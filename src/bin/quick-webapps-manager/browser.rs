use crate::launcher::installed_webapps;
use serde::{Deserialize, Serialize};
use webapps::WebviewArgs;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Browser {
    pub app_id: String,
    pub webview_args: WebviewArgs,
}

impl Browser {
    pub fn new(window_title: &str, name: &str, url: &str, with_profile: bool) -> Self {
        let profile_path = if with_profile {
            let xdg_data = dirs::data_dir().unwrap_or_default();
            Some(xdg_data.join("quick-webapps/profiles").join(name))
        } else {
            None
        };

        let webview_args = WebviewArgs {
            app_id: name.to_string(),
            window_title: window_title.to_string(),
            url: url.to_string(),
            profile: profile_path,
        };

        Self {
            app_id: name.to_string(),
            webview_args,
        }
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
}
