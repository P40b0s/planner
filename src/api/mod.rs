mod auth_api;
mod cors;
mod test_api;
mod router;
mod authorization;
mod server;
use std::sync::Arc;
use axum::{extract::FromRequestParts, http::{request::Parts, HeaderValue}, response::{IntoResponseParts, Response, ResponseParts}};
use cors::cors_layer;
use crate::{configuration::Configuration, db::Session};

