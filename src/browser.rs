use crate::{
    common::{fd_entries, home_dir},
    LOCALES,
};
use freedesktop_desktop_entry::{matching::find_entry_from_appid, DesktopEntry, PathSource};
use std::path::PathBuf;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter)]
pub enum BrowserModel {
    Brave,
    Chrome,
    Chromium,
    Cromite,
    Falkon,
    Floorp,
    Firefox,
    Librewolf,
    MicrosoftEdge,
    Vivaldi,
    Waterfox,
    Zen,
}

impl AsRef<str> for BrowserModel {
    fn as_ref(&self) -> &str {
        match &self {
            BrowserModel::Brave => "brave",
            BrowserModel::Chrome => "google-chrome",
            BrowserModel::Chromium => "chromium",
            BrowserModel::Cromite => "cromite",
            BrowserModel::Falkon => "falkon",
            BrowserModel::Floorp => "floorp",
            BrowserModel::Firefox => "firefox",
            BrowserModel::Librewolf => "librewolf",
            BrowserModel::MicrosoftEdge => "microsoft-edge",
            BrowserModel::Vivaldi => "vivaldi",
            BrowserModel::Waterfox => "waterfox",
            BrowserModel::Zen => "zen_browser",
        }
    }
}

impl BrowserModel {
    pub fn from(app_id: &str) -> Option<BrowserModel> {
        let normalize_appid = app_id.to_lowercase().replace("-", ".");
        BrowserModel::iter().find(|b| {
            let compare_b = b.as_ref().to_lowercase().replace("-", ".");
            normalize_appid.contains(&compare_b)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserSource {
    Flatpak,
    Native,
    NativeLocal,
    Nix,
    Snap,
    SystemFlatpak,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Browser {
    pub model: Option<BrowserModel>,
    pub source: BrowserSource,
    pub entry: Option<DesktopEntry>,
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
    pub fn none() -> Self {
        Self {
            model: None,
            source: BrowserSource::Native,
            entry: None,
            name: String::new(),
            exec: String::new(),
            profile_path: PathBuf::new(),
        }
    }

    fn create(entry: DesktopEntry) -> Self {
        let mut name = entry.name(&LOCALES).unwrap_or_default().to_string();
        let exec = entry.exec().unwrap_or_default().to_string();
        let is_flatpak = entry.flatpak().is_some();
        let profile_path = match is_flatpak {
            true => home_dir()
                .join(".var/app/")
                .join(&entry.appid)
                .join("data/profiles"),
            false => home_dir()
                .join(".local/share/quick-webapps/")
                .join(&entry.appid),
        };

        if let Some(model) = BrowserModel::from(&entry.appid) {
            let source = match PathSource::guess_from(&entry.path) {
                PathSource::Local | PathSource::LocalDesktop | PathSource::SystemLocal => {
                    BrowserSource::NativeLocal
                }
                PathSource::System => BrowserSource::Native,
                PathSource::LocalFlatpak => BrowserSource::Flatpak,
                PathSource::LocalNix | PathSource::Nix => BrowserSource::Nix,
                PathSource::SystemFlatpak => BrowserSource::SystemFlatpak,
                PathSource::SystemSnap => BrowserSource::Snap,
                PathSource::Other(_) => {
                    let str_path = entry.path.to_string_lossy();

                    if str_path.contains(".local/share/flatpak/") {
                        BrowserSource::Flatpak
                    } else if str_path.contains("/var/lib/flatpak/") {
                        BrowserSource::SystemFlatpak
                    } else {
                        BrowserSource::Native
                    }
                }
            };

            match source {
                BrowserSource::Flatpak => name.push_str(" (User Flatpak)"),
                BrowserSource::Snap => name.push_str(" (Snap)"),
                BrowserSource::SystemFlatpak => name.push_str(" (System Flatpak)"),
                BrowserSource::NativeLocal => name.push_str(" (User)"),
                _ => (),
            }

            return Self {
                model: Some(model),
                source,
                entry: Some(entry),
                name,
                exec,
                profile_path,
            };
        }

        Self::none()
    }

    pub fn from_appid(appid: String) -> Self {
        let entries = fd_entries();
        let entry = find_entry_from_appid(&entries, &appid);

        if let Some(entry) = entry {
            return Self::create(entry.clone());
        };

        Self::none()
    }

    pub fn from_path(path: &PathBuf) -> Self {
        let entry = DesktopEntry::from_path(path, Some(&LOCALES));

        if let Ok(entry) = entry {
            return Self::create(entry);
        }

        Self::none()
    }
}

pub fn installed_browsers() -> Vec<Browser> {
    let mut apps: Vec<Browser> = Vec::new();

    for entry in fd_entries() {
        if let Some(comments) = entry.comment(&LOCALES) {
            if comments.contains("Quick Web Apps") {
                continue;
            }
        }

        let browser = Browser::from_path(&entry.path);

        if browser.model.is_some() {
            apps.push(browser);
        }
    }

    apps
}
