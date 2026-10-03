use std::{fs, path::PathBuf};

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
    pub fn profile_path(&self, id: &str) -> Option<PathBuf> {
        match self {
            Installation::System => {
                if let Some(mut dir) = dirs::data_local_dir() {
                    dir.push(APP_ID);
                    dir.push(id);

                    if !dir.exists() {
                        let _ = fs::create_dir_all(&dir);
                    }

                    return Some(dir.into());
                }
            }
            Installation::Flatpak => {
                if let Some(mut dir) = dirs::home_dir() {
                    dir.push(".var");
                    dir.push("app");
                    dir.push(id);
                    dir.push("data");

                    if !dir.exists() {
                        let _ = fs::create_dir_all(&dir);
                    }

                    return Some(dir);
                }
            }
            Installation::Snap => {
                if let Some(mut dir) = dirs::home_dir() {
                    dir.push("snap");
                    dir.push(id);
                    dir.push("common");

                    if !dir.exists() {
                        let _ = fs::create_dir_all(&dir);
                    }

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
    Zen,
}

pub type ArgKey = String;
pub type ArgValue = String;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrowserArg(ArgKey, Option<ArgValue>);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Browser {
    pub display_name: String,
    pub app_id: String,
    pub executable_name: String,
    pub executable_path: Option<PathBuf>,
    pub install_type: Option<Installation>,
    pub browser_type: BrowserT,
    pub browser_args: Vec<BrowserArg>,
}

impl Browser {
    pub fn new(display_name: &str, app_id: &str, exe_name: &str, browser_type: BrowserT) -> Self {
        Self {
            display_name: display_name.to_string(),
            app_id: app_id.to_string(),
            executable_name: exe_name.to_string(),
            executable_path: None,
            install_type: None,
            browser_type,
            browser_args: Vec::new(),
        }
    }

    pub fn update_with_path(&mut self, path: PathBuf) {
        self.install_type = Some(Installation::from(&path));
        self.executable_path = Some(path);

        if let Some(i) = &self.install_type {
            match i {
                Installation::Flatpak => self.display_name.push_str(" (Flatpak)"),
                Installation::Snap => self.display_name.push_str(" (Snap)"),
                _ => {}
            }
        }
    }

    pub fn update_arg(&mut self, arg: BrowserArg) {
        self.browser_args.push(arg);
    }

    pub fn display_args(&self) -> String {
        let mut result_string = String::new();

        for args in self.browser_args.iter() {
            result_string.push_str(&args.0);
            result_string.push_str(" ");

            if let Some(val) = &args.1 {
                result_string.push_str(&val);
                result_string.push_str(" ");
            }
        }

        result_string
    }

    pub fn display_preview_string(&self) -> String {
        let mut preview_string = String::new();

        if let Some(path) = &self.executable_path {
            preview_string.push_str(&path.display().to_string());
        }

        preview_string.push_str(&self.display_args());
        preview_string
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
