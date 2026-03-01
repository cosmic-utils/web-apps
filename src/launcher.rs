use ashpd::desktop::{
    Icon,
    dynamic_launcher::{
        DynamicLauncherProxy, InstallOptions, PrepareInstallOptions, UninstallOptions,
    },
};
use serde::{Deserialize, Serialize};
use std::{io::Read as _, path::PathBuf};
use tokio::{fs::remove_file, io::AsyncReadExt};

use crate::APP_ID;

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
pub struct WebAppLauncher {
    pub browser: crate::browser::Browser,
    pub name: String,
    pub icon: String,
    pub category: crate::Category,
}

impl WebAppLauncher {
    pub async fn create(&self) -> anyhow::Result<bool> {
        let mut desktop_entry = String::new();

        let Some(exe) = self.browser.get_exec() else {
            return Ok(false);
        };

        let icon_path = PathBuf::from(&self.icon);

        if !icon_path.exists() {
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

        let proxy = DynamicLauncherProxy::new()
            .await
            .expect("Failed to create DynamicLauncherProxy");

        if let Ok(mut file) = tokio::fs::File::open(&self.icon).await {
            let mut buff = Vec::new();

            if let Ok(_size) = file.read_to_end(&mut buff).await {
                let icon = Icon::Bytes(buff);

                let prepare_opts = PrepareInstallOptions::default().set_editable_icon(true);

                let response = proxy
                    .prepare_install(None, &self.name, icon, prepare_opts)
                    .await
                    .expect("Failed to prepare install")
                    .response()
                    .expect("Failed to get response");

                let token = response.token();

                tracing::info!("{}", desktop_entry);

                proxy
                    .install(
                        &token,
                        &format!("{}.{}.desktop", &APP_ID, self.browser.app_id.id),
                        &desktop_entry,
                        InstallOptions::default(),
                    )
                    .await
                    .expect("installing");

                return Ok(true);
            }

            return Ok(false);
        }

        return Ok(false);
    }

    pub async fn delete(&self) -> std::io::Result<()> {
        let proxy = DynamicLauncherProxy::new()
            .await
            .expect("Failed to create DynamicLauncherProxy");

        proxy
            .uninstall(
                &format!("{}.{}.desktop", &APP_ID, self.browser.app_id.id,),
                UninstallOptions::default(),
            )
            .await
            .expect("Failed to uninstall");

        if let Some(path) = crate::database_path(&format!("{}.ron", self.browser.app_id.as_ref())) {
            remove_file(path).await?;
        }

        self.browser.delete();

        Ok(())
    }
}
