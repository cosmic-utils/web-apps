use clap::Parser;
use cosmic::desktop::fde::get_languages_from_env;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fmt::Display,
    fs::{self, copy, create_dir_all, File},
    io::{Cursor, Read},
    path::PathBuf,
    str::FromStr,
    vec::IntoIter,
};

use base64::prelude::*;
use bytes::Bytes;
use cosmic::{iced_core, widget};
use image::ImageReader;
use image::{load_from_memory, GenericImageView};
use svg::node::element::Image;
use svg::Document;
use url::Url;
use walkdir::WalkDir;

pub mod favicon;
pub mod localize;

lazy_static::lazy_static! {
    pub static ref LOCALES: Vec<String> = get_languages_from_env();
}

pub const ICON_SIZE: u32 = 42;
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
pub const CONFIG_VERSION: u64 = 1;
pub const APP_ID: &str = "dev.heppen.QuickWebApps.Manager";
pub const APP_ICON: &[u8] =
    include_bytes!("../res/icons/hicolor/256x256/apps/dev.heppen.webapps.png");
pub const WEBVIEW_APP_ID: &str = "dev.heppen.QuickWebApp";
pub const MOBILE_UA: &str = "Mozilla/5.0 (Android 16; Mobile; rv:68.0) Gecko/68.0 Firefox/142.0";

pub fn url_valid(url: &str) -> bool {
    Url::parse(url).is_ok()
}

pub fn is_svg(path: &str) -> bool {
    if !url_valid(path) {
        let Ok(pb) = PathBuf::from_str(path);

        if pb.extension() == Some(OsStr::new("svg")) {
            return true;
        }
    }
    false
}

pub fn themes_path(theme_file: &str) -> PathBuf {
    if let Some(xdg_data) = dirs::data_dir() {
        let path = xdg_data.join("quick-webapps/themes");

        if !path.exists() {
            create_dir_all(&path).unwrap();
        }

        return path.join(theme_file);
    }

    PathBuf::new()
}

pub fn database_path(entry: &str) -> PathBuf {
    if let Some(xdg_data) = dirs::data_dir() {
        let path = xdg_data.join("quick-webapps/database");

        if !path.exists() {
            create_dir_all(&path).unwrap();
        }

        return path.join(entry);
    }

    PathBuf::new()
}

pub fn desktop_files_location(filename: &str) -> PathBuf {
    if let Some(xdg_data) = dirs::data_dir() {
        let dir = xdg_data.join("applications");

        if !dir.exists() {
            let _ = create_dir_all(&dir);
        }

        return dir.join(format!("{filename}.desktop"));
    }

    PathBuf::new()
}

pub fn icons_location() -> PathBuf {
    if let Some(home_dir) = dirs::home_dir() {
        return home_dir.join(".local/share/icons");
    }

    PathBuf::new()
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
                                if size.width() >= ICON_SIZE as f32
                                    && size.height() >= ICON_SIZE as f32
                                    && !icons.contains(&path.to_string())
                                {
                                    icons.push(path.to_string())
                                }
                            }
                        }
                    }
                } else if let Some(path) = entry.path().to_str() {
                    if let Ok(image) = ImageReader::open(path) {
                        if let Ok(img) = image.decode() {
                            if img.width() >= ICON_SIZE
                                && img.height() >= ICON_SIZE
                                && !icons.contains(&path.to_string())
                            {
                                icons.push(path.to_string())
                            }
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

    if url_valid(&url) {
        if let Ok(data) = crate::favicon::download_favicon(&url).await {
            result.extend(data)
        }
    };

    result.extend(find_icon(icons_location(), icon_name.clone()).await);

    result
}

pub fn convert_raster_to_svg_format(img_slice: Bytes, icon_name: &str) -> String {
    let save_path = icon_save_path(icon_name);

    if let Ok(data) = load_from_memory(&img_slice) {
        let (width, height) = data.dimensions();
        let mut image_buffer = Vec::new();
        let mut image_cursor = Cursor::new(&mut image_buffer);

        data.write_to(&mut image_cursor, image::ImageFormat::Png)
            .unwrap();

        let encoded_img = BASE64_STANDARD.encode(image_buffer);

        // Create an SVG document and embed the image
        let image_element = Image::new()
            .set("x", 0)
            .set("y", 0)
            .set("width", width)
            .set("height", height)
            .set("href", format!("data:image/png;base64,{encoded_img}"));

        let document = Document::new()
            .set("width", width)
            .set("height", height)
            .add(image_element);

        // Save the SVG document

        let _ = svg::save(&save_path, &document).is_ok();
    }

    save_path
}

fn icon_save_path(icon_name: &str) -> String {
    icons_location()
        .join(format!("{icon_name}.svg"))
        .to_str()
        .unwrap()
        .to_string()
}

pub async fn move_icon(path: &str, output_name: &str) -> String {
    create_dir_all(icons_location()).expect("cant create folder for your icons");

    let icon_name = output_name.replace(' ', "");

    let Ok(p) = PathBuf::from_str(&icon_save_path(&icon_name));

    if p.exists() {
        std::fs::remove_file(p).unwrap();
    }

    if url_valid(path) {
        let response = reqwest::get(path).await.expect("sending request");

        if response.status().is_success() {
            let content: Bytes = response.bytes().await.expect("getting image bytes");

            return convert_raster_to_svg_format(content, &icon_name);
        }

        return String::new();
    };

    if !is_svg(path) {
        if let Ok(mut file) = File::open(path) {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            let content = Bytes::from(buffer);

            return convert_raster_to_svg_format(content, &icon_name);
        }
    };

    let save_path = icon_save_path(&icon_name);
    let _ = copy(path, &save_path);

    save_path
}

pub async fn image_handle(path: String) -> Option<Icon> {
    if url_valid(&path) {
        if let Ok(response) = reqwest::Client::new().get(&path).send().await {
            if let Ok(bytes) = response.bytes().await {
                let options = usvg::Options::default();
                if let Ok(parsed) = usvg::Tree::from_data(&bytes, &options) {
                    let size = parsed.size();
                    if size.width() >= 96.0 && size.height() >= 96.0 {
                        let handle = widget::svg::Handle::from_memory(bytes.to_vec());
                        return Some(Icon::new(IconType::Svg(handle), path, true));
                    }
                }
                if let Ok(image_reader) =
                    ImageReader::new(Cursor::new(&bytes)).with_guessed_format()
                {
                    if let Ok(image) = image_reader.decode() {
                        if image.width() >= ICON_SIZE && image.height() >= ICON_SIZE {
                            let handle = iced_core::image::Handle::from_bytes(bytes);
                            return Some(Icon::new(IconType::Raster(handle), path, true));
                        }
                    };
                }
            }
        }
    };

    let Ok(result_path) = PathBuf::from_str(&path);

    if result_path.is_file() {
        if is_svg(&path) {
            let handle = widget::svg::Handle::from_path(&result_path);

            return Some(Icon::new(IconType::Svg(handle), path, false));
        } else {
            let mut data: Vec<_> = Vec::new();

            if let Ok(mut file) = fs::File::open(&result_path) {
                let _ = file.read_to_end(&mut data);
            }

            if let Ok(image_reader) = ImageReader::new(Cursor::new(&data)).with_guessed_format() {
                if let Ok(image) = image_reader.decode() {
                    if image.width() >= ICON_SIZE && image.height() >= ICON_SIZE {
                        let handle = iced_core::image::Handle::from_bytes(data);

                        return Some(Icon::new(IconType::Raster(handle), path, false));
                    }
                };
            }
        }
    };

    None
}

#[derive(Debug, Clone, PartialEq)]
pub enum IconType {
    Raster(widget::image::Handle),
    Svg(widget::svg::Handle),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Icon {
    pub icon: IconType,
    pub path: String,
    pub is_favicon: bool,
}

impl Icon {
    pub fn new(icon: IconType, path: String, is_favicon: bool) -> Self {
        Self {
            icon,
            path,
            is_favicon,
        }
    }
}

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct WebviewArgs {
    pub app_id: String,
    #[clap(short, long)]
    pub window_title: String,
    #[clap(short, long)]
    pub url: String,
    #[clap(short, long)]
    pub profile: Option<PathBuf>,
    #[clap(short, long)]
    pub window_size: Option<WindowSize>,
    #[clap(short, long)]
    pub window_decorations: Option<bool>,
    #[clap(short, long)]
    pub private_mode: Option<bool>,
    #[clap(short, long)]
    pub try_simulate_mobile: Option<bool>,
}

pub struct WebViewArg(pub String);

impl Display for WebViewArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<OsStr> for WebViewArg {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl IntoIterator for WebviewArgs {
    type Item = WebViewArg;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut args: Vec<WebViewArg> = Vec::new();

        args.push(WebViewArg("--window-title".to_string()));
        args.push(WebViewArg(self.window_title));
        args.push(WebViewArg("--url".to_string()));
        args.push(WebViewArg(self.url));

        if self.profile.is_some() {
            args.push(WebViewArg("--profile".to_string()));
            args.push(WebViewArg(
                self.profile
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            ));
        }

        if self.window_size.is_some() {
            args.push(WebViewArg("--window-size".to_string()));
            args.push(WebViewArg(self.window_size.unwrap_or_default().to_string()));
        }

        if self.window_decorations.is_some() {
            args.push(WebViewArg("--window-decorations".to_string()));
            args.push(WebViewArg(
                self.window_decorations.unwrap_or_default().to_string(),
            ));
        }

        if self.private_mode.is_some() {
            args.push(WebViewArg("--private-mode".to_string()));
            args.push(WebViewArg(
                self.private_mode.unwrap_or_default().to_string(),
            ));
        }

        if self.try_simulate_mobile.is_some() {
            args.push(WebViewArg("--try-simulate-mobile".to_string()));
            args.push(WebViewArg(
                self.try_simulate_mobile.unwrap_or_default().to_string(),
            ));
        }

        args.push(WebViewArg(self.app_id));

        args.into_iter()
    }
}

impl Display for WebviewArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let exec: String = self
            .clone()
            .into_iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        write!(f, "{}", exec)
    }
}

impl WebviewArgs {
    pub fn get_exec(&self, from_flatpak: bool) -> String {
        let mut exec = String::new();

        if from_flatpak {
            exec.push_str("flatpak run --command=quick-webapps-webview dev.heppen.webapps");
        } else {
            exec.push_str("quick-webapps-webview");
        }

        exec.push_str(&format!(" {}", self));

        exec
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebviewArgsBuilder {
    app_id: String,
    window_title: String,
    url: String,
    profile: Option<PathBuf>,
    window_size: Option<WindowSize>,
    window_decorations: Option<bool>,
    private_mode: Option<bool>,
    try_simulate_mobile: Option<bool>,
}

impl WebviewArgsBuilder {
    pub fn new(app_id: String, window_title: String, url: String) -> Self {
        Self {
            app_id,
            window_title,
            url,
            profile: None,
            window_size: None,
            window_decorations: None,
            private_mode: None,
            try_simulate_mobile: None,
        }
    }

    pub fn profile(&mut self, profile: &PathBuf) {
        self.profile = Some(profile.clone());
    }

    pub fn window_size(&mut self, window_size: WindowSize) {
        self.window_size = Some(window_size);
    }

    pub fn window_decorations(&mut self, window_decorations: bool) {
        self.window_decorations = Some(window_decorations);
    }

    pub fn set_incognito(&mut self, private_mode: bool) {
        self.private_mode = Some(private_mode);
    }

    pub fn try_simulate_mobile(&mut self, try_simulate_mobile: bool) {
        self.try_simulate_mobile = Some(try_simulate_mobile);
    }

    pub fn build(self) -> WebviewArgs {
        WebviewArgs {
            app_id: self.app_id,
            window_title: self.window_title,
            url: self.url,
            profile: self.profile,
            window_size: self.window_size,
            window_decorations: self.window_decorations,
            private_mode: self.private_mode,
            try_simulate_mobile: self.try_simulate_mobile,
        }
    }
}

#[derive(Debug)]
pub enum WindowSizeError {
    ParsingError,
    BadArgument,
}

impl Display for WindowSizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowSizeError::ParsingError => write!(f, "Failed to parse window size"),
            WindowSizeError::BadArgument => write!(f, "Invalid window size argument"),
        }
    }
}

impl std::error::Error for WindowSizeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WindowSizeError::ParsingError => None,
            WindowSizeError::BadArgument => None,
        }
    }

    fn description(&self) -> &str {
        match self {
            WindowSizeError::ParsingError => "Failed to parse window size",
            WindowSizeError::BadArgument => "Invalid window size argument",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

pub type WindowWidth = f64;
pub type WindowHeight = f64;

pub const DEFAULT_WINDOW_WIDTH: WindowWidth = 800.0;
pub const DEFAULT_WINDOW_HEIGHT: WindowHeight = 600.0;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowSize(pub WindowWidth, pub WindowHeight);

impl Default for WindowSize {
    fn default() -> Self {
        WindowSize(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
    }
}

impl Display for WindowSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.0, self.1)
    }
}

impl FromStr for WindowSize {
    type Err = WindowSizeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            Err(WindowSizeError::BadArgument)
        } else {
            let width = parts[0]
                .parse()
                .map_err(|_| WindowSizeError::ParsingError)?;
            let height = parts[1]
                .parse()
                .map_err(|_| WindowSizeError::ParsingError)?;
            Ok(WindowSize(width, height))
        }
    }
}

pub fn is_flatpak() -> bool {
    let Ok(path) = PathBuf::from_str("/.flatpak-info");
    path.exists()
}
