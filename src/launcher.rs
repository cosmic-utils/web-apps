use std::{
    fs::{self, create_dir_all},
    path::PathBuf,
};

use crate::{WebappIcon, browser::Browser};

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

pub fn create_desktop_entry(
    browser: &Browser,
    webapp_id: &str,
    webapp_name: &str,
    webapp_icon: &Option<WebappIcon>,
    category: &str,
) -> anyhow::Result<bool> {
    let mut desktop_entry = String::new();

    let exec = browser.get_exec_string();

    let Some(webapp_icon) = webapp_icon else {
        return Ok(false);
    };

    let Some(icon_path) = &webapp_icon.source_path else {
        return Ok(false);
    };

    desktop_entry.push_str("[Desktop Entry]\n");
    desktop_entry.push_str("Version=1.0\n");
    desktop_entry.push_str("Type=Application\n");
    desktop_entry.push_str(&format!("Name={}\n", webapp_name));
    desktop_entry.push_str(&format!("Comment=Quick WebApp\n",));
    desktop_entry.push_str(&format!("Exec={}\n", exec));
    desktop_entry.push_str(&format!("Icon={}\n", icon_path.display()));
    desktop_entry.push_str(&format!("StartupWMClass={}\n", browser.config.class_name));
    desktop_entry.push_str(&format!("Categories={}\n", category));
    desktop_entry.push_str(&format!("X-WebApp-Browser={}\n", browser.display_name));
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

    tracing::info!("{}", desktop_entry);

    if !PathBuf::from(icon_path).exists() {
        let _ = fs::write(icon_path, &webapp_icon.buffer);
    }

    if let Some(path) = crate::launcher_desktop_entry_path(&webapp_id) {
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
