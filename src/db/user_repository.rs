use std::{pin::Pin, sync::Arc};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Pool, Row, Sqlite, SqlitePool};
use utilites::Date;
use crate::{error, Error, Role};

pub struct UserRepository
{
    pub connection: Arc<SqlitePool>,
}


#[derive(Debug, Clone)]
pub struct ContactDbo
{
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub contact_type: String,
    pub verified: bool,
    pub contact: String,
}

fn create_contacts_table_sql<'a>() -> &'a str
{
    "BEGIN;
    CREATE TABLE IF NOT EXISTS contacts (
    id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    contact_type TEXT NOT NULL,
    verified INTEGER NOT NULL DEFAULT 0,
    contact TEXT NOT NULL,
    PRIMARY KEY(id),
    FOREIGN KEY (user_id)  REFERENCES users (Id) ON DELETE CASCADE
    );
    CREATE INDEX IF NOT EXISTS 'contacts_idx' ON contacts (id, user_id, contact, verified);
    COMMIT;"
}
impl FromRow<'_, SqliteRow> for ContactDbo 
{
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> 
    {
        let id: &str =  row.try_get("id")?;
        let user_id: &str =  row.try_get("user_id")?;
        let contact_type: String =  row.try_get("contact_type")?;
        let verified: bool = row.try_get("verified")?;
        let contact: String = row.try_get("contact")?;
        let obj = ContactDbo   
        {
            id: id.parse().unwrap(),
            user_id: user_id.parse().unwrap(),
            contact_type,
            verified,
            contact
        };
        Ok(obj)
    }
}

#[derive(Debug, Clone)]
pub struct ContactVerificationDbo
{
    contact_id: uuid::Uuid,
    code: u32,
    expiration_time: Date
}
fn create_verification_table_sql<'a>() -> &'a str
{
    "BEGIN;
    CREATE TABLE IF NOT EXISTS contacts_verification (
    contact_id TEXT NOT NULL,
    code INTEGER NOT NULL,
    expiration_time TEXT NOT NULL,
    PRIMARY KEY(contact_id)
    );
    CREATE INDEX IF NOT EXISTS 'users_idx' ON users (contact_id, code, expiration_time);
    COMMIT;"
}
impl FromRow<'_, SqliteRow> for ContactVerificationDbo 
{
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> 
    {
        let contact_id: &str =  row.try_get("contact_id")?;
        let code: u32 =  row.try_get("code")?;
        let expiration_time: &str = row.try_get("expiration_time")?;
        let expiration_time = Date::parse(expiration_time).unwrap();
        let obj = ContactVerificationDbo   
        {
            contact_id: contact_id.parse().unwrap(),
            code,
            expiration_time
        };
        Ok(obj)
    }
}

///юзеры
#[derive(Debug, Clone)]
pub struct UserDbo
{
    pub id: uuid::Uuid,
    pub username: String,
    pub password: String,
    pub is_active: bool,
    pub role: Role,
    pub audiences: Vec<String>,
    pub contacts: Vec<ContactDbo>
}
impl UserDbo
{
    pub fn add_contact(mut self, contact_type: &str, contact: &str) -> Self
    {
        let contact = ContactDbo
        {
            id: uuid::Uuid::now_v7(),
            user_id: self.id.clone(),
            contact_type: contact_type.to_owned(),
            verified: false,
            contact: contact.to_owned()
        };
        self.contacts.push(contact);
        self
    }
}
fn create_users_table_sql<'a>() -> &'a str
{
    "BEGIN;
    CREATE TABLE IF NOT EXISTS users (
    id TEXT NOT NULL,
    username TEXT NOT NULL,
    password TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 0,
    role TEXT NOT NULL,
    audiences BLOB,
    PRIMARY KEY(id)
    );
    CREATE INDEX IF NOT EXISTS 'users_idx' ON users (id, username, is_active, role);
    COMMIT;"
}

impl FromRow<'_, SqliteRow> for UserDbo 
{
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> 
    {
        let id: &str =  row.try_get("id")?;
        let username: String =  row.try_get("username")?;
        let password: String =  row.try_get("password")?;
        let is_active: bool = row.try_get("is_active")?;
        let role: &str = row.try_get("role")?;
        let audiences: &str = row.try_get("audiences")?;
        let audiences: Vec<String> = serde_json::from_str(&audiences).unwrap();
        let obj = UserDbo   
        {
            id: id.parse().unwrap(),
            username,
            password,
            is_active,
            role: role.parse().unwrap(),
            audiences,
            contacts: Vec::new()
        };
        Ok(obj)
    }
}
//тут мы просто создаем удаляем юзера, нужен дополнительный слой для сведения логики авторизации
pub trait IUserRepository
{
    fn login<'a>(&'a self, username: &'a str, password: &'a str) -> Pin<Box<dyn Future<Output = Result<UserDbo, Error>> + Send + 'a>>;
    ///self user info update
    fn update_info<'a>(&'a self, user: UserDbo) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>;
    fn update_password<'a>(&'a self, user_id: &'a uuid::Uuid, old_password: &'a str, new_password: &'a str) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>;
    ///update user info by admin privilegy
    fn update<'a>(&'a self, user: UserDbo) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>;
    fn create<'a>(&'a self, user: UserDbo) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>;
    fn username_is_busy<'a>(&'a self, username: &'a str) -> Pin<Box<dyn Future<Output = Result<bool, Error>> + Send + 'a>>;
    fn get_user<'a>(&'a self, user_id: &'a uuid::Uuid) -> Pin<Box<dyn Future<Output = Result<UserDbo, Error>> + Send + 'a>>;
    fn contact_verification_request<'a>(&'a self, contact_id: &'a uuid::Uuid) -> Pin<Box<dyn Future<Output = Result<u32, Error>> + Send + 'a>>;
    fn contact_verification_accept<'a>(&'a self, contact_id: &'a uuid::Uuid, code: u32) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>;
}

impl IUserRepository for UserRepository
{
    fn login<'a>(&'a self, username: &'a str, password: &'a str) -> Pin<Box<dyn Future<Output = Result<UserDbo, Error>> + Send + 'a>>
    {
        let connection = Arc::clone(&self.connection);
        Box::pin(async move 
        {
            let sql = "SELECT id, username, password, is_active, role, json(audiences) as audiences FROM users WHERE username = $1";
            let user = sqlx::query_as::<_, UserDbo>(&sql)
            .bind(username)
            .fetch_one(&*connection).await;
            if let Ok(user) = user
            {
                let pass_and_sailt = utilites::Hasher::hash_from_strings([password, &user.id.to_string()]);
                if &pass_and_sailt == &user.password
                {
                    let sql = "SELECT id, user_id, contact_type, verified, contact FROM contacts WHERE user_id = $1";
                    let contacts = sqlx::query_as::<_, ContactDbo>(&sql)
                    .bind(user.id.to_string())
                    .fetch_all(&*connection).await?;
                    let user = UserDbo 
                    {  
                        contacts,
                        ..user
                    };
                    Ok(user)
                }
                else
                {
                    Err(error::Error::AuthError(["Ошибка пароля для `", username, "`"].concat()))
                }
            }
            else 
            {
                logger::error!("{}", user.err().unwrap());
                Err(error::Error::AuthError(["Пользователь `", username, "` не зарегистрирован"].concat()))
            }
        })
    }
    fn update_password<'a>(&'a self, user_id: &'a uuid::Uuid, old_password: &'a str, new_password: &'a str) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>
    {
        let connection = Arc::clone(&self.connection);
        Box::pin(async move 
        {
            let sql = "SELECT password FROM users WHERE id = $1";
            let current_password: String = sqlx::query_scalar(&sql)
            .bind(user_id.to_string())
            .fetch_one(&*connection).await?;
            
            let check_old_password_and_sailt = utilites::Hasher::hash_from_strings([old_password, &user_id.to_string()]);
            let new_pass_and_sailt = utilites::Hasher::hash_from_strings([new_password, &user_id.to_string()]);
            if &current_password == &check_old_password_and_sailt
            {
                let sql = "UPDATE users SET password = $1 WHERE id = $2";
                let _ = sqlx::query(&sql)
                .bind(new_pass_and_sailt)
                .bind(user_id.to_string())
                .execute(&*connection).await?;
                Ok(())
            }
            else
            {
                Err(error::Error::AuthError("Неверный старый пароль, попробуйте еще раз".to_owned()))
            }
        })
    }
    ///partialy user itself update (only contacts)
    fn update_info<'a>(&'a self, user: UserDbo) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>
    {
        let connection = Arc::clone(&self.connection);
        Box::pin(async move 
        {
            let sql = "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)";
            let exists: bool = sqlx::query_scalar(&sql)
            .bind(user.id.to_string())
            .fetch_one(&*connection).await?;
            if exists
            {
                let sql = "INSERT OR REPLACE INTO contacts (id, user_id, contact_type, contact) VALUES ($1, $2, $3, $4)";
                let mut tx = connection.begin().await?;
                for c in user.contacts
                {
                    let _ = sqlx::query(&sql)
                    .bind(c.id.to_string())
                    .bind(c.user_id.to_string())
                    .bind(&c.contact_type)
                    .bind(&c.contact)
                    .execute(&mut *tx).await?;
                }
                tx.commit().await;
                Ok(())
            }
            else 
            {
                Err(error::Error::AuthError(["Ошибка обновления данных для  ", &user.username].concat()))
            }
        })
    }
    ///full update user info for admin rights
    fn update<'a>(&'a self, user: UserDbo) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>
    {
        let connection = Arc::clone(&self.connection);
        Box::pin(async move 
        {
            let sql = "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)";
            let exists: bool = sqlx::query_scalar(&sql)
            .bind(user.id.to_string())
            .fetch_one(&*connection).await?;
            if exists
            {
                let sql = "UPDATE users SET is_active = $2, role = $3, audiences = jsonb($4) WHERE id = $1";
                let _ = sqlx::query(&sql)
                .bind(user.id.to_string())
                .bind(user.is_active)
                .bind(user.role.to_string())
                .bind(serde_json::to_string(&user.audiences).unwrap())
                .execute(&*connection).await?;
                let sql = "INSERT OR REPLACE INTO contacts (id, user_id, contact_type, contact) VALUES ($1, $2, $3, $4)";
                let mut tx = connection.begin().await?;
                for c in user.contacts
                {
                    let _ = sqlx::query(&sql)
                    .bind(c.id.to_string())
                    .bind(c.user_id.to_string())
                    .bind(&c.contact_type)
                    .bind(&c.contact)
                    .execute(&mut *tx).await?;
                }
                tx.commit().await;
                Ok(())
            }
            else 
            {
                Err(error::Error::AuthError(["Ошибка обновления данных для  ", &user.username].concat()))
            }
        })
    }
    //is_active set by default - 0;
    fn create<'a>(&'a self, user: UserDbo) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>
    {
        let connection = Arc::clone(&self.connection);
        Box::pin(async move 
        {
            let pass_and_sailt = utilites::Hasher::hash_from_strings([&user.password, &user.id.to_string()]);
            let sql = "INSERT INTO users (id, username, password, role, audiences) VALUES ($1, $2, $3, $4, jsonb($5))";
            let _ = sqlx::query(&sql)
            .bind(user.id.to_string())
            .bind(&user.username)
            .bind(&pass_and_sailt)
            .bind(user.role.to_string())
            .bind(serde_json::to_string(&user.audiences).unwrap())
            .execute(&*connection).await?;
            let sql = "INSERT INTO contacts (id, user_id, contact_type, contact) VALUES ($1, $2, $3, $4)";
            let mut tx = connection.begin().await?;
            for c in user.contacts
            {
                let _ = sqlx::query(&sql)
                .bind(c.id.to_string())
                .bind(c.user_id.to_string())
                .bind(&c.contact_type)
                .bind(&c.contact)
                .execute(&mut *tx).await?;
            }
            tx.commit().await;
            Ok(())
        })
    }

    fn username_is_busy<'a>(&'a self, username: &'a str) -> Pin<Box<dyn Future<Output = Result<bool, Error>> + Send + 'a>>
    {
        let connection = Arc::clone(&self.connection);
        Box::pin(async move 
        {
            let sql = ["SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)"].concat();
            let exists: bool = sqlx::query_scalar(&sql)
            .bind(username)
            .fetch_one(&*connection).await?;
            Ok(exists)
        })
    }
    fn get_user<'a>(&'a self, user_id: &'a uuid::Uuid) -> Pin<Box<dyn Future<Output = Result<UserDbo, Error>> + Send + 'a>>
    {
        let connection = Arc::clone(&self.connection);
        Box::pin(async move 
        {
            let sql = "SELECT id, username, password, is_active, role, json(audiences) as audiences FROM users WHERE id = $1";
            let user = sqlx::query_as::<_, UserDbo>(&sql)
            .bind(user_id.to_string())
            .fetch_one(&*connection).await;
            if let Ok(user) = user
            {
                let sql = "SELECT id, user_id, contact_type, verified, contact FROM contacts WHERE user_id = $1";
                let contacts = sqlx::query_as::<_, ContactDbo>(&sql)
                .bind(user.id.to_string())
                .fetch_all(&*connection).await?;
                let user = UserDbo 
                {  
                    contacts,
                    ..user
                };
               Ok(user)
            }
            else 
            {
                logger::error!("{}", user.err().unwrap());
                Err(error::Error::AuthError(["Пользователь `", &user_id.to_string(), "` не найден"].concat()))
            }
        })
    }

    fn contact_verification_request<'a>(&'a self, contact_id: &'a uuid::Uuid) -> Pin<Box<dyn Future<Output = Result<u32, Error>> + Send + 'a>>
    {
        let connection = Arc::clone(&self.connection);
        Box::pin(async move 
        {
            let mut rand = utilites::SimpleRand::new(666);
            let code = rand.rand_range(1000, 9999);
            let sql = "INSERT INTO contacts_verification (contact_id, code, expiration_time) VALUES ($1, $2, $3)";
            let _ = sqlx::query(&sql)
            .bind(contact_id.to_string())
            .bind(&code)
            .bind(Date::now().add_minutes(10).format(utilites::DateFormat::Serialize))
            .execute(&*connection).await?;
            Ok(code as u32)
        })
    }
    fn contact_verification_accept<'a>(&'a self, contact_id: &'a uuid::Uuid, code: u32) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>
    {
        let connection = Arc::clone(&self.connection);
        Box::pin(async move 
        {
            let sql = "SELECT contact_id, code, expiration_time FROM contacts_verification WHERE contact_id = $1";
            let verify = sqlx::query_as::<_, ContactVerificationDbo>(&sql)
            .bind(contact_id.to_string())
            .fetch_one(&*connection).await;
            if let Ok(v) = verify
            {
                if v.expiration_time < Date::now()
                {
                    Err(Error::VerificationCodeExpired)
                }
                else 
                {
                    if code != v.code
                    {
                        Err(Error::VerificationCodeWrong)
                    }
                    else 
                    {
                        let sql = "DELETE FROM contacts_verification WHERE contact_id = $1";
                        let _ = sqlx::query(&sql)
                        .bind(v.contact_id.to_string())
                        .execute(&*connection).await?;
                        let sql = "UPDATE users SET is_active = 1 WHERE id = (SELECT user_id from contacts WHERE id = $1)";
                        let _ = sqlx::query(&sql)
                        .bind(v.contact_id.to_string())
                        .execute(&*connection).await?;
                        Ok(())
                    }
                }
            }
            else 
            {
                Err(Error::VerificationNotFound)
            }
        })
    }
}


impl UserRepository
{
    pub async fn new(pool: Arc<Pool<Sqlite>>) -> Result<Self, Error>
    {
        let _ = sqlx::query(create_users_table_sql()).execute(&*pool).await?;
        let _ = sqlx::query(create_contacts_table_sql()).execute(&*pool).await?;
        let _ = sqlx::query(create_verification_table_sql()).execute(&*pool).await?;
        Ok(Self
        {
            connection: pool,
        })
    }
}
#[cfg(test)]
mod tests
{
    use std::sync::Arc;

    use crate::{db::{connection, user_repository::{UserDbo}, IUserRepository}, Role};

    
    #[tokio::test]
    async fn test_create_1()
    {
        let pool = Arc::new(connection::new_connection("planner").await.unwrap());
        let repo: Box<dyn IUserRepository + Send + Sync> = Box::new(super::UserRepository::new(pool).await.unwrap());
        let user = UserDbo
        {
            id: "0195ae79-6004-76b2-8dd4-8e94d6e5bddb".parse().unwrap(),
            username: "TestUser1".to_owned(),
            password: "test_password".to_owned(),
            is_active: true,
            role: Role::Administrator,
            audiences: Vec::new(),
            contacts: Vec::new()
        }.add_contact("мобильный телефон", "111-222-333")
        .add_contact("e-mail", "aaa@bbb.ru");
        let username_is_exists = repo.username_is_busy(&user.username).await.unwrap();
        if !username_is_exists
        {
            let _ = repo.create(user).await.unwrap();
        }
    }
    #[tokio::test]
    async fn test_create_2()
    {
        let pool = Arc::new(connection::new_connection("planner").await.unwrap());
        let repo: Box<dyn IUserRepository + Send + Sync> = Box::new(super::UserRepository::new(pool).await.unwrap());
        let user = UserDbo
        {
            id: "0195ae79-dcb1-7943-ba11-99dccc909833".parse().unwrap(),
            username: "TestUser2".to_owned(),
            password: "test_password".to_owned(),
            is_active: true,
            role: Role::Administrator,
            audiences: Vec::new(),
            contacts: Vec::new()
        }.add_contact("мобильный телефон", "999-666-333")
        .add_contact("e-mail", "test@test.ru");
        let username_is_exists = repo.username_is_busy(&user.username).await.unwrap();
        if !username_is_exists
        {
            let _ = repo.create(user).await.unwrap();
        }
    }
    #[tokio::test]
    async fn test_create_3()
    {
        let pool = Arc::new(connection::new_connection("planner").await.unwrap());
        let repo: Box<dyn IUserRepository + Send + Sync> = Box::new(super::UserRepository::new(pool).await.unwrap());
        let user = UserDbo
        {
            id: "0195ae7a-3cda-7b11-aa6b-46992a3e209f".parse().unwrap(),
            username: "TestUser3".to_owned(),
            password: "test_password".to_owned(),
            is_active: true,
            role: Role::Administrator,
            audiences: Vec::new(),
            contacts: Vec::new()
        }.add_contact("мобильный телефон", "000-000-000")
        .add_contact("e-mail", "test222@test.ru");
        let username_is_exists = repo.username_is_busy(&user.username).await.unwrap();
        if !username_is_exists
        {
            let _ = repo.create(user).await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_update()
    {
        let pool = Arc::new(connection::new_connection("planner").await.unwrap());
        let repo: Box<dyn IUserRepository + Send + Sync> = Box::new(super::UserRepository::new(pool).await.unwrap());
        let user = UserDbo
        {
            id: "0195ae79-6004-76b2-8dd4-8e94d6e5bddb".parse().unwrap(),
            username: "TestUser1".to_owned(),
            password: "test_password".to_owned(),
            is_active: true,
            role: Role::User,
            audiences: vec!["www.111.ru".to_owned(), "www.222.ru".to_owned()],
            contacts: Vec::new(),
        }.add_contact("мобильный телефон", "111-222-333")
        .add_contact("e-mail", "111@bbb.ru");
        
        let _ = repo.update(user).await.unwrap();
        
    }
    #[tokio::test]
    async fn test_partialy_update()
    {
        let pool = Arc::new(connection::new_connection("planner").await.unwrap());
        let repo: Box<dyn IUserRepository + Send + Sync> = Box::new(super::UserRepository::new(pool).await.unwrap());
        let user = UserDbo
        {
            id: "0195ae79-dcb1-7943-ba11-99dccc909833".parse().unwrap(),
            username: "TestUser666".to_owned(),
            password: "test_password".to_owned(),
            is_active: true,
            role: Role::Administrator,
            audiences: Vec::new(),
            contacts: Vec::new(),
        }.add_contact("мобильный телефон", "999-666-333")
        .add_contact("e-mail", "abyrvalg@ebb.ru");
        let _ = repo.update_info(user).await.unwrap();
        
    }

    #[tokio::test]
    async fn test_change_password()
    {
        let pool = Arc::new(connection::new_connection("planner").await.unwrap());
        let repo: Box<dyn IUserRepository + Send + Sync> = Box::new(super::UserRepository::new(pool).await.unwrap());
        let user = UserDbo
        {
            id: "0195ae7a-3cda-7b11-aa6b-46992a3e209f".parse().unwrap(),
            username: "TestUser666".to_owned(),
            password: "test_password".to_owned(),
            is_active: true,
            role: Role::User,
            audiences: Vec::new(),
            contacts: Vec::new(),
        }.add_contact("мобильный телефон", "999-666-333")
        .add_contact("e-mail", "abyrvalg@ebb.ru");
        let _ = repo.update_password(&user.id, "test_password", "test_password2").await.unwrap();
        
    }
    #[tokio::test]
    async fn test_login()
    {
        logger::StructLogger::new_default();
        let pool = Arc::new(connection::new_connection("planner").await.unwrap());
        let repo: Box<dyn IUserRepository + Send + Sync> = Box::new(super::UserRepository::new(pool).await.unwrap());
        let user = repo.login("TestUser3", "test_password2").await.unwrap();
        assert_eq!(user.id.to_string(), "0195ae7a-3cda-7b11-aa6b-46992a3e209f");
        
    }
}