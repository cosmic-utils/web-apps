use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::APP_ID;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Installation {
    System,
    Flatpak,
    Snap,
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
pub struct Browser {
    pub id: String,
    pub name: String,
    pub executable_name: String,
    pub executable_path: PathBuf,
    pub install_type: Installation,
}

impl Browser {
    pub fn new(
        id: &str,
        name: &str,
        executable_name: &str,
        executable_path: &str,
        install_type: Installation,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            executable_name: executable_name.to_string(),
            executable_path: PathBuf::from(executable_path),
            install_type,
        }
    }

    pub fn installed_browsers() -> Vec<Self> {
        crate::supported_browsers::supported_browsers()
            .into_iter()
            .filter(|b| b.executable_path.exists())
            .collect()
    }
}
