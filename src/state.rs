use std::sync::Arc;
use jwt_authentification::JWT;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::{configuration::Configuration, db::{self, DatabaseService, IUserRepository, UserRepository}, services::{self, JwtService, UserService}};

pub struct Services
{
    /// Сервис базы данных предоставляет только пул соединений
    pub database_service: Arc<crate::db::DatabaseService>,
    ///JWT сервис предоставляет методы для валидации ключа доступа и создания нового ключа доступа
    pub jwt_service: JwtService,
    pub user_service: UserService
    // Сервис предоставляет доступ к отправке сообщений Server Send Events всем подключенным клиентам
    //pub sse_service: SSEService,
}

pub struct AppState
{
    pub configuration: Arc<Configuration>,
    pub services: Services,
    pub errors: Mutex<Vec<String>>
}

impl AppState
{
    pub async fn initialize() -> Result<AppState, crate::Error>
    {
        let cfg = Arc::new(Configuration::load());
        let database_service = Arc::new(super::db::DatabaseService::new(cfg.max_sessions_count).await?);
        let jwt_service = JwtService::new();
        let user_service = UserService::new(database_service.clone(), jwt_service.clone(), cfg.clone());
      
        let services = Services
        {
            database_service,
            jwt_service,
            user_service
        };
        Ok(Self
        {
            services,
            configuration: cfg,
            errors: Mutex::new(Vec::new())
        })
    }
    pub fn get_services(&self) -> &Services
    {
        &self.services
    }
}