// use std::{net::SocketAddr, sync::Arc, time::Duration};
// use axum::{extract::{ConnectInfo, Query, Request, State}, response::{IntoResponse, Response}, routing::{get, post}, Json, Router};
// use axum::http::header::SET_COOKIE;
// use serde::{Deserialize, Serialize};
// use tower_http::trace::{DefaultMakeSpan, TraceLayer};
// use crate::{auth_route::{AuthRoute, AuthRouteParams}, state::AppState, Error};
// use utilites::http::HeaderMap;
// use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};

// use super::{auth_middleware::AuthLayer, roles::Roles};

// const REFRESH_KEY_COOKIE: &'static str = "refresh";
// //TODO не создает сессию! проверить
// pub fn authorization_and_users_router(app_state: Arc<AppState>) -> Router
// {   
//     Router::new()      
//         .route("/auth/login", post(login))
//         .auth_route("/auth/logout",
//          post(close_session),
//          Arc::clone(&app_state),
//             AuthRouteParams::new()
//                 .with_roles(&[Roles::Administrator, Roles::User])
//                 .with_audience(&["http://google.com"]))
//         .route("/test", post(close_session))
//         .route_layer(AuthLayer::new(Arc::clone(&app_state), &["role1", "role2"], &["http://google.com"]))
//         .auth_route("/auth/logout_full", post(close_sessions), Arc::clone(&app_state))
//         .route("/auth/update_tokens", get(update_tokens))
//         .auth_route("/users/create", post(create), Arc::clone(&app_state))
//         .auth_route("/users/update", post(update), Arc::clone(&app_state))
//         .auth_roles_route("/users/update_role", post(update), &[Roles::Administrator], Arc::clone(&app_state))
//         .with_state(app_state)
//         .layer(crate::api::cors_layer())
//         .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)))
// }











// #[derive(Debug, Deserialize, Clone)]
// pub struct LoginPayload
// {
//     pub login: String,
//     pub password: String
// }
// #[derive(Debug, Clone, Serialize)]
// pub struct AuthorizationInfo<R> where R: for<'de> Deserialize<'de> + Serialize + Clone + Send + Sync
// {
//     pub id: String,
//     pub name: String,
//     pub surname_1: String,
//     pub surname_2: String,
//     pub role: R,
//     pub access_key: String,
//     ///дата до которой годен рефреш токен
//     pub expiration_date: String,
//     pub avatar: Option<String>

// }
// fn add_refresh_cookie(cookies: CookieJar, rfr: &str) -> CookieJar
// {
//     let exp = time::Duration::days(6);
//     let offset = time::OffsetDateTime::now_utc().checked_add(exp);
//     let cookie = Cookie::build((REFRESH_KEY_COOKIE, rfr.to_owned()))
//     .path("/")
//     .expires(offset)
//     //.max_age(time::Duration::minutes(120))
//     .same_site(SameSite::Lax)
//     .secure(false)
//     .http_only(true);
//     cookies.add(cookie.build())
// }

// fn update_refresh_cookie(cookies: CookieJar, rfr: &str) -> CookieJar
// {
//     let c = cookies.remove(Cookie::from(REFRESH_KEY_COOKIE));
//     add_refresh_cookie(c, rfr)
// }

// fn remove_refresh_cookie(cookies: CookieJar) -> CookieJar
// {
//     cookies.remove(Cookie::from(REFRESH_KEY_COOKIE))
// }


// //json экстрактор последний!
// pub async fn login(
//     ConnectInfo(addr): ConnectInfo<SocketAddr>,
//     cookies: CookieJar,
//     State(app_state): State<Arc<AppState>>,
//     Json(payload): Json<LoginPayload>) 
// -> impl IntoResponse
// {
//     let ip = addr.ip().to_string();
//     let logged = app_state.services.user_service.login(&payload.login, &payload.password).await;
//     if logged.is_err()
//     {
//         return logged.err().unwrap().into_response()
//     }
//     let logged = logged.unwrap();
//     let session = app_state.services.user_session_service.new_session(logged.get_id(), &ip).await;
//     if session.is_err()
//     {
//         return AppError::DbError(session.err().unwrap()).into_response()
//     }
//     let session = session.unwrap();
//     let role: Roles = serde_json::from_str(&logged.user_role).unwrap();
//     let access_key = app_state.services.jwt_service.gen_key(&logged.id, role.clone());
//     let cookies = add_refresh_cookie(cookies, session.get_id());
//     let authorized = AuthorizationInfo::<Roles>
//     {
//         id: logged.id,
//         name: logged.user_name,
//         surname_1: logged.surname_1,
//         surname_2: logged.surname_2,
//         role,
//         access_key,
//         expiration_date: session.key_expiration_time.to_string(),
//         avatar: logged.avatar
//     };
//     (cookies, Json(authorized)).into_response()
// }


// pub async fn close_session(
//     ConnectInfo(addr): ConnectInfo<SocketAddr>,
//     headers: HeaderMap,
//     cookies: CookieJar,
//     State(app_state): State<Arc<AppState>>)
// -> impl IntoResponse
// {
//     let ip = addr.ip().to_string();
//     let claims = app_state.services.jwt_service.get_claims(headers).await;
//     if claims.is_err()
//     {
//         return claims.err().unwrap().into_response();
//     }
//     let claims = claims.unwrap();
//     let _ = app_state.services.user_session_service.delete_session(claims.user_id(), &ip).await;
//     let cookie = remove_refresh_cookie(cookies);
//     (cookie).into_response()
// }
// pub async fn close_sessions(
//     headers: HeaderMap,
//     cookies: CookieJar,
//     State(app_state): State<Arc<AppState>>)
// -> impl IntoResponse
// {
//     let claims = app_state.services.jwt_service.get_claims(headers).await;
//     if claims.is_err()
//     {
//         return claims.err().unwrap().into_response();
//     }
//     let claims = claims.unwrap();
//     let _ = app_state.services.user_session_service.delete_all_sessions(claims.user_id()).await;
//     let cookie = remove_refresh_cookie(cookies);
//     (cookie).into_response()
// }


// #[derive(Debug, Deserialize, Clone)]
// pub struct TokensPayload
// {
//     pub refresh: String,
//     pub accsess: String
// }
// #[derive(Debug, Deserialize, Clone)]
// pub struct UpdateTokenPayload
// {
//     pub refresh: String
// }

// // pub async fn get_claims<R>(app_state: Arc<AppState>, headers: HeaderMap) -> Result<Claims<R>, AppError>
// // where R: for<'de> Deserialize<'de> + Serialize + PartialEq + Clone
// // {
// //     match headers.get(AUTHORIZATION) 
// //     {
// //         Some(value) => 
// //         {
// //             //let token_str = value.to_str().unwrap_or("")[6..].replace("Bearer ", "");
// //             let token_str = &value.to_str().unwrap_or("")[6..];
// //             logger::info!("Проверка токена->{}", token_str);
// //             let v = jwt.validate_access::<R>(&token_str)?;
// //             Ok(v.claims)
// //         },
// //         None => 
// //         {
// //             Err(AppError::AuthError(authentification::AuthError::AuthorizationError("Отсуствует заголовок Authorization".to_owned())))
// //         }
// //     }
// // }
// // pub async fn verify_token<R>(jwt: Arc<JWT>, headers: HeaderMap) -> Result<(), AppError>
// // where R: for<'de> Deserialize<'de> + Serialize + PartialEq + Clone
// // {
// //     let _ = get_claims::<R>(jwt, headers).await?;
// //     Ok(())
// // }

// pub async fn update_tokens(
//     ConnectInfo(addr): ConnectInfo<SocketAddr>,
//     cookies: CookieJar,
//     State(state): State<Arc<AppState>>)
// -> impl IntoResponse
// {
//     let ip = addr.ip().to_string();
//     if let Some(refresh) = cookies.get(REFRESH_KEY_COOKIE)
//     {
//         logger::debug!("на обновление пришел рефреш: {}", refresh.value());
//         let updated_session = state.services.user_session_service.update_key(refresh.value(), &ip).await;
//         if updated_session.is_err()
//         {
//             return AppError::AuthError(updated_session.err().unwrap()).into_response();
//         }
//         let updated_session = updated_session.unwrap();
//         let user_role: Result<Roles, db_service::DbError> = state.services.user_service.get_role(&updated_session.user_id).await;
//         if user_role.is_err()
//         {
//             return AppError::DbError(user_role.err().unwrap()).into_response();
//         }
//         let user_role = user_role.unwrap();
//         let access_key = state.services.jwt_service.gen_key(&updated_session.user_id, user_role);
//         let cookie = update_refresh_cookie(cookies, &updated_session.id);
//         (cookie, access_key).into_response()
//     }
//     else
//     {
//         AppError::AuthError(authentification::AuthError::UpdateRefreshKeyError("refresh-key необходимый для обновления токенов не найден!".to_owned())).into_response()
//     }
// }

// // pub async fn update_tokens(
// //     ConnectInfo(addr): ConnectInfo<SocketAddr>,
// //     cookies: Cookies,
// //     State(state): State<Arc<AppState>>,
// //     Json(payload): Json<UpdateTokenPayload>)
// // -> Result<Json<UpdateTokens>, AppError>
// // {
// //     if let Some(refresh) = cookies.get("refresh-key")
// //     {

// //     }
// //     else
// //     {
// //         Err(AppError::AuthError(authentification::AuthError::UpdateRefreshKeyError("refresh-key необходимый для обновления токенов не найден!".to_owned())))
// //     }
// //     let updated_session = state.services.user_session_service.update_key(&payload.refresh, &addr.to_string()).await?;
// //     let user_role: Roles = state.services.user_service.get_role(&updated_session.user_id).await?;
// //     let access_key = state.services.jwt_service.gen_key(&updated_session.user_id, user_role);
// //     Ok(Json(UpdateTokens { access: access_key, refresh: updated_session.id }))
// // }

// #[derive(Deserialize, Clone)]
// pub struct CreateUserPayload
// {
//     pub login: String,
//     pub password: String,
//     pub role: Roles,
//     pub name: String,
//     pub surn_1: String,
//     pub surn_2: String,
//     pub avatar: Option<String>,
// }

// pub async fn create(
//     State(state): State<Arc<AppState>>,
//     Json(payload): Json<CreateUserPayload>)
// -> Result<(), AppError>
// {
//     let _ = state.services.user_service.create(&payload.login, &payload.password, &payload.name, &payload.surn_1, &payload.surn_2, payload.role, payload.avatar).await?;
//     Ok(())
// }

// #[derive(Deserialize, Clone)]
// pub struct UpdateUserPayload
// {
//     pub id: String,
//     pub password: String,
//     pub name: String,
//     pub surn_1: String,
//     pub surn_2: String,
//     pub avatar: Option<String>
// }

// pub async fn update(
//     State(state): State<Arc<AppState>>,
//     Json(payload): Json<UpdateUserPayload>)
// -> Result<(), AppError>
// {
//     let _ = state.services.user_service.update(&payload.id, &payload.password, &payload.name, &payload.surn_1, &payload.surn_2, payload.avatar).await?;
//     Ok(())
// }

// #[derive(Deserialize, Clone)]
// pub struct UpdateRoleQuery
// {
//     /// кому надо сменить роль
//     pub id: String,
//     /// на какую роль
//     pub role: Roles
// }

// pub async fn update_role(
//     query: Query<UpdateRoleQuery>,
//     State(state): State<Arc<AppState>>)
// -> Result<(), AppError>
// {
//     let UpdateRoleQuery {id, role}  = query.0;
//     let _ = state.services.user_service.update_role(&id, role).await?;
//     Ok(())
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct UpdateTokens
// {
//     pub access: String,
//     pub refresh: String
// }