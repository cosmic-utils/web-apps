use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::cef_path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Browser {
    pub app_id: crate::WebviewArgs,
    pub window_title: Option<String>,
    pub url: Option<String>,
    pub profile: Option<PathBuf>,
    pub window_size: Option<crate::WindowSize>,
    pub try_simulate_mobile: Option<bool>,
}

impl Browser {
    pub fn new(app_id: &str) -> Self {
        let mut browser = Self {
            app_id: crate::WebviewArgs {
                id: app_id.to_string(),
                url: None,
            },
            window_title: None,
            url: None,
            profile: None,
            window_size: None,
            try_simulate_mobile: None,
        };

        let xdg_data = dirs::data_dir().unwrap_or_default();
        let path = xdg_data.join(crate::APP_ID).join("profiles").join(&app_id);
        browser.profile = Some(path);

        browser
    }

    pub fn from_appid(id: &str) -> Option<Self> {
        if let Some(launcher) = crate::launcher::installed_webapps()
            .iter()
            .find(|launcher| launcher.browser.app_id.as_ref() == id)
        {
            return Some(launcher.browser.clone());
        };

        None
    }

    pub fn get_exec(&self) -> Option<String> {
        let Some(cef_path) = cef_path() else {
            return None;
        };

        Some(format!(
            "env LD_LIBRARY_PATH={} {}.webview {}",
            cef_path.display(),
            crate::APP_ID,
            self.app_id.as_ref()
        ))
    }

    pub fn delete(&self) {
        if self.profile.is_some() {
            let xdg_data = dirs::data_dir().unwrap_or_default();
            let path = xdg_data
                .join(crate::APP_ID)
                .join("profiles")
                .join(self.app_id.as_ref());
            if let Err(e) = std::fs::remove_dir_all(&path) {
                eprintln!("Failed to delete profile directory: {}", e);
            }
        }
    }
}
