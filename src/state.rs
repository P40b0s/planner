use std::sync::Arc;

use authentification::{Claims, JWT};
use db_service::{Operations, QuerySelector, Selector, SqlitePool};
use hyper::{header::AUTHORIZATION, HeaderMap};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{api::Roles, db::{self, PublicationDocumentTypeDbo}, services::{DbServiceInstance, DocumentTypeService, JwtService, PublicationService, SSEService, UserService, UserSessionService}, AppError};

#[derive(Clone)]
pub struct Settings
{
    //pub processing_documets_types : Vec<PublicationDocumentTypeDbo>
}
pub struct Services
{
    /// Сервис базы данных предоставляет только пул соединений
    pub db_service: super::services::DbService,
    ///JWT сервис предоставляет методы для валидации ключа доступа и создания нового ключа доступа
    pub jwt_service: JwtService<Roles>,
    /// Сервис предоставляет доступ к отправке сообщений Server Send Events всем подключенным клиентам
    pub sse_service: SSEService,
    ///  Сервис предоставляет методы для работы с пользователями приложения
    pub user_service: UserService,
    /// Сервис для работы с сессией пользователя работа с рефреш ключом и валидация времени
    pub user_session_service: UserSessionService,
    /// Сервис для работы с документами с сервера опубликования publication.pravo.gov.ru  
    /// хранения их в БД и связывание с новыми редакциями
    pub publication_service: PublicationService,
    /// Сервис для определения типов документов по которым  
    /// в БД будут добавлятся новые опубликованные документы
    pub document_type_service: DocumentTypeService
}

pub struct AppState
{
    pub settings: Mutex<Settings>,
    pub services: Services,
    db_pool: Arc<SqlitePool>,
    jwt: Arc<JWT>,
    pub errors: Mutex<Vec<String>>
}

impl AppState
{
    pub async fn initialize() -> Result<AppState, crate::AppError>
    {
        let db_service = super::services::DbService::new("db").await?;
        let pool = db_service.get_db_pool();
        let sse_service = SSEService::new();
        let jwt_service = JwtService::<Roles>::new();
        let user_service = UserService::new(db_service.get_db_pool()).await;
        let user_session_service = UserSessionService::new(db_service.get_db_pool()).await;
        let document_type_service = DocumentTypeService::new(db_service.get_db_pool()).await;
        let publication_service = PublicationService::new(db_service.get_db_pool()).await;
        let services = Services
        {
            db_service,
            jwt_service,
            sse_service,
            user_service,
            user_session_service,
            publication_service,
            document_type_service
        };
        Ok(Self
        {
            services,
            settings: Mutex::new(Settings
            {
                //processing_documets_types: types
            }),
            db_pool: pool,
            jwt: Arc::new(JWT::new()),
            errors: Mutex::new(Vec::new())
        })
    }
    pub fn get_db_pool(&self) -> Arc<SqlitePool>
    {
        Arc::clone(&self.db_pool)
    }
    pub fn get_jwt(&self) -> Arc<JWT>
    {
        Arc::clone(&self.jwt)
    }
    pub async fn get_settings(&self) -> Settings
    {
        let guard = self.settings.lock().await;
        guard.clone()
    }
}