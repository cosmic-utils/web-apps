use crate::{
    common::{self, home_dir},
    fl, supported_browsers,
};

use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter, PathSource};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserType {
    NoBrowser,
    Firefox,
    FirefoxFlatpak,
    Zen,
    ZenFlatpak,
    Chromium,
    ChromiumFlatpak,
    Falkon,
    FalkonFlatpak,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryLocation {
    System,
    Nix,
    FlatpakLocal,
    FlatpakSystem,
    Snap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Browser {
    pub _type: BrowserType,
    pub _binary: BinaryLocation,
    pub name: String,
    pub exec: String,
    pub profile_path: PathBuf,
}

impl AsRef<str> for Browser {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl Browser {
    pub fn new(
        _type: BrowserType,
        _binary: BinaryLocation,
        name: &str,
        exec: &str,
        profile_path: &str,
    ) -> Self {
        let name = name.to_string();
        let mut exe_path = PathBuf::new();

        let base = common::home_dir();
        let data_home = base.join(".local/share");

        if exec.starts_with(".local/share/") {
            let flatpak_path: Vec<&str> = exec.split(".local/share/").collect();
            let path = data_home.join(flatpak_path[1]);
            exe_path.push(path);
        } else {
            exe_path.push(exec)
        }

        let exec = exe_path.to_str().unwrap().to_string();
        let profile_path = base.join(profile_path);

        Self {
            _type,
            _binary,
            name,
            exec,
            profile_path,
        }
    }

    pub fn update_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn update_binary_location(&mut self, location: BinaryLocation) {
        self._binary = location;
    }

    pub fn web_browser(name: String) -> Option<Browser> {
        let supported = get_supported_browsers();
        supported.into_iter().find(|b| b.name == name)
    }

    pub fn is_installed(&self) -> bool {
        !matches!(self._type, BrowserType::NoBrowser)
    }
}

#[derive(Debug)]
enum AppSource {
    Native,
    Nix,
    Flatpak,
    Snap,
    SystemFlatpak,
}

#[derive(Debug)]
struct App {
    pub app_source: AppSource,
    pub name: String,
    pub exec: String,
}

impl App {
    pub fn new(app_source: AppSource, name: String, exec: String) -> Self {
        Self {
            app_source,
            name,
            exec,
        }
    }
}

fn installed_apps() -> Vec<App> {
    let mut apps: Vec<App> = Vec::new();

    let locales = get_languages_from_env();
    let mut paths = Vec::new();
    default_paths().for_each(|path| paths.push(path));

    // this is workaround for flatpak sandbox
    if PathBuf::from("/.flatpak-info").exists() {
        paths.push(home_dir().join(".local/share/applications"));
        paths.push(home_dir().join(".local/share/flatpak/exports/share/applications"));
        paths.push("/var/lib/flatpak/exports/share/applications".into());
        paths.push("/run/host/usr/share/applications".into());
        paths.push("/run/host/usr/local/share/applications".into());
    }

    for entry in Iter::new(paths.into_iter()).entries(Some(&locales)) {
        if let Some(comments) = entry.comment(&locales) {
            if comments.to_lowercase().contains("web app") {
                continue;
            }
        }

        match PathSource::guess_from(&entry.path) {
            PathSource::Local => {
                if let Some(exec) = entry.exec() {
                    apps.push(App::new(
                        AppSource::Native,
                        format!("{} (User)", entry.name(&locales).unwrap_or_default()),
                        exec.to_string(),
                    ))
                }
            }
            PathSource::LocalDesktop => {
                if let Some(exec) = entry.exec() {
                    apps.push(App::new(
                        AppSource::Native,
                        format!("{} (User)", entry.name(&locales).unwrap_or_default()),
                        exec.to_string(),
                    ))
                }
            }
            PathSource::LocalFlatpak => {
                if let Some(exec) = entry.exec() {
                    apps.push(App::new(
                        AppSource::Flatpak,
                        format!(
                            "{} (User Flatpak)",
                            entry.name(&locales).unwrap_or_default()
                        ),
                        exec.to_string(),
                    ))
                }
            }
            PathSource::LocalNix => {
                if let Some(exec) = entry.exec() {
                    apps.push(App::new(
                        AppSource::Nix,
                        format!("{} (User)", entry.name(&locales).unwrap_or_default()),
                        exec.to_string(),
                    ))
                }
            }
            PathSource::Nix => {
                if let Some(exec) = entry.exec() {
                    apps.push(App::new(
                        AppSource::Nix,
                        entry.name(&locales).unwrap_or_default().to_string(),
                        exec.to_string(),
                    ))
                }
            }
            PathSource::System => {
                if let Some(exec) = entry.exec() {
                    apps.push(App::new(
                        AppSource::Native,
                        entry.name(&locales).unwrap_or_default().to_string(),
                        exec.to_string(),
                    ))
                }
            }
            PathSource::SystemLocal => {
                if let Some(exec) = entry.exec() {
                    apps.push(App::new(
                        AppSource::Native,
                        format!("{} (User)", entry.name(&locales).unwrap_or_default()),
                        exec.to_string(),
                    ))
                }
            }
            PathSource::SystemFlatpak => {
                if let Some(exec) = entry.exec() {
                    apps.push(App::new(
                        AppSource::SystemFlatpak,
                        format!("{} (Flatpak)", entry.name(&locales).unwrap_or_default()),
                        exec.to_string(),
                    ))
                }
            }
            PathSource::SystemSnap => {
                if let Some(exec) = entry.exec() {
                    apps.push(App::new(
                        AppSource::Snap,
                        format!("{} (Snap)", entry.name(&locales).unwrap_or_default()),
                        exec.to_string(),
                    ))
                }
            }
            PathSource::Other(_) => {
                if let Some(exec) = entry.exec() {
                    let str_path = entry.path.to_string_lossy();

                    if str_path.contains(".local/share/flatpak/") {
                        apps.push(App::new(
                            AppSource::Flatpak,
                            format!(
                                "{} (User Flatpak)",
                                entry.name(&locales).unwrap_or_default()
                            ),
                            exec.to_string(),
                        ))
                    } else if str_path.contains("/var/lib/flatpak/") {
                        apps.push(App::new(
                            AppSource::SystemFlatpak,
                            format!("{} (Flatpak)", entry.name(&locales).unwrap_or_default()),
                            exec.to_string(),
                        ))
                    } else {
                        apps.push(App::new(
                            AppSource::Native,
                            entry.name(&locales).unwrap_or_default().to_string(),
                            exec.to_string(),
                        ))
                    }
                }
            }
        };
    }

    apps
}

pub fn get_supported_browsers() -> Vec<Browser> {
    let mut native_browsers: Vec<Browser> = supported_browsers::native_browsers();
    let mut nix_browsers: Vec<Browser> = supported_browsers::nix_browsers();
    let mut flatpak_browsers: Vec<Browser> = supported_browsers::flatpak_browsers();
    let mut snap_browsers: Vec<Browser> = supported_browsers::snap_browsers();

    let mut browsers = Vec::new();

    for app in installed_apps().iter() {
        match app.app_source {
            AppSource::Native => {
                if let Some(installed) = native_browsers
                    .iter_mut()
                    .find(|browser| app.exec.contains(&browser.exec))
                {
                    if !app.name.is_empty() {
                        installed.update_name(app.name.clone())
                    }
                    browsers.push(installed.clone())
                }
            }
            AppSource::Nix => {
                if let Some(installed) = nix_browsers
                    .iter_mut()
                    .find(|browser| app.exec.contains(&browser.exec))
                {
                    if !app.name.is_empty() {
                        installed.update_name(app.name.clone())
                    }
                    browsers.push(installed.clone())
                }
            }
            AppSource::Flatpak => {
                if let Some(installed) = flatpak_browsers
                    .iter_mut()
                    .find(|browser| app.exec.contains(&browser.exec))
                {
                    if !app.name.is_empty() {
                        installed.update_name(app.name.clone())
                    }
                    browsers.push(installed.clone())
                }
            }
            AppSource::Snap => {
                if let Some(installed) = snap_browsers
                    .iter_mut()
                    .find(|browser| app.exec.contains(&browser.exec))
                {
                    if !app.name.is_empty() {
                        installed.update_name(app.name.clone())
                    }
                    browsers.push(installed.clone())
                }
            }
            AppSource::SystemFlatpak => {
                if let Some(installed) = flatpak_browsers
                    .iter_mut()
                    .find(|browser| app.exec.contains(&browser.exec))
                {
                    if !app.name.is_empty() {
                        installed.update_name(app.name.clone())
                    }
                    installed.update_binary_location(BinaryLocation::FlatpakSystem);
                    browsers.push(installed.clone())
                }
            }
        }
    }

    if browsers.is_empty() {
        browsers.push(Browser::new(
            BrowserType::NoBrowser,
            BinaryLocation::System,
            &fl!("select-browser"),
            "",
            "",
        ));
    }

    browsers
}
