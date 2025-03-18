use std::sync::Arc;
use sqlx::{sqlite::SqliteRow, FromRow, Row, SqlitePool};
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
    pub avatar: Option<String>
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

        let obj = UserDbo   
        {
            id: id.parse().unwrap(),
            username,
            password,
            name,
            surname_1,
            surname_2,
            is_active,
            avatar
        };
        Ok(obj)
    }
}
//тут мы просто создаем удаляем юзера, нужен дополнительный слой для сведения логики авторизации
pub trait IUserRepository
{
    fn login(&self, username: &str, password: &str) -> impl std::future::Future<Output = Result<UserDbo, Error>> + Send;
    ///self user info update
    fn update_info(&self, user: UserDbo) -> impl std::future::Future<Output = Result<(), Error>> + Send;
    fn update_password(&self, user_id: &uuid::Uuid, old_password: &str, new_password: &str) -> impl std::future::Future<Output = Result<(), Error>> + Send;
    ///update user info by admin privilegy
    fn update(&self, target: UserDbo) -> impl std::future::Future<Output = Result<(), Error>> + Send;
    fn create(&self, user: UserDbo) -> impl std::future::Future<Output = Result<(), Error>> + Send;
    fn username_is_busy(&self, username: &str) -> impl std::future::Future<Output = Result<(), Error>> + Send;
}

impl IUserRepository for UserRepository
{
    fn login(&self, username: &str, password: &str) -> impl std::future::Future<Output = Result<UserDbo, Error>> + Send
    {
        let connection = Arc::clone(&self.connection);
        Box::pin(async move 
        {
            let sql = ["SELECT * FROM users WHERE username = $1 "].concat();
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
                    Err(error::Error::AuthError(["неверные авторизационные данные ", username].concat()))
                }
            }
            else 
            {
                Err(error::Error::AuthError(["неверные авторизационные данные ", username].concat()))
            }
        })
    }
    fn update_password(&self, user_id: &uuid::Uuid, old_password: &str, new_password: &str) -> impl std::future::Future<Output = Result<(), Error>> + Send
    {
        let connection = Arc::clone(&self.connection);
        Box::pin(async move 
        {
            let sql = ["SELECT * FROM users WHERE id = $1 "].concat();
            let mut user = sqlx::query_as::<_, UserDbo>(&sql)
            .bind(user_id.to_string())
            .fetch_one(&*connection).await;
            if let Ok(user) = user.as_mut()
            {
                let old_pass_and_sailt = utilites::Hasher::hash_from_strings([&user.password, &user.id.to_string()]);
                let check_old_password_and_sailt = utilites::Hasher::hash_from_strings([old_password, &user.id.to_string()]);
                let new_pass_and_sailt = utilites::Hasher::hash_from_strings([new_password, &user.id.to_string()]);
                if &old_pass_and_sailt == &check_old_password_and_sailt
                {
                    let sql = ["UPDATE users SET password = &1 WHERE id = $2 "].concat();
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
            }
            else 
            {
                Err(error::Error::AuthError("Некорректный id юзера ".to_owned()))
            }
        })
    }

    fn update_info(&self, user: UserDbo) -> impl std::future::Future<Output = Result<(), Error>> + Send
    {
        let connection = Arc::clone(&self.connection);
        Box::pin(async move 
        {
            let sql = ["SELECT * FROM users WHERE id = $1 "].concat();
            let mut user_from_db = sqlx::query_as::<_, UserDbo>(&sql)
            .bind(user.id.to_string())
            .fetch_one(&*connection).await;
            if let Ok(user_from_db) = user_from_db.as_mut()
            {
                user_from_db.avatar = user.avatar;
                user_from_db.name = user.name;
                user_from_db.surname_1 = user.surname_1;
                user_from_db.surname_2 = user.surname_2;
                Ok(())
            }
            else 
            {
                Err(error::Error::AuthError(["Ошибка обновления данных для  ", &user.username].concat()))
            }
        })
    }
    fn update(&self, user: UserDbo) -> impl std::future::Future<Output = Result<(), Error>> + Send
    {
        let connection = Arc::clone(&self.connection);
        Box::pin(async move 
        {
            let sql = ["SELECT * FROM users WHERE id = $1 "].concat();
            let mut user_from_db = sqlx::query_as::<_, UserDbo>(&sql)
            .bind(user.id.to_string())
            .fetch_one(&*connection).await;
            if let Ok(user_from_db) = user_from_db.as_mut()
            {
                user_from_db.avatar = user.avatar;
                user_from_db.name = user.name;
                user_from_db.surname_1 = user.surname_1;
                user_from_db.surname_2 = user.surname_2;
                user_from_db.is_active = user.is_active;
                user_from_db.password = user.password;
                Ok(())
            }
            else 
            {
                Err(error::Error::AuthError(["Ошибка обновления данных для  ", &user.username].concat()))
            }
        })
    }

    fn create(&self, user: UserDbo) -> impl std::future::Future<Output = Result<(), Error>> + Send
    {
        let connection = Arc::clone(&self.connection);
        Box::pin(async move 
        {
            let pass_and_sailt = utilites::Hasher::hash_from_strings([&user.password, &user.id.to_string()]);
            let sql = ["INSERT INTO users (id, username, password, name, surname_1, surname_2, avatar) VALUES ($1, $2, $3, $4, $5, $6, $7)"].concat();
            let _ = sqlx::query(&sql)
            .bind(user.id.to_string())
            .bind(&user.username)
            .bind(&pass_and_sailt)
            .bind(&user.name)
            .bind(&user.surname_1)
            .bind(&user.surname_2)
            .bind(&user.avatar)
            .execute(&*connection).await?;
            Ok(())
        })
    }

    fn username_is_busy(&self, username: &str) -> impl std::future::Future<Output = Result<(), Error>> + Send
    {
        let connection = Arc::clone(&self.connection);
        Box::pin(async move 
        {
            let sql = ["SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)"].concat();
            let exists: bool = sqlx::query_scalar(&sql)
            .bind(username)
            .fetch_one(&*connection).await?;
            if exists
            {
                Err(error::Error::AuthError("Это имя пользователя уже занято".to_owned()))
            }
            else 
            {
                Ok(())
            }
        })
    }
}
