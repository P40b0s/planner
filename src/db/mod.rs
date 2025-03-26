mod user_repository;
mod connection;
mod session_repository;
pub use session_repository::{Session, SessionRepository, ISessionRepository};
use std::sync::Arc;
pub use user_repository::{UserRepository, IUserRepository, UserDbo, ContactDbo, ContactVerificationDbo};

use crate::Error;
pub struct DatabaseService
{
    pub user_repository: Box<dyn IUserRepository + Sync + Send>,
    pub session_repository: SessionRepository
}
impl DatabaseService
{
    pub async fn new(max_sessions_count: u8) -> Result<Self, Error>
    {
        let pool = Arc::new(connection::new_connection("planner").await?);
        let user_repository = UserRepository::new(pool.clone()).await?;
        let session_repository = session_repository::SessionRepository::new(max_sessions_count).await?;
        Ok(Self
        {
            user_repository: Box::new(user_repository),
            session_repository: session_repository
        })
    }
}