use std::{borrow::Cow, net::SocketAddr, ops::{Deref, DerefMut}, sync::Arc};
use auth_service::IAuthorizationRepository;
use axum::{body::Body, extract::{ConnectInfo, FromRequestParts, Request, State}, http::{request::Parts, HeaderValue}, response::{IntoResponse, IntoResponseParts, Response, ResponseParts}, routing::{get, patch, post}, Extension, Json, Router};
use cookie::{Cookie, CookieJar};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use utilites::Date;
use crate::{middleware::{Cookies, CookiesExtractor}, state::AppState, Error};
use logger::debug;
use crate::Roles;
use crate::{middleware::AuthLayer, middleware::CookieLayer};

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
        //.route_layer(AuthLayer::new(Arc::clone(&app_state), &[Roles::User, Roles::Administrator], &["http://google.com"]))
        // .route("/test/create_session", method_router)
        .route("/auth/update_key", post(update_access))
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


//session-key=0195aec0-1f11-7692-b390-36010f5ace45
//accsess eyJ0eXAiOiJKV1QiLCJhbGciOiJFZERTQSJ9.eyJzdWIiOiIwMTk1YWU3YS0zY2RhLTdiMTEtYWE2Yi00Njk5MmEzZTIwOWYiLCJleHAiOjE3NDIzOTM4ODEsImlhdCI6MTc0MjM5MzU4MSwicm9sZSI6IkFkbWluaXN0cmF0b3IiLCJhdWQiOm51bGx9.w6Fe6ZXn9bVZ1PTfZmgcPwcctJpi6Q2HeMmm8Wg5EFNO_yzY9JkORrLyxSj6NnMDN1DytTnLNAPCCPORMDgcBg
//json экстрактор последний!
pub async fn login(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    cookie_jar: CookiesExtractor,
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<LoginPayload>) 
-> Result<impl IntoResponse, Error>
{
    if let Some(cookie) = cookie_jar.get("session-key") {
        println!("Получена cookie: {}", cookie.value());
    }
    let ip = addr.ip().to_string();
    let fingerprint = "123321123";
    let user = app_state.get_services().user_service.login(&payload.login, &payload.password, &ip, fingerprint).await;
    if let Ok((user_info, refresh_key)) = user
    {
        let mut new_cookie_jar = Cookies::new();
        new_cookie_jar.add("session-key", refresh_key.to_string());
        Ok((
            StatusCode::OK,
            new_cookie_jar,
            Json(user_info),
        ))
    }
    else 
    {
        Err(user.err().unwrap())
    }
    // //получаем юзера из БД и берем роль и аудит
    // //let access_key = app_state.services.jwt_service.gen_key(&logged.id, role.clone());
    // let access_key =  "TEST_ACCESS_KEY".to_owned();
    // let authorized = AuthorizationInfo::<Roles>
    // {
    //     id: "123321".to_owned(),
    //     name: "test_username".to_owned(),
    //     surname_1: "test_surname_1".to_owned(),
    //     surname_2: "test_surname_2".to_owned(),
    //     role: Roles::User,
    //     access_key,
    //     expiration_date: Date::now().add_minutes(666).to_string(),
    //     avatar: None
    // };
     
    //  let mut new_cookie_jar = Cookies::new();
    //  new_cookie_jar.add("example_cookie", ip);
    // (
    //     StatusCode::OK,
    //     new_cookie_jar,
    //     Json(authorized),
    // )
}

pub async fn admin_section(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    cookie_jar: CookiesExtractor,
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<LoginPayload>) 
-> Result<impl IntoResponse, Error>
{
    if let Some(cookie) = cookie_jar.get("session-key") {
        println!("Получена cookie: {}", cookie.value());
    }
    let ip = addr.ip().to_string();
    let fingerprint = "123321123";
    let user = app_state.get_services().user_service.login(&payload.login, &payload.password, &ip, fingerprint).await;
    if let Ok((user_info, refresh_key)) = user
    {
        let mut new_cookie_jar = Cookies::new();
        new_cookie_jar.add("session-key", refresh_key.to_string());
        Ok((
            StatusCode::OK,
            new_cookie_jar,
            Json(user_info),
        ))
    }
    else 
    {
        Err(user.err().unwrap())
    }
    // //получаем юзера из БД и берем роль и аудит
    // //let access_key = app_state.services.jwt_service.gen_key(&logged.id, role.clone());
    // let access_key =  "TEST_ACCESS_KEY".to_owned();
    // let authorized = AuthorizationInfo::<Roles>
    // {
    //     id: "123321".to_owned(),
    //     name: "test_username".to_owned(),
    //     surname_1: "test_surname_1".to_owned(),
    //     surname_2: "test_surname_2".to_owned(),
    //     role: Roles::User,
    //     access_key,
    //     expiration_date: Date::now().add_minutes(666).to_string(),
    //     avatar: None
    // };
     
    //  let mut new_cookie_jar = Cookies::new();
    //  new_cookie_jar.add("example_cookie", ip);
    // (
    //     StatusCode::OK,
    //     new_cookie_jar,
    //     Json(authorized),
    // )
}
pub async fn update_access(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    cookie_jar: CookiesExtractor,
    State(app_state): State<Arc<AppState>>) 
-> Result<impl IntoResponse, Error>
{
    if let Some(cookie) = cookie_jar.get("session-key") 
    {
        println!("Получена cookie: {}", cookie.value());
        let user = app_state.get_services().database_service.authorization_repository.get_session(&cookie.value().parse().unwrap()).await?;
        //TODO сравниваем  fingerprint
        let new_access = app_state.get_services().jwt_service.gen_key(&user.id, user.role, &user.audience).await;
        logger::info!("Обновлен access key `{}` для сессии {}", &new_access, user.id.to_string());
        let mut new_cookie_jar = Cookies::new();
        new_cookie_jar.add("session-key", user.id.to_string());
        Ok((
            StatusCode::OK,
            new_cookie_jar,
            new_access,
        ))
    }
    else 
    {
        Err(Error::SessionExpired)    
    }
}


pub async fn create_session(
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
     
    //куки устанавливаем только если их еще нет, при обновлении ключа они у клиента есть
    (
        StatusCode::OK,
        Json(authorized),
    )
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
        logger::StructLogger::new_default();
        super::test_api().await;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(60000)).await;
        }
    }
}