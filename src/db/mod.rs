mod user_repository;
mod connection;
use std::sync::Arc;

use auth_service::{AuthorizationRepository, IAuthorizationRepository};
pub use user_repository::{UserRepository, IUserRepository, UserDbo};

use crate::Error;
pub struct DatabaseService
{
    pub user_repository: Box<dyn IUserRepository + Sync + Send>,
    pub authorization_repository: AuthorizationRepository
}
impl DatabaseService
{
    pub async fn new(max_sessions_count: u8) -> Result<Self, Error>
    {
        let pool = Arc::new(connection::new_connection("planner").await?);
        let user_repository = UserRepository::new(pool.clone()).await?;
        Ok(Self
        {
            authorization_repository: AuthorizationRepository::new(max_sessions_count).await?,
            user_repository: Box::new(user_repository) 
        })
    }
}