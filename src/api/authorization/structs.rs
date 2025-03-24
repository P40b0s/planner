use serde::{Deserialize, Serialize};

use crate::Role;



#[derive(Debug, Deserialize, Clone)]
pub struct LoginPayload
{
    pub login: String,
    pub password: String,
    pub device: String
}
#[derive(Debug, Deserialize, Clone)]
pub struct PasswordPayload
{
    pub old_password: String,
    pub new_password: String
}
#[derive(Debug, Deserialize, Clone)]
pub struct SessionPayload
{
    pub session_id: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UserInformationPayload
{
    pub phones: Option<Vec<String>>,
    pub email: Option<String>
}

#[derive(Deserialize, Debug, Clone)]
pub struct UserUpdatePayload
{
    pub name: String,
    pub surname_1: String,
    pub surname_2: String,
    pub is_active: bool,
    pub avatar: Option<String>,
    pub role: Role,
    pub audiences: Vec<String>,
    pub information: UserInformationPayload
}


#[derive(Debug, Clone, Serialize)]
pub struct AuthorizationInfo<R> where R: ToString + Serialize
{
    pub id: String,
    pub name: String,
    pub surname_1: String,
    pub surname_2: String,
    pub role: R,
    pub access_key: String,
    ///дата до которой годен рефреш токен
    pub expiration_date: String,
    pub avatar: Option<String>
}
