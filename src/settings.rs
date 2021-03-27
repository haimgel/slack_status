use std::collections::HashMap;
use dirs::home_dir;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub status: HashMap<String, super::slack_api::SlackStatus>,
    pub accounts: HashMap<String, String>,
}

impl AppSettings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let mut settings_path = home_dir().unwrap();
        settings_path.push(".slack_status");
        let mut cfg = config::Config::default();
        cfg.merge(config::File::from(settings_path))?
            .merge(config::Environment::with_prefix("APP"))?;
        cfg.try_into()
    }
}

