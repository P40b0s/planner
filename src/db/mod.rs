mod user_repository;
mod connection;
use std::sync::Arc;

use auth_service::{AuthorizationRepository, IAuthorizationRepository};
pub use user_repository::{UserRepository, IUserRepository};

use crate::Error;
pub struct DatabaseService<UR: IUserRepository, AR: IAuthorizationRepository> 
{
    pub user_repository: UR,
    pub authorization_repository: AR
}
impl DatabaseService<UserRepository, AuthorizationRepository>
{
    pub async fn new(max_sessions_count: u8) -> Result<Self, Error>
    {
        let pool = Arc::new(connection::new_connection("planner").await?);
        //создать все таблицы
        //let r1 = sqlx::query(create_table_sql()).execute(&*pool).await;
        // if r1.is_err()
        // {
        //     logger::error!("{}", r1.as_ref().err().unwrap());
        //     let _ = r1?;
        // };
        Ok(Self
        {
            authorization_repository: AuthorizationRepository::new(max_sessions_count).await?,
            user_repository: UserRepository 
            { 
                connection: pool
            }
        })
    }
}