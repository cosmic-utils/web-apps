use crate::launcher::installed_webapps;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Browser {
    pub app_id: String,
    pub url: String,
    pub with_profile: bool,
    pub args: webapps::WebviewArgsBuilder,
}

impl Browser {
    pub fn new(app_id: &str, window_title: &str, url: &str, with_profile: bool) -> Self {
        let app_id = format!("{}.{}", webapps::WEBVIEW_APP_ID, app_id);

        let mut args = webapps::WebviewArgsBuilder::new(
            app_id.to_string(),
            window_title.to_string(),
            url.to_string(),
        );

        if with_profile {
            let xdg_data = dirs::data_dir().unwrap_or_default();
            let path = xdg_data.join("quick-webapps/profiles").join(&app_id);
            args.profile(&path);
        };

        Self {
            app_id,
            url: url.to_string(),
            with_profile,
            args,
        }
    }

    pub fn set_window_size(&mut self, window_size: &webapps::WindowSize) {
        self.args.window_size(window_size.clone());
    }

    pub fn set_window_decorations(&mut self, window_decorations: bool) {
        self.args.window_decorations(window_decorations);
    }

    pub fn set_private_mode(&mut self, private_mode: bool) {
        self.args.set_incognito(private_mode);
    }

    pub fn set_simulate_mobile(&mut self, simulate_mobile: bool) {
        self.args.try_simulate_mobile(simulate_mobile);
    }

    pub(crate) fn from_appid(id: &str) -> Option<Self> {
        if let Some(launcher) = installed_webapps()
            .iter()
            .find(|launcher| launcher.browser.app_id == id)
        {
            return Some(launcher.browser.clone());
        };

        None
    }

    pub(crate) fn delete(&self) {
        if self.with_profile {
            let xdg_data = dirs::data_dir().unwrap_or_default();
            let path = xdg_data.join("quick-webapps/profiles").join(&self.app_id);
            if let Err(e) = std::fs::remove_dir_all(&path) {
                eprintln!("Failed to delete profile directory: {}", e);
            }
        }
    }
}
