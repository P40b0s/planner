use std::borrow::Cow;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::task::{Context, Poll};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::HeaderValue;
use axum::response::{IntoResponse, IntoResponseParts, Response, ResponseParts};
use tower::{Service, Layer};
use axum::http::{Request, header::SET_COOKIE};
use axum::body::Body;
use cookie::{Cookie, CookieJar};
use futures::future::BoxFuture;
use futures::FutureExt;
//TODO переписать комменты на английский
/// Структура для управления cookies
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

/// Слой для обработки cookies
/// указать для роута:
/// ```  
/// Router::new()      
/// .route("/auth/login", post(login))
/// ...
/// .with_state(app_state)
/// .layer(CookieLayer)
/// ```  
/// указать в обработчике:
/// ```
/// pub async fn login(
/// ConnectInfo(addr): ConnectInfo<SocketAddr>,
/// cookie_jar: CookiesExtractor,
/// State(app_state): State<Arc<AppState>>,
/// Json(payload): Json<LoginPayload>) 
/// -> impl IntoResponse
/// {
///    //прочитать cookie из запроса
///    if let Some(cookie) = cookie_jar.get("example_cookie") 
///    {
///           println!("Получена cookie: {}", cookie.value());
///    }
///    //добавить cookie в запрос
///    let mut new_cookie_jar = Cookies::new();
///    new_cookie_jar.add("example_cookie", "blah blah blah");
/// 
///    (
///        StatusCode::OK,
///        new_cookie_jar,
///        Json(authorized),
///    )
/// }```
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


///Структура для создания cookies
/// ```
/// let mut new_cookie_jar = Cookies::new();
/// new_cookie_jar.add("example_cookie", "blah blah blah");
/// ```
/// имплементирует `FromRequestParts`, можно сразу добавлять в response
pub struct Cookies(CookieJar);
impl Cookies
{
    pub fn new() -> Self
    {
        Self(CookieJar::new())
    }
    pub fn add<N, V>(&mut self, name: N, value: V)
    where
        N: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        self.0.add(Cookie::new(name, value));
    }
}
impl Deref for Cookies
{
    type Target = CookieJar;
    fn deref(&self) -> &Self::Target 
    {
        &self.0
    }
}
impl DerefMut for Cookies
{
    fn deref_mut(&mut self) -> &mut Self::Target 
    {
        &mut self.0
    }
}
impl IntoResponseParts for Cookies
{
    type Error = std::convert::Infallible;

    fn into_response_parts(self, mut response: ResponseParts) -> Result<ResponseParts, Self::Error> 
    {
        // Добавляем cookies в заголовки ответа
        let headers = response.headers_mut();
        for cookie in self.0.iter() 
        {
            let header_value = HeaderValue::from_str(&cookie.to_string());
            if let Ok(hv) = header_value
            {
                headers.append("Set-Cookie", hv);
            }
            else 
            {
                logger::error!("Failed to convert cookie `{}` to header value", cookie.to_string());    
            }
        }
        Ok(response)
    }
}


///Для извлечения cookies в хендлере:
/// ```
/// pub async fn login(
/// ...,
/// cookie_jar: CookiesExtractor,
/// ...,
/// -> impl IntoResponse
/// {
///    //прочитать cookie из запроса
///    if let Some(cookie) = cookie_jar.get("example_cookie") 
///    {
///           println!("Получена cookie: {}", cookie.value());
///    }
/// }
pub struct CookiesExtractor(pub Arc<CookieJar>);
impl Deref for CookiesExtractor
{
    type Target = Arc<CookieJar>;
    fn deref(&self) -> &Self::Target 
    {
        &self.0
    }
}

impl<S> FromRequestParts<S> for CookiesExtractor
where
    S: Send + Sync,
{
    type Rejection = Response;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> 
    {
        if let Some(cookie_jar) = parts.extensions.get::<Arc<CookieJar>>() 
        {
            Ok(CookiesExtractor(cookie_jar.clone()))
        } 
        else 
        {
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}