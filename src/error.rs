use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error 
{
    #[error(transparent)]
    DeserializeError(#[from] serde_json::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error("Ошибка авторизации: `{0}`")]
    AuthError(String),
	#[error("Время сессии закончилось, необходимо зайти в систему заново")]
	SessionExpired,
    #[error(transparent)]
    AuthServiceError(#[from] auth_service::Error)
}

impl serde::Serialize for Error 
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
	S: serde::ser::Serializer,
	{
		serializer.serialize_str(self.to_string().as_ref())
	}
}