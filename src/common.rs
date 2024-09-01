#![allow(clippy::too_many_arguments)]

use std::{
    ffi::OsStr,
    fs::{self, copy, create_dir_all, remove_dir_all, remove_file, File},
    io::{self, BufRead, Cursor, Read, Write},
    path::PathBuf,
    str::FromStr,
    sync::Mutex,
};

use anyhow::{anyhow, Error, Result};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use bytes::Bytes;
use cosmic::widget;
use image::GenericImageView;
use image::ImageReader;
use rand::{thread_rng, Rng};
use reqwest::Client;
use scraper::{Html, Selector};
use svg::node::element::Image;
use svg::Document;
use url::Url;
use walkdir::WalkDir;

use crate::{
    fl,
    icon_cache::IconCache,
    pages,
    supported_browsers::{flatpak_browsers, native_browsers, nix_browsers},
};

lazy_static::lazy_static! {
    static ref ICON_CACHE: Mutex<IconCache> = Mutex::new(IconCache::new());
}

pub fn icon_cache_get(name: &'static str, size: u16) -> widget::icon::Icon {
    let mut icon_cache = ICON_CACHE.lock().unwrap();
    icon_cache.get(name, size)
}

pub fn url_valid(url: &str) -> bool {
    Url::parse(url).is_ok()
}

pub fn is_svg(path: &str) -> bool {
    !url_valid(path) && PathBuf::from_str(path).unwrap().extension() == Some(OsStr::new("svg"))
}

pub fn home_dir() -> PathBuf {
    let home = std::env::var("HOME");

    if let Ok(path) = home {
        return PathBuf::from_str(&path).unwrap();
    }

    PathBuf::new()
}

pub fn desktop_filepath(filename: &str) -> PathBuf {
    let mut home = home_dir();
    home.push(".local/share/applications");
    home.join(filename)
}

pub fn icons_location() -> PathBuf {
    home_dir().join(".local/share/icons")
}

pub fn system_fonts() -> PathBuf {
    if let Ok(path) = PathBuf::from_str("/usr/share/icons") {
        path
    } else {
        PathBuf::new()
    }
}

pub fn my_icons_location() -> PathBuf {
    icons_location().join("QuickWebApps")
}

pub fn webapplauncher_is_valid(
    webbrowser: &Browser,
    icon: &str,
    codename: &str,
    name: &str,
    url: &str,
) -> bool {
    let installed = get_webapps();

    for app in installed.iter().flatten() {
        if !url_valid(url)
            || !webbrowser.is_installed()
            || (name.is_empty() || app.name == name)
            || icon.is_empty()
            || (codename.is_empty() || app.codename == codename)
            || url.is_empty()
        {
            return false;
        }
    }

    true
}

#[derive(Debug, Clone)]
pub struct WebAppLauncher {
    pub path: PathBuf,
    pub codename: String,
    pub web_browser: Browser,
    pub name: String,
    pub icon: String,
    pub is_valid: bool,
    pub exec: String,
    // pub args: Vec<String>,
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
        let path = desktop_filepath(&filename);
        let web_browser = browser;
        let exec = web_browser.exec.clone();
        // let args = Vec::new();
        let isolate_profile = isolated;
        let is_incognito = privatewindow;

        let is_valid = webapplauncher_is_valid(&web_browser, &icon, &codename, &name, &url);

        Self {
            path,
            codename,
            web_browser,
            name,
            icon,
            is_valid,
            exec,
            // args,
            category,
            url,
            custom_parameters,
            isolate_profile,
            navbar,
            is_incognito,
        }
    }

    pub fn read(path: PathBuf, codename: String) -> Result<WebAppLauncher, Error> {
        let file = File::open(&path)?;
        let mut browser_name = String::new();
        let mut name = String::new();
        let mut icon = String::new();
        let mut is_valid = false;
        let mut exec = String::new();
        let mut args = Vec::new();
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
            Some(web_browser) => {
                exec.split(' ').enumerate().for_each(|(n, arg)| {
                    if n > 0 && !arg.is_empty() {
                        args.push(arg.to_string())
                    }
                });

                Ok(WebAppLauncher {
                    path,
                    codename,
                    web_browser,
                    name,
                    icon,
                    is_valid,
                    exec,
                    // args,
                    category,
                    url,
                    custom_parameters,
                    isolate_profile,
                    navbar,
                    is_incognito,
                })
            }
            None => Err(anyhow!("Cannot read web app launcher.")),
        }
    }

    fn create_firefox_userjs(&self, is_zen_browser: bool, path: PathBuf) -> bool {
        let content = match is_zen_browser {
            true => include_bytes!("../data/runtime/zen-browser/profile/user.js"),
            false => include_bytes!("../data/runtime/firefox/profile/user.js"),
        };

        let mut file = File::create(&path)
            .unwrap_or_else(|_| panic!("failed to create user.js in {:?}", path));

        file.write_all(content).is_ok()
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
        let profile_path = self.web_browser.profile_path.join(&self.codename);
        let user_js_path = profile_path.join("user.js");
        let mut user_chrome_css = profile_path.join("chrome");

        tracing::info!("Creating profile directory in: {:?}", &profile_path);
        create_dir_all(&profile_path)
            .unwrap_or_else(|_| panic!("cant create profile dir in {:?}", &profile_path));
        create_dir_all(&user_chrome_css)
            .unwrap_or_else(|_| panic!("cant create chrome dir in {:?}", &user_chrome_css));

        user_chrome_css = user_chrome_css.join("userChrome.css");

        self.create_firefox_userjs(is_zen_browser, user_js_path);
        self.create_user_chrome_css(is_zen_browser, user_chrome_css, self.navbar);

        let profile_path = profile_path.to_str().unwrap();

        let mut exec_string = format!(
            "{} --class WebApp-{} --name WebApp-{} --profile {} --no-remote ",
            self.exec, self.codename, self.codename, profile_path
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
            "{} --app={} --class=WebApp-{} --name=WebApp-{} ",
            self.exec, self.url, self.codename, self.codename
        );

        if self.isolate_profile {
            let profile_dir = self.web_browser.profile_path.join(&self.codename);

            tracing::info!("Creating profile directory in: {:?}", &profile_dir);
            let _ = create_dir_all(&profile_dir);
            let profile_path = profile_dir.to_str().unwrap();
            exec_string.push_str(&format!("--user-data-dir={} ", profile_path));
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

    fn exec_falkon(&self) -> String {
        let mut exec_string = String::new();

        if self.isolate_profile {
            let profile_dir = self.web_browser.profile_path.join(&self.codename);
            tracing::info!("Creating profile directory in: {:?}", &profile_dir);
            let _ = create_dir_all(&profile_dir);

            let profile_path = profile_dir.to_str().unwrap();

            exec_string = format!(
                "{} --portable --wmclass WebApp-{} --profile {} ",
                self.exec, self.codename, profile_path
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
        match self.web_browser._type {
            BrowserType::Firefox => self.exec_firefox(false),
            BrowserType::FirefoxFlatpak => self.exec_firefox(false),
            BrowserType::ZenFlatpak => self.exec_firefox(true),
            BrowserType::Chromium => self.exec_chromium(),
            BrowserType::ChromiumFlatpak => self.exec_chromium(),
            BrowserType::Falkon => self.exec_falkon(),
            BrowserType::FalkonFlatpak => self.exec_falkon(),
            BrowserType::NoBrowser => String::new(),
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
        let exist = self.path.as_path().exists();

        match exist {
            true => {
                remove_file(&self.path)?;
            }
            false => {
                tracing::error!("file not found");
            }
        }

        let profile_path = self.web_browser.profile_path.join(&self.codename);

        if remove_dir_all(&profile_path).is_ok() {
            tracing::info!(
                "Removed profile directory, from: {}",
                profile_path.to_str().unwrap()
            );
        };

        Ok(())
    }
}

pub fn get_webapps() -> Vec<Result<WebAppLauncher>> {
    let mut webapps = Vec::new();

    match fs::read_dir(desktop_filepath("")) {
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
            create_dir_all(desktop_filepath("")).expect("Cannot create local applications dir");
        }
    }

    webapps
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserType {
    NoBrowser,
    Firefox,
    FirefoxFlatpak,
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

        let base = home_dir();
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

    let native_browsers = native_browsers();
    let flatpak_browsers = flatpak_browsers();
    let nix_browsers: Vec<Browser> = nix_browsers();

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

pub async fn find_icon(path: PathBuf, icon_name: String) -> Vec<String> {
    let mut icons: Vec<String> = Vec::new();

    for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
        if let Some(filename) = entry.file_name().to_str() {
            if filename.contains(&icon_name) {
                if is_svg(filename) {
                    if let Some(path) = entry.path().to_str() {
                        if let Ok(buffer) = tokio::fs::read_to_string(&mut path.to_string()).await {
                            let options = usvg::Options::default();
                            if let Ok(parsed) = usvg::Tree::from_str(&buffer, &options) {
                                let size = parsed.size();
                                if size.width() >= 64.0
                                    && size.height() >= 64.0
                                    && !icons.contains(&path.to_string())
                                {
                                    icons.push(path.to_string())
                                }
                            }
                        }
                    }
                } else if let Some(path) = entry.path().to_str() {
                    let image = ImageReader::open(path).unwrap().decode();

                    if let Ok(img) = image {
                        if img.width() >= 64
                            && img.height() >= 64
                            && !icons.contains(&path.to_string())
                        {
                            icons.push(path.to_string())
                        }
                    }
                }
            }
        }
    }

    icons
}

pub async fn find_icons(icon_name: String, url: String) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();

    result.extend(find_icon(icons_location(), icon_name.clone()).await);
    result.extend(find_icon(system_fonts(), icon_name).await);

    if url_valid(&url) {
        if let Ok(data) = download_favicon(&url).await {
            result.extend(data)
        }
    };

    result
}

pub async fn download_favicon(url: &str) -> Result<Vec<String>> {
    let mut favicons = Vec::new();

    if let Ok(request) = Client::new().get(url).send().await {
        if let Ok(content) = request.text().await {
            let document = Html::parse_document(&content);
            let head = Selector::parse("head").unwrap();
            let link = Selector::parse("link").unwrap();
            let meta = Selector::parse("meta").unwrap();

            for head in document.select(&head) {
                let fragment = Html::parse_document(&head.html());

                for link in fragment.select(&link) {
                    if link.attr("rel") == Some("icon") {
                        let val = link.value().attr("href").unwrap();

                        favicons.push(val.to_string());
                    }
                }

                for meta in fragment.select(&meta) {
                    if meta.value().attr("property") == Some("og:image") {
                        let val = meta.value().attr("content").unwrap();

                        favicons.push(val.to_string());
                    }
                }
            }
        }
    }

    Ok(favicons)
}

pub fn convert_raster_to_svg_format(img_slice: Bytes, save_path: &str) -> Result<()> {
    let encoded_img = BASE64_STANDARD.encode(&img_slice);

    if let Ok(image) = image::load_from_memory(&img_slice) {
        let (width, height) = image.dimensions();

        // Create an SVG document and embed the image
        let image_element = Image::new()
            .set("x", 0)
            .set("y", 0)
            .set("width", width)
            .set("height", height)
            .set("href", format!("data:image/png;base64,{}", encoded_img));

        let document = Document::new()
            .set("width", width)
            .set("height", height)
            .add(image_element);

        // Save the SVG document
        svg::save(save_path, &document)?;
    }

    Ok(())
}

pub fn move_icon(path: String, output_name: String) -> String {
    create_dir_all(my_icons_location()).expect("cant create folder for your icons");

    let save_path = my_icons_location()
        .join(format!("{}.svg", output_name.replace(' ', "")))
        .to_str()
        .unwrap()
        .to_string();

    if url_valid(&path) {
        let response = reqwest::blocking::get(&path).expect("sending request");

        if response.status().is_success() {
            let content: Bytes = response.bytes().expect("getting image bytes");

            let _ = convert_raster_to_svg_format(content, &save_path);

            // let mut file = File::create(&save_path).expect("creating file");
            // file.write_all(&content).expect("saving image");
        }
    } else if !path.contains(&save_path) {
        if !is_svg(&path) {
            let file = File::open(&path);
            if let Ok(mut opened) = file {
                let mut buffer = Vec::new();
                opened.read_to_end(&mut buffer).unwrap();

                let content = Bytes::from(buffer);

                let _ = convert_raster_to_svg_format(content, &save_path);
            }
        } else {
            copy(&path, &save_path).unwrap();
        }
    }

    save_path
}

pub async fn image_handle(path: String) -> Option<pages::iconpicker::Icon> {
    let mut data: Vec<_> = Vec::new();
    let pathbuf = PathBuf::from_str(&path).unwrap();

    if url_valid(&path) {
        data.extend(
            Client::new()
                .get(&path)
                .send()
                .await
                .unwrap()
                .bytes()
                .await
                .unwrap()
                .to_vec(),
        );
    } else if let Ok(mut file) = File::open(&pathbuf) {
        let mut buffer = Vec::new();

        if pathbuf.is_file() {
            file.read_to_end(&mut buffer).unwrap();
        };

        data.extend(buffer);
    };

    if is_svg(&path) {
        let handle = widget::svg::Handle::from_memory(data);

        return Some(pages::iconpicker::Icon::new(
            pages::iconpicker::IconType::Svg(handle),
            path,
        ));
    } else if let Ok(image) = ImageReader::new(Cursor::new(&data))
        .with_guessed_format()
        .unwrap()
        .decode()
    {
        if image.width() >= 96 && image.height() >= 96 {
            let handle = widget::image::Handle::from_memory(data);

            return Some(pages::iconpicker::Icon::new(
                pages::iconpicker::IconType::Raster(handle),
                path,
            ));
        }
    };

    None
}
