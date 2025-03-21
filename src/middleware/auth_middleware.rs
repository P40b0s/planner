use std::sync::Arc;
use std::task::{Context, Poll};
use axum::body::Body;
use axum::response::IntoResponse;
use hyper::header::{AUTHORIZATION, COOKIE};
use hyper::HeaderMap;
use jwt_authentification::{Cookie, CookieJar, Duration as CookieMaxLife};
use tower::{Service, Layer};
use axum::http::{Extensions, Request, Response, StatusCode};
use futures::future::BoxFuture;
use futures::FutureExt;
use crate::configuration::Configuration;
use crate::db::{ISessionRepository, Session};
use crate::state::AppState;
use super::SessionExtension;
#[derive(Copy, Clone)]
pub enum AuthCheck
{
    Session,
    All
}

/// Слой для проверки авторизации пользователей
#[derive(Clone)]
pub struct AuthMiddleware<S> 
{
    inner: S,
    state: Arc<AppState>,
    roles: Arc<Vec<String>>,
    audience: Arc<Vec<String>>,
    check: AuthCheck
}

impl<S> AuthMiddleware<S> 
{
    pub fn new(inner: S, check: AuthCheck, state: Arc<AppState>, roles: Arc<Vec<String>>, audience: Arc<Vec<String>>) -> Self 
    {
        Self 
        {
            inner,
            state,
            roles,
            audience,
            check
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

    fn call(&mut self, mut req: Request<axum::body::Body>) -> Self::Future 
    {
        let state = self.state.clone();
        let roles = self.roles.clone();
        let audience = self.audience.clone();
        let mut inner = self.inner.clone();
        let check = self.check;
        //let mut inner: S = std::mem::replace(&mut self.inner, inner);
        async move 
        {
            let headers = req.headers();
            let session = cookie_checker(headers, state.clone()).await;
            if session.is_err()
            {
                Ok(session.err().unwrap())
            }
            else
            {
                let fingerprint = fingerprint_checker(headers, state.clone()).await;
                if fingerprint.is_err()
                {
                    Ok(fingerprint.err().unwrap())
                }
                else
                {
                    if let AuthCheck::Session = check
                    {
                        let fingerprint = fingerprint.unwrap().to_owned();
                        let session_extension = SessionExtension
                        {
                            session: Arc::new(session.unwrap()),
                            fingerprint: Arc::new(fingerprint)
                        };
                        let ext = req.extensions_mut();
                        ext.insert(session_extension);
                        inner.call(req).await
                    }
                    else
                    {
                        if let Err(e) = bearer_checker(headers, session.as_ref().unwrap(), state, roles, audience).await
                        {
                            Ok(e)
                        }
                        else
                        {
                            let fingerprint = fingerprint.unwrap().to_owned();
                            let session_extension = SessionExtension
                            {
                                session: Arc::new(session.unwrap()),
                                fingerprint: Arc::new(fingerprint)
                            };
                            let ext = req.extensions_mut();
                            ext.insert(session_extension);
                            inner.call(req).await
                        }
                    }
                }
            }
        }
        .boxed()
    }
}

fn error_response<T: ToString>(body: T) -> Response<axum::body::Body>
{
    let err = body.to_string();
    logger::error!("{}", &err);
    let body = axum::body::Body::new(err);
    Response::builder()
    .status(StatusCode::UNAUTHORIZED)
    .body(body)
    .unwrap()
}
///возврат ошибки клиенту с удалением cookie рефреш ключа (сессии)
pub fn cookie_error_response<T: ToString>(body: T, cfg: &Configuration) -> Response<axum::body::Body>
{
    let err = body.to_string();
    logger::error!("{}", &err);
    let body = axum::body::Body::new(err);
    let cookie: Cookie = Cookie::build((&cfg.session_cookie_name, "")).path("/").max_age(CookieMaxLife::seconds(0)).into();
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::SET_COOKIE,
        cookie.to_string().parse().unwrap(),
    );
    let mut resp = Response::builder()
    .status(StatusCode::UNAUTHORIZED)
    .body(body)
    .unwrap();
    *resp.headers_mut() = headers;
    resp
}

async fn cookie_checker(headers: &HeaderMap, state: Arc<AppState>) -> Result<Session, Response<Body>>
{
    if let Some(cookie_header) = headers.get(COOKIE)
    {
        let mut cookie_jar = CookieJar::new();
        // Извлечение cookies из запроса
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
        
        if let Some(cookie) = cookie_jar.get(&state.configuration.session_cookie_name)
        {
            let session = state.services.database_service.session_repository.get_session(&cookie.value().parse().unwrap()).await;
            if let Ok(session) = session
            {
                if !session.is_expired()
                {
                    Ok(session)
                }
                else
                {
                    let response = cookie_error_response("Время вашей сессии истекло, необходимо зайти в систему заново", &state.configuration);
                    Err(response)
                }
            }
            else
            {
                //let response = error_response(session.err().unwrap().to_string());
                let error = session.err().unwrap();
                logger::error!("{}", error.to_string());
                Err(error.into_response())
            }

        }
        else
        {
            let response = cookie_error_response("Ошибка авторизации, отсуствует cookie вашей сессии", &state.configuration);
            Err(response)
        }
    }
    else
    {
        let response = cookie_error_response("Ошибка авторизации, отсуствует cookie вашей сессии", &state.configuration);
        Err(response)
    }
}

async fn bearer_checker(headers: &HeaderMap, session: &Session, state: Arc<AppState>, roles: Arc<Vec<String>>, audience: Arc<Vec<String>>) -> Result<(), Response<Body>>
{
    if let Some(authorization) = headers.get(AUTHORIZATION)
    {
        //get key after Bearer 
        if let Ok(token_str) = authorization.to_str()
        {
            if token_str.len() < 10
            {
                let response = error_response("Bearer не распознан");
                Err(response)
            }
            else
            {
                //cut Bearer
                let token_str =  token_str[7..].trim();
                let user_claims = state.services.jwt_service.validate(&session.user_id, token_str, &*roles, &audience).await;
                if let Ok(_) = user_claims 
                {
                    Ok(())
                }
                else
                {
                    let response = error_response(user_claims.err().unwrap());
                    Err(response)
                }
            }
        }
        else
        {
            let response = error_response("Ошибка авторизации, заголовок Authorization имеет ошибки в кодировке");
            Err(response)
        }
    }
    else
    {
        let response = error_response("Ошибка авторизации, отсуствует заголовок Authorization");
        Err(response)
    }
}

async fn fingerprint_checker<'a >(headers: &'a HeaderMap, state: Arc<AppState>) -> Result<&'a str, Response<Body>>
{
    if let Some(authorization) = headers.get(&state.configuration.fingerprint_header_name)
    {
        //get key after Bearer 
        if let Ok(fingerprint) = authorization.to_str()
        {
            Ok(fingerprint)
        }
        else
        {
            let response = error_response(["Ошибка, заголовок ", &state.configuration.fingerprint_header_name, " имеет ошибки в кодировке"].concat());
            Err(response)
        }
    }
    else
    {
        let response = error_response("Ошибка, отсуствует уникальный отпечаток сессии");
        Err(response)
    }
}

/// Слой обработки маршрута с авторизацией
#[derive(Clone)]
pub struct AuthLayer 
{
    state: Arc<AppState>,
    roles: Arc<Vec<String>>,
    audience: Arc<Vec<String>>,
    check: AuthCheck
}

impl AuthLayer 
{
    pub fn with_roles<R: ToString>(check: AuthCheck, state: Arc<AppState>, roles: &[R]) -> Self 
    {
        Self 
        {
            state,
            roles: Arc::new(roles.into_iter().map(|v| v.to_string()).collect()),
            audience: Arc::new(Vec::new()),
            check
        }
    }
    pub fn with_audiences<R: ToString, A: ToString>(check: AuthCheck, state: Arc<AppState>, roles: &[R], audience: &[A]) -> Self 
    {
        Self 
        {
            state,
            roles: Arc::new(roles.into_iter().map(|v| v.to_string()).collect()),
            audience: Arc::new(audience.into_iter().map(|v| v.to_string()).collect()),
            check
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
        AuthMiddleware::new(inner, self.check, self.state.clone(), self.roles.clone(), self.audience.clone())
    }
}