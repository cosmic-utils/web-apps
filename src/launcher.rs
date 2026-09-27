use serde::{Deserialize, Serialize};
use std::{io::Read as _, path::PathBuf};
use tokio::fs::remove_file;

use crate::{APP_ID, handle_icon};

pub fn webapplauncher_is_valid(name: &str, url: &Option<String>) -> bool {
    if let Some(url) = url {
        if crate::url_valid(url) && !name.is_empty() && !url.is_empty() {
            return true;
        }
    }

    false
}

pub fn installed_webapps() -> Vec<WebAppLauncher> {
    let mut webapps = Vec::new();

    if let Some(data_dir) = dirs::data_dir() {
        if let Ok(entries) = std::fs::read_dir(data_dir.join(APP_ID).join("database")) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let file = std::fs::File::open(entry.path());
                    let mut content = String::new();

                    if let Ok(mut f) = file {
                        f.read_to_string(&mut content).unwrap();
                        if let Ok(launcher) = ron::from_str::<WebAppLauncher>(&content) {
                            webapps.push(launcher);
                        }
                    }
                }
            }
        }
    }

    webapps
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebappIcon {
    pub path: PathBuf,
    pub buffer: Vec<u8>,
}

impl WebappIcon {
    pub fn to_icon(&self) -> crate::Icon {
        handle_icon(self.path.clone())
    }
}

pub fn webapp_icon_valid(icon: &WebappIcon) -> bool {
    icon.path.exists() && !icon.buffer.is_empty()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebAppLauncher {
    pub browser: crate::browser::Browser,
    pub name: String,
    pub icon: WebappIcon,
    pub category: crate::Category,
}

impl WebAppLauncher {
    pub async fn create(&self) -> anyhow::Result<bool> {
        let mut desktop_entry = String::new();

        let Some(exe) = self.browser.get_exec() else {
            return Ok(false);
        };

        if !webapp_icon_valid(&self.icon) {
            tracing::warn!("icon invalid!");
            return Ok(false);
        };

        desktop_entry.push_str("[Desktop Entry]\n");
        desktop_entry.push_str("Version=1.0\n");
        desktop_entry.push_str("Type=Application\n");
        desktop_entry.push_str(&format!("Name={}\n", self.name));
        desktop_entry.push_str(&format!("Comment=Quick WebApp\n",));
        desktop_entry.push_str(&format!("Exec={}\n", exe));
        desktop_entry.push_str(&format!("StartupWMClass={}\n", self.browser.app_id.id));
        desktop_entry.push_str(&format!("Categories={}\n", self.category.as_ref()));

        tracing::info!("{}", desktop_entry);

        return Ok(true);
    }

    pub async fn delete(&self) -> std::io::Result<()> {
        if let Some(path) = crate::launcher_desktop_entry_path(&self.browser.app_id.id) {
            remove_file(path).await?;
        }

        if let Some(path) = crate::database_path(&format!("{}.ron", &self.browser.app_id.id)) {
            remove_file(path).await?;
        }

        self.browser.delete();

        Ok(())
    }
}
