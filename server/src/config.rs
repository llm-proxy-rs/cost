use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct AppConfig {
    pub cognito_client_id: String,
    pub cognito_client_secret: String,
    pub cognito_domain: String,
    pub cognito_redirect_uri: String,
    pub cognito_region: String,
    pub cognito_user_pool_id: String,
    #[serde(default = "default_database_url_gateway_ro")]
    pub database_url_gateway_ro: String,
    #[serde(default = "default_database_url_cost")]
    pub database_url_cost: String,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_base_path")]
    pub base_path: String,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_database_url_gateway_ro() -> String {
    "postgres://postgres:postgres@localhost/gateway".to_string()
}

fn default_database_url_cost() -> String {
    "postgres://postgres:postgres@localhost/cost".to_string()
}

fn default_base_path() -> String {
    "/".to_string()
}

pub async fn load_config(config_file: &str) -> anyhow::Result<AppConfig> {
    let app_config: AppConfig = Config::builder()
        .add_source(File::with_name(config_file).required(false))
        .add_source(Environment::default())
        .build()?
        .try_deserialize()?;
    Ok(app_config)
}
