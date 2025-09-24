use clap::Parser;
use cosmic::{iced_core, iced_winit::graphics::image::image_rs::ImageReader, widget};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fmt::Display,
    fs::{self, create_dir_all},
    io::{Cursor, Read},
    os::unix::fs::PermissionsExt as _,
    path::PathBuf,
    str::FromStr,
};
use tokio::{fs::File, io::AsyncWriteExt as _, process::Child};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use url::Url;
use walkdir::WalkDir;

pub mod browser;
pub mod launcher;
pub mod localize;

pub const DEFAULT_WINDOW_WIDTH: WindowWidth = 800.0;
pub const DEFAULT_WINDOW_HEIGHT: WindowHeight = 600.0;
pub const ICON_SIZE: u32 = 42;
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
pub const CONFIG_VERSION: u64 = 1;
pub const APP_ID: &str = "dev.heppen.webapps";
pub const APP_ICON: &[u8] =
    include_bytes!("../resources/icons/hicolor/256x256/apps/dev.heppen.webapps.png");
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

pub fn themes_path(theme_file: &str) -> Option<PathBuf> {
    if let Some(xdg_data) = dirs::data_dir() {
        let path = xdg_data.join(APP_ID).join("themes");

        if !path.exists() {
            create_dir_all(&path).unwrap();
        }

        return Some(path.join(theme_file));
    }

    None
}

pub fn database_path(entry: &str) -> Option<PathBuf> {
    if let Some(xdg_data) = dirs::data_dir() {
        let path = xdg_data.join(APP_ID).join("database");

        if !path.exists() {
            create_dir_all(&path).unwrap();
        }

        return Some(path.join(entry));
    }

    None
}

pub fn profiles_path(app_id: &str) -> Option<PathBuf> {
    if let Some(xdg_data) = dirs::data_dir() {
        return Some(xdg_data.join(APP_ID).join("profiles").join(app_id));
    }

    None
}

pub fn icons_location() -> Option<PathBuf> {
    if let Some(xdg_data) = dirs::data_dir() {
        return Some(xdg_data.join(APP_ID).join("icons"));
    }
    None
}

pub fn move_icon(path: &str, icon_name: &str, extension: &str) -> Option<PathBuf> {
    if let Some(icons_dir) = icons_location() {
        if !icons_dir.exists() {
            if let Err(e) = create_dir_all(&icons_dir) {
                eprintln!("Failed to create icons directory: {}", e);
                return None;
            }
        }

        let dest_path = icons_dir.join(format!("{}.{}", icon_name, extension));

        if let Err(e) = fs::copy(path, &dest_path) {
            eprintln!("Failed to copy icon: {}", e);
            return None;
        }

        return Some(dest_path);
    }

    None
}

pub fn icon_pack_installed() -> bool {
    let packs: Vec<&str> = vec!["Papirus", "Papirus-Dark", "Papirus-Light"];
    let mut directories = 0;

    let mut icons_dir = match icons_location() {
        Some(dir) => dir,
        None => PathBuf::from(env!("HOME"))
            .join(".local")
            .join("share")
            .join("icons"),
    };

    for theme in packs.iter() {
        icons_dir.push(theme);

        if icons_dir.exists() {
            directories += 1;
        };
    }

    directories > 0
}

pub async fn add_icon_packs_install_script() -> String {
    let install_script = include_bytes!("../resources/scripts/icon-installer.sh");
    let temp_file = format!("/tmp/{}.sh", APP_ID);

    // Create a temporary file
    let mut file = File::create(&temp_file).await.unwrap();

    file.write_all(install_script).await.unwrap();

    // Make the script executable
    let mut perms = file.metadata().await.unwrap().permissions();
    perms.set_mode(0o755);
    file.set_permissions(perms).await.unwrap();

    temp_file.to_string()
}

pub async fn execute_script(script: String) -> Child {
    tokio::process::Command::new(script)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("cant execute script")
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

pub async fn find_icons(icon_name: String) -> Vec<String> {
    if let Some(path) = icons_location() {
        find_icon(path, icon_name).await
    } else {
        Vec::new()
    }
}

pub async fn image_handle(path: String) -> Option<Icon> {
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

#[repr(u8)]
#[derive(Debug, Default, Clone, EnumIter, PartialEq, Eq, Deserialize, Serialize)]
pub enum Category {
    #[default]
    None = 0,
    Audio = 1,
    AudioVideo = 2,
    Video = 3,
    Development = 4,
    Education = 5,
    Game = 6,
    Graphics = 7,
    Network = 8,
    Office = 9,
    Science = 10,
    Settings = 11,
    System = 12,
    Utility = 13,
}

impl AsRef<str> for Category {
    fn as_ref(&self) -> &str {
        match self {
            Category::None => "None",
            Category::Audio => "Audio",
            Category::AudioVideo => "AudioVideo",
            Category::Video => "Video",
            Category::Development => "Development",
            Category::Education => "Education",
            Category::Game => "Game",
            Category::Graphics => "Graphics",
            Category::Network => "Network",
            Category::Office => "Office",
            Category::Science => "Science",
            Category::Settings => "Settings",
            Category::System => "System",
            Category::Utility => "Utility",
        }
    }
}

impl From<String> for Category {
    fn from(value: String) -> Self {
        match value.as_str() {
            "None" => Category::None,
            "Audio" => Category::Audio,
            "AudioVideo" => Category::AudioVideo,
            "Video" => Category::Video,
            "Development" => Category::Development,
            "Education" => Category::Education,
            "Game" => Category::Education,
            "Graphics" => Category::Graphics,
            "Network" => Category::Network,
            "Office" => Category::Office,
            "Science" => Category::Science,
            "Settings" => Category::Settings,
            "System" => Category::System,
            "Utility" => Category::Utility,
            _ => Self::default(),
        }
    }
}

impl Category {
    pub fn name(&self) -> String {
        match self {
            Category::None => String::from("None"),
            Category::Audio => String::from("Audio"),
            Category::AudioVideo => String::from("Audio & Video"),
            Category::Video => String::from("Video"),
            Category::Development => String::from("Development"),
            Category::Education => String::from("Education"),
            Category::Game => String::from("Game"),
            Category::Graphics => String::from("Graphics"),
            Category::Network => String::from("Network"),
            Category::Office => String::from("Office"),
            Category::Science => String::from("Science"),
            Category::Settings => String::from("Settings"),
            Category::System => String::from("System"),
            Category::Utility => String::from("Utility"),
        }
    }

    pub fn from_index(index: u8) -> Self {
        Self::iter()
            .find(|i| i.to_owned() as u8 == index)
            .unwrap_or_default()
    }

    pub fn to_vec() -> Vec<String> {
        Self::iter().map(|c| c.name()).collect()
    }
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

pub type WindowWidth = f64;
pub type WindowHeight = f64;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowSize(pub WindowWidth, pub WindowHeight);

impl Display for WindowSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.0, self.1)
    }
}

impl Default for WindowSize {
    fn default() -> Self {
        WindowSize(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
    }
}

#[derive(Parser, Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct WebviewArgs {
    pub id: String,
}

impl AsRef<str> for WebviewArgs {
    fn as_ref(&self) -> &str {
        &self.id
    }
}

impl IntoIterator for WebviewArgs {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self.id].into_iter()
    }
}
