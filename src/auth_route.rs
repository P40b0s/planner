use std::sync::Arc;
use axum::{extract::{Request, State}, middleware::{self, Next}, response::Response, routing::MethodRouter, Extension, Router};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use utilites::http::{StatusCode, HeaderMap};

async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    ext: Extension<Vec<String>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode>
{
    let roles = ext.0;
    //роли не переданы, доступ имеют все
    let user_claims = state.services.jwt_service.get_claims(headers).await;
    if let Ok(claims) = user_claims
    {
        if roles.is_empty()
        {
            let response = next.run(request).await;
            Ok(response)
        }
        else 
        {
            if let Some(role) = claims.role()
            {
                if roles.contains(role)
                {
                    let response = next.run(request).await;
                    Ok(response)
                }
                else 
                {
                    logger::error!("Отсуствует необходимая роль для доступа к маршруту {}, текущаяя роль: {}, требуемые роли: {:?}", request.uri().to_string(), role, roles);
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
            else 
            {
                logger::error!("Отсуствует роль для доступа к маршруту {}, перечень требуемых ролей: {:?}", request.uri().to_string(), roles);
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }
    else
    {
        logger::error!("{}", user_claims.err().unwrap());
        Err(StatusCode::UNAUTHORIZED)
    }

}

pub trait AuthRoute<S>
where
    S: Clone + Send + Sync + 'static
{
    ///Возможность указать маршрут с авторизацией по выбранным ролям
    fn auth_route(self, path: &str, method_router: MethodRouter<S>, state: Arc<AppState>) -> Self;
    fn auth_roles_route<R: ToString>(self, path: &str, method_router: MethodRouter<S>, roles: &[R], state: Arc<AppState>) -> Self;
}
//TODO add audience in route
impl<S> AuthRoute<S> for Router<S>
where
    S: Clone + Send + Sync + 'static
{
    fn auth_route(self, path: &str, method_router: MethodRouter<S>, state: Arc<AppState>) -> Self 
    {
        self.route(path, method_router
            .route_layer(middleware::from_fn_with_state(state, auth_middleware))
            .route_layer(avalible_for_roles::<String>(&[])))
    }
    fn auth_roles_route<R: ToString>(self, path: &str, method_router: MethodRouter<S>, roles: &[R], state: Arc<AppState>) -> Self 
    {
        self.route(path, method_router
            .route_layer(middleware::from_fn_with_state(state, auth_middleware))
            .route_layer(avalible_for_roles(roles)))
    }
}
fn avalible_for_roles<R: ToString>(roles: &[R]) -> Extension<Vec<String>>
{
    Extension(roles.into_iter().map(|m| m.to_string()).collect::<Vec<String>>())
}