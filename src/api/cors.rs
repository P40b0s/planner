use axum::{http::{HeaderName, HeaderValue, Method}, response::Response};//http::{Request, Response, Method, header};
use axum::body::Body;
use axum::http::header::{ACCEPT, ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS, AUTHORIZATION, CONTENT_TYPE, ORIGIN, SET_COOKIE};
use tower::{ServiceBuilder, ServiceExt, Service};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use std::{convert::Infallible, sync::Arc};

use crate::state::AppState;

pub fn cors_layer(state: Arc<AppState>) -> CorsLayer
{
    // let origins = [
    //     "http://193.109.69.132".parse().unwrap(),
    //     "http://localhost:9090".parse().unwrap(),
    //     "http://xarman.space".parse().unwrap()
    // ];
    let origins: Vec<HeaderValue> = state.configuration.origins.iter().map(|v| v.parse().unwrap()).collect();
    let fingerprint_header_name: HeaderName = state.configuration.fingerprint_header_name.parse().unwrap();
    let cors_layer = CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::PUT, Method::HEAD, Method::PATCH])
            .allow_headers([ORIGIN, ACCEPT, CONTENT_TYPE, ACCESS_CONTROL_ALLOW_HEADERS, AUTHORIZATION, fingerprint_header_name])
            .allow_credentials(true);
        //"Access-Control-Allow-Headers", "Access-Control-Allow-Headers, Origin,Accept, X-Requested-With, Content-Type, Access-Control-Request-Method, Access-Control-Request-Headers"
            //.allow_headers(vec![AUTHORIZATION, ACCEPT]);
    return cors_layer;
}