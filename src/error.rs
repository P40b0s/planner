use axum::response::{IntoResponse, Response};
use hyper::{HeaderMap, StatusCode};
use jwt_authentification::{Cookie, CookieJar, Duration as CookieMaxLife};
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
    #[error("Пользователь с такими данными не найден")]
	UserNotFound,
    #[error("Сессия не найдена")]
	SessionNotFound,
    #[error("Ваш код для верификации был просрочен, попробуйте еще раз")]
	VerificationCodeExpired,
    #[error("Неверный код верификации, попробуйте еще раз")]
	VerificationCodeWrong,
    #[error("Запись верификации контакта не обнаружена, попробуйте запросить верификацию повторно")]
	VerificationNotFound,
    #[error(transparent)]
    JwtError(#[from] jwt_authentification::JwtError),
    #[error("Отпечаток сессии не совпадает, сессия будет удалена, необходимо зайти заново")]
    WrongFingerprintError(String),
    #[error("Уникальный идетификатор клиента не найден или имеет неверный формат")]
    FingerprintNotFound
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

impl IntoResponse for Error
{
    fn into_response(self) -> axum::response::Response 
    {
        let message = self.to_string();
        match self
        {
            Error::WrongFingerprintError(cookie_name) =>
            {
                cookie_remove_error_response(&message, &cookie_name)
            }
            Error::SqlxError(e) => 
            {
                let body = "Ошибка базы данных";
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            },
            Error::VerificationCodeExpired =>
            {
                let body = self.to_string();
                (StatusCode::REQUEST_TIMEOUT, body).into_response()
            }
            _ => 
            {
                let body = self.to_string();
                (StatusCode::BAD_REQUEST, body).into_response()
            }
        }
    }
}


pub fn cookie_remove_error_response<T: ToString>(body: T, cookie_name: &str) -> Response<axum::body::Body>
{
    
    let error  = body.to_string();
    let body = axum::body::Body::new(error);
    let cookie: Cookie = Cookie::build((cookie_name, "")).path("/").max_age(CookieMaxLife::seconds(0)).into();
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::SET_COOKIE,
        cookie.to_string().parse().unwrap(),
    );
    let mut resp = Response::builder()
    .status(StatusCode::UNAUTHORIZED)
    .body(body)
    .unwrap();
    *resp.headers_mut() = headers;
    resp
}
