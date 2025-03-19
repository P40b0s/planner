use std::sync::Arc;
use std::task::{Context, Poll};
use hyper::header::AUTHORIZATION;
use tower::{Service, Layer};
use axum::http::{Request, Response, StatusCode};
use futures::future::BoxFuture;
use futures::FutureExt;
use crate::state::AppState;

/// Слой для проверки авторизации пользователей
#[derive(Clone)]
pub struct AuthMiddleware<S> 
{
    inner: S,
    state: Arc<AppState>,
    roles: Arc<Vec<String>>,
    audience: Arc<Vec<String>>,
}

impl<S> AuthMiddleware<S> 
{
    pub fn new(inner: S, state: Arc<AppState>, roles: Arc<Vec<String>>, audience: Arc<Vec<String>>) -> Self 
    {
        Self 
        {
            inner,
            state,
            roles,
            audience
        }
    }
}

/// Реализация трейта `Service` для middleware
impl<S> Service<Request<axum::body::Body>> for AuthMiddleware<S>
where
    S: Service<Request<axum::body::Body>, Response = Response<axum::body::Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<axum::body::Body>;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> 
    {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<axum::body::Body>) -> Self::Future 
    {
        let state = self.state.clone();
        let roles = self.roles.clone();
        let audience = self.audience.clone();
        let mut inner = self.inner.clone();
        //let mut inner: S = std::mem::replace(&mut self.inner, inner);
        async move 
        {
            // Извлечение заголовков
            let headers = req.headers().clone();
            match headers.get(AUTHORIZATION) 
            {
                Some(value) => 
                {
                    //get key after Bearer 
                    if let Ok(token_str) = value.to_str()
                    {
                        if token_str.len() < 10
                        {
                            let response = error_response("Bearer не распознан");
                            Ok(response)
                        }
                        else
                        {
                            //cut Bearer
                            let token_str =  token_str[7..].trim();
                            let user_claims = state.services.jwt_service.validate(token_str, &*roles, &audience).await;
                            if let Ok(_) = user_claims 
                            {
                                inner.call(req).await
                            }
                            else
                            {
                                let response = error_response(user_claims.err().unwrap());
                                Ok(response)
                            }
                        }
                    }
                    else
                    {
                        let response = error_response("Ошибка авторизации, заголовок Authorization имеет ошибки в кодировке");
                        Ok(response)
                    }
                },
                None => 
                {
                    let response = error_response("Ошибка авторизации, отсуствует заголовок Authorization");
                    Ok(response)
                }
            }
        }
        .boxed()
    }
}

fn error_response<T: ToString>(body: T) -> Response<axum::body::Body>
{
    let body = axum::body::Body::new(body.to_string());
    Response::builder()
    .status(StatusCode::UNAUTHORIZED)
    .body(body)
    .unwrap()
}

/// Слой обработки маршрута с авторизацией
#[derive(Clone)]
pub struct AuthLayer 
{
    state: Arc<AppState>,
    roles: Arc<Vec<String>>,
    audience: Arc<Vec<String>>,
}

impl AuthLayer 
{
    pub fn new<R: ToString, A: ToString>(state: Arc<AppState>, roles: &[R], audience: &[A]) -> Self 
    {
        Self 
        {
            state,
            roles: Arc::new(roles.into_iter().map(|v| v.to_string()).collect()),
            audience: Arc::new(audience.into_iter().map(|v| v.to_string()).collect()),
        }
    }
}

impl<S> Layer<S> for AuthLayer
where
    S: Service<Request<axum::body::Body>, Response = Response<axum::body::Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Service = AuthMiddleware<S>;
    fn layer(&self, inner: S) -> Self::Service 
    {
        AuthMiddleware::new(inner, self.state.clone(), self.roles.clone(), self.audience.clone())
    }
}