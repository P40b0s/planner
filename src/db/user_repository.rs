use std::{pin::Pin, sync::Arc};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Pool, Row, Sqlite, SqlitePool};
use crate::{error, Error};

pub struct UserRepository
{
    pub connection: Arc<SqlitePool>,
}

///юзеры
#[derive(Debug, Clone)]
pub struct UserDbo
{
    pub id: uuid::Uuid,
    pub username: String,
    pub password: String,
    pub name: String,
    pub surname_1: String,
    pub surname_2: String,
    pub is_active: bool,
    pub avatar: Option<String>,
    pub information: InformationDbo
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InformationDbo
{
    pub phones: Option<Vec<String>>,
    pub email: Option<String>
}

fn create_table_sql<'a>() -> &'a str
{
    "BEGIN;
    CREATE TABLE IF NOT EXISTS users (
    id TEXT NOT NULL,
    username TEXT NOT NULL,
    password TEXT NOT NULL,
    name TEXT NOT NULL,
    surname_1 TEXT NOT NULL,
    surname_2 TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 0,
    avatar TEXT,
    information BLOB,
    PRIMARY KEY(id)
    );
    CREATE INDEX IF NOT EXISTS 'users_idx' ON users (id, username, is_active, name);
    COMMIT;"
}

impl FromRow<'_, SqliteRow> for UserDbo 
{
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> 
    {
        let id: &str =  row.try_get("id")?;
        let username: String =  row.try_get("username")?;
        let password: String =  row.try_get("password")?;
        let name: String =  row.try_get("name")?;
        let surname_1: String =  row.try_get("surname_1")?;
        let surname_2: String =  row.try_get("surname_2")?;
        let is_active: bool = row.try_get("is_active")?;
        let avatar: Option<String> = row.try_get("avatar")?;
        let information: &str = row.try_get("information")?;
        let information = serde_json::from_str(&information).unwrap();
        let obj = UserDbo   
        {
            id: id.parse().unwrap(),
            username,
            password,
            name,
            surname_1,
            surname_2,
            is_active,
            avatar,
            information
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
}

impl IUserRepository for UserRepository
{
    fn login<'a>(&'a self, username: &'a str, password: &'a str) -> Pin<Box<dyn Future<Output = Result<UserDbo, Error>> + Send + 'a>>
    {
        let connection = Arc::clone(&self.connection);
        Box::pin(async move 
        {
            let sql = "SELECT id, username, password, name, surname_1, surname_2, is_active, avatar, json(information) as information FROM users WHERE username = $1";
            let user = sqlx::query_as::<_, UserDbo>(&sql)
            .bind(username)
            .fetch_one(&*connection).await;
            if let Ok(user) = user
            {
                let pass_and_sailt = utilites::Hasher::hash_from_strings([password, &user.id.to_string()]);
                if &pass_and_sailt == &user.password
                {
                    Ok(user)
                }
                else
                {
                    Err(error::Error::AuthError(["Ошибка ввода пароля для `", username, "`"].concat()))
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
    ///partialy user itself update
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
                let sql = "UPDATE users SET avatar = $2, name = $3, surname_1 = $4, surname_2 = $5, information = jsonb($6) WHERE id = $1";
                let _ = sqlx::query(&sql)
                .bind(user.id.to_string())
                .bind(user.avatar.as_ref())
                .bind(&user.name)
                .bind(&user.surname_1)
                .bind(&user.surname_2)
                .bind(serde_json::to_string(&user.information).unwrap())
                .execute(&*connection).await?;
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
                let sql = "UPDATE users SET avatar = $2, name = $3, surname_1 = $4, surname_2 = $5, information = jsonb($6), is_active = $7 WHERE id = $1";
                let _ = sqlx::query(&sql)
                .bind(user.id.to_string())
                .bind(user.avatar.as_ref())
                .bind(&user.name)
                .bind(&user.surname_1)
                .bind(&user.surname_2)
                .bind(serde_json::to_string(&user.information).unwrap())
                .bind(user.is_active)
                .execute(&*connection).await?;
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
            let sql = "INSERT INTO users (id, username, password, name, surname_1, surname_2, avatar, information) VALUES ($1, $2, $3, $4, $5, $6, $7, jsonb($8))";
            let _ = sqlx::query(&sql)
            .bind(user.id.to_string())
            .bind(&user.username)
            .bind(&pass_and_sailt)
            .bind(&user.name)
            .bind(&user.surname_1)
            .bind(&user.surname_2)
            .bind(&user.avatar)
            .bind(serde_json::to_string(&user.information).unwrap())
            .execute(&*connection).await?;
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
}


impl UserRepository
{
    pub async fn new(pool: Arc<Pool<Sqlite>>) -> Result<Self, Error>
    {
        let r1 = sqlx::query(create_table_sql()).execute(&*pool).await;
        if r1.is_err()
        {
            logger::error!("{}", r1.as_ref().err().unwrap());
            let _ = r1?;
        };
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

    use crate::db::{connection, user_repository::{InformationDbo, UserDbo}, IUserRepository};

    #[tokio::test]
    async fn test_create()
    {
        let pool = Arc::new(connection::new_connection("planner").await.unwrap());
        let repo: Box<dyn IUserRepository + Send + Sync> = Box::new(super::UserRepository::new(pool).await.unwrap());
        let user = UserDbo
        {
            id: uuid::Uuid::now_v7(),
            username: "TestUser3".to_owned(),
            password: "test_password".to_owned(),
            name: "Тест".to_owned(),
            surname_1: "Тестович".to_owned(),
            surname_2: "Тестов".to_owned(),
            is_active: true,
            avatar: None,
            information: InformationDbo
            {
                phones: Some(vec![
                    "111-444-555".to_owned(),
                    "222-555-999".to_owned()
                ]),
                email: Some("aaa@bbb.ru".to_owned())
            }
        };
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
            id: "0195ae24-b77a-7f42-afb7-cbda6279d455".parse().unwrap(),
            username: "TestUser1".to_owned(),
            password: "test_password".to_owned(),
            name: "Тест".to_owned(),
            surname_1: "Тестович".to_owned(),
            surname_2: "Тестов-Обновленный".to_owned(),
            is_active: true,
            avatar: None,
            information: InformationDbo
            {
                phones: Some(vec![
                    "999-666-666".to_owned(),
                    "111-222-333".to_owned()
                ]),
                email: Some("eva@vae.ru".to_owned())
            }
        };
        let _ = repo.update(user).await.unwrap();
        
    }
    #[tokio::test]
    async fn test_partialy_update()
    {
        let pool = Arc::new(connection::new_connection("planner").await.unwrap());
        let repo: Box<dyn IUserRepository + Send + Sync> = Box::new(super::UserRepository::new(pool).await.unwrap());
        let user = UserDbo
        {
            id: "0195ae2d-bc05-74b0-b0ca-69c7fe70938a".parse().unwrap(),
            username: "TestUser666".to_owned(),
            password: "test_password".to_owned(),
            name: "Тест".to_owned(),
            surname_1: "Тестович".to_owned(),
            surname_2: "Тестов-Обновленный-Частично".to_owned(),
            is_active: true,
            avatar: Some("AVA".to_owned()),
            information: InformationDbo
            {
                phones: Some(vec![
                    "000-000-000".to_owned()
                ]),
                email: Some("valle@omega.ru".to_owned())
            }
        };
        let _ = repo.update_info(user).await.unwrap();
        
    }

    #[tokio::test]
    async fn test_change_password()
    {
        let pool = Arc::new(connection::new_connection("planner").await.unwrap());
        let repo: Box<dyn IUserRepository + Send + Sync> = Box::new(super::UserRepository::new(pool).await.unwrap());
        let user = UserDbo
        {
            id: "0195ae30-07b5-7f62-b9d1-2e4f643031b2".parse().unwrap(),
            username: "TestUser666".to_owned(),
            password: "test_password".to_owned(),
            name: "Тест".to_owned(),
            surname_1: "Тестович".to_owned(),
            surname_2: "Тестов-Обновленный-Частично".to_owned(),
            is_active: true,
            avatar: Some("AVA".to_owned()),
            information: InformationDbo
            {
                phones: Some(vec![
                    "000-000-000".to_owned()
                ]),
                email: Some("valle@omega.ru".to_owned())
            }
        };
        let _ = repo.update_password(&user.id, "test_password", "test_password2").await.unwrap();
        
    }
    #[tokio::test]
    async fn test_login()
    {
        logger::StructLogger::new_default();
        let pool = Arc::new(connection::new_connection("planner").await.unwrap());
        let repo: Box<dyn IUserRepository + Send + Sync> = Box::new(super::UserRepository::new(pool).await.unwrap());
        let user = repo.login("TestUser3", "test_password2").await.unwrap();
        assert_eq!(user.id.to_string(), "0195ae30-07b5-7f62-b9d1-2e4f643031b2");
        
    }
}