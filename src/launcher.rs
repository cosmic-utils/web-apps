use crate::{
    browser::{Browser, BrowserModel, Chromium, Falkon, Firefox},
    common::{self},
    pages::editor::Category,
    LOCALES,
};
use anyhow::Result;
use ashpd::desktop::dynamic_launcher::{DynamicLauncherProxy, PrepareInstallOptions};
use freedesktop_desktop_entry::DesktopEntry;
use std::{
    fs::{self, create_dir_all, remove_dir_all, File},
    io::Read,
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
    pub category: Category,
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
                category: Category::from(value.categories().unwrap_or_default().concat()),
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
                browser: Browser::default(),
                name: String::new(),
                icon: String::new(),
                category: Category::default(),
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
    fn exec_firefox(&self, zen_browser: bool) -> String {
        let profile_path = self.browser.profile_path.join(&self.codename);

        Firefox::builder(self.browser.exec.clone())
            .url(self.url.clone())
            .codename(self.codename.clone())
            .navbar(self.navbar)
            .isolated(self.isolate_profile)
            .profile_path(profile_path)
            .zen_browser(zen_browser)
            .private_mode(self.is_incognito)
            .custom_args(self.custom_parameters.clone())
            .build()
    }

    fn exec_chromium(&self, microsoft_edge: bool) -> String {
        let profile_dir = self.browser.profile_path.join(&self.codename);

        Chromium::builder(self.browser.exec.clone())
            .url(self.url.clone())
            .codename(self.codename.clone())
            .isolated(self.isolate_profile)
            .profile_path(profile_dir)
            .ms_edge(microsoft_edge)
            .private_mode(self.is_incognito)
            .custom_args(self.custom_parameters.clone())
            .build()
    }

    fn exec_falkon(&self) -> String {
        let profile_dir = self.browser.profile_path.join(&self.codename);

        Falkon::builder(self.browser.exec.clone())
            .url(self.url.clone())
            .codename(self.codename.clone())
            .isolated(self.isolate_profile)
            .profile_path(profile_dir)
            .private_mode(self.is_incognito)
            .custom_args(self.custom_parameters.clone())
            .build()
    }

    fn exec_string(&self) -> String {
        if let Some(model) = &self.browser.model {
            return match model {
                BrowserModel::Brave => self.exec_chromium(false),
                BrowserModel::Chrome => self.exec_chromium(false),
                BrowserModel::Chromium => self.exec_chromium(false),
                BrowserModel::Cromite => self.exec_chromium(false),
                BrowserModel::Falkon => self.exec_falkon(),
                BrowserModel::Firefox => self.exec_firefox(false),
                BrowserModel::Floorp => self.exec_firefox(false),
                BrowserModel::Librewolf => self.exec_firefox(false),
                BrowserModel::MicrosoftEdge => self.exec_chromium(true),
                BrowserModel::Vivaldi => self.exec_chromium(false),
                BrowserModel::Waterfox => self.exec_firefox(false),
                BrowserModel::Zen => self.exec_firefox(true),
            };
        };

        String::new()
    }

    pub async fn create(&self) -> Result<()> {
        if let Some(entry) = &self.browser.entry {
            let proxy = DynamicLauncherProxy::new().await?;

            let mut desktop_entry = String::from("[Desktop Entry]\n");
            desktop_entry.push_str("Comment=Quick Web App\n");
            desktop_entry.push_str(&format!("Exec={}\n", self.exec_string()));
            desktop_entry.push_str("Terminal=false\n");
            desktop_entry.push_str("Type=Application\n");
            desktop_entry.push_str(&format!("Categories={}\n", self.category.as_ref()));
            desktop_entry.push_str("MimeType=text/html;text/xml;application/xhtml_xml;\n");
            desktop_entry.push_str(&format!("StartupWMClass=QuickWebApp-{}\n", self.codename));
            desktop_entry.push_str("StartupNotify=true\n");
            desktop_entry.push_str(&format!("X-QWA-Codename={}\n", self.codename));
            desktop_entry.push_str(&format!("X-QWA-Browser-Id={}\n", entry.appid));
            desktop_entry.push_str(&format!("X-QWA-Url={}\n", self.url));
            desktop_entry.push_str(&format!("X-QWA-Navbar={}\n", self.navbar));
            desktop_entry.push_str(&format!("X-QWA-Private={}\n", self.is_incognito));
            desktop_entry.push_str(&format!("X-QWA-Isolated={}\n", self.isolate_profile));
            desktop_entry.push_str(&format!("X-QWA-Parameters={}\n", self.custom_parameters));

            if let Ok(mut f) = File::open(&self.icon) {
                let metadata = std::fs::metadata(&self.icon).expect("unable to read metadata");
                let mut buffer = vec![0; metadata.len() as usize];
                f.read_exact(&mut buffer).expect("buffer overflow");

                let icon = ashpd::desktop::Icon::Bytes(buffer);

                let response = proxy
                    .prepare_install(None, &self.name, icon, PrepareInstallOptions::default())
                    .await?
                    .response()?;
                let token = response.token();

                proxy
                    .install(
                        token,
                        &format!("QuickWebApp-{}.desktop", self.codename),
                        &desktop_entry,
                    )
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        let proxy = DynamicLauncherProxy::new().await?;
        proxy
            .uninstall(&format!("QuickWebApp-{}.desktop", self.codename))
            .await?;
        let profile_path = self.browser.profile_path.join(&self.codename);
        remove_dir_all(&profile_path)?;

        Ok(())
    }
}
