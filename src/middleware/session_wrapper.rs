use std::sync::Arc;

use axum::{extract::{FromRef, FromRequestParts}, http::{request::Parts, HeaderValue}, response::{IntoResponse, IntoResponseParts, Response, ResponseParts}};
use jwt_authentification::{Cookie, CookieJar, Duration};
use utilites::Date;

use crate::{configuration::Configuration, db::Session, state::AppState, Error};

#[derive(Clone)]
pub struct ResponseSessionWrapper
{
    session: Arc<Session>,
    cfg: Arc<Configuration>
}
impl ResponseSessionWrapper
{
    pub fn new(session: Arc<Session>, cfg: Arc<Configuration>) -> Self
    {
        Self
        {
            session,
            cfg
        }
    }
    pub fn to_cookie(&self) -> String
    {
        
        let cookie: Cookie = Cookie::build((&self.cfg.session_cookie_name, self.session.session_id.to_string()))
        .max_age(Duration::days(self.cfg.session_life_time as i64))
        .path("/")
        .into();
        cookie.to_string()
    }
}

pub struct FingerprintExtractor(pub Arc<String>);
impl<S> FromRequestParts<S> for FingerprintExtractor
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> 
    {
        // Извлекаем заголовки из запроса
        let app_state = Arc::<AppState>::from_ref(state);
        if let Some(fingerprint) = parts.headers.get(&app_state.configuration.fingerprint_header_name)
        {
            if let Ok(f) = fingerprint.to_str()
            {
                return Ok(FingerprintExtractor(Arc::new(f.to_string())));
            }
        }
        Err(Error::FingerprintNotFound)
    }
}


impl IntoResponseParts for ResponseSessionWrapper
{
    type Error = std::convert::Infallible;
    fn into_response_parts(self, mut response: ResponseParts) -> Result<ResponseParts, Self::Error> 
    {
        // Добавляем cookies в заголовки ответа
        let headers = response.headers_mut();
        let cookie = self.to_cookie();
        let header_value = HeaderValue::from_str(&cookie);
        if let Ok(hv) = header_value
        {
            headers.append("Set-Cookie", hv);
        }
        else 
        {
            logger::error!("Failed to convert cookie `{}` to header value", cookie);    
        }
        Ok(response)
    }
}
#[derive(Clone)]
pub struct SessionExtension
{
    pub session: Arc<Session>,
    pub fingerprint: Arc<String>,
    pub role: Arc<Option<String>>
}
impl<S> FromRequestParts<S> for SessionExtension
where
    S: Send + Sync,
{
    type Rejection = Response;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> 
    {
        if let Some(session) = parts.extensions.get::<SessionExtension>()
        {
            Ok(session.clone())
        }
        else 
        {
            Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Ошибка экстрактора").into_response())
        }
    }
}