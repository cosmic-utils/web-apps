#![allow(clippy::too_many_arguments)]

use rand::{thread_rng, Rng};
use std::{
    fmt::Display,
    fs::{self, remove_dir_all, remove_file, File},
    io::{self, BufRead, Result, Write},
    path::PathBuf,
};
use url::Url;
use walkdir::WalkDir;
use xdg::BaseDirectories;

#[derive(Debug, Clone)]
pub struct WebAppLauncher {
    pub path: PathBuf,
    pub codename: String,
    pub web_browser: Browser,
    pub name: String,
    pub icon: String,
    pub is_valid: bool,
    pub exec: String,
    pub category: String,
    pub url: String,
    pub custom_parameters: String,
    pub isolate_profile: bool,
    pub navbar: bool,
    pub is_incognito: bool,
}

impl WebAppLauncher {
    pub fn new(
        name: String,
        url: String,
        icon: String,
        category: String,
        browser: Browser,
        custom_parameters: String,
        isolated: bool,
        navbar: bool,
        privatewindow: bool,
    ) -> Self {
        let random_code: u16 = thread_rng().gen_range(1000..10000);
        let base_dir = BaseDirectories::new().expect("base directories not found");
        let mut path = base_dir.get_data_home();

        let codename = format!("{}{}", name, random_code);
        path.push("applications");
        let filename = format!("webapp-{}.desktop", codename);
        path.push(filename);
        let web_browser = browser;
        let is_valid = !name.is_empty() && !icon.is_empty();
        let exec = String::new(); // TODO: Implement this exec_string
        let isolate_profile = isolated;
        let is_incognito = privatewindow;

        Self {
            path,
            codename,
            web_browser,
            name,
            icon,
            is_valid,
            exec,
            category,
            url,
            custom_parameters,
            isolate_profile,
            navbar,
            is_incognito,
        }
    }

    pub fn read(path: PathBuf, codename: String) -> Result<WebAppLauncher> {
        let file = File::open(&path)?;
        let mut browser_name = String::new();
        let mut name = String::new();
        let mut icon = String::new();
        let mut is_valid = false;
        let mut exec = String::new();
        let mut category = String::new();
        let mut url = String::new();
        let mut custom_parameters = String::new();
        let mut isolate_profile = false;
        let mut navbar = false;
        let mut is_incognito = false;

        let reader = io::BufReader::new(file);

        let mut is_webapp = false;

        for line_result in reader.lines() {
            match line_result {
                Ok(line) => {
                    if line.contains("StartupWMClass=WebApp")
                        || line.contains("StartupWMClass=Chromium")
                        || line.contains("StartupWMClass=ICE-SSB")
                    {
                        is_webapp = true;
                    };

                    if line.contains("Name=") {
                        name = line.replace("Name=", "");
                    };

                    if line.contains("Icon=") {
                        icon = line.replace("Icon=", "");
                    };

                    if line.contains("Exec=") {
                        exec = line.replace("Exec=", "");
                    };

                    if line.contains("Categories=") {
                        category = line
                            .replace("Categories=", "")
                            .replace("GTK;", "")
                            .replace(';', "");
                    };

                    if line.contains("X-WebApp-Browser=") {
                        browser_name = line.replace("X-WebApp-Browser=", "");
                    };

                    if line.contains("X-WebApp-URL=") {
                        url = line.replace("X-WebApp-URL=", "");
                    };

                    if line.contains("X-WebApp-CustomParameters=") {
                        custom_parameters = line.replace("X-WebApp-CustomParameters=", "");
                    };

                    if line.contains("X-WebApp-Isolated=") {
                        isolate_profile = line.replace("X-WebApp-Isolated=", "") == "true"
                    };

                    if line.contains("X-WebApp-Navbar=") {
                        navbar = line.replace("X-WebApp-Navbar=", "") == "true"
                    };

                    if line.contains("X-WebApp-PrivateWindow=") {
                        is_incognito = line.replace("X-WebApp-PrivateWindow=", "") == "true"
                    };
                }
                Err(e) => eprintln!("Error reading line: {}", e),
            }
        }

        if is_webapp && !name.is_empty() && !icon.is_empty() {
            is_valid = true
        }

        let web_browser = Browser::web_browser(browser_name);

        match web_browser {
            Some(web_browser) => Ok(WebAppLauncher {
                path,
                codename,
                web_browser,
                name,
                icon,
                is_valid,
                exec,
                category,
                url,
                custom_parameters,
                isolate_profile,
                navbar,
                is_incognito,
            }),
            None => {
                let supported = get_supported_browsers();
                let web_browser = supported[0].clone();

                Ok(WebAppLauncher {
                    path,
                    codename,
                    web_browser,
                    name,
                    icon,
                    is_valid,
                    exec,
                    category,
                    url,
                    custom_parameters,
                    isolate_profile,
                    navbar,
                    is_incognito,
                })
            }
        }
    }

    pub fn exec_string(&self) -> String {
        self.exec.to_string()
    }

    pub fn create(&self) -> Result<()> {
        let mut output = File::create(&self.path)?;

        writeln!(output, "[Desktop Entry]")?;
        writeln!(output, "Version=1.0")?;
        writeln!(output, "Name={}", self.name)?;
        writeln!(output, "Comment=Web App")?;
        writeln!(output, "Exec={}", self.exec_string())?;
        writeln!(output, "Terminal=false")?;
        writeln!(output, "Type=Application")?;
        writeln!(output, "Icon={}", self.icon)?;
        writeln!(output, "Categories=GTK;{};", self.category)?;
        writeln!(output, "MimeType=text/html;text/xml;application/xhtml_xml;")?;
        writeln!(output, "StartupWMClass=WebApp-{}", self.codename)?;
        writeln!(output, "StartupNotify=true")?;
        writeln!(output, "X-MultipleArgs=false")?;
        writeln!(output, "X-WebApp-Browser={}", self.web_browser.name)?;
        writeln!(output, "X-WebApp-URL={}", self.url)?;
        writeln!(output, "X-WebApp-Navbar={}", self.navbar)?;
        writeln!(output, "X-WebApp-PrivateWindow={}", self.is_incognito)?;
        writeln!(output, "X-WebApp-Isolated={}", self.isolate_profile)?;
        writeln!(
            output,
            "X-WebApp-CustomParameters={}",
            self.custom_parameters
        )?;

        Ok(())
    }

    pub fn delete(&self) -> Result<()> {
        let base_dir = BaseDirectories::new().expect("no base directories found");
        let ice_dir = base_dir.get_data_home().join("ice");
        let profiles_dir = ice_dir.join("profiles").join(&self.codename);
        let firefox_profiles_dir = profiles_dir.join("firefox").join(&self.codename);

        let exist = self.path.as_path().exists();

        match exist {
            true => {
                remove_file(&self.path)?;
            }
            false => {
                tracing::error!("file not found");
            }
        }

        match remove_dir_all(firefox_profiles_dir) {
            Ok(_) => {
                tracing::info!("Removed firefox profile directory.");

                match remove_dir_all(profiles_dir) {
                    Ok(_) => tracing::info!("Removed profiles directories."),
                    Err(_) => tracing::info!("Trying remove profiles directories."),
                }
            }
            Err(_) => tracing::info!("Trying remove firefox profile directory."),
        }

        // TODO: Implement epiphany deletion

        Ok(())
    }
}

fn apps_dir() -> PathBuf {
    let base = BaseDirectories::new().expect("base directories not found");
    let mut data_home = base.get_data_home();

    data_home.push("applications");

    data_home
}

pub fn get_webapps() -> Vec<Result<WebAppLauncher>> {
    let mut webapps = Vec::new();

    match fs::read_dir(apps_dir()) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let entry_fn = entry.file_name();
                        let filename = entry_fn.to_str().unwrap();

                        if filename.starts_with("webapp-") && filename.ends_with(".desktop") {
                            let codename = filename.replace("webapp-", "").replace(".desktop", "");

                            let launcher = WebAppLauncher::read(entry.path(), codename);
                            webapps.push(launcher);
                        }
                    }
                    Err(e) => tracing::error!("Error reading directory: {}", e),
                }
            }
        }
        Err(e) => tracing::error!("Error opening directory: {}", e),
    }

    webapps
}

use crate::supported_browsers::supported_browsers;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserType {
    Firefox,
    FirefoxFlatpak,
    Librewolf,
    WaterfoxFlatpak,
    Chromium,
    Epiphany,
    Falkon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Browser {
    _type: BrowserType,
    pub name: String,
    pub exec: String,
    test_path: PathBuf,
}

impl Display for Browser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Browser {
    pub fn new(_type: BrowserType, name: &str, exec: &str, test: &str) -> Self {
        let name = name.to_string();
        let exec = exec.to_string();

        let mut test_path = PathBuf::new();

        if test.starts_with(".local/share/") {
            let base = BaseDirectories::new().expect("base directories not found");
            let mut data_home = base.get_data_home();
            let flatpak_path: Vec<_> = test.split(".local/share/").collect();
            data_home.push(flatpak_path[1]);

            test_path = data_home;
        }

        Self {
            _type,
            name,
            exec,
            test_path,
        }
    }

    pub fn web_browser(name: String) -> Option<Browser> {
        let supported = get_supported_browsers();
        supported.into_iter().find(|b| b.name == name)
    }
}

pub fn get_supported_browsers() -> Vec<Browser> {
    let test_browsers = supported_browsers();

    let mut browsers = Vec::new();

    for browser in test_browsers {
        let exists = browser.test_path.as_path().try_exists();

        match exists {
            Ok(found) => match found {
                true => browsers.push(browser),
                false => continue,
            },
            Err(_) => continue,
        }
    }

    browsers
}

pub fn get_icon_name_from_url(url: &str) -> String {
    match Url::parse(url) {
        Ok(url) => match url.host_str() {
            Some(host) => {
                let parts: Vec<&str> = host.split('.').collect();
                parts[parts.len() - 2].to_string()
            }
            None => String::new(),
        },
        Err(_) => String::new(),
    }
}

pub fn find_icon(path: &str, icon_name: &str) -> Option<Vec<String>> {
    let mut icons: Vec<String> = Vec::new();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if let Some(filename) = entry.file_name().to_str() {
            if filename.contains(icon_name) {
                if let Some(path) = entry.path().to_str() {
                    icons.push(path.to_string())
                }
            }
        }
    }
    Some(icons)
}

pub fn find_icons(icon_name: &str) -> Option<Vec<String>> {
    let base_dir = BaseDirectories::new().expect("no base directories found");
    let mut local_dir = base_dir.get_data_home();
    local_dir.push("icons");
    let local_dir = local_dir
        .to_str()
        .expect("cant convert local path to string");
    let system_dir = "/usr/share/icons";

    let local_icons = find_icon(local_dir, icon_name).unwrap();
    let system_icons = find_icon(system_dir, icon_name).unwrap();

    let mut result: Vec<String> = Vec::new();

    result.extend(local_icons);
    result.extend(system_icons);

    Some(result)
}
