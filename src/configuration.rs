use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DataBaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DataBaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DataBaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        let exposed = format!(
            "postgres://{username}:{password}@{host}:{port}/{db_name}",
            username = self.username,
            password = self.password.expose_secret(),
            host = self.host,
            port = self.port,
            db_name = self.database_name
        );
        Secret::new(exposed)
    }

    pub fn connection_string_wihtout_db(&self) -> Secret<String> {
        let exposed = format!(
            "postgres://{username}:{password}@{host}:{port}",
            username = self.username,
            password = self.password.expose_secret(),
            host = self.host,
            port = self.port
        );
        Secret::new(exposed)
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new(
            "configuration.yaml",
            config::FileFormat::Yaml,
        ))
        .build()?;
    settings.try_deserialize()
}
