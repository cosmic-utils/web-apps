use std::fs;

use serde::{Deserialize, Serialize};

use crate::{APP_ID, browser::Browser};

pub fn webapplauncher_is_valid(name: &str, url: &Option<String>) -> bool {
    if let Some(url) = url {
        if crate::url_valid(url) && !name.is_empty() && !url.is_empty() {
            return true;
        }
    }

    false
}

// pub fn installed_webapps() -> Vec<WebAppLauncher> {
//     let mut webapps = Vec::new();

//     if let Some(data_dir) = dirs::data_dir() {
//         if let Ok(entries) = std::fs::read_dir(data_dir.join(APP_ID).join("database")) {
//             for entry in entries {
//                 if let Ok(entry) = entry {
//                     let file = std::fs::File::open(entry.path());
//                     let mut content = String::new();

//                     if let Ok(mut f) = file {
//                         f.read_to_string(&mut content).unwrap();
//                         if let Ok(launcher) = ron::from_str::<WebAppLauncher>(&content) {
//                             webapps.push(launcher);
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     webapps
// }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebAppLauncher {
    pub webapp_name: String,
    pub filename: String,
    pub icon_file_path: String,
    pub browser_data: Browser,
    pub category: crate::Category,
}

impl WebAppLauncher {
    pub fn create(&self) -> anyhow::Result<bool> {
        let mut desktop_entry = String::new();

        let app_id = format!("{}_{}", APP_ID, self.webapp_name.to_lowercase());

        let exe = self.browser_data.executable_name.clone();
        let args = self.browser_data.executable_name.clone();

        desktop_entry.push_str("[Desktop Entry]\n");
        desktop_entry.push_str("Version=1.0\n");
        desktop_entry.push_str("Type=Application\n");
        desktop_entry.push_str(&format!("Name={}\n", self.webapp_name));
        desktop_entry.push_str(&format!("Comment=Quick WebApp\n",));
        desktop_entry.push_str(&format!("Exec={} {}\n", exe, args));
        desktop_entry.push_str(&format!("StartupWMClass={}\n", app_id));
        desktop_entry.push_str(&format!("Categories={}\n", self.category.as_ref()));

        tracing::info!("{}", desktop_entry);

        if let Some(mut path) = crate::launcher_desktop_entry_path(&self.filename) {
            path.push(&self.filename);
            let _ = fs::write(path, &desktop_entry);
        }

        Ok(false)
    }

    pub fn delete(&self) -> std::io::Result<()> {
        if let Some(path) = crate::launcher_desktop_entry_path(&self.filename) {
            fs::remove_file(path)?;
        }

        // TODO: delete profile path + icon

        Ok(())
    }
}
