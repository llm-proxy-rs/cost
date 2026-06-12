use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct LegacyEmailMapping {
    pub from: String,
    pub to: String,
}

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
    #[serde(default)]
    pub legacy_email_map: Vec<LegacyEmailMapping>,
    #[serde(default = "default_export_row_cap")]
    pub export_row_cap: usize,
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

fn default_export_row_cap() -> usize {
    templates::DEFAULT_EXPORT_ROW_CAP
}

pub async fn load_config(config_file: &str) -> anyhow::Result<AppConfig> {
    let app_config: AppConfig = Config::builder()
        .add_source(File::with_name(config_file).required(false))
        .add_source(Environment::default())
        .build()?
        .try_deserialize()?;
    Ok(app_config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_row_cap_default_is_1000() {
        assert_eq!(default_export_row_cap(), 1000);
    }

    #[test]
    fn export_row_cap_defaults_when_absent() {
        // Required fields supplied via env; export_row_cap omitted should
        // fall back to the serde default rather than failing to deserialize.
        let cfg: AppConfig = Config::builder()
            .add_source(
                Environment::default().source(Some(std::collections::HashMap::from([
                    ("cognito_client_id".into(), "id".into()),
                    ("cognito_client_secret".into(), "secret".into()),
                    ("cognito_domain".into(), "domain".into()),
                    ("cognito_redirect_uri".into(), "uri".into()),
                    ("cognito_region".into(), "region".into()),
                    ("cognito_user_pool_id".into(), "pool".into()),
                ]))),
            )
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();
        assert_eq!(cfg.export_row_cap, 1000);
    }
}
