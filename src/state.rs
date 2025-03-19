use std::sync::Arc;
use auth_service::{AuthorizationRepository, IAuthorizationRepository};
//use authentification::{Claims, JWT};
//use db_service::{Operations, QuerySelector, Selector, SqlitePool};
//use hyper::{header::AUTHORIZATION, HeaderMap};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::{db::{self, DatabaseService, IUserRepository, UserRepository}, services::{self, UserService}};

#[derive(Clone)]
pub struct Settings
{
    //pub processing_documets_types : Vec<PublicationDocumentTypeDbo>
}
pub struct Services
{
    /// Сервис базы данных предоставляет только пул соединений
    pub database_service: Arc<crate::db::DatabaseService>,
    ///JWT сервис предоставляет методы для валидации ключа доступа и создания нового ключа доступа
    pub jwt_service: auth_service::JwtService,
    pub user_service: UserService
    // Сервис предоставляет доступ к отправке сообщений Server Send Events всем подключенным клиентам
    //pub sse_service: SSEService,
}

pub struct AppState
{
    pub settings: Mutex<Settings>,
    pub services: Services,
    pub errors: Mutex<Vec<String>>
}

impl AppState
{
    pub async fn initialize() -> Result<AppState, crate::Error>
    {
        //TODO перенести сессии и время жизни ключа в настройки!
        let database_service = Arc::new(super::db::DatabaseService::new(3).await?);
        let jwt_service = auth_service::JwtService::new();
        let user_service = UserService::new(database_service.clone(), jwt_service.clone(), 5);
        //let pool = db_service.get_db_pool();
        //let sse_service = SSEService::new();
        //let jwt_service = JwtService::<Roles>::new();
        //let user_service = UserService::new(db_service.get_db_pool()).await;
        //let user_session_service = UserSessionService::new(db_service.get_db_pool()).await;
        //let document_type_service = DocumentTypeService::new(db_service.get_db_pool()).await;
        //let publication_service = PublicationService::new(db_service.get_db_pool()).await;
        let services = Services
        {
            database_service,
            jwt_service,
            user_service
        };
        Ok(Self
        {
            services,
            settings: Mutex::new(Settings
            {
                //processing_documets_types: types
            }),
            errors: Mutex::new(Vec::new())
        })
    }
    pub fn get_services(&self) -> &Services
    {
        &self.services
    }
    pub async fn get_settings(&self) -> Settings
    {
        let guard = self.settings.lock().await;
        guard.clone()
    }
}