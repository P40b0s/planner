use std::{borrow::Cow, net::SocketAddr, ops::{Deref, DerefMut}, sync::Arc};
use axum::{body::Body, extract::{ConnectInfo, FromRequestParts, Request, State}, http::{request::Parts, HeaderValue}, response::{IntoResponse, IntoResponseParts, Response, ResponseParts}, routing::{get, patch, post}, Extension, Json, Router};
use cookie::{Cookie, CookieJar};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use utilites::Date;
use crate::state::AppState;
use logger::debug;

use super::{auth_middleware::AuthLayer, cookie_middleware::CookieLayer, roles::Roles};

async fn test_api() -> Result<(), crate::Error>
{
    let state = Arc::new(AppState::initialize().await?);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8888));
    debug!("Апи сервера доступно на {}", &addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router(state).into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
    Ok(())
}

pub fn authorization_and_users_router(app_state: Arc<AppState>) -> Router
{   
    Router::new()      
        .route("/auth/login", post(login))
        .route_layer(AuthLayer::new(Arc::clone(&app_state), &["role1", "role2"], &["http://google.com"]))
        .with_state(app_state)
        .layer(crate::api::cors_layer())
        .layer(CookieLayer)
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)))
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoginPayload
{
    pub login: String,
    pub password: String
}
#[derive(Debug, Clone, Serialize)]
pub struct AuthorizationInfo<R> where R: ToString + Serialize
{
    pub id: String,
    pub name: String,
    pub surname_1: String,
    pub surname_2: String,
    pub role: R,
    pub access_key: String,
    ///дата до которой годен рефреш токен
    pub expiration_date: String,
    pub avatar: Option<String>

}

//json экстрактор последний!
pub async fn login(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    cookie_jar: CookiesExtractor,
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<LoginPayload>) 
-> impl IntoResponse
{
    

    if let Some(cookie) = cookie_jar.get("example_cookie") {
        println!("Получена cookie: {}", cookie.value());
    }
    let ip = addr.ip().to_string();
    //получаем юзера из БД и берем роль и аудит
    //let access_key = app_state.services.jwt_service.gen_key(&logged.id, role.clone());
    let access_key =  "TEST_ACCESS_KEY".to_owned();
    let authorized = AuthorizationInfo::<Roles>
    {
        id: "123321".to_owned(),
        name: "test_username".to_owned(),
        surname_1: "test_surname_1".to_owned(),
        surname_2: "test_surname_2".to_owned(),
        role: Roles::User,
        access_key,
        expiration_date: Date::now().add_minutes(666).to_string(),
        avatar: None
    };
     
     let mut new_cookie_jar = Cookies::new();
     new_cookie_jar.add("example_cookie", ip);
    (
        StatusCode::OK,
        new_cookie_jar,
        Json(authorized),
    )
}
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
pub struct CookieWrapper(CookieJar);
// impl IntoResponse for CookieWrapper 
// {
//     fn into_response(self) -> Response 
//     {
//         let mut response = Response::new(axum::body::Body::empty());
//         let headers = response.headers_mut();
//         for cookie in self.0.iter() 
//         {
//             let header_value = HeaderValue::from_str(&cookie.to_string());
//             if let Ok(hv) = header_value
//             {
//                 headers.append("Set-Cookie", hv);
//             }
//             else 
//             {
//                 logger::error!("Failed to convert cookie `{}` to header value", cookie.to_string());    
//             }
//         }
//         response
//     }
// }
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

pub fn router(app_state: Arc<AppState>) -> Router
{   
    let auth_router = authorization_and_users_router(Arc::clone(&app_state));
    Router::new()
        .fallback(handler_404)      
        .with_state(app_state)
        .layer(super::cors::cors_layer())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        ).merge(auth_router)
}

async fn handler_404() -> impl IntoResponse 
{
    (StatusCode::NOT_FOUND, "Такого пути нет")
}

#[cfg(test)]
mod tests
{
    #[tokio::test]
    async fn test_running()
    {
        super::test_api().await;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(60000)).await;
        }
    }
}