use std::{
    fs::{self, File},
    io::{self, BufRead, Result},
    path::PathBuf,
};
use xdg::BaseDirectories;

#[derive(Debug, Clone)]
pub struct WebAppLauncher {
    pub path: PathBuf,
    pub codename: String,
    pub web_browser: String,
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
    pub fn new(path: PathBuf, codename: String) -> Result<WebAppLauncher> {
        let file = File::open(&path)?;
        let mut web_browser = String::new();
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
                        web_browser = line.replace("X-WebApp-Browser=", "");
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

                            let launcher = WebAppLauncher::new(entry.path(), codename);
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

#[derive(Debug)]
pub enum BrowserType {
    Firefox,
    FirefoxFlatpak,
    Librewolf,
    WaterfoxFlatpak,
    Chromium,
    Epiphany,
    Falkon,
}

#[derive(Debug)]
pub struct Browser {
    _type: BrowserType,
    pub name: String,
    pub exec: String,
    test_path: PathBuf,
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
