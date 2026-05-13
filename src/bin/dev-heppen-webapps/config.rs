use std::collections::HashMap;

use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use webapps::{APP_ID, CONFIG_VERSION};

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct Config {
    pub app_theme: String,
    pub apps: HashMap<String, webapps::AppConfig>,
}

impl Config {
    pub fn config_handler() -> Option<cosmic_config::Config> {
        cosmic_config::Config::new(APP_ID, CONFIG_VERSION).ok()
    }
    pub fn config() -> Config {
        match Self::config_handler() {
            Some(config_handler) => {
                Config::get_entry(&config_handler).unwrap_or_else(|(errs, config)| {
                    tracing::info!("errors loading config: {:?}", errs);
                    config
                })
            }
            None => Config::default(),
        }
    }
}
