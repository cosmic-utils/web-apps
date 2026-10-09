use std::{
    fs::{self, create_dir_all},
    io::Read,
    path::PathBuf,
};

use crate::{APP_ID, WebappIcon, browser::Browser, desktop_files};

pub fn webapplauncher_is_valid(name: &str, url: &Option<String>) -> bool {
    if let Some(url) = url {
        if crate::url_valid(url) && !name.is_empty() && !url.is_empty() {
            return true;
        }
    }

    false
}

pub fn installed_webapps() -> Vec<WebappLauncher> {
    let mut webapps = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&desktop_files()) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if file_name.starts_with(APP_ID) {
                        let file = std::fs::File::open(entry.path());
                        let mut content = String::new();

                        if let Ok(mut f) = file {
                            let _ = f.read_to_string(&mut content);
                            webapps.push(WebappLauncher::from_str(&content));
                        }
                    }
                }
            }
        }
    }

    webapps
}

#[derive(Debug)]
pub struct WebappLauncher {
    pub browser: Option<Browser>,
    pub webapp_id: String,
    pub webapp_name: String,
    pub webapp_icon: Option<WebappIcon>,
    pub category: String,
}

impl WebappLauncher {
    pub fn from_str(content: &str) -> Self {
        let mut launcher = Self {
            browser: None,
            webapp_id: String::new(),
            webapp_name: String::new(),
            webapp_icon: None,
            category: String::new(),
        };

        for line in content.lines() {
            let line_split = line.split('=').collect::<Vec<&str>>();

            if line_split.len() < 2 {
                continue;
            }

            let key = line_split[0];
            let value = line_split[1];

            // fill browser config
            if let Some(browser) = launcher.browser.as_mut() {
                match key {
                    "StartupWMClass" => browser.config.class_name = value.to_string(),
                    "X-WebApp-Isolated" => {
                        browser.config.isolated_profile = match value {
                            "true" => true,
                            "false" => false,
                            _ => true,
                        }
                    }
                    "X-WebApp-PrivateMode" => {
                        browser.config.private_mode = match value {
                            "true" => true,
                            "false" => false,
                            _ => false,
                        }
                    }
                    "X-WebApp-ProfilePath" => browser.config.profile_path = Some(value.to_string()),
                    "X-WebApp-CustomParameters" => {
                        browser.config.custom_parameters = value.to_string()
                    }
                    "X-WebApp-URL" => browser.config.url = value.to_string(),
                    _ => {
                        continue;
                    }
                }
            }

            match key {
                "X-WebApp-Browser-Id" => launcher.browser = Browser::from_id(value),
                "X-WebApp-Id" => launcher.webapp_id = value.to_string(),
                "Name" => launcher.webapp_name = value.to_string(),
                "Icon" => launcher.webapp_icon = Some(WebappIcon::build_from_path(value)),
                "Categories" => launcher.category = value.to_string(),
                _ => {
                    continue;
                }
            }
        }

        launcher
    }

    pub fn create_desktop_entry(&self) -> anyhow::Result<bool> {
        let mut desktop_entry = String::new();

        let Some(browser) = &self.browser else {
            return Ok(false);
        };

        let exec = browser.get_exec_string();

        let Some(webapp_icon) = &self.webapp_icon else {
            return Ok(false);
        };

        let Some(icon_path) = &webapp_icon.source_path else {
            return Ok(false);
        };

        desktop_entry.push_str("[Desktop Entry]\n");
        desktop_entry.push_str("Version=1.0\n");
        desktop_entry.push_str("Type=Application\n");
        desktop_entry.push_str(&format!("Name={}\n", self.webapp_name));
        desktop_entry.push_str(&format!("Comment=Quick WebApp\n",));
        desktop_entry.push_str(&format!("Exec={}\n", exec));
        desktop_entry.push_str(&format!("Icon={}\n", icon_path.display()));
        desktop_entry.push_str(&format!("StartupWMClass={}\n", browser.config.class_name));
        desktop_entry.push_str(&format!("Categories={}\n", self.category));
        desktop_entry.push_str(&format!("X-WebApp-Id={}\n", self.webapp_id));
        desktop_entry.push_str(&format!("X-WebApp-Browser={}\n", browser.display_name));
        desktop_entry.push_str(&format!("X-WebApp-Browser-Id={}\n", browser.app_id));
        desktop_entry.push_str(&format!("X-WebApp-URL={}\n", browser.config.url));
        desktop_entry.push_str(&format!(
            "X-WebApp-CustomParameters={}\n",
            browser.config.custom_parameters
        ));
        desktop_entry.push_str(&format!(
            "X-WebApp-PrivateMode={}\n",
            browser.config.private_mode
        ));
        desktop_entry.push_str(&format!(
            "X-WebApp-Isolated={}\n",
            browser.config.isolated_profile
        ));
        desktop_entry.push_str(&format!(
            "X-WebApp-ProfilePath={}\n",
            browser
                .config
                .profile_path
                .clone()
                .unwrap_or_else(|| String::new())
        ));

        tracing::info!("{}", desktop_entry);

        if !PathBuf::from(icon_path).exists() {
            let _ = match webapp_icon.icon {
                crate::IconType::Raster => fs::write(icon_path, webapp_icon.buffer.clone()),
                crate::IconType::Svg => fs::write(
                    icon_path,
                    String::from_utf8_lossy_owned(webapp_icon.buffer.clone()),
                ),
            };
        }

        if let Some(path) = crate::launcher_desktop_entry_path(&self.webapp_id) {
            let _ = fs::write(path, &desktop_entry);
        }

        if let Some(profile) = &browser.config.profile_path {
            let path = PathBuf::from(profile);

            if !path.exists() {
                let _ = create_dir_all(path);
            }
        }

        Ok(true)
    }

    pub fn delete(&self, webapp_id: &str) -> bool {
        if let Some(path) = crate::launcher_desktop_entry_path(&webapp_id) {
            if path.exists() {
                let _ = fs::remove_file(path);
            }
        }

        let Some(browser) = &self.browser else {
            return false;
        };

        if let Some(profile) = &browser.config.profile_path {
            let path = PathBuf::from(profile);

            if path.exists() {
                let _ = fs::remove_dir(profile);
            }
        }

        true
    }
}
