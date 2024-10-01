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
pub struct Browser {
    pub _type: BrowserType,
    pub name: String,
    pub exec: String,
    pub id: String,
    pub profile_path: PathBuf,
}

impl AsRef<str> for Browser {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl Browser {
    pub fn new(_type: BrowserType, name: &str, exec: &str, id: &str, profile_path: &str) -> Self {
        let name = name.to_string();
        let id = id.to_string();
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
            name,
            exec,
            id,
            profile_path,
        }
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
    SystemFlatpak,
}

#[derive(Debug)]
struct App {
    pub app_source: AppSource,
    pub id: String,
}

impl App {
    pub fn new(app_source: AppSource, id: String) -> Self {
        Self { app_source, id }
    }
}

fn installed_apps() -> Vec<App> {
    let mut apps: Vec<App> = Vec::new();

    let locales = get_languages_from_env();
    let mut paths = Vec::new();
    default_paths().for_each(|path| paths.push(path));

    // this is workaround for flatpak sandbox
    if PathBuf::from("/.flatpak-info").exists() {
        paths.push(home_dir().join(".local/share/flatpak/exports/share/applications"));
        paths.push("/var/lib/flatpak/exports/share/applications".into());
        paths.push("/run/host/usr/share/applications".into());
        paths.push("/run/host/usr/local/share/applications".into());
    }

    for entry in Iter::new(paths.into_iter()).entries(Some(&locales)) {
        match PathSource::guess_from(&entry.path) {
            PathSource::Local => apps.push(App::new(AppSource::Native, entry.id().to_string())),
            PathSource::LocalDesktop => {
                apps.push(App::new(AppSource::Native, entry.id().to_string()))
            }
            PathSource::LocalFlatpak => {
                apps.push(App::new(AppSource::Flatpak, entry.id().to_string()))
            }
            PathSource::LocalNix => apps.push(App::new(AppSource::Nix, entry.id().to_string())),
            PathSource::Nix => apps.push(App::new(AppSource::Nix, entry.id().to_string())),
            PathSource::System => apps.push(App::new(AppSource::Native, entry.id().to_string())),
            PathSource::SystemLocal => {
                apps.push(App::new(AppSource::Native, entry.id().to_string()))
            }
            PathSource::SystemFlatpak => {
                apps.push(App::new(AppSource::SystemFlatpak, entry.id().to_string()))
            }
            PathSource::Other(_) => {
                let str_path = entry.path.to_string_lossy();

                if str_path.contains(".local/share/flatpak/") {
                    apps.push(App::new(AppSource::Flatpak, entry.id().to_string()))
                } else if str_path.contains("/var/lib/flatpak/") {
                    apps.push(App::new(AppSource::SystemFlatpak, entry.id().to_string()))
                } else {
                    apps.push(App::new(AppSource::Native, entry.id().to_string()))
                }
            }
            _ => continue,
        };
    }

    apps
}

pub fn get_supported_browsers() -> Vec<Browser> {
    let native_browsers: Vec<Browser> = supported_browsers::native_browsers();
    let nix_browsers: Vec<Browser> = supported_browsers::nix_browsers();
    let flatpak_browsers: Vec<Browser> = supported_browsers::flatpak_browsers();
    let flatpak_system_browsers: Vec<Browser> = supported_browsers::flatpak_system_browsers();

    let mut browsers = Vec::new();

    for app in installed_apps().iter() {
        match app.app_source {
            AppSource::Native => {
                if let Some(installed) = native_browsers.iter().find(|browser| browser.id == app.id)
                {
                    browsers.push(installed.clone())
                }
            }
            AppSource::Nix => {
                if let Some(installed) = nix_browsers.iter().find(|browser| browser.id == app.id) {
                    browsers.push(installed.clone())
                }
            }
            AppSource::Flatpak => {
                if let Some(installed) =
                    flatpak_browsers.iter().find(|browser| browser.id == app.id)
                {
                    browsers.push(installed.clone())
                }
            }
            AppSource::SystemFlatpak => {
                if let Some(installed) = flatpak_system_browsers
                    .iter()
                    .find(|browser| browser.id == app.id)
                {
                    browsers.push(installed.clone())
                }
            }
        }
    }

    if browsers.is_empty() {
        browsers.push(Browser::new(
            BrowserType::NoBrowser,
            &fl!("select-browser"),
            "",
            "",
            "",
        ));
    }

    browsers
}
