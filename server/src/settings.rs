use actix_multipart::form;
use config::{Config, ConfigError, File};
use serde::Deserialize;


#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct FrontendSettings {
    pub debug: bool,
    pub ssl: bool,
    pub host: String,
    pub port: usize,
}




#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Database {
    pub connect_type: String,
    pub host: String,
    pub port: u16,
    pub options: String,
}

impl Database {
    pub fn get_uri(&self) -> String {
        format!("{}://{}:{}/?{}", self.connect_type, self.host, self.port, self.options)
    }
}


#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct DiscordAuth {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub port: usize,
}





#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct BackendSettings {
    pub debug: bool,
    pub ssl: bool,
    pub host: String,
    pub port: usize,
    pub uri: String,	
}




impl BackendSettings {

    pub fn get_host(&self) -> String {
        format!("{}://{}:{}", if self.ssl { "https" } else { "http" }, self.uri, self.port)
    }

    pub fn get_ws_host(&self) -> String {
        format!("{}://{}:{}", if self.ssl { "wss" } else { "ws" }, self.host, self.port)
    }
    

}



#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub frontend_settings: FrontendSettings,
    pub database: Database,
    pub discord_auth: DiscordAuth,
    pub backend_settings: BackendSettings,
}


impl Settings {
    pub  fn new() -> Result<Self, ConfigError> {
        let source_file = File::with_name("../Settings");
        let s = Config::builder().add_source(source_file).build() .expect("Failed to load configuration");
        s.try_deserialize()
    }

    pub fn get_discord_auth_uri(&self) -> String {
        format!("{}://{}:{}", if self.backend_settings.ssl { "https" } else { "http" }, self.discord_auth.redirect_uri, self.discord_auth.port)
    }

}