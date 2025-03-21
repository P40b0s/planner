use serde::{Deserialize, Serialize};


const FILENAME: &str = "configuration.toml";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configuration
{
    ///session life time in days
    pub session_life_time: u8,
    ///access key lifetime un minutes
    pub access_key_lifetime: u8,
    ///maximum sessions on one user
    pub max_sessions_count: u8,
    ///cookie name with session key
    pub session_cookie_name: String,
    ///every user request update session life time
    pub update_session_time_on_request: bool,
    pub fingerprint_header_name: String,
    pub origins: Vec<String>,
    pub server_port: u16,
}
impl Default for Configuration
{
    fn default() -> Self 
    {
        Self
        {
            session_life_time: 5,
            access_key_lifetime: 5,
            max_sessions_count: 3,
            session_cookie_name: "session-key".to_string(),
            fingerprint_header_name: "x-unique".to_string(),
            update_session_time_on_request: true,
            origins: vec![
                "http://localhost:8888".to_owned()
            ],
            server_port: 8888
        }
    }
}
impl Configuration
{
    pub fn load() -> Self
    {
        let cfg = utilites::deserialize(FILENAME, false, utilites::Serializer::Toml);
        if cfg.is_err()
        {
            logger::error!("Ошибка десериализации настроек, {}, будут установлены настройки по умолчанию", cfg.err().unwrap());
            Self::default()
        }
        else 
        {
            cfg.unwrap()    
        }
    }
    pub fn save(&self)
    {
        let _ = utilites::serialize(&self, FILENAME, false, utilites::Serializer::Toml);
    }
}

#[cfg(test)]
mod tests
{
    #[test]
    fn save_cfg()
    {
        let cfg = super::Configuration::default();
        cfg.save();
    }
}