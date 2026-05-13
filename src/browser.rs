use std::collections::HashMap;

use crate::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, WebviewArgs, WindowSize, cef_path};
use cosmic::cosmic_config::{self, ConfigGet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Browser {
    pub app_id: crate::WebviewArgs,
    pub window_title: Option<String>,
    pub url: Option<String>,
    pub window_size: Option<crate::WindowSize>,
    pub persistent_profile: Option<bool>,
    pub try_simulate_mobile: Option<bool>,
}

impl Browser {
    pub fn new(app_id: String) -> Browser {
        Browser {
            app_id: crate::WebviewArgs {
                id: app_id.to_string(),
            },
            window_title: None,
            url: None,
            window_size: None,
            persistent_profile: None,
            try_simulate_mobile: None,
        }
    }

    pub fn from_appid(app_id: &str) -> Option<Browser> {
        let Ok(config) = cosmic_config::Config::new(crate::APP_ID, crate::CONFIG_VERSION) else {
            return None;
        };

        let Ok(apps) = config.get::<HashMap<String, crate::AppConfig>>("apps") else {
            return None;
        };

        fn parse_window_width(w: String) -> u32 {
            if let Ok(w) = w.parse::<u32>() {
                w
            } else {
                DEFAULT_WINDOW_WIDTH
            }
        }

        fn parse_window_height(h: String) -> u32 {
            if let Ok(h) = h.parse::<u32>() {
                h
            } else {
                DEFAULT_WINDOW_HEIGHT
            }
        }

        match apps.get(app_id) {
            Some(a) => Some(Browser {
                app_id: WebviewArgs {
                    id: a.id.to_string(),
                },
                window_title: Some(a.title.to_string()),
                url: Some(a.url.to_string()),
                window_size: Some(WindowSize(
                    parse_window_width(a.window_width.to_string()),
                    parse_window_height(a.window_height.to_string()),
                )),
                persistent_profile: Some(a.persistent_profile),
                try_simulate_mobile: Some(a.simulate_mobile),
            }),
            None => None,
        }
    }

    pub fn get_exec(&self) -> Option<String> {
        let Some(cef_path) = cef_path() else {
            eprintln!("CEF cannot be found. Is it not installed?");
            return None;
        };

        Some(format!(
            "env LD_LIBRARY_PATH={} {}.webview {}",
            cef_path.display(),
            crate::APP_ID,
            self.app_id.as_ref()
        ))
    }
}
