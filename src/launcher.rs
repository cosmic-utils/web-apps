use ashpd::desktop::{
    dynamic_launcher::{DynamicLauncherProxy, PrepareInstallOptions},
    Icon,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self},
    io::Read,
};
use tokio::fs::remove_file;

use crate::APP_ID;

pub fn webapplauncher_is_valid(
    icon: &str,
    name: &str,
    url: &Option<String>,
    category: &crate::Category,
) -> bool {
    if let Some(url) = url {
        if crate::url_valid(url)
            && !name.is_empty()
            && !icon.is_empty()
            && !url.is_empty()
            && category != &crate::Category::None
        {
            return true;
        }
    }

    false
}

pub fn installed_webapps() -> Vec<WebAppLauncher> {
    let mut webapps = Vec::new();

    if let Some(data_dir) = dirs::data_dir() {
        if let Ok(entries) = fs::read_dir(data_dir.join(APP_ID).join("database")) {
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
    pub async fn create(&self) -> std::io::Result<()> {
        let mut desktop_entry = String::new();

        desktop_entry.push_str("[Desktop Entry]\n");
        desktop_entry.push_str("Version=1.0\n");
        desktop_entry.push_str("Type=Application\n");
        desktop_entry.push_str(&format!("Name={}\n", self.name));
        desktop_entry.push_str(&format!("Comment=Quick WebApp\n",));
        desktop_entry.push_str(&format!("Exec={}\n", self.browser.get_exec()));
        desktop_entry.push_str(&format!("StartupWMClass={}\n", self.browser.app_id.id));
        desktop_entry.push_str(&format!("Categories={}\n", self.category.as_ref()));

        let proxy = DynamicLauncherProxy::new()
            .await
            .expect("Failed to create DynamicLauncherProxy");

        let mut f = std::fs::File::open(&self.icon).expect("no file found");
        let metadata = std::fs::metadata(&self.icon).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");

        let icon = Icon::Bytes(buffer);
        let response = proxy
            .prepare_install(None, &self.name, icon, PrepareInstallOptions::default())
            .await
            .expect("Failed to prepare install")
            .response()
            .expect("Failed to get response");

        let token = response.token();

        println!("{}", desktop_entry);

        proxy
            .install(
                &token,
                &format!("{}.{}.desktop", &APP_ID, self.browser.app_id.id),
                &desktop_entry,
            )
            .await
            .expect("Failed to install");

        Ok(())
    }

    pub async fn delete(&self) -> std::io::Result<()> {
        let proxy = DynamicLauncherProxy::new()
            .await
            .expect("Failed to create DynamicLauncherProxy");

        proxy
            .uninstall(&format!("{}.{}.desktop", &APP_ID, self.browser.app_id.id))
            .await
            .expect("Failed to uninstall");

        if let Some(path) = crate::database_path(&format!("{}.ron", self.browser.app_id.as_ref())) {
            remove_file(path).await?;
        }

        self.browser.delete();

        Ok(())
    }
}
