mod structs;

use std::{net::SocketAddr, sync::Arc};
use axum::{body::Body, extract::{ConnectInfo, State}, response::{IntoResponse, Response}, routing::{get, post}, Extension, Json, Router};
use hyper::StatusCode;
use structs::{LoginPayload, PasswordPayload, SessionPayload, UserUpdatePayload};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use crate::{db::{InformationDbo, UserDbo}, middleware::{AuthCheck, FingerprintExtractor, ResponseSessionWrapper, SessionExtension}, state::AppState, Error};
use crate::Role;
use crate::middleware::AuthLayer;



pub fn authorization_router(app_state: Arc<AppState>) -> Router
{   
    Router::new()      
        .route("/auth/login", post(login))

        .route("/auth/update_key", get(update_access)
            .route_layer(AuthLayer::with_roles(
                AuthCheck::Session,
                Arc::clone(&app_state),
                &[Role::User, Role::Administrator])))

        .route("/auth/change_password", post(change_password)
            .route_layer(AuthLayer::with_roles(
                AuthCheck::All,
                Arc::clone(&app_state),
                &[Role::User, Role::Administrator])))
            
        .route("/auth/admin", get(admin_section)
            .route_layer(AuthLayer::with_roles(
                AuthCheck::All,
                Arc::clone(&app_state),
                &[Role::Administrator])))

        .route("/auth/exit", get(exit)
            .route_layer(AuthLayer::with_roles(
                AuthCheck::All,
                Arc::clone(&app_state),
                &[Role::User, Role::Administrator])))

        .route("/auth/exit_from", post(exit_from)
            .route_layer(AuthLayer::with_roles(
                AuthCheck::All,
                Arc::clone(&app_state),
                &[Role::User, Role::Administrator])))

        .route("/auth/exit_all", get(exit_all)
            .route_layer(AuthLayer::with_roles(
                AuthCheck::All,
                Arc::clone(&app_state),
                &[Role::User, Role::Administrator])))
            
        .route("/auth/update_user_info", post(update_user_info)
            .route_layer(AuthLayer::with_roles(
                AuthCheck::All,
                Arc::clone(&app_state),
                &[Role::User, Role::Administrator])))
                
        .route("/auth/update_user", post(update_user_info_by_admin)
            .route_layer(AuthLayer::with_roles(
                AuthCheck::All,
                Arc::clone(&app_state),
                &[Role::Administrator])))

        .with_state(app_state.clone())
        //.layer(crate::api::cors_layer(app_state.clone()))
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)))
}

pub async fn login(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(app_state): State<Arc<AppState>>,
    FingerprintExtractor(fp): FingerprintExtractor,
    Json(payload): Json<LoginPayload>) 
-> Result<impl IntoResponse, Error>
{
    let ip = addr.ip().to_string();
    let user = app_state.get_services().user_service.login(&payload.login, &payload.password, &ip, &fp, &payload.device).await;
    
    if let Ok((user_info, session)) = user
    {
        logger::debug!("Юзер {} прошел авторизацию", &payload.login);
        let session_wrapper = ResponseSessionWrapper::new(Arc::new(session), app_state.configuration.clone());
        Ok((
            StatusCode::OK,
            session_wrapper,
            Json(user_info),
        ))
    }
    else 
    {
        Err(user.err().unwrap())
    }
}
pub async fn change_password(
    State(app_state): State<Arc<AppState>>,
    Extension(session_wrapper): Extension<SessionExtension>,
    Json(payload): Json<PasswordPayload>) 
-> Result<Response<Body>, Error>
{
    let result = app_state
        .services
        .user_service
        .change_password(&session_wrapper.session.user_id, &payload.old_password, &payload.new_password).await?;
    Ok(result.into_response())
}

pub async fn admin_section(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(app_state): State<Arc<AppState>>,
    Extension(session_wrapper): Extension<SessionExtension>) 
-> Result<Response<Body>, Error>
{
    let user = app_state.services.database_service.user_repository.get_user(&session_wrapper.session.user_id).await?;
    Ok((
        StatusCode::OK,
        "вы зашли в админский роут",
    ).into_response())
}
pub async fn exit(
    State(app_state): State<Arc<AppState>>,
    Extension(session_wrapper): Extension<SessionExtension>) 
-> Result<impl IntoResponse, Error>
{
    let result = app_state.services.user_service.exit_from_session(&session_wrapper.session.session_id).await?;
    Ok(result.into_response())
}
pub async fn exit_from(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<SessionPayload>)
-> Result<impl IntoResponse, Error>
{
    let session_uid= payload.session_id.parse::<uuid::Uuid >();
    if let Ok(id) = session_uid
    {
        let result = app_state.services.user_service.exit_from_session(&id).await?;
        Ok(result.into_response())
    }
    else 
    {
        logger::error!("Ошибка парсинга uid {}", &payload.session_id);
        Ok((
            StatusCode::BAD_REQUEST,
            format!("id сессии {} не валиден", payload.session_id),
        ).into_response())
    }
}

pub async fn exit_all(
    State(app_state): State<Arc<AppState>>,
    Extension(session_wrapper): Extension<SessionExtension>)
-> Result<impl IntoResponse, Error>
{
    let result = app_state.services.user_service.exit_from_all_sessions(&session_wrapper.session.session_id).await?;
    Ok(result.into_response())
}

pub async fn update_access(
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    Extension(session_wrapper): Extension<SessionExtension>,
    State(app_state): State<Arc<AppState>>) 
-> Result<impl IntoResponse, Error>
{
    //TODO надо ли проверить ip адрес? не всегда он будет совпадать так как везде почти динамический
    let key = app_state.services.user_service.update_access_key(&session_wrapper.session, &session_wrapper.fingerprint).await?;
    let response_wrapper = ResponseSessionWrapper::new(session_wrapper.session, app_state.configuration.clone());
    Ok((
        StatusCode::OK,
        response_wrapper,
        key,
    ))
}

pub async fn update_user_info(
    State(app_state): State<Arc<AppState>>,
    Extension(session_wrapper): Extension<SessionExtension>,
    Json(payload): Json<UserUpdatePayload>)
-> Result<impl IntoResponse, Error>
{
    let user = app_state.services.database_service.user_repository.get_user(&session_wrapper.session.user_id).await;
    if let Ok(user) = user
    {
        match user.role
        {
            Role::Administrator =>
            {
                let user = UserDbo
                {
                    id: session_wrapper.session.user_id,
                    username: String::new(),
                    password: String::new(),
                    name: payload.name,
                    surname_1: payload.surname_1,
                    surname_2: payload.surname_2,
                    is_active: payload.is_active,
                    avatar: payload.avatar,
                    role: payload.role,
                    audiences: payload.audiences,
                    information: InformationDbo
                    {
                        phones: payload.information.phones,
                        email: payload.information.email
                    }
                };
                let result = app_state.services.user_service.update_user_by_admin(user).await?;
                Ok(result.into_response())
            }
            _ => 
            {
                let user = UserDbo
                {
                    id: session_wrapper.session.user_id,
                    username: String::new(),
                    password: String::new(),
                    name: payload.name,
                    surname_1: payload.surname_1,
                    surname_2: payload.surname_2,
                    is_active: false,
                    avatar: payload.avatar,
                    role: Role::NonPrivileged,
                    audiences: Vec::new(),
                    information: InformationDbo
                    {
                        phones: payload.information.phones,
                        email: payload.information.email
                    }
                };
                let result = app_state.services.user_service.update_user_info(user).await?;
                Ok(result.into_response())
            }
        }
    }
    else 
    {
        let error = user.err().unwrap();
        logger::error!("{}", error.to_string());
        Err(error)    
    }
}

pub async fn update_user_info_by_admin(
    State(app_state): State<Arc<AppState>>,
    Extension(session_wrapper): Extension<SessionExtension>,
    Json(payload): Json<UserUpdatePayload>)
-> Result<impl IntoResponse, Error>
{
    let user = UserDbo
    {
        id: session_wrapper.session.user_id,
        username: String::new(),
        password: String::new(),
        name: payload.name,
        surname_1: payload.surname_1,
        surname_2: payload.surname_2,
        is_active: payload.is_active,
        avatar: payload.avatar,
        role: payload.role,
        audiences: payload.audiences,
        information: InformationDbo
        {
            phones: payload.information.phones,
            email: payload.information.email
        }
    };
    let result = app_state.services.user_service.update_user_by_admin(user).await?;
    Ok(result.into_response())
}


#[cfg(test)]
mod tests
{
    #[tokio::test]
    async fn test_running()
    {
        logger::StructLogger::new_default();
        
    }
}