#![allow(clippy::too_many_arguments)]

use dircpy::copy_dir;
use iced::widget::{image, svg};
use rand::{thread_rng, Rng};
use reqwest::Client;
use scraper::{Html, Selector};
use std::{
    fmt::Display,
    fs::{self, copy, create_dir_all, remove_dir_all, remove_file, File},
    io::{self, BufRead, Read, Result, Write},
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
        codename: Option<String>,
        url: String,
        icon: String,
        category: String,
        browser: Browser,
        custom_parameters: String,
        isolated: bool,
        navbar: bool,
        privatewindow: bool,
    ) -> Self {
        let codename = if let Some(codename) = codename {
            codename
        } else {
            let random_code: u16 = thread_rng().gen_range(1000..10000);
            format!("{}{}", name.replace(' ', ""), random_code)
        };
        let filename = format!("webapp-{}.desktop", codename);
        let web_browser = browser;
        let is_valid = !name.is_empty() && !icon.is_empty();
        let exec = web_browser.exec.clone();
        let isolate_profile = isolated;
        let is_incognito = privatewindow;

        let base_dir = BaseDirectories::new().expect("base directories not found");
        let mut path = base_dir.get_data_home();
        path.push("applications");
        path.push(filename);

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

    fn exec_firefox(&self, is_flatpak: bool) -> String {
        let mut profile_dir = PathBuf::new();

        if is_flatpak {
            let home = dirs::home_dir().expect("cant get home dir");
            profile_dir.push(home);
            profile_dir.push(".var/app/org.mozilla.firefox/data/ice/firefox");
        } else {
            let base_dir = BaseDirectories::new().expect("no base directories found");
            let ice_dir = base_dir.get_data_home().join("ice");
            let profiles_dir = ice_dir.join("profiles").join(&self.codename);
            profile_dir = profiles_dir.join("firefox").join(&self.codename);
        }

        let profile_path = profile_dir.join(&self.codename);
        let profile_dir = profile_dir.to_str().unwrap();

        create_dir_all(profile_dir).expect("cant create profile dir");

        // TODO: copy firefox default profile from webapp-manager
        copy_dir("firefox/profile", profile_path.clone()).expect("cant copy firefox profile dir");

        if self.navbar {
            let profile_path = profile_path.join("chrome/userChrome.css");
            copy("firefox/userChrome-with-navbar.css", profile_path)
                .expect("cannot copy userChrome.css");
        }

        let profile_path = profile_path.to_str().unwrap();
        let mut exec_string = format!(
            r#"sh -c 'XAPP_FORCE_GTKWINDOW_ICON="{}" {} --class WebApp-{} --name WebApp-{} --profile {} --no-remote "#,
            self.icon, self.exec, self.codename, self.codename, profile_path
        );

        if self.is_incognito {
            exec_string.push_str("--private-window ");
        }

        if !self.custom_parameters.is_empty() {
            exec_string.push_str(&self.custom_parameters);
        }

        exec_string.push_str(&format!(r#" "{}"'"#, &self.url));

        exec_string
    }

    fn exec_chromium(&self) -> String {
        let mut exec_string = String::new();

        if self.isolate_profile {
            let mut profile_dir = PathBuf::new();

            let base_dir = BaseDirectories::new().expect("no base directories found");
            let ice_dir = base_dir.get_data_home().join("ice");
            profile_dir.push(ice_dir.join("profiles").join(&self.codename));

            let profile_path = profile_dir.to_str().unwrap();

            exec_string = format!(
                r#"{} --app="{}" --class=WebApp-{} --name=WebApp-{} --user-data-dir={} "#,
                self.exec, self.url, self.codename, self.codename, profile_path
            );
        }

        if self.is_incognito {
            if self.web_browser.name.starts_with("Microsoft Edge") {
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

    fn exec_string(&self) -> String {
        match self.web_browser._type {
            BrowserType::Firefox => self.exec_firefox(false),
            BrowserType::FirefoxFlatpak => self.exec_firefox(true),
            BrowserType::Librewolf => todo!(),
            BrowserType::WaterfoxFlatpak => todo!(),
            BrowserType::Chromium => self.exec_chromium(),
            BrowserType::Epiphany => todo!(),
            BrowserType::Falkon => todo!(),
        }
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

        if remove_dir_all(firefox_profiles_dir).is_ok() {
            tracing::info!("Removed firefox profile directory.");

            match remove_dir_all(profiles_dir) {
                Ok(_) => tracing::info!("Removed profiles directories."),
                Err(_) => tracing::info!("Trying remove profiles directories."),
            }
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
        Err(_) => {
            create_dir_all(apps_dir()).expect("Cannot create local applications dir");
        }
    }

    webapps
}

use crate::{
    gui::{Icon, IconType},
    supported_browsers::supported_browsers,
};

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
    pub _type: BrowserType,
    pub name: String,
    pub exec: String,
    test: PathBuf,
}

impl Display for Browser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Browser {
    pub fn new(_type: BrowserType, name: &str, exec: &str, test_path: &str) -> Self {
        let name = name.to_string();
        let exec = exec.to_string();

        let mut test = PathBuf::new();

        if test_path.starts_with(".local/share/") {
            let base = BaseDirectories::new().expect("base directories not found");
            let mut data_home = base.get_data_home();
            let flatpak_path: Vec<_> = test_path.split(".local/share/").collect();
            data_home.push(flatpak_path[1]);

            test.push(data_home)
        } else {
            test.push(test_path)
        }

        Self {
            _type,
            name,
            exec,
            test,
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
        let exists = browser.test.as_path().try_exists();

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

pub fn get_icon_name_from_url(url: String) -> String {
    match Url::parse(&url) {
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

pub async fn find_icon(path: &str, icon_name: &str) -> Vec<String> {
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

    icons
}

pub async fn find_icons(icon_name: String, url: Option<String>) -> Vec<String> {
    let base_dir = BaseDirectories::new().expect("no base directories found");
    let mut local_dir = base_dir.get_data_home();
    local_dir.push("icons");
    let local_dir = local_dir
        .to_str()
        .expect("cant convert local path to string");
    let system_dir = "/usr/share/icons";
    let mut home_dir = dirs::home_dir().expect("cant get home directory");
    home_dir.push(".icons");
    let home_dir = home_dir.to_str().expect("cant convert path to string");

    let home_icons = find_icon(home_dir, &icon_name).await;
    let local_icons = find_icon(local_dir, &icon_name).await;
    let system_icons = find_icon(system_dir, &icon_name).await;

    let mut result: Vec<String> = Vec::new();

    result.extend(home_icons);
    result.extend(local_icons);
    result.extend(system_icons);

    if let Some(u) = url {
        if u.starts_with("http") {
            if let Ok(data) = download_favicon(&u).await {
                result.extend(data)
            }
        }
    }

    result
}

pub async fn download_favicon(url: &str) -> Result<Vec<String>> {
    let mut favs = Vec::new();

    let content = Client::new()
        .get(url)
        .send()
        .await
        .expect("sending request")
        .text()
        .await
        .expect("getting content");

    let document = Html::parse_document(&content);
    let head = Selector::parse("head").unwrap();
    let link = Selector::parse("link").unwrap();
    let meta = Selector::parse("meta").unwrap();

    for head in document.select(&head) {
        let fragment = Html::parse_document(&head.html());

        for link in fragment.select(&link) {
            if link.attr("rel") == Some("icon") {
                let val = link.value().attr("href").unwrap();

                favs.push(val.to_string());
            }
        }

        for meta in fragment.select(&meta) {
            if meta.value().attr("property") == Some("og:image") {
                let val = meta.value().attr("content").unwrap();

                favs.push(val.to_string());
            }
        }
    }

    Ok(favs)
}

pub fn move_icon(path: String, output_name: String) -> Result<String> {
    let base_dir = BaseDirectories::new().expect("not found base directories");
    let mut icons_folder = base_dir.get_data_home();
    icons_folder.push("ice/icons");

    create_dir_all(&icons_folder).expect("cant create icons folder");

    let ext = if path.ends_with(".svg") {
        ".svg"
    } else {
        ".png"
    };

    let save_path = icons_folder
        .join(format!("{}{}", output_name.replace(' ', ""), ext))
        .to_str()
        .unwrap()
        .to_string();

    if path.starts_with("http") {
        let response = reqwest::blocking::get(path).expect("sending request");

        if response.status().is_success() {
            let content = response.bytes().expect("getting image bytes");

            let mut file = File::create(&save_path).expect("creating file");
            file.write_all(&content).expect("saving image");
        }
    } else if !path.contains(&save_path) {
        copy(&path, &save_path).expect("saving image");
    }

    Ok(save_path)
}

pub async fn image_from_memory(path: String) -> Result<Icon> {
    let img_bytes = if path.starts_with("http") {
        Client::new()
            .get(path.clone())
            .send()
            .await
            .expect("sending request")
            .bytes()
            .await
            .expect("getting content")
            .to_vec()
    } else {
        let mut file = File::open(path.clone())?;
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)?;

        buffer.to_vec()
    };

    let icon = image::Handle::from_memory(img_bytes.to_vec());

    Ok(Icon::new(IconType::Raster(icon), path))
}

pub async fn svg_from_memory(path: String) -> Result<Icon> {
    let img_bytes = if path.starts_with("http") {
        Client::new()
            .get(path.clone())
            .send()
            .await
            .expect("sending request")
            .bytes()
            .await
            .expect("getting content")
            .to_vec()
    } else {
        let mut file = File::open(path.clone())?;
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)?;

        buffer.to_vec()
    };

    let icon = svg::Handle::from_memory(img_bytes.to_vec());

    Ok(Icon::new(IconType::Svg(icon), path))
}
