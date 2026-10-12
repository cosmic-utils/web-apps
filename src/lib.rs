use dircpy::copy_dir;
use serde::{Deserialize, Serialize};
use std::fs::Permissions;
use std::{ffi::OsStr, os::unix::fs::PermissionsExt as _, path::PathBuf, str::FromStr};
use tokio::{fs::File, io::AsyncWriteExt as _, process::Child};

use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use cosmic::cosmic_theme::{self, ThemeBuilder};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use url::Url;
use walkdir::WalkDir;

pub mod browser;
pub mod launcher;
pub mod localize;
pub mod supported_browsers;

pub const ICON_SIZE: u32 = 42;
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
pub const CONFIG_VERSION: u64 = 1;
pub const APP_ID: &str = "dev.heppen.webapps";
pub const APP_ICON: &[u8] =
    include_bytes!("../resources/icons/hicolor/256x256/apps/dev.heppen.webapps.png");

#[derive(Debug, Default, Clone)]
pub enum Theme {
    #[default]
    Light,
    Dark,
    Custom((String, Box<cosmic_theme::Theme>)),
}

impl AsRef<str> for Theme {
    fn as_ref(&self) -> &str {
        match self {
            Theme::Light => "COSMIC Light",
            Theme::Dark => "COSMIC Dark",
            Theme::Custom(theme) => &theme.0,
        }
    }
}

impl Theme {
    pub fn build(name: String, value: String) -> Self {
        if let Ok(palette) = ron::from_str::<ThemeBuilder>(&value) {
            return Self::Custom((name, Box::new(palette.build())));
        }

        Self::Light
    }

    pub fn to_cosmic_theme(&self) -> cosmic::Theme {
        match self {
            Theme::Light => cosmic::theme::system_light(),
            Theme::Dark => cosmic::theme::system_dark(),
            _ => cosmic::theme::system_preference(),
        }
    }
}

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct AppConfig {
    pub app_theme: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct WebAppConfig {
    pub id: String,
}

impl AppConfig {
    pub fn config_handler() -> Option<cosmic_config::Config> {
        cosmic_config::Config::new(APP_ID, CONFIG_VERSION).ok()
    }
    pub fn config() -> AppConfig {
        match Self::config_handler() {
            Some(config_handler) => {
                AppConfig::get_entry(&config_handler).unwrap_or_else(|(errs, config)| {
                    tracing::info!("errors loading config: {:?}", errs);
                    config
                })
            }
            None => AppConfig::default(),
        }
    }
}

pub fn webapp_id(name: String, num_id: u16) -> String {
    if name.is_empty() || name.len() < 3 {
        return format!("{}.{}", APP_ID.to_owned(), num_id);
    };

    let name = name.replace(' ', "");
    format!("{}.{}{}", APP_ID, name, num_id)
}

pub fn url_valid(url: &str) -> bool {
    if Url::parse(url).is_ok() {
        return true;
    }
    false
}

/// Checks if /.flatpak-info exists so we can assume
/// Its a flatpak installation
pub fn is_flatpak() -> bool {
    PathBuf::from("/.flatpak-info").exists() || std::env::var("FLATPAK_ID").is_ok()
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

/// Local cache path for storing icon installer script
pub fn cache_path() -> Option<PathBuf> {
    if let Some(cache) = dirs::cache_dir() {
        return Some(cache.join(APP_ID));
    }

    None
}

/// Local xdg_data path for storing icons mostly and profiles
pub fn data_path() -> Option<PathBuf> {
    if let Some(data) = dirs::data_dir() {
        return Some(data.join(APP_ID));
    }

    None
}

/// Desktop entries path, common data_dir/applications
pub fn desktop_files() -> PathBuf {
    let mut pathbuf = PathBuf::new();

    if let Some(xdg_data) = dirs::data_dir() {
        pathbuf.push(xdg_data.join("applications"));

        if !pathbuf.exists() {
            let _ = std::fs::create_dir_all(&pathbuf);
        }
    }

    pathbuf
}

pub fn launcher_desktop_entry_path(appid: &str) -> Option<PathBuf> {
    let filename = format!("{}.desktop", appid);

    if let Some(mut xdg_data) = dirs::data_dir() {
        xdg_data = xdg_data.join("applications");

        if !xdg_data.exists() {
            let _ = std::fs::create_dir_all(&xdg_data);
        }

        xdg_data = xdg_data.join(filename);

        return Some(xdg_data);
    }

    None
}

pub fn themes_path(theme_file: &str) -> Option<PathBuf> {
    if let Some(mut data) = data_path() {
        data.push("themes");

        if !data.exists() {
            let _ = std::fs::create_dir_all(&data);
        }

        data.push(theme_file);

        return Some(data);
    }

    None
}

pub fn profiles_path(app_id: &str) -> Option<PathBuf> {
    if let Some(xdg_data) = dirs::data_dir() {
        let final_path = xdg_data.join(APP_ID).join("profiles").join(app_id);

        if !final_path.exists() {
            if let Err(e) = std::fs::create_dir_all(&final_path) {
                eprintln!("Failed to create profile directory: {}", e);
                return None;
            }
        }

        return Some(final_path);
    }

    None
}

pub fn icons_location() -> Option<PathBuf> {
    if let Some(data) = data_path() {
        let directory = data.join("icons");

        if !directory.exists() {
            if let Err(e) = std::fs::create_dir_all(&directory) {
                eprintln!("Failed to create icons directory: {}", e);
                return None;
            }
        };

        return Some(directory);
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
            .join(APP_ID)
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

pub async fn add_icon_packs_install_script() -> Option<String> {
    let install_script = include_bytes!("../resources/scripts/icon-installer.sh");

    let Some(cache) = cache_path() else {
        return None;
    };

    let script_file = cache.join(format!("{}-icon-installer.sh", APP_ID));

    let _ = tokio::fs::create_dir_all(&cache).await;

    // Create a temporary file
    let mut file = File::create(&script_file)
        .await
        .expect("creating script file");

    let _ = file.write_all(install_script).await;

    // Make the script executable
    if let Ok(metadata) = file.metadata().await {
        let mut perms = metadata.permissions();
        perms.set_mode(0o755);
        let _ = file.set_permissions(perms).await;

        return Some(script_file.display().to_string());
    }
    None
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
                    if !icons.contains(&path.to_string()) {
                        icons.push(path.to_string())
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

#[repr(u8)]
#[derive(Debug, Default, Clone, EnumIter, PartialEq, Eq, Deserialize, Serialize)]
pub enum Category {
    Audio = 0,
    AudioVideo = 1,
    Video = 2,
    Development = 3,
    Education = 4,
    Game = 5,
    Graphics = 6,
    Network = 7,
    Office = 8,
    Science = 9,
    Settings = 10,
    System = 11,
    #[default]
    Utility = 12,
}

impl AsRef<str> for Category {
    fn as_ref(&self) -> &str {
        match self {
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
    Raster,
    Svg,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WebappIcon {
    pub icon: IconType,
    pub source_path: Option<PathBuf>,
    pub buffer: Vec<u8>,
}

impl WebappIcon {
    pub fn build_from_path(path: &str) -> Self {
        let source_path = if path.is_empty() {
            None
        } else {
            Some(PathBuf::from(&path))
        };

        let icon_t = match is_svg(&path) {
            true => IconType::Svg,
            false => IconType::Raster,
        };

        let buffer = std::fs::read(path).unwrap_or_default();

        Self {
            icon: icon_t,
            source_path,
            buffer,
        }
    }
}

pub fn generate_icon(name: &str) -> Option<WebappIcon> {
    let mut color = random_color::RandomColor::new();
    let hex_color = color.to_hex();
    let file_name = format!("{}_{}.svg", name, &hex_color);

    let svg_document = format!(
        r#"<?xml version="1.0" ?>
<!-- Uploaded to: SVG Repo, www.svgrepo.com, Generator: SVG Repo Mixer Tools -->
<svg version="1.1" id="Icons" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" 
	 viewBox="0 0 32 32" xml:space="preserve">
<style type="text/css">
	.st0{{fill:none;stroke:#000000;stroke-width:2;stroke-linecap:round;stroke-linejoin:round;stroke-miterlimit:10;}}
</style>
<path d="M27,4H5C3.3,4,2,5.3,2,7v18c0,1.7,1.3,3,3,3h2.8C7.3,27,7,26,7,24.9c-0.1-2.7,0.7-5.3,2.5-7.5L9.8,17c0.8-1,1.6-2,2-3
	c0.2-0.6,0.7-1.1,1.2-1.5c1.2-0.8,2.9-0.6,3.9,0.4c1.5,1.6,2.4,3.3,2.7,5c0,0.2,0.1,0.4,0.1,0.6c0.5-0.3,1.1-0.5,1.7-0.5
	c1.1,0,2.1,0.7,2.6,1.7c0.9,1.9,1.2,4.3,0.8,6.4c-0.1,0.6-0.4,1.3-0.6,1.8H27c1.7,0,3-1.3,3-3V7C30,5.3,28.7,4,27,4z M7.9,8.4
	C7.9,8.5,7.8,8.6,7.7,8.7C7.5,8.9,7.3,9,7,9S6.5,8.9,6.3,8.7C6.1,8.5,6,8.3,6,8c0-0.3,0.1-0.5,0.3-0.7c0,0,0.1-0.1,0.1-0.1
	c0.1,0,0.1-0.1,0.2-0.1C6.7,7,6.7,7,6.8,7c0.1,0,0.3,0,0.4,0c0.1,0,0.1,0,0.2,0.1c0.1,0,0.1,0.1,0.2,0.1c0,0,0.1,0.1,0.1,0.1
	c0.1,0.1,0.2,0.2,0.2,0.3C8,7.7,8,7.9,8,8C8,8.1,8,8.3,7.9,8.4z M10.7,8.7C10.5,8.9,10.3,9,10,9C9.7,9,9.5,8.9,9.3,8.7
	C9.1,8.5,9,8.3,9,8c0-0.1,0-0.3,0.1-0.4c0.1-0.1,0.1-0.2,0.2-0.3c0.1-0.1,0.2-0.2,0.3-0.2C10,6.9,10.4,7,10.7,7.3
	c0.1,0.1,0.2,0.2,0.2,0.3C11,7.7,11,7.9,11,8C11,8.3,10.9,8.5,10.7,8.7z M13.9,8.4c-0.1,0.1-0.1,0.2-0.2,0.3C13.5,8.9,13.3,9,13,9
	c-0.1,0-0.3,0-0.4-0.1c-0.1-0.1-0.2-0.1-0.3-0.2c-0.1-0.1-0.2-0.2-0.2-0.3C12,8.3,12,8.1,12,8c0-0.1,0-0.3,0.1-0.4
	c0.1-0.1,0.1-0.2,0.2-0.3c0.4-0.4,1-0.4,1.4,0c0.1,0.1,0.2,0.2,0.2,0.3C14,7.7,14,7.9,14,8C14,8.1,14,8.3,13.9,8.4z" fill="{hex_color}"/>
<path d="M22.2,20.7c-0.2-0.3-0.5-0.5-0.9-0.6c-0.4,0-0.7,0.2-0.9,0.5l-0.1,0.2c0,0.1-0.1,0.2-0.1,0.2c0,0,0,0,0,0
	c-0.5,0.7-1.1,1.1-2.1,1.2c0,0-0.1,0-0.1,0c0,0,0,0,0-0.1l0-0.6c-0.1-1.1-0.1-2.1-0.3-3.2c-0.3-1.4-1-2.7-2.2-4
	c-0.3-0.3-0.9-0.4-1.3-0.1c-0.2,0.1-0.4,0.3-0.5,0.5c-0.5,1.3-1.3,2.3-2.3,3.5l-0.3,0.4c-1.5,1.9-2.2,4-2.1,6.2
	c0.1,3.3,3.3,6.2,6.9,6.2c0,0,0,0,0,0c3.3,0,6.2-2.2,6.9-5.3C23.2,24.1,23,22.2,22.2,20.7z"/>
</svg>
"#
    );

    if let Some(mut source) = icons_location() {
        source.push(file_name);

        return Some(WebappIcon {
            icon: IconType::Svg,
            source_path: Some(source),
            buffer: svg_document.as_bytes().to_vec(),
        });
    }

    None
}

fn profile_templates() -> PathBuf {
    let prefix = if is_flatpak() {
        PathBuf::from("/app")
    } else {
        dirs::home_dir()
            .and_then(|d| Some(d.join(".local")))
            .unwrap_or(PathBuf::from("/usr"))
    };

    prefix.join("share").join(APP_ID)
}

pub fn install_firefox_empty_profile(dst: &str) -> std::io::Result<()> {
    let dst = PathBuf::from(dst);

    std::fs::create_dir_all(&dst)?;

    let mut data = profile_templates();
    data.push("firefox");
    data.push("profile");

    let _ = copy_dir(data, dst);

    Ok(())
}

pub fn install_zen_empty_profile(dst: &str) -> std::io::Result<()> {
    let dst = PathBuf::from(dst);

    std::fs::create_dir_all(&dst)?;

    let mut data = profile_templates();
    data.push("zen");
    data.push("profile");

    let _ = copy_dir(data, dst);

    Ok(())
}

pub fn is_profile_path_readonly(profile_path: &PathBuf) -> bool {
    if let Some(parent) = profile_path.parent() {
        let Ok(metadata) = parent.metadata() else {
            return true;
        };

        return Permissions::readonly(&metadata.permissions());
    }

    true
}
