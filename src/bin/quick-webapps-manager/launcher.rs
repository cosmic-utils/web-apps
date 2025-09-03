use crate::{
    browser::Browser,
    common::{self, database_path, desktop_files_location},
    pages::editor::Category,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self},
    io::Read,
};
use tokio::{
    fs::{remove_dir_all, remove_file, File},
    io::AsyncWriteExt,
};
use tracing::debug;

pub fn webapplauncher_is_valid(icon: &str, name: &str, url: &str) -> bool {
    if !common::url_valid(url) || name.is_empty() || icon.is_empty() || url.is_empty() {
        return false;
    }

    true
}

pub fn installed_webapps() -> Vec<WebAppLauncher> {
    let mut webapps = Vec::new();

    if let Ok(entries) = fs::read_dir(database_path("")) {
        for entry in entries {
            match entry {
                Ok(entry) => {
                    let file = std::fs::File::open(entry.path());
                    let mut content = String::new();

                    if let Ok(mut f) = file {
                        f.read_to_string(&mut content).unwrap();
                        if let Ok(launcher) = ron::from_str::<WebAppLauncher>(&content) {
                            webapps.push(launcher);
                        }
                    }
                }
                Err(e) => tracing::error!("Error reading directory: {}", e),
            }
        }
    }

    webapps
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebAppLauncher {
    pub browser: Browser,
    pub name: String,
    pub icon: String,
    pub category: Category,
    pub isolated_profile: bool,
}

impl WebAppLauncher {
    pub async fn create(&self) -> Result<()> {
        debug!("create {:?}", self);

        let entry_location = desktop_files_location(&self.browser.app_id);
        if entry_location.exists() {
            let _ = std::fs::remove_file(&entry_location);
        }

        let exec = match &self.browser.webview_args.profile {
            Some(profile) => {
                format!(
                    "quick-webapps-webview --window-title \"{}\" --url \"{}\" --profile \"{}\" {}",
                    self.browser.webview_args.window_title,
                    self.browser.webview_args.url,
                    profile.display().to_string(),
                    self.browser.webview_args.app_id
                )
            }
            None => {
                format!(
                    "quick-webapps-webview --window-title \"{}\" --url \"{}\" {}",
                    self.browser.webview_args.window_title,
                    self.browser.webview_args.url,
                    self.browser.webview_args.app_id
                )
            }
        };

        let mut desktop_entry = String::from("[Desktop Entry]\n");
        desktop_entry.push_str(&format!("Name={}\n", self.name));
        desktop_entry.push_str("Comment=Quick Web App\n");
        desktop_entry.push_str(&format!("Exec={}\n", exec));
        desktop_entry.push_str(&format!("Icon={}\n", self.icon));
        desktop_entry.push_str(&format!(
            "StartupWMClass={}\n",
            self.browser.webview_args.app_id
        ));
        desktop_entry.push_str("Terminal=false\n");
        desktop_entry.push_str("Type=Application\n");
        desktop_entry.push_str(&format!("Categories={}\n", self.category.as_ref()));
        desktop_entry.push_str("MimeType=text/html;text/xml;application/xhtml_xml;\n");

        if let Ok(mut f) = File::create(entry_location).await {
            f.write_all(desktop_entry.as_bytes()).await?;
        }

        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        remove_file(desktop_files_location(&self.browser.app_id)).await?;
        if let Some(path) = &self.browser.webview_args.profile {
            remove_dir_all(path).await?;
        }
        remove_file(database_path(&format!("{}.ron", self.browser.app_id))).await?;

        Ok(())
    }
}
