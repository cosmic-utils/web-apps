use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};
use webapps::{APP_ID, CONFIG_VERSION};

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct AppConfig {
    pub app_theme: String,
}

impl AppConfig {
    pub fn config_handler() -> Option<cosmic_config::Config> {
        cosmic_config::Config::new(APP_ID, CONFIG_VERSION).ok()
    }
    pub fn config() -> AppConfig {
        match Self::config_handler() {
            Some(config_handler) => {
                AppConfig::get_entry(&config_handler).unwrap_or_else(|(errs, config)| {
                    tracing::info!("errors loading config: {:?}", errs);
                    config
                })
            }
            None => AppConfig::default(),
        }
    }
}
