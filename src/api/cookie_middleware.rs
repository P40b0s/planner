use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Service, Layer};
use axum::http::{Request, Response, StatusCode, header::SET_COOKIE};
use axum::body::Body;
use cookie::{Cookie, CookieJar};
use futures::future::BoxFuture;
use futures::FutureExt;

/// Структура middleware для управления cookies
#[derive(Clone)]
pub struct CookieMiddleware<S> 
{
    inner: S,
}

impl<S> CookieMiddleware<S> 
{
    pub fn new(inner: S) -> Self 
    {
        Self { inner }
    }
}

/// Реализация трейта `Service` для middleware
impl<S> Service<Request<Body>> for CookieMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> 
    {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future 
    {
        let mut inner = self.inner.clone();
        //let mut inner = std::mem::replace(&mut self.inner, inner);
        let mut cookie_jar = CookieJar::new();
        // Извлечение cookies из запроса
        let cookie_header = req.headers().get("Cookie");
        if let Some(cookie_header) = cookie_header
        {
            if let Ok(cookie_str) = cookie_header.to_str().and_then(|c| Ok(c.to_string())) 
            {
                for cookie in Cookie::split_parse(cookie_str) 
                {
                    if let Ok(cookie) = cookie 
                    {
                        cookie_jar.add(cookie);
                    }
                }
            }
        }
        // Передача cookie_jar в запрос
        req.extensions_mut().insert(Arc::new(cookie_jar));

        async move 
        {
            // Вызов внутреннего сервиса
            let mut response = inner.call(req).await?;

            // Добавление cookies в ответ
            let cookie_jar = response.extensions().get::<Arc<CookieJar>>().and_then(|cj| Some(cj.clone()));
            if let Some(cookie_jar) = cookie_jar
            {
                for cookie in cookie_jar.iter() 
                {
                    response.headers_mut().append(
                        SET_COOKIE,
                        cookie.to_string().parse().unwrap(),
                    );
                }
            }
            Ok(response)
        }
        .boxed()
    }
}

/// Слой для добавления middleware
#[derive(Clone)]
pub struct CookieLayer;

impl<S> Layer<S> for CookieLayer
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Service = CookieMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service 
    {
        CookieMiddleware::new(inner)
    }
}