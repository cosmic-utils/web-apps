use crate::{
    browser::{Browser, BrowserModel},
    common::{self},
    LOCALES,
};
use anyhow::Result;
use freedesktop_desktop_entry::DesktopEntry;
use std::{
    fs::{self, create_dir_all, remove_dir_all, remove_file, File},
    io::Write,
    path::PathBuf,
};

pub fn webapplauncher_is_valid(icon: &str, name: &str, url: &str) -> bool {
    if !common::url_valid(url) || name.is_empty() || icon.is_empty() || url.is_empty() {
        return false;
    }

    true
}

pub fn installed_webapps() -> Vec<WebAppLauncher> {
    let mut webapps = Vec::new();

    match fs::read_dir(common::desktop_filepath("")) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let entry_fn = entry.file_name();
                        let filename = entry_fn.to_str().unwrap();

                        if filename.starts_with("QuickWebApp-") && filename.ends_with(".desktop") {
                            let fde = DesktopEntry::from_path(entry.path(), Some(&LOCALES));

                            match fde {
                                Ok(fde) => webapps.push(WebAppLauncher::from(fde)),
                                Err(e) => tracing::error!(
                                    "Error reading desktop entry for {}: \n{}",
                                    filename,
                                    e
                                ),
                            }
                        }
                    }
                    Err(e) => tracing::error!("Error reading directory: {}", e),
                }
            }
        }
        Err(_) => {
            create_dir_all(common::desktop_filepath(""))
                .expect("Cannot create local applications dir");
        }
    }

    webapps
}

#[derive(Debug, Clone)]
pub struct WebAppLauncher {
    pub codename: String,
    pub browser: Browser,
    pub name: String,
    pub icon: String,
    pub category: String,
    pub url: String,
    pub custom_parameters: String,
    pub isolate_profile: bool,
    pub navbar: bool,
    pub is_incognito: bool,
}

impl From<DesktopEntry> for WebAppLauncher {
    fn from(value: DesktopEntry) -> Self {
        let group = value.groups.group("Desktop Entry");

        match group {
            Some(group) => Self {
                codename: group
                    .entry("X-QWA-Codename")
                    .unwrap_or_default()
                    .to_string(),
                browser: Browser::from_appid(
                    group
                        .entry("X-QWA-Browser-Id")
                        .unwrap_or_default()
                        .to_string(),
                ),
                name: value.name(&LOCALES).unwrap_or_default().to_string(),
                icon: value.icon().unwrap_or_default().to_string(),
                category: value.categories().unwrap_or_default().concat(),
                url: group.entry("X-QWA-Url").unwrap_or_default().to_string(),
                custom_parameters: group
                    .entry("X-QWA-Parameters")
                    .unwrap_or_default()
                    .to_string(),
                isolate_profile: group
                    .entry("X-QWA-Isolated")
                    .unwrap_or_default()
                    .parse::<bool>()
                    .unwrap_or_default(),
                navbar: group
                    .entry("X-QWA-Navbar")
                    .unwrap_or_default()
                    .parse::<bool>()
                    .unwrap_or_default(),
                is_incognito: group
                    .entry("X-QWA-Private")
                    .unwrap_or_default()
                    .parse::<bool>()
                    .unwrap_or_default(),
            },
            None => Self {
                codename: String::new(),
                browser: Browser::none(),
                name: String::new(),
                icon: String::new(),
                category: String::new(),
                url: String::new(),
                custom_parameters: String::new(),
                isolate_profile: false,
                navbar: false,
                is_incognito: false,
            },
        }
    }
}

impl WebAppLauncher {
    fn create_firefox_userjs(
        &self,
        is_zen_browser: bool,
        path: PathBuf,
        create_navbar: bool,
    ) -> bool {
        let mut file = File::create(&path)
            .unwrap_or_else(|_| panic!("failed to create user.js in {:?}", path));

        let navbar_pref = if create_navbar {
            b"user_pref(\"browser.tabs.inTitlebar\", 2);\n"
        } else {
            b"user_pref(\"browser.tabs.inTitlebar\", 0);\n"
        };
        let Ok(_) = file.write_all(navbar_pref) else {
            return false;
        };

        match is_zen_browser {
            true => file
                .write_all(include_bytes!(
                    "../data/runtime/zen-browser/profile/user.js"
                ))
                .is_ok(),
            false => file
                .write_all(include_bytes!("../data/runtime/firefox/profile/user.js"))
                .is_ok(),
        }
    }

    fn create_user_chrome_css(
        &self,
        is_zen_browser: bool,
        path: PathBuf,
        create_navbar: bool,
    ) -> bool {
        let mut file = File::create(&path)
            .unwrap_or_else(|_| panic!("cant create userChrome.css in {:?}", path));

        if create_navbar {
            file.write_all(b"").is_ok()
        } else {
            match is_zen_browser {
                true => file
                    .write_all(include_bytes!(
                        "../data/runtime/zen-browser/profile/chrome/userChrome.css"
                    ))
                    .is_ok(),
                false => file
                    .write_all(include_bytes!(
                        "../data/runtime/firefox/profile/chrome/userChrome.css"
                    ))
                    .is_ok(),
            }
        }
    }

    fn exec_firefox(&self, is_zen_browser: bool) -> String {
        let profile_path = self.browser.profile_path.join(&self.codename);
        let user_js_path = profile_path.join("user.js");
        let mut user_chrome_css = profile_path.join("chrome");

        tracing::info!("Creating profile directory in: {:?}", &profile_path);
        create_dir_all(&profile_path)
            .unwrap_or_else(|_| panic!("cant create profile dir in {:?}", &profile_path));
        create_dir_all(&user_chrome_css)
            .unwrap_or_else(|_| panic!("cant create chrome dir in {:?}", &user_chrome_css));

        user_chrome_css = user_chrome_css.join("userChrome.css");

        self.create_firefox_userjs(is_zen_browser, user_js_path, self.navbar);
        self.create_user_chrome_css(is_zen_browser, user_chrome_css, self.navbar);

        let profile_path = profile_path.to_str().unwrap();

        let mut exec_string = format!(
            "{} --class QuickWebApp-{} --name QuickWebApp-{} --profile {} --no-remote ",
            self.browser.exec, self.codename, self.codename, profile_path
        );

        if self.is_incognito {
            exec_string.push_str("--private-window ");
        }

        if !self.custom_parameters.is_empty() {
            exec_string.push_str(&format!("{} ", self.custom_parameters));
        }

        exec_string.push_str(&self.url);

        exec_string
    }

    fn exec_chromium(&self) -> String {
        let mut exec_string = format!(
            "{} --app={} --class=QuickWebApp-{} --name=QuickWebApp-{} ",
            self.browser.exec, self.url, self.codename, self.codename
        );

        if self.isolate_profile {
            let profile_dir = self.browser.profile_path.join(&self.codename);

            tracing::info!("Creating profile directory in: {:?}", &profile_dir);
            let _ = create_dir_all(&profile_dir);
            let profile_path = profile_dir.to_str().unwrap();
            exec_string.push_str(&format!("--user-data-dir={} ", profile_path));
        }

        if self.is_incognito {
            if self.browser.name.starts_with("Microsoft Edge") {
                exec_string.push_str("--inprivate ");
            } else {
                exec_string.push_str("--incognito ");
            }
        }

        if !self.custom_parameters.is_empty() {
            exec_string.push_str(&format!("{} ", self.custom_parameters));
        }

        exec_string
    }

    fn exec_falkon(&self) -> String {
        let mut exec_string = String::new();

        if self.isolate_profile {
            let profile_dir = self.browser.profile_path.join(&self.codename);
            tracing::info!("Creating profile directory in: {:?}", &profile_dir);
            let _ = create_dir_all(&profile_dir);

            let profile_path = profile_dir.to_str().unwrap();

            exec_string = format!(
                "{} --portable --wmclass QuickWebApp-{} --profile {} ",
                self.browser.exec, self.codename, profile_path
            );
        }

        if self.is_incognito {
            exec_string.push_str("--private-browsing ");
        }

        if !self.custom_parameters.is_empty() {
            exec_string.push_str(&format!("{} ", self.custom_parameters));
        }

        exec_string.push_str(&format!("--no-remote --current-tab {}", self.url));

        exec_string
    }

    fn exec_string(&self) -> String {
        if let Some(model) = &self.browser.model {
            return match model {
                BrowserModel::Brave => self.exec_chromium(),
                BrowserModel::Chrome => self.exec_chromium(),
                BrowserModel::Chromium => self.exec_chromium(),
                BrowserModel::Cromite => self.exec_chromium(),
                BrowserModel::Falkon => self.exec_falkon(),
                BrowserModel::Firefox => self.exec_firefox(false),
                BrowserModel::Floorp => self.exec_firefox(false),
                BrowserModel::Librewolf => self.exec_firefox(false),
                BrowserModel::MicrosoftEdge => self.exec_chromium(),
                BrowserModel::Vivaldi => self.exec_chromium(),
                BrowserModel::Waterfox => self.exec_firefox(false),
                BrowserModel::Zen => self.exec_firefox(true),
            };
        };

        String::new()
    }

    fn qwa_desktop_path(&self) -> PathBuf {
        let filename = format!("QuickWebApp-{}.desktop", self.codename);
        common::desktop_filepath(&filename)
    }

    pub fn create(&self) -> Result<()> {
        if let Some(entry) = &self.browser.entry {
            if self.qwa_desktop_path().exists() {
                remove_file(self.qwa_desktop_path())?;
            }

            let mut output = File::create(self.qwa_desktop_path())?;

            writeln!(output, "[Desktop Entry]")?;
            writeln!(output, "Version=1.0")?;
            writeln!(output, "Name={}", self.name)?;
            writeln!(output, "Comment=Quick Web Apps")?;
            writeln!(output, "Exec={}", self.exec_string())?;
            writeln!(output, "Terminal=false")?;
            writeln!(output, "Type=Application")?;
            writeln!(output, "Icon={}", self.icon)?;
            writeln!(output, "Categories={};", self.category)?;
            writeln!(output, "MimeType=text/html;text/xml;application/xhtml_xml;")?;
            writeln!(output, "StartupWMClass=QuickWebApp-{}", self.codename)?;
            writeln!(output, "StartupNotify=true")?;
            writeln!(output, "X-QWA-Codename={}", self.codename)?;
            writeln!(output, "X-QWA-Browser-Id={}", entry.appid)?;
            writeln!(output, "X-QWA-Url={}", self.url)?;
            writeln!(output, "X-QWA-Navbar={}", self.navbar)?;
            writeln!(output, "X-QWA-Private={}", self.is_incognito)?;
            writeln!(output, "X-QWA-Isolated={}", self.isolate_profile)?;
            writeln!(output, "X-QWA-Parameters={}", self.custom_parameters)?;
        }

        Ok(())
    }

    pub fn delete(&self) -> Result<()> {
        if self.qwa_desktop_path().exists() {
            let profile_path = self.browser.profile_path.join(&self.codename);
            remove_file(self.qwa_desktop_path())?;
            remove_dir_all(&profile_path)?;
        }

        Ok(())
    }
}
