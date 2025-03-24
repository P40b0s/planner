use std::sync::Arc;
use sqlx::{query::Query, sqlite::{SqliteArguments, SqliteRow}, FromRow, Row, Sqlite, SqlitePool};
use utilites::Date;
use crate::error::Error;

#[derive(Clone)]
pub struct SessionRepository
{
    connection: Arc<SqlitePool>,
    max_sessions_count: u8
}

impl SessionRepository
{
    pub async fn new(max_sessions_count: u8) -> Result<Self, Error>
    {
        let pool = Arc::new(super::connection::new_connection("sessions").await?);
        let r1 = sqlx::query(create_table_sql()).execute(&*pool).await;
        if r1.is_err()
        {
            logger::error!("{}", r1.as_ref().err().unwrap());
            let _ = r1?;
        };
        Ok(Self
        {
            connection: pool,
            max_sessions_count
        })
    }
}
pub trait ISessionRepository
{
    fn create_session(&self, user_id: &uuid::Uuid, refresh_key_lifetime_days: u8, ip_addr: &str, fingerprint: &str, device: &str) -> impl std::future::Future<Output = Result<Session, Error>> + Send;
    fn get_session(&self, session_id: &uuid::Uuid) -> impl std::future::Future<Output = Result<Session, Error>> + Send;
    fn insert_or_replace_session(&self, session: &SessionDbo) -> impl std::future::Future<Output = Result<(), Error>> + Send;
    fn sessions_count(&self, user_id: &uuid::Uuid) -> impl std::future::Future<Output = Result<u32, Error>> + Send;
    fn delete_all_sessions(&self, user_id: &uuid::Uuid) -> impl std::future::Future<Output = Result<u64, Error>> + Send;
    fn delete_session(&self, session_id: &uuid::Uuid) -> impl std::future::Future<Output = Result<(), Error>> + Send;
    fn update_session_key(&self, session_id: &uuid::Uuid, refresh_key_lifetime_days: u8) -> impl std::future::Future<Output = Result<(), Error>>;
}

fn create_table_sql<'a>() -> &'a str
{
    "BEGIN;
    CREATE TABLE IF NOT EXISTS sessions (
    session_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    logged_in TEXT NOT NULL,
    key_expiration_time TEXT NOT NULL,
    ip_addr TEXT NOT NULL,
    fingerprint TEXT,
    device TEXT NOT NULL DEFAULT 'unknown',
    PRIMARY KEY(user_id, session_id)
    );
    CREATE INDEX IF NOT EXISTS 'session_idx' ON sessions (user_id, session_id);
    COMMIT;"
}

enum SessionTable
{
    SessionId,
    UserId,
    LoggedIn,
    KeyExpirationTime,
    IpAddr,
    Fingerprint,
    Device
}

impl SessionTable
{
    pub fn get_all() -> String
    {
        [
            SessionTable::SessionId.as_ref(), ",", 
            SessionTable::UserId.as_ref(), ",", 
            SessionTable::LoggedIn.as_ref(), ",", 
            SessionTable::KeyExpirationTime.as_ref(), ",", 
            SessionTable::IpAddr.as_ref(), ",", 
            SessionTable::Fingerprint.as_ref(), ",", 
            SessionTable::Device.as_ref()
        ].concat()
    }
}

impl AsRef<str> for SessionTable
{
    fn as_ref(&self) -> &str 
    {
        match self
        {
            SessionTable::SessionId => "session_id",
            SessionTable::UserId => "user_id",
            SessionTable::LoggedIn => "logged_in",
            SessionTable::KeyExpirationTime => "key_expiration_time",
            SessionTable::IpAddr => "ip_addr",
            SessionTable::Fingerprint => "fingerprint",
            SessionTable::Device => "device"
        }
    }
}

#[derive(Debug)]
pub struct SessionDbo 
{
   
    pub session_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub logged_in: Date,
    pub key_expiration_time: Date,
    pub ip_addr: String,
    pub fingerprint: String,
    pub device: String
}

impl SessionDbo
{
    fn bind_all<'a>(&'a self, sql: &'a str) -> Query<'a, Sqlite, SqliteArguments<'a>>
    {
        sqlx::query(&sql)
        .bind(self.session_id.to_string())
        .bind(self.user_id.to_string())
        .bind(self.logged_in.to_string())
        .bind(self.key_expiration_time.to_string())
        .bind(&self.ip_addr)
        .bind(&self.fingerprint)
        .bind(&self.device)
    }
}

#[derive(Debug, Clone)]
pub struct Session
{
   
    pub session_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub logged_in: Date,
    pub key_expiration_time: Date,
    pub ip_addr: String,
    pub fingerprint: String,
    pub device: String
}
impl Session
{
    pub fn is_expired(&self) -> bool
    {
        self.key_expiration_time <= Date::now()
    }
}
impl Into<Session> for SessionDbo
{
    fn into(self) -> Session 
    {
        Session 
        { 
           
            session_id: self.session_id,
            user_id: self.user_id,
            logged_in: self.logged_in,
            key_expiration_time: self.key_expiration_time,
            ip_addr: self.ip_addr,
            fingerprint: self.fingerprint,
            device: self.device
        }
    }
}
impl Into<SessionDbo> for Session
{
    fn into(self) -> SessionDbo 
    {
        SessionDbo 
        { 
            session_id: self.session_id,
            user_id: self.user_id,
            logged_in: self.logged_in,
            key_expiration_time: self.key_expiration_time,
            ip_addr: self.ip_addr,
            fingerprint: self.fingerprint,
            device: self.device
        }
    }
}

impl FromRow<'_, SqliteRow> for SessionDbo 
{
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> 
    {
      
        let session_id: &str =  row.try_get(SessionTable::SessionId.as_ref())?;
        let user_id: &str =  row.try_get(SessionTable::UserId.as_ref())?;
        let logged_in: &str =  row.try_get(SessionTable::LoggedIn.as_ref())?;
        let key_expiration_time: &str = row.try_get(SessionTable::KeyExpirationTime.as_ref())?;
        let ip_addr: &str = row.try_get(SessionTable::IpAddr.as_ref())?;
        let fingerprint: String = row.try_get(SessionTable::Fingerprint.as_ref())?;
        let device: String = row.try_get(SessionTable::Device.as_ref())?;
        let obj = SessionDbo   
        {
            
            session_id: session_id.parse().unwrap(),
            user_id: user_id.parse().unwrap(),
            logged_in: Date::parse(logged_in).unwrap(),
            key_expiration_time: Date::parse(key_expiration_time).unwrap(),
            ip_addr: ip_addr.to_owned(),
            fingerprint,
            device
        };
        Ok(obj)
    }
}

impl ISessionRepository for SessionRepository
{
    fn create_session(&self, user_id: &uuid::Uuid, refresh_key_lifetime_days: u8, ip_addr: &str, fingerprint: &str, device: &str) -> impl std::future::Future<Output = Result<Session, Error>> + Send
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = ["SELECT ", &SessionTable::get_all(), " FROM sessions WHERE ", SessionTable::UserId.as_ref(), " = $1 ORDER BY ", SessionTable::LoggedIn.as_ref()].concat();
            let mut current_sessions = sqlx::query_as::<_, SessionDbo>(&sql)
            .bind(user_id.to_string())
            .fetch_all(&*connection).await?;
            //sessions for current user not exists
            if current_sessions.is_empty()
            {
                let session = new_session(user_id,  refresh_key_lifetime_days, ip_addr, fingerprint, device);
                let _ = self.insert_or_replace_session(&session).await?;
                Ok(session.into())
            }
            //sessions count bigger than 3, replace older session with updated session
            else if current_sessions.len() > self.max_sessions_count as usize
            {
                let old_session = current_sessions.swap_remove(0);
                //if fingerprint equalis
                if let Some(mut session) = current_sessions.into_iter().find(|f|f.fingerprint == fingerprint)
                {
                    session.ip_addr = ip_addr.to_owned();
                    session.logged_in = Date::now();
                    session.key_expiration_time = Date::now().add_minutes(get_key_update_in_days(refresh_key_lifetime_days));
                    let _ = self.insert_or_replace_session(&session).await?;
                    Ok(session.into())
                }
                else 
                {
                    self.delete_session(&old_session.session_id).await?;
                    let session = new_session(user_id, refresh_key_lifetime_days, ip_addr, fingerprint, device);
                    let _ = self.insert_or_replace_session(&session).await?;
                    logger::warn!("Превышено максимальное количество одновременных сессий `{}` сессия `{}` заменена на {}", self.max_sessions_count, &old_session.session_id.to_string(), &session.session_id.to_string());
                    Ok(session.into())
                }
            }
            else 
            {
                //sessions with equalis fingerprint is found, update session and return new keys
                if let Some(mut session) = current_sessions.into_iter().find(|f|f.fingerprint == fingerprint)
                {
                    session.ip_addr = ip_addr.to_owned();
                    session.logged_in = Date::now();
                    session.key_expiration_time = Date::now().add_minutes(get_key_update_in_days(refresh_key_lifetime_days));
                    let _ = self.insert_or_replace_session(&session).await?;
                    Ok(session.into())
                }
                //add new session for this user
                else 
                {
                    let session = new_session(user_id, refresh_key_lifetime_days, ip_addr, fingerprint, device);
                    let _ = self.insert_or_replace_session(&session).await?;
                    Ok(session.into())
                }
            }
        })
    }
    //update current session lifetime
    fn update_session_key(&self, session_id: &uuid::Uuid, refresh_key_lifetime_days: u8) -> impl std::future::Future<Output = Result<(), Error>>
    {
        Box::pin(async move 
        {
            let mut session = self.get_session(session_id).await?;
            if session.key_expiration_time > Date::now()
            {
                session.key_expiration_time = Date::now().add_minutes(get_key_update_in_days(refresh_key_lifetime_days));
                self.insert_or_replace_session(&session.into()).await?;
                Ok(())
            }
            else 
            {
                Err(Error::SessionExpired)
            }
        })
    }
    fn get_session(&self, session_id: &uuid::Uuid) -> impl std::future::Future<Output = Result<Session, Error>> + Send
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = ["SELECT ", &SessionTable::get_all(), " FROM sessions WHERE ", SessionTable::SessionId.as_ref(), " = $1"].concat();
            let  current_session = sqlx::query_as::<_, SessionDbo>(&sql)
            .bind(session_id.to_string())
            .fetch_one(&*connection).await;
            if let Ok(session) = current_session
            {
                Ok(session.into())
            }
            else 
            {
                Err(Error::SessionNotFound)
            }
            
        })
        
    }
    fn insert_or_replace_session(&self, session: &SessionDbo) -> impl std::future::Future<Output = Result<(), Error>> + Send
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = ["INSERT OR REPLACE INTO sessions (", &SessionTable::get_all(), ") VALUES ($1, $2, $3, $4, $5, $6, $7)"].concat();
            let _ = session.bind_all(&sql)
            .execute(&*connection).await?;
            Ok(())
        })
    }

    fn sessions_count(&self, user_id: &uuid::Uuid) -> impl std::future::Future<Output = Result<u32, Error>> + Send
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = ["SELECT COUNT(*) FROM sessions WHERE ", SessionTable::UserId.as_ref(), " = $1"].concat();
            let count: u32 = sqlx::query_scalar(&sql)
            .bind(user_id.to_string())
            .fetch_one(&*connection).await?;
            Ok(count)
        })
    }
    fn delete_all_sessions(&self, user_id: &uuid::Uuid) -> impl std::future::Future<Output = Result<u64, Error>> + Send
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = ["DELETE FROM sessions WHERE ", SessionTable::UserId.as_ref(), " = $1"].concat();
            let count = sqlx::query(&sql)
            .bind(user_id.to_string())
            .execute(&*connection).await?;
            let count = count.rows_affected();
            logger::info!("Для `{}` удалено `{}` сессий", user_id.to_string(), count);
            Ok(count)
        })
    }
    fn delete_session(&self, session_id: &uuid::Uuid) -> impl std::future::Future<Output = Result<(), Error>> + Send
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = ["DELETE FROM sessions WHERE ", SessionTable::SessionId.as_ref(), " = $1"].concat();
            let _ = sqlx::query(&sql)
            .bind(session_id.to_string())
            .execute(&*connection).await?;
            logger::info!("Удалена сессия `{}`", session_id.to_string());
            Ok(())
        })
    }
}

fn new_session(user_id: &uuid::Uuid, refresh_key_lifetime_days: u8, ip_addr: &str, fingerprint: &str, device: &str) -> SessionDbo
{
    
    SessionDbo
    {
        session_id: uuid::Uuid::now_v7(),
        user_id: user_id.clone(),
        logged_in: Date::now(),
        key_expiration_time: Date::now().add_minutes(get_key_update_in_days(refresh_key_lifetime_days)),
        ip_addr: ip_addr.to_owned(),
        fingerprint: fingerprint.to_owned(),
        device: device.to_owned()
    }
}

fn get_key_update_in_days(key_lifetime: u8) -> i64
{
    (key_lifetime as i64)*60*24
}