use crate::{common::fd_entries, LOCALES};
use freedesktop_desktop_entry::{find_app_by_id, unicase, DesktopEntry, PathSource};
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, remove_file, File},
    io::Write,
    path::PathBuf,
    str::FromStr,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone)]
pub struct Firefox {
    pub exec: String,
    pub codename: String,
    pub url: String,
    pub user_dir: String,
    pub navbar: bool,
    pub isolated: bool,
    pub zen_browser: bool,
    pub private: bool,
    pub custom_args: String,
}

impl Firefox {
    pub fn builder(browser_exec: String) -> Self {
        Self {
            exec: browser_exec,
            codename: String::new(),
            url: String::new(),
            user_dir: String::new(),
            navbar: false,
            isolated: true,
            zen_browser: false,
            private: false,
            custom_args: String::new(),
        }
    }

    pub fn codename(&mut self, codename: String) -> &mut Self {
        self.codename = format!(
            "--class dev.heppen.webapps.{} --name dev.heppen.webapps.{}",
            codename, codename
        );
        self
    }

    pub fn url(&mut self, app_url: String) -> &mut Self {
        self.url = app_url;
        self
    }

    pub fn navbar(&mut self, flag: bool) -> &mut Self {
        self.navbar = flag;
        self
    }

    pub fn isolated(&mut self, flag: bool) -> &mut Self {
        self.isolated = flag;
        self
    }

    fn create_firefox_userjs(&self, path: PathBuf) {
        let file = File::create(path.join("user.js"));

        if let Ok(mut file) = file {
            let navbar_pref = if self.navbar {
                b"user_pref(\"browser.tabs.inTitlebar\", 2);\n"
            } else {
                b"user_pref(\"browser.tabs.inTitlebar\", 0);\n"
            };
            file.write_all(navbar_pref).unwrap();

            if self.zen_browser {
                file.write_all(include_bytes!(
                    "../data/runtime/zen-browser/profile/user.js"
                ))
                .unwrap();
            } else {
                file.write_all(include_bytes!("../data/runtime/firefox/profile/user.js"))
                    .unwrap();
            };
        }
    }

    fn create_user_chrome_css(&self, path: PathBuf) {
        let chrome = path.join("chrome");
        let user_chrome = chrome.join("userChrome.css");
        let _ = remove_file(&user_chrome);

        if !self.navbar {
            let _ = create_dir_all(&chrome);

            let file = File::create(user_chrome);

            if let Ok(mut file) = file {
                if self.zen_browser {
                    file.write_all(include_bytes!(
                        "../data/runtime/zen-browser/profile/chrome/userChrome.css"
                    ))
                    .unwrap();
                } else {
                    file.write_all(include_bytes!(
                        "../data/runtime/firefox/profile/chrome/userChrome.css"
                    ))
                    .unwrap();
                }
            }
        }
    }

    pub fn profile_path(&mut self, path: PathBuf) -> &mut Self {
        if self.isolated {
            tracing::info!("Creating profile directory in: {:?}", &path);

            let _ = create_dir_all(&path);

            self.create_firefox_userjs(path.clone());
            self.create_user_chrome_css(path.clone());
        }

        self.user_dir = format!(" --profile \"{}\"", path.to_str().unwrap());
        self
    }

    pub fn zen_browser(&mut self, flag: bool) -> &mut Self {
        self.zen_browser = flag;
        self
    }

    pub fn private_mode(&mut self, flag: bool) -> &mut Self {
        self.private = flag;
        self
    }

    pub fn custom_args(&mut self, args: String) -> &mut Self {
        self.custom_args = args;
        self
    }

    pub fn build(&mut self) -> String {
        let mut exec = format!("{} --no-remote {}", self.exec, self.codename);

        if self.private {
            exec.push_str(" --private-window");
        }

        if self.isolated {
            exec.push_str(&self.user_dir);
        }

        if !self.custom_args.is_empty() {
            exec.push_str(&self.custom_args);
        }

        exec.push_str(&format!(" --new-window {}", self.url));

        exec
    }
}

#[derive(Debug, Clone)]
pub struct Falkon {
    pub exec: String,
    pub codename: String,
    pub url: String,
    pub user_dir: String,
    pub isolated: bool,
    pub private: bool,
    pub custom_args: String,
}

impl Falkon {
    pub fn builder(browser_exec: String) -> Self {
        Self {
            exec: browser_exec,
            codename: String::new(),
            url: String::new(),
            user_dir: String::new(),
            isolated: true,
            private: false,
            custom_args: String::new(),
        }
    }

    pub fn codename(&mut self, codename: String) -> &mut Self {
        self.codename = format!("--wmclass dev.heppen.webapps.{}", codename);
        self
    }

    pub fn url(&mut self, app_url: String) -> &mut Self {
        self.url = app_url;
        self
    }

    pub fn isolated(&mut self, flag: bool) -> &mut Self {
        self.isolated = flag;
        self
    }

    pub fn profile_path(&mut self, path: PathBuf) -> &mut Self {
        if self.isolated {
            tracing::info!("Creating profile directory in: {:?}", &path);
            let _ = create_dir_all(&path);
        }

        self.user_dir = format!(" --profile \"{}\"", path.to_str().unwrap());
        self
    }

    pub fn private_mode(&mut self, flag: bool) -> &mut Self {
        self.private = flag;
        self
    }

    pub fn custom_args(&mut self, args: String) -> &mut Self {
        self.custom_args = args;
        self
    }

    pub fn build(&mut self) -> String {
        let mut exec = format!("{} --no-remote --current-tab {}", self.exec, self.codename);

        if self.private {
            exec.push_str(" --private-browsing");
        }

        if self.isolated {
            exec.push_str(&self.user_dir);
        }

        if !self.custom_args.is_empty() {
            exec.push_str(&format!(" {}", self.custom_args));
        }

        exec.push_str(&format!(" {}", self.url));

        exec
    }
}

#[derive(Debug, Clone)]
pub struct Chromium {
    pub exec: String,
    pub codename: String,
    pub url: String,
    pub user_dir: String,
    pub isolated: bool,
    pub ms_edge: bool,
    pub private: bool,
    pub custom_args: String,
}

impl Chromium {
    pub fn builder(browser_exec: String) -> Self {
        Self {
            exec: browser_exec,
            codename: String::new(),
            url: String::new(),
            user_dir: String::new(),
            isolated: true,
            ms_edge: false,
            private: false,
            custom_args: String::new(),
        }
    }

    pub fn codename(&mut self, codename: String) -> &mut Self {
        self.codename = format!(
            "--class=dev.heppen.webapps.{} --name=dev.heppen.webapps.{}",
            codename, codename
        );
        self
    }

    pub fn url(&mut self, app_url: String) -> &mut Self {
        self.url = format!("--app={}", app_url);
        self
    }

    pub fn isolated(&mut self, flag: bool) -> &mut Self {
        self.isolated = flag;
        self
    }

    pub fn profile_path(&mut self, path: PathBuf) -> &mut Self {
        if self.isolated {
            tracing::info!("Creating profile directory in: {:?}", &path);
            let _ = create_dir_all(&path);
        }

        self.user_dir = format!(" --user-data-dir=\"{}\"", path.to_str().unwrap());
        self
    }

    pub fn ms_edge(&mut self, flag: bool) -> &mut Self {
        self.ms_edge = flag;
        self
    }

    pub fn private_mode(&mut self, flag: bool) -> &mut Self {
        self.private = flag;
        self
    }

    pub fn custom_args(&mut self, args: String) -> &mut Self {
        self.custom_args = args;
        self
    }

    pub fn build(&mut self) -> String {
        let mut exec = format!("{} {} {}", self.exec, self.url, self.codename);

        if self.private {
            if self.ms_edge {
                exec.push_str(" --inprivate");
            } else {
                exec.push_str(" --incognito");
            }
        }

        if self.isolated {
            exec.push_str(&self.user_dir);
        }

        if !self.custom_args.is_empty() {
            exec.push_str(&self.custom_args);
        }

        exec
    }
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, Deserialize, Serialize)]
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

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum BrowserSource {
    Flatpak,
    #[default]
    Native,
    NativeLocal,
    Nix,
    Snap,
    SystemFlatpak,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Browser {
    pub model: Option<BrowserModel>,
    pub source: BrowserSource,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
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
    fn create(entry: DesktopEntry) -> Self {
        let mut name = entry.name(&LOCALES).unwrap_or_default().to_string();
        let xdg_data = dirs::data_dir().unwrap_or_default();
        let profile_path = xdg_data.join("quick-webapps/profiles").join(&entry.appid);

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

            let executable = match source {
                BrowserSource::Native
                | BrowserSource::NativeLocal
                | BrowserSource::Nix
                | BrowserSource::Snap => entry.exec().unwrap_or_default().to_string(),
                BrowserSource::Flatpak => dirs::home_dir()
                    .unwrap_or_default()
                    .join(".local/share/flatpak/exports/bin")
                    .join(&entry.appid)
                    .to_str()
                    .unwrap()
                    .to_string(),
                BrowserSource::SystemFlatpak => PathBuf::from_str("/var/lib/flatpak/exports/bin")
                    .unwrap_or_default()
                    .join(&entry.appid)
                    .to_str()
                    .unwrap()
                    .to_string(),
            };

            return Self {
                model: Some(model),
                source,
                entry: Some(entry),
                name,
                exec: executable,
                profile_path,
            };
        }

        Browser::default()
    }

    pub fn from_appid(appid: String) -> Self {
        let entries = fd_entries();
        let entry = find_app_by_id(&entries, unicase::Ascii::new(&appid));

        if let Some(entry) = entry {
            return Self::create(entry.clone());
        };

        Browser::default()
    }

    pub fn from_path(path: &PathBuf) -> Self {
        let entry = DesktopEntry::from_path(path, Some(&LOCALES));

        if let Ok(entry) = entry {
            return Self::create(entry);
        }

        Browser::default()
    }
}

pub fn installed_browsers() -> Vec<Browser> {
    let mut apps: Vec<Browser> = Vec::new();

    for entry in fd_entries() {
        if let Some(comments) = entry.comment(&LOCALES) {
            if comments.contains("Quick Web App") {
                continue;
            }
        }

        let browser = Browser::from_path(&entry.path);

        if browser.model.is_some() && !apps.contains(&browser) {
            apps.push(browser);
        }
    }

    apps
}
