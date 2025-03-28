// use std::sync::Arc;
// use axum::{extract::{Request, State}, middleware::{self, Next}, response::Response, routing::MethodRouter, Extension, Router};
// use serde::{Deserialize, Serialize};
// use crate::state::AppState;
// use utilites::http::{StatusCode, HeaderMap};

// async fn auth_middleware(
//     State(state): State<Arc<AppState>>,
//     //ext: Extension<Vec<String>>,
//     Extension(args): Extension<ArgsExtension>,
//     headers: HeaderMap,
//     request: Request,
//     next: Next,
// ) -> Result<Response, StatusCode>
// {
//     let roles = args.roles;
//     let audience = args.audience;
//     //роли не переданы, доступ имеют все
//     let user_claims = state.services.jwt_service.get_claims(headers).await;
//     if let Ok(claims) = user_claims
//     {
//         if roles.is_empty()
//         {
//             let response = next.run(request).await;
//             Ok(response)
//         }
//         else 
//         {
//             if let Some(role) = claims.role()
//             {
//                 if roles.contains(role)
//                 {
//                     let response = next.run(request).await;
//                     Ok(response)
//                 }
//                 else 
//                 {
//                     logger::error!("Отсуствует необходимая роль для доступа к маршруту {}, текущаяя роль: {}, требуемые роли: {:?}", request.uri().to_string(), role, roles);
//                     Err(StatusCode::UNAUTHORIZED)
//                 }
//             }
//             else 
//             {
//                 logger::error!("Отсуствует роль для доступа к маршруту {}, перечень требуемых ролей: {:?}", request.uri().to_string(), roles);
//                 Err(StatusCode::UNAUTHORIZED)
//             }
//         }
//     }
//     else
//     {
//         logger::error!("{}", user_claims.err().unwrap());
//         Err(StatusCode::UNAUTHORIZED)
//     }

// }

// pub struct AuthRouteParams<'a, R: ToString, A: ToString>
// {
//     roles: Option<&'a [R]>,
//     audience: Option<&'a [A]>,
// }
// impl<'a, R: ToString, A: ToString> AuthRouteParams<'a, R, A>
// {
//     pub fn new() -> Self
//     {
//         Self
//         {
//             roles: None::<&'a [R]>,
//             audience: None::<&'a [A]>,
//         }
//     }
//     pub fn with_roles(mut self, roles: &'a [R]) -> Self
//     {
//         self.roles = Some(roles);
//         self
//     }
//     pub fn with_audience(mut self, audience: &'a [A]) -> Self
//     {
//         self.audience = Some(audience);
//         self
//     }
// }
// #[derive(Debug, Clone)]
// struct ArgsExtension
// {
//     roles: Vec<String>,
//     audience: Vec<String>
// }
// impl<'a, R: ToString, A: ToString> Into<ArgsExtension> for AuthRouteParams<'a, R, A>
// {
//     fn into(self) -> ArgsExtension 
//     {
//         let roles = if let Some(roles) = self.roles
//         {
//             roles.into_iter().map(|v| v.to_string()).collect()
//         }
//         else
//         {
//             Vec::new()
//         };
//         let audience = if let Some(audience) = self.audience
//         {
//             audience.into_iter().map(|v| v.to_string()).collect()
//         }
//         else
//         {
//             Vec::new()
//         };
//         ArgsExtension
//         {
//             roles,
//             audience
//         }
//     }
// }   

// pub trait AuthRoute<S>
// where
//     S: Clone + Send + Sync + 'static
// {
//     ///Возможность указать маршрут с авторизацией по выбранным ролям
//     fn auth_route<'a, R: ToString, A: ToString>(self, path: &str, method_router: MethodRouter<S>, state: Arc<AppState>, args: AuthRouteParams<R, A>) -> Self;

// }
// //TODO add audience in route
// impl<S> AuthRoute<S> for Router<S>
// where
//     S: Clone + Send + Sync + 'static
// {
//     fn auth_route<'a, R: ToString, A: ToString>(self, path: &str, method_router: MethodRouter<S>, state: Arc<AppState>, args: AuthRouteParams<R, A>) -> Self 
//     {
//         self.route(path, method_router
//             .route_layer(middleware::from_fn_with_state(state, auth_middleware)))
//             .route_layer(with_args(args))
//     }
// }
// fn with_args<'a, R: ToString, A: ToString>(args: AuthRouteParams<R, A>) -> Extension<ArgsExtension>
// {
//     Extension(args.into())
// }