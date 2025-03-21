mod auth_route;
mod roles;
mod configuration;
pub use roles::Role;
mod api;
mod middleware;
mod state;
mod error;
mod services;
pub use error::Error;
mod db;
fn main() {
    println!("Hello, world!");
}
