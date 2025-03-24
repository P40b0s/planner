// use std::{borrow::Cow, net::SocketAddr, ops::{Deref, DerefMut}, sync::Arc};
// use axum::{body::Body, extract::{ConnectInfo, FromRequestParts, Request, State}, http::{request::Parts, HeaderValue}, response::{IntoResponse, IntoResponseParts, Response, ResponseParts}, routing::{get, patch, post}, Extension, Json, Router};
// use hyper::StatusCode;
// use serde::{Deserialize, Serialize};
// use tower_http::trace::{DefaultMakeSpan, TraceLayer};
// use utilites::Date;
// use crate::{db::ISessionRepository, middleware::{AuthCheck, FingerprintExtractor, ResponseSessionWrapper, SessionExtension}, state::AppState, Error};
// use logger::debug;
// use crate::Role;
// use crate::{middleware::AuthLayer,};

// async fn test_api() -> Result<(), crate::Error>
// {
//     let state = Arc::new(AppState::initialize().await?);
//     let addr = SocketAddr::from(([0, 0, 0, 0], state.configuration.server_port));
//     debug!("Апи сервера доступно на {}", &addr);
//     let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
//     axum::serve(listener, router(state).into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
//     Ok(())
// }

// pub fn authorization_and_users_router(app_state: Arc<AppState>) -> Router
// {   
//     Router::new()      
//         .route("/auth/login", post(login))
//         //.route_layer(AuthLayer::new(Arc::clone(&app_state), &[Roles::User, Roles::Administrator], &["http://google.com"]))
//         // .route("/test/create_session", method_router)
//         .route("/auth/update_key", get(update_access)
//             .route_layer(AuthLayer::with_roles(AuthCheck::Session, Arc::clone(&app_state), &[Role::User, Role::Administrator])))
            
//         .route("/auth/admin", get(admin_section)
//             .route_layer(AuthLayer::with_roles(AuthCheck::All, Arc::clone(&app_state), &[Role::Administrator])))
            
//         .with_state(app_state.clone())
//         //.layer(crate::api::cors_layer(app_state.clone()))
//         .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)))
// }



// //session-key=0195aec0-1f11-7692-b390-36010f5ace45
// //accsess eyJ0eXAiOiJKV1QiLCJhbGciOiJFZERTQSJ9.eyJzdWIiOiIwMTk1YWU3YS0zY2RhLTdiMTEtYWE2Yi00Njk5MmEzZTIwOWYiLCJleHAiOjE3NDIzOTM4ODEsImlhdCI6MTc0MjM5MzU4MSwicm9sZSI6IkFkbWluaXN0cmF0b3IiLCJhdWQiOm51bGx9.w6Fe6ZXn9bVZ1PTfZmgcPwcctJpi6Q2HeMmm8Wg5EFNO_yzY9JkORrLyxSj6NnMDN1DytTnLNAPCCPORMDgcBg
// //json экстрактор последний!
// pub async fn login(
//     ConnectInfo(addr): ConnectInfo<SocketAddr>,
//     State(app_state): State<Arc<AppState>>,
//     FingerprintExtractor(fp): FingerprintExtractor,
//     Json(payload): Json<LoginPayload>) 
// -> Result<impl IntoResponse, Error>
// {
//     let ip = addr.ip().to_string();
//     let user = app_state.get_services().user_service.login(&payload.login, &payload.password, &ip, &fp).await;
    
//     if let Ok((user_info, session)) = user
//     {
//         logger::debug!("Юзер {} прошел авторизацию", &payload.login);
//         let session_wrapper = ResponseSessionWrapper::new(Arc::new(session), app_state.configuration.clone());
//         Ok((
//             StatusCode::OK,
//             session_wrapper,
//             Json(user_info),
//         ))
//     }
//     else 
//     {
//         Err(user.err().unwrap())
//     }
// }

// pub async fn admin_section(
//     ConnectInfo(addr): ConnectInfo<SocketAddr>,
//     State(app_state): State<Arc<AppState>>,
//     Extension(session_wrapper): Extension<SessionExtension>) 
// -> Result<impl IntoResponse, Error>
// {
//     let user = app_state.services.database_service.user_repository.get_user(&session_wrapper.session.user_id).await?;
//     Ok((
//         StatusCode::OK,
//         "вы зашли в админский роут",
//     ))
// }
// pub async fn update_access(
//     ConnectInfo(_addr): ConnectInfo<SocketAddr>,
//     Extension(session_wrapper): Extension<SessionExtension>,
//     State(app_state): State<Arc<AppState>>) 
// -> Result<impl IntoResponse, Error>
// {
//     //TODO надо ли проверить ip адрес? не всегда он будет совпадать так как везде почти динамический
//     let new_fingerprint = &session_wrapper.fingerprint;
//     logger::debug!("новый fp:{} старый fp: {} из fp extractor", new_fingerprint, &session_wrapper.session.fingerprint);
//     if &session_wrapper.session.fingerprint != new_fingerprint.as_str()
//     {
//         logger::error!("Ошибка, новый fingerprint {} не совпадает с отпечатком сеcсии {}", new_fingerprint, &session_wrapper.session.fingerprint);
//         let _ = app_state.services.database_service.session_repository.delete_session(&session_wrapper.session.session_id).await;
//         return Err(Error::WrongFingerprintError(app_state.configuration.session_cookie_name.clone()));
//     }

//     let user = app_state.services.database_service.user_repository.get_user(&session_wrapper.session.user_id).await?;
//     let new_access = app_state.services.jwt_service.gen_key(&user.id, user.role, &user.audiences, app_state.configuration.access_key_lifetime).await;
//     if app_state.configuration.update_session_time_on_request
//     {
//         let _ = app_state.services.database_service.session_repository.update_session_key(&session_wrapper.session.session_id, app_state.configuration.access_key_lifetime).await?;
//     }
//     logger::debug!("Обновлен access key `{}` для сессии {}", &new_access, user.id.to_string());
//     let s = ResponseSessionWrapper::new(session_wrapper.session, app_state.configuration.clone());
//     Ok((
//         StatusCode::OK,
//         s,
//         new_access,
//     ))
// }


// // pub async fn create_session(
// //     ConnectInfo(addr): ConnectInfo<SocketAddr>,
// //     cookie_jar: CookiesExtractor,
// //     State(app_state): State<Arc<AppState>>,
// //     Json(payload): Json<LoginPayload>) 
// // -> impl IntoResponse
// // {
    

// //     if let Some(cookie) = cookie_jar.get("example_cookie") {
// //         println!("Получена cookie: {}", cookie.value());
// //     }
// //     let ip = addr.ip().to_string();
// //     //получаем юзера из БД и берем роль и аудит
// //     //let access_key = app_state.services.jwt_service.gen_key(&logged.id, role.clone());
// //     let access_key =  "TEST_ACCESS_KEY".to_owned();
// //     let authorized = AuthorizationInfo::<Role>
// //     {
// //         id: "123321".to_owned(),
// //         name: "test_username".to_owned(),
// //         surname_1: "test_surname_1".to_owned(),
// //         surname_2: "test_surname_2".to_owned(),
// //         role: Role::User,
// //         access_key,
// //         expiration_date: Date::now().add_minutes(666).to_string(),
// //         avatar: None
// //     };
     
// //     //куки устанавливаем только если их еще нет, при обновлении ключа они у клиента есть
// //     (
// //         StatusCode::OK,
// //         Json(authorized),
// //     )
// // }

// //pub struct CookieWrapper(CookieJar);
// // impl IntoResponse for CookieWrapper 
// // {
// //     fn into_response(self) -> Response 
// //     {
// //         let mut response = Response::new(axum::body::Body::empty());
// //         let headers = response.headers_mut();
// //         for cookie in self.0.iter() 
// //         {
// //             let header_value = HeaderValue::from_str(&cookie.to_string());
// //             if let Ok(hv) = header_value
// //             {
// //                 headers.append("Set-Cookie", hv);
// //             }
// //             else 
// //             {
// //                 logger::error!("Failed to convert cookie `{}` to header value", cookie.to_string());    
// //             }
// //         }
// //         response
// //     }
// // }



// pub fn router(app_state: Arc<AppState>) -> Router
// {   
//     let auth_router = authorization_and_users_router(Arc::clone(&app_state));
//     Router::new()
//         .fallback(handler_404)      
//         .with_state(app_state.clone())
//         .layer(super::cors::cors_layer(app_state.clone()))
//         .layer(
//             TraceLayer::new_for_http()
//                 .make_span_with(DefaultMakeSpan::default().include_headers(true)),
//         ).merge(auth_router)
// }

// async fn handler_404() -> impl IntoResponse 
// {
//     (StatusCode::NOT_FOUND, "Такого пути нет")
// }

// #[cfg(test)]
// mod tests
// {
//     #[tokio::test]
//     async fn test_running()
//     {
//         logger::StructLogger::new_default();
//         super::test_api().await;
//         loop {
//             tokio::time::sleep(tokio::time::Duration::from_millis(60000)).await;
//         }
//     }
// }