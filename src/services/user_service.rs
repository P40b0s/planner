use std::{fmt::Display, sync::Arc};
use axum::response::IntoResponse;
use hyper::StatusCode;
use jwt_authentification::JWT;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::{configuration::Configuration, db::{ContactDbo, DatabaseService, ISessionRepository, Session, SessionRepository, UserDbo}, Error, Role};

use super::JwtService;

pub trait IUserService
{

}

pub struct UserService
{
    database_service: Arc<DatabaseService>,
    jwt_service: JwtService,
    configuration: Arc<Configuration>
}
impl UserService
{
    pub fn new(database_service: Arc<DatabaseService>, jwt_service: JwtService, config: Arc<Configuration>) -> Self
    {
        Self
        {
            database_service,
            jwt_service,
            configuration: config,
        }
    }
    ///Result -> (user_information, refresh_key)
    /// запускаем все это из хэндлера маршрута
    pub async fn login(&self, username: &str, password: &str, ip_addr: &str, fingerprint: &str, device: &str) -> Result<(UserInformation, Session), Error>
    {
        let user_dbo = self.database_service.user_repository.login(username, password).await;
        if let Ok(user) = user_dbo
        {
            let session = self.database_service.session_repository.create_session(&user.id,  self.configuration.session_life_time, ip_addr, fingerprint, device).await;
            if let Ok(s) = session
            {
                let access_key = self.jwt_service.gen_key(&user.id, user.role, &user.audiences, self.configuration.access_key_lifetime).await;
                let mut user: UserInformation = user.into();
                if let Some(auth) = user.authorization_information.as_mut()
                {
                    auth.access_key = Some(access_key);
                }
                Ok((user, s))
            }
            else 
            {
                let error = session.err().unwrap();
                logger::error!("{}", error.to_string());
                Err(error)
            }
        }
        else 
        {
            let error = user_dbo.err().unwrap();
            logger::error!("{}", error.to_string());
            Err(error)
        }
    }

    pub async fn change_password<'a,'s >(&'s self, user_id: &'a uuid::Uuid, old_password: &'a str, new_password: &'a str) -> Result<impl IntoResponse + use<'a>, Error>
    {
        let result = self.database_service.user_repository.update_password(user_id, old_password, new_password).await;
        if let Ok(_) = result
        {
            Ok((
                StatusCode::OK,
                "Пароль успешно изменен"
            ))
        }
        else
        {
            let error= result.err().unwrap();
            logger::error!("{}", error.to_string());
            Err(error)
        }
    }

    pub async fn update_user_info(&self, user: UserInformation) -> Result<impl IntoResponse, Error>
    {
        let user = user.into();
        let result = self.database_service.user_repository.update_info(user).await;
        if let Ok(_) = result
        {
            Ok((
                StatusCode::OK,
                "Данные успешно обновлены"
            ))
        }
        else
        {
            let error= result.err().unwrap();
            logger::error!("{}", error.to_string());
            Err(error)
        }
    }
    pub async fn update_user_by_admin(&self, user: UserInformation) -> Result<impl IntoResponse, Error>
    {
        let user = user.into();
        let result = self.database_service.user_repository.update(user).await;
        if let Ok(_) = result
        {
            Ok((
                StatusCode::OK,
                "Данные успешно обновлены"
            ))
        }
        else
        {
            let error= result.err().unwrap();
            logger::error!("{}", error.to_string());
            Err(error)
        }
    }

    pub async fn exit_from_session(&self, session_id: &uuid::Uuid) -> Result<impl IntoResponse, Error>
    {
        let result = self.database_service.session_repository.delete_session(&session_id).await;
        if result.is_ok()
        {
            Ok((
                StatusCode::OK,
                format!("Вы успешно вышли из сессии {}", session_id),
            ))
        }
        else
        {
            let error = result.err().unwrap();
            logger::error!("{}", error.to_string());
            Err(error)
        }
    }

    pub async fn exit_from_all_sessions(&self, user_id: &uuid::Uuid) -> Result<impl IntoResponse, Error>
    {
       
        let result = self.database_service.session_repository.delete_all_sessions(&user_id).await;
        if result.is_ok()
        {
            Ok((
                StatusCode::OK,
                format!("Сессий успешно удалено: `{}`", result.unwrap()),
            ))
        }
        else
        {
            let error = result.err().unwrap();
            logger::error!("{}", error.to_string());
            Err(error)
        }
    }
    pub async fn update_access_key(&self, session: &Session, fingerprint: &str) -> Result<String, Error>
    {
        if &session.fingerprint != fingerprint
        {
            logger::error!("Ошибка, новый fingerprint {} не совпадает с отпечатком сеcсии {}", fingerprint, &session.fingerprint);
            let _ = self.database_service.session_repository.delete_session(&session.session_id).await;
            return Err(Error::WrongFingerprintError(self.configuration.session_cookie_name.clone()));
        }

        let user = self.database_service.user_repository.get_user(&session.user_id).await;
        if user.is_err()
        {
            let error = user.err().unwrap();
            logger::error!("{}", error.to_string());
            Err(error)
        }
        else
        {
            let user = user.unwrap();
            let new_access = self.jwt_service.gen_key(&user.id, user.role, &user.audiences, self.configuration.access_key_lifetime).await;
            let result = self.database_service.session_repository.update_session_key(&session.session_id, self.configuration.access_key_lifetime).await;
            if result.is_err()
            {
                let error = result.err().unwrap();
                logger::error!("{}", error.to_string());
                Err(error)
            }
            else
            {
                logger::debug!("Обновлен access key `{}` для сессии {}", &new_access, user.id.to_string());
                Ok(new_access)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInformation
{
    pub id: String,
    pub username: String,
    pub contacts: Vec<Contact>,
    pub authorization_information: Option<AuthorizationInformation>
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Contact
{
    pub id: String,
    pub contact_type: String,
    pub contact: String
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthorizationInformation
{
    pub is_active: bool,
    pub role: Role,
    pub audiences: Vec<String>,
    pub access_key: Option<String>,
}
impl Into<Contact> for ContactDbo
{
    fn into(self) -> Contact 
    {
        Contact 
        { 
            id: self.id.to_string(),
            contact_type: self.contact_type,
            contact: self.contact
        }
    }
}
fn to_contact_dbo(contact: Contact, user_id: &uuid::Uuid) -> ContactDbo
{
    ContactDbo 
    { 
        id: contact.id.parse().unwrap(),
        user_id: *user_id,
        contact_type: contact.contact_type,
        verified: false,
        contact: contact.contact
    }
}



impl Into<UserInformation> for UserDbo
{
    fn into(self) -> UserInformation 
    {
        UserInformation
        {
            id: self.id.to_string(),
            username: self.username,
            contacts: self.contacts.into_iter().map(|m| m.into()).collect(),
            authorization_information: Some(AuthorizationInformation 
            { 
                is_active: self.is_active,
                role: self.role,
                audiences: self.audiences,
                access_key: None
            })
        }
    }
}

impl Into<UserDbo> for UserInformation
{
    fn into(self) -> UserDbo 
    {
        let user_id: uuid::Uuid =  self.id.parse().unwrap();
        if let Some(auth) = self.authorization_information
        {
           
            UserDbo
            {
                id: user_id,
                username: self.username,
                contacts: self.contacts.into_iter().map(|m| to_contact_dbo(m, &user_id)).collect(),
                is_active: auth.is_active,
                role: auth.role,
                password: "".to_owned(),
                audiences: auth.audiences
            }
        }
        else 
        {
            UserDbo
            {
                id: user_id,
                username: self.username,
                contacts: self.contacts.into_iter().map(|m| to_contact_dbo(m, &user_id)).collect(),
                is_active: false,
                role: Role::NonPrivileged,
                password: "".to_owned(),
                audiences: Vec::new()
            }
        }
       
    }
}

pub fn log_result<T, E: IntoResponse + Display>(result: Result<T, E>) -> Result<T, E>
{
    if result.is_err()
    {
        logger::error!("{}", result.as_ref().err().as_ref().unwrap().to_string());
        Err(result.err().unwrap())
    }
    else 
    {
        result
    }
}