use anyhow::anyhow;
use ashpd::desktop::{
    Icon,
    dynamic_launcher::{
        DynamicLauncherProxy, InstallOptions, LauncherType, PrepareInstallOptions, UninstallOptions,
    },
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DesktopIcon {
    pub path: PathBuf,
    pub buffer: Vec<u8>,
}

impl DesktopIcon {
    pub fn to_icon(&self) -> crate::Icon {
        crate::handle_icon(self.path.clone())
    }
}

pub async fn create(
    browser: crate::browser::Browser,
    app_config: crate::AppConfig,
) -> anyhow::Result<(String, crate::AppConfig), anyhow::Error> {
    let mut desktop_entry = String::new();

    let Some(exe) = browser.get_exec() else {
        return Err(anyhow!("Failed to get browser executable."));
    };

    let Some(ref icon) = app_config.icon else {
        return Err(anyhow!("Failed! Icon is wrong."));
    };

    desktop_entry.push_str("[Desktop Entry]\n");
    desktop_entry.push_str("Version=1.0\n");
    desktop_entry.push_str("Type=Application\n");
    desktop_entry.push_str(&format!("Name={}\n", app_config.title));
    desktop_entry.push_str(&format!("Comment=Quick WebApp\n",));
    desktop_entry.push_str(&format!("Exec={}\n", exe));
    desktop_entry.push_str(&format!("StartupWMClass={}\n", &browser.app_id.id));
    desktop_entry.push_str(&format!("Categories={}\n", app_config.category.as_ref()));

    let proxy = DynamicLauncherProxy::new()
        .await
        .expect("Failed to create DynamicLauncherProxy");

    let icon = Icon::Bytes(icon.buffer.clone());

    let prepare_opts = PrepareInstallOptions::default()
        .set_editable_icon(true)
        .set_launcher_type(LauncherType::Application);

    let response = proxy
        .prepare_install(None, &app_config.title, icon, prepare_opts)
        .await
        .expect("Failed to prepare install")
        .response()
        .expect("Failed to get response");

    let token = response.token();

    tracing::info!("{}", desktop_entry);

    proxy
        .install(
            &token,
            &format!("{}.desktop", browser.app_id.id),
            &desktop_entry,
            InstallOptions::default(),
        )
        .await
        .expect("installing");

    Ok((app_config.id.to_string(), app_config.to_owned()))
}

pub async fn uninstall(
    app_config: crate::AppConfig,
) -> anyhow::Result<(String, crate::AppConfig), anyhow::Error> {
    let proxy = DynamicLauncherProxy::new()
        .await
        .expect("Failed to create DynamicLauncherProxy");

    proxy
        .uninstall(
            &format!("{}.desktop", app_config.id),
            UninstallOptions::default(),
        )
        .await
        .expect("installing");

    Ok((app_config.id.to_string(), app_config.to_owned()))
}
