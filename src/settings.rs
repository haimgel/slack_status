use dirs::home_dir;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub status: HashMap<String, super::slack_api::SlackStatus>,
    pub accounts: HashMap<String, String>,
}

impl AppSettings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let mut settings_path = home_dir().unwrap();
        settings_path.push(".slack_status");

        let builder = config::Config::builder()
            .add_source(config::File::from(settings_path))
            .add_source(config::Environment::with_prefix("APP"));

        builder.build()?.try_deserialize()
    }
}
