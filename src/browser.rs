use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{APP_ID, supported_browsers::supported_browsers};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Installation {
    System,
    Flatpak,
    Snap,
}

impl From<&PathBuf> for Installation {
    fn from(value: &PathBuf) -> Self {
        if value.starts_with("/usr/bin") || value.starts_with("/usr/local/bin") {
            return Installation::System;
        }

        if value.starts_with("/snap/bin") {
            return Installation::Snap;
        }

        if value.starts_with("/var/lib/flatpak") || value.starts_with("/home") {
            return Installation::Flatpak;
        }

        Installation::System
    }
}

impl Installation {
    pub fn profile_path(
        &self,
        browser_id: &str,
        browser_exe: &str,
        webapp_id: &str,
    ) -> Option<PathBuf> {
        match self {
            Installation::System => {
                if let Some(mut dir) = dirs::data_local_dir() {
                    dir.push(APP_ID);
                    dir.push(browser_id);
                    dir.push(webapp_id);
                    return Some(dir.into());
                }
            }
            Installation::Flatpak => {
                if let Some(mut dir) = dirs::home_dir() {
                    dir.push(".var");
                    dir.push("app");
                    dir.push(browser_id);
                    dir.push("data");
                    dir.push(webapp_id);
                    return Some(dir);
                }
            }
            Installation::Snap => {
                if let Some(mut dir) = dirs::home_dir() {
                    dir.push("snap");
                    dir.push(browser_exe);
                    dir.push("common");
                    dir.push(webapp_id);
                    return Some(dir);
                }
            }
        }

        None
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum BrowserT {
    Chromium,
    Epiphany,
    Falkon,
    Firefox,
    Floorp,
    MsEdge,
    Zen,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrowserConfig {
    pub class_name: String,
    pub isolated_profile: bool,
    pub private_mode: bool,
    pub profile_path: Option<String>,
    pub custom_parameters: String,
    pub url: String,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        BrowserConfig {
            class_name: String::new(),
            isolated_profile: true,
            private_mode: false,
            profile_path: None,
            custom_parameters: String::new(),
            url: String::new(),
        }
    }
}

impl BrowserConfig {
    pub fn set_profile_path(&mut self, path_str: Option<String>) {
        self.profile_path = path_str;
    }

    pub fn set_custom_parameters(&mut self, params: &str) {
        self.custom_parameters = params.to_string();
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Browser {
    pub display_name: String,
    pub app_id: String,
    pub executable_name: String,
    pub executable_path: Option<PathBuf>,
    pub install_t: Option<Installation>,
    pub browser_t: BrowserT,
    pub config: BrowserConfig,
}

impl Browser {
    pub fn new(display_name: &str, app_id: &str, exe_name: &str, browser_t: BrowserT) -> Self {
        Self {
            display_name: display_name.to_string(),
            app_id: app_id.to_string(),
            executable_name: exe_name.to_string(),
            executable_path: None,
            install_t: None,
            browser_t,
            config: BrowserConfig::default(),
        }
    }

    pub fn update_with_path(&mut self, path: PathBuf) {
        self.install_t = Some(Installation::from(&path));
        self.executable_path = Some(path);

        if let Some(i) = &self.install_t {
            match i {
                Installation::Flatpak => self.display_name.push_str(" (Flatpak)"),
                Installation::Snap => self.display_name.push_str(" (Snap)"),
                _ => {}
            }
        }
    }

    fn starting_args(&self) -> String {
        let mut starting_args = String::new();

        match self.browser_t {
            BrowserT::Chromium | BrowserT::MsEdge => {
                starting_args.push_str("--app");
            }
            BrowserT::Epiphany => {
                starting_args.push_str("--application-mode");
            }
            BrowserT::Falkon | BrowserT::Firefox | BrowserT::Floorp | BrowserT::Zen => {
                starting_args.push_str("--no-remote");
            }
        }

        starting_args
    }

    pub fn profile_path_arg(&self) -> String {
        let mut arg = String::new();

        if let Some(path) = &self.config.profile_path {
            if self.config.isolated_profile {
                match &self.browser_t {
                    BrowserT::Chromium | BrowserT::MsEdge => {
                        arg.push_str(&format!("--user-data-dir={}", path))
                    }
                    BrowserT::Epiphany => arg.push_str(&format!("--profile={}", path)),
                    BrowserT::Falkon => arg.push_str(&format!("--profile={}", path)),
                    BrowserT::Firefox | BrowserT::Floorp | BrowserT::Zen => {
                        arg.push_str(&format!("--profile={}", path))
                    }
                }
            }
        }

        arg
    }

    pub fn private_mode_arg(&self) -> String {
        let mut arg = String::new();

        if self.config.private_mode {
            match &self.browser_t {
                BrowserT::Chromium => arg.push_str("--incognito"),
                BrowserT::Falkon => arg.push_str("--private-browsing"),
                BrowserT::Firefox | BrowserT::Floorp | BrowserT::Zen => {
                    arg.push_str("--private-window")
                }
                BrowserT::MsEdge => arg.push_str("--inprivate"),
                _ => {}
            }
        }

        arg
    }

    pub fn class_name_arg(&self) -> Option<String> {
        if self.config.class_name.is_empty() {
            return None;
        };

        let mut arg = String::new();

        match &self.browser_t {
            BrowserT::Falkon => arg.push_str(&format!("--wmclass={}", &self.config.class_name)),
            _ => {
                arg.push_str(&format!("--class={}", &self.config.class_name));
                arg.push_str(" ");
                arg.push_str(&format!("--name={}", &self.config.class_name));
            }
        }

        Some(arg)
    }

    pub fn get_exec_string(&self) -> String {
        let mut exec = String::new();

        if let Some(path) = &self.executable_path {
            if let Some(path_str) = path.to_str() {
                exec.push_str(path_str);
                exec.push_str(" ");
            }
        }

        exec.push_str(&self.starting_args());
        exec.push_str(" ");
        if let Some(class) = self.class_name_arg().as_ref() {
            exec.push_str(class);
            exec.push_str(" ");
        }
        exec.push_str(&self.private_mode_arg());
        exec.push_str(" ");
        exec.push_str(&self.profile_path_arg());
        exec.push_str(" ");
        exec.push_str(&self.config.custom_parameters);
        exec.push_str(" ");

        exec.push_str(&self.config.url);

        exec
    }
}

pub fn common_install_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("/usr/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/snap/bin"),
        PathBuf::from("/var/lib/flatpak/exports/bin"),
    ];

    if let Some(mut data_dir) = dirs::data_dir() {
        data_dir.push("flatpak");
        data_dir.push("exports");
        data_dir.push("bin");
        paths.push(data_dir);
    }

    paths
}

pub fn installed_browsers() -> Vec<Browser> {
    let mut installed: Vec<Browser> = vec![];

    for browser in &mut supported_browsers() {
        for path in &common_install_paths() {
            let final_path = path.join(&browser.executable_name);

            if final_path.exists() {
                browser.update_with_path(final_path);
                installed.push(browser.clone());
            }
        }
    }

    installed
}
