use std::sync::Arc;

use axum::{response::IntoResponse, Router};
use hyper::StatusCode;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use crate::state::AppState;


pub fn router(app_state: Arc<AppState>) -> Router
{   
    let auth_router = super::authorization::authorization_router(Arc::clone(&app_state));
    Router::new()
        .fallback(handler_404)      
        .with_state(app_state.clone())
        .layer(super::cors::cors_layer(app_state.clone()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        ).merge(auth_router)
}

async fn handler_404() -> impl IntoResponse 
{
    (StatusCode::NOT_FOUND, "Такого пути нет")
}