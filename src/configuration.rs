use {
    config::{Config, ConfigError, Environment, File, FileFormat},
    secrecy::SecretString,
    serde::Deserialize,
    serde_with::serde_as,
    std::env,
};

pub const CONFIG_PATH: &str = "configuration.toml";

#[serde_as]
#[derive(Deserialize, Clone)]
pub struct AppConfig {
    /// Log level for the application layer
    #[serde(default = "default_loglevel")]
    pub log_level: String,

    /// Whether to use JSON logging
    #[serde(default = "default_is_json_logging")]
    pub is_json_logging: bool,

    /// The address to listen on
    #[serde(default = "default_listener")]
    pub listener: String,

    /// Redis uri
    #[serde(default = "default_redis_uri")]
    pub redis_uri: SecretString,

    /// Signing keypair
    pub signing_key: SecretString,
}

fn default_is_json_logging() -> bool {
    true
}

fn default_loglevel() -> String {
    String::from("info")
}

fn default_listener() -> String {
    String::from("0.0.0.0:3000")
}

fn default_redis_uri() -> SecretString {
    SecretString::from("redis://localhost:6379")
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| CONFIG_PATH.to_string());

        let settings = Config::builder()
            .add_source(
                File::with_name(&config_path)
                    .format(FileFormat::Toml)
                    .required(false),
            )
            .add_source(
                Environment::with_prefix("APP")
                    .try_parsing(true)
                    .ignore_empty(true),
            )
            .build()?;

        settings.try_deserialize::<Self>()
    }
}
