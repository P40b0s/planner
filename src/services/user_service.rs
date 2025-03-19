use std::sync::Arc;

use auth_service::{AuthorizationRepository, IAuthorizationRepository, JwtService};
use serde::{Deserialize, Serialize};

use crate::{db::{DatabaseService, IUserRepository, UserDbo}, Error, Roles};

pub trait IUserService
{

}

pub struct UserService
{
    database_service: Arc<DatabaseService>,
    jwt_service: JwtService,
    refresh_key_lifetime: i64
}
impl UserService
{
    pub fn new(database_service: Arc<DatabaseService>, jwt_service: JwtService, refresh_key_lifetime: i64) -> Self
    {
        Self
        {
            database_service,
            jwt_service,
            refresh_key_lifetime
        }
    }
    ///Result -> (user_information, refresh_key)
    /// запускаем все это из хэндлера маршрута
    pub async fn login(&self, username: &str, password: &str, ip_addr: &str, fingerprint: &str) -> Result<(UserInformation, uuid::Uuid), Error>
    {
        let user_dbo = self.database_service.user_repository.login(username, password).await?;
        let refresh_key = self.database_service.authorization_repository.create_session(&user_dbo.id, user_dbo.role, self.refresh_key_lifetime, ip_addr, fingerprint, Some(&user_dbo.audiences)).await?;
        let access_key = self.jwt_service.gen_key(&user_dbo.id, user_dbo.role, &user_dbo.audiences).await;
        let mut user: UserInformation = user_dbo.into();
        user.authorization_information.access_key = Some(access_key);
        Ok((user, refresh_key))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInformation
{
    pub id: String,
    pub username: String,
    pub name: String,
    pub surname_1: String,
    pub surname_2: String,
    pub avatar: Option<String>,
    pub information: ExtendedUserInformation,
    pub authorization_information: AuthorizationInformation
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtendedUserInformation
{
    pub phones: Option<Vec<String>>,
    pub email: Option<String>
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthorizationInformation
{
    pub is_active: bool,
    pub role: Roles,
    pub audiences: Vec<String>,
    pub access_key: Option<String>,
}


impl Into<UserInformation> for UserDbo
{
    fn into(self) -> UserInformation 
    {
        UserInformation
        {
            id: self.id.to_string(),
            username: self.username,
            name: self.name,
            surname_1: self.surname_1,
            surname_2: self.surname_2,
            avatar: self.avatar,
            information: ExtendedUserInformation 
            { 
                phones: self.information.phones,
                email: self.information.email
            },
            authorization_information: AuthorizationInformation 
            { 
                is_active: self.is_active,
                role: self.role,
                audiences: self.audiences,
                access_key: None
            }
        }
    }
}