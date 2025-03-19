mod auth_route;
mod roles;
pub use roles::Roles;
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
