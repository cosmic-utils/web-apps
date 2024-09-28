use std::path::PathBuf;

use crate::{common, fl, supported_browsers};

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
    pub test: PathBuf,
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
        name: &str,
        exec: &str,
        test_path: &str,
        profile_path: &str,
    ) -> Self {
        let name = name.to_string();

        let mut test = PathBuf::new();
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

        if test_path.starts_with(".local/share") {
            let flatpak_path: Vec<&str> = test_path.split(".local/share/").collect();
            let path = data_home.join(flatpak_path[1]);
            test.push(path);
        } else {
            test.push(test_path)
        }

        let exec = exe_path.to_str().unwrap().to_string();

        let profile_path = base.join(profile_path);

        Self {
            _type,
            name,
            exec,
            test,
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

pub fn get_supported_browsers() -> Vec<Browser> {
    let mut test_browsers: Vec<Browser> = Vec::new();

    let native_browsers: Vec<Browser> = supported_browsers::native_browsers();
    let flatpak_browsers: Vec<Browser> = supported_browsers::flatpak_browsers();
    let nix_browsers: Vec<Browser> = supported_browsers::nix_browsers();

    test_browsers.extend(native_browsers);
    test_browsers.extend(flatpak_browsers);
    test_browsers.extend(nix_browsers);

    let mut browsers = Vec::new();

    for browser in test_browsers {
        let exists = browser.test.as_path().try_exists();

        match exists {
            Ok(found) => match found {
                true => browsers.push(browser),
                false => continue,
            },
            Err(_) => continue,
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
