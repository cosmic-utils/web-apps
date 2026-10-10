use dircpy::copy_dir;
use rand::RngExt;
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

pub fn database_path(entry: &str) -> Option<PathBuf> {
    if let Some(xdg_data) = dirs::data_dir() {
        let path = xdg_data.join(APP_ID).join("database");

        if !path.exists() {
            std::fs::create_dir_all(&path).unwrap();
        }

        return Some(path.join(entry));
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter, Hash)]
pub enum SvgColor {
    AliceBlue,
    AntiqueWhite,
    Aqua,
    Aquamarine,
    Azure,
    Beige,
    Bisque,
    Black,
    BlanchedAlmond,
    Blue,
    BlueViolet,
    Brown,
    BurlyWood,
    CadetBlue,
    Chartreuse,
    Chocolate,
    Coral,
    CornflowerBlue,
    Cornsilk,
    Crimson,
    Cyan,
    DarkBlue,
    DarkCyan,
    DarkGoldenRod,
    DarkGray,
    DarkGreen,
    DarkKhaki,
    DarkMagenta,
    DarkOliveGreen,
    DarkOrange,
    DarkOrchid,
    DarkRed,
    DarkSalmon,
    DarkSeaGreen,
    DarkSlateBlue,
    DarkSlateGray,
    DarkTurquoise,
    DarkViolet,
    DeepPink,
    DeepSkyBlue,
    DimGray,
    DodgerBlue,
    FireBrick,
    FloralWhite,
    ForestGreen,
    Fuchsia,
    Gainsboro,
    GhostWhite,
    #[default]
    Gold,
    GoldenRod,
    Gray,
    Green,
    GreenYellow,
    HoneyDew,
    HotPink,
    IndianRed,
    Indigo,
    Ivory,
    Khaki,
    Lavender,
    LavenderBlush,
    LawnGreen,
    LemonChiffon,
    LightBlue,
    LightCoral,
    LightCyan,
    LightGoldenRodYellow,
    LightGray,
    LightGreen,
    LightPink,
    LightSalmon,
    LightSeaGreen,
    LightSkyBlue,
    LightSlateGray,
    LightSteelBlue,
    LightYellow,
    Lime,
    LimeGreen,
    Linen,
    Magenta,
    Maroon,
    MediumAquaMarine,
    MediumBlue,
    MediumOrchid,
    MediumPurple,
    MediumSeaGreen,
    MediumSlateBlue,
    MediumSpringGreen,
    MediumTurquoise,
    MediumVioletRed,
    MidnightBlue,
    MintCream,
    MistyRose,
    Moccasin,
    NavajoWhite,
    Navy,
    OldLace,
    Olive,
    OliveDrab,
    Orange,
    OrangeRed,
    Orchid,
    PaleGoldenRod,
    PaleGreen,
    PaleTurquoise,
    PaleVioletRed,
    PapayaWhip,
    PeachPuff,
    Peru,
    Pink,
    Plum,
    PowderBlue,
    Purple,
    RebeccaPurple,
    Red,
    RosyBrown,
    RoyalBlue,
    SaddleBrown,
    Salmon,
    SandyBrown,
    SeaGreen,
    SeaShell,
    Sienna,
    Silver,
    SkyBlue,
    SlateBlue,
    SlateGray,
    Snow,
    SpringGreen,
    SteelBlue,
    Tan,
    Teal,
    Thistle,
    Tomato,
    Turquoise,
    Violet,
    Wheat,
    White,
    WhiteSmoke,
    Yellow,
    YellowGreen,
}

impl std::fmt::Display for SvgColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::AliceBlue => "aliceblue",
            Self::AntiqueWhite => "antiquewhite",
            Self::Aqua => "aqua",
            Self::Aquamarine => "aquamarine",
            Self::Azure => "azure",
            Self::Beige => "beige",
            Self::Bisque => "bisque",
            Self::Black => "black",
            Self::BlanchedAlmond => "blanchedalmond",
            Self::Blue => "blue",
            Self::BlueViolet => "blueviolet",
            Self::Brown => "brown",
            Self::BurlyWood => "burlywood",
            Self::CadetBlue => "cadetblue",
            Self::Chartreuse => "chartreuse",
            Self::Chocolate => "chocolate",
            Self::Coral => "coral",
            Self::CornflowerBlue => "cornflowerblue",
            Self::Cornsilk => "cornsilk",
            Self::Crimson => "crimson",
            Self::Cyan => "cyan",
            Self::DarkBlue => "darkblue",
            Self::DarkCyan => "darkcyan",
            Self::DarkGoldenRod => "darkgoldenrod",
            Self::DarkGray => "darkgray",
            Self::DarkGreen => "darkgreen",
            Self::DarkKhaki => "darkkhaki",
            Self::DarkMagenta => "darkmagenta",
            Self::DarkOliveGreen => "darkolivegreen",
            Self::DarkOrange => "darkorange",
            Self::DarkOrchid => "darkorchid",
            Self::DarkRed => "darkred",
            Self::DarkSalmon => "darksalmon",
            Self::DarkSeaGreen => "darkseagreen",
            Self::DarkSlateBlue => "darkslateblue",
            Self::DarkSlateGray => "darkslategray",
            Self::DarkTurquoise => "darkturquoise",
            Self::DarkViolet => "darkviolet",
            Self::DeepPink => "deeppink",
            Self::DeepSkyBlue => "deepskyblue",
            Self::DimGray => "dimgray",
            Self::DodgerBlue => "dodgerblue",
            Self::FireBrick => "firebrick",
            Self::FloralWhite => "floralwhite",
            Self::ForestGreen => "forestgreen",
            Self::Fuchsia => "fuchsia",
            Self::Gainsboro => "gainsboro",
            Self::GhostWhite => "ghostwhite",
            Self::Gold => "gold",
            Self::GoldenRod => "goldenrod",
            Self::Gray => "gray",
            Self::Green => "green",
            Self::GreenYellow => "greenyellow",
            Self::HoneyDew => "honeydew",
            Self::HotPink => "hotpink",
            Self::IndianRed => "indianred",
            Self::Indigo => "indigo",
            Self::Ivory => "ivory",
            Self::Khaki => "khaki",
            Self::Lavender => "lavender",
            Self::LavenderBlush => "lavenderblush",
            Self::LawnGreen => "lawngreen",
            Self::LemonChiffon => "lemonchiffon",
            Self::LightBlue => "lightblue",
            Self::LightCoral => "lightcoral",
            Self::LightCyan => "lightcyan",
            Self::LightGoldenRodYellow => "lightgoldenrodyellow",
            Self::LightGray => "lightgray",
            Self::LightGreen => "lightgreen",
            Self::LightPink => "lightpink",
            Self::LightSalmon => "lightsalmon",
            Self::LightSeaGreen => "lightseagreen",
            Self::LightSkyBlue => "lightskyblue",
            Self::LightSlateGray => "lightslategray",
            Self::LightSteelBlue => "lightsteelblue",
            Self::LightYellow => "lightyellow",
            Self::Lime => "lime",
            Self::LimeGreen => "limegreen",
            Self::Linen => "linen",
            Self::Magenta => "magenta",
            Self::Maroon => "maroon",
            Self::MediumAquaMarine => "mediumaquamarine",
            Self::MediumBlue => "mediumblue",
            Self::MediumOrchid => "mediumorchid",
            Self::MediumPurple => "mediumpurple",
            Self::MediumSeaGreen => "mediumseagreen",
            Self::MediumSlateBlue => "mediumslateblue",
            Self::MediumSpringGreen => "mediumspringgreen",
            Self::MediumTurquoise => "mediumturquoise",
            Self::MediumVioletRed => "mediumvioletred",
            Self::MidnightBlue => "midnightblue",
            Self::MintCream => "mintcream",
            Self::MistyRose => "mistyrose",
            Self::Moccasin => "moccasin",
            Self::NavajoWhite => "navajowhite",
            Self::Navy => "navy",
            Self::OldLace => "oldlace",
            Self::Olive => "olive",
            Self::OliveDrab => "olivedrab",
            Self::Orange => "orange",
            Self::OrangeRed => "orangered",
            Self::Orchid => "orchid",
            Self::PaleGoldenRod => "palegoldenrod",
            Self::PaleGreen => "palegreen",
            Self::PaleTurquoise => "paleturquoise",
            Self::PaleVioletRed => "palevioletred",
            Self::PapayaWhip => "papayawhip",
            Self::PeachPuff => "peachpuff",
            Self::Peru => "peru",
            Self::Pink => "pink",
            Self::Plum => "plum",
            Self::PowderBlue => "powderblue",
            Self::Purple => "purple",
            Self::RebeccaPurple => "rebeccapurple",
            Self::Red => "red",
            Self::RosyBrown => "rosybrown",
            Self::RoyalBlue => "royalblue",
            Self::SaddleBrown => "saddlebrown",
            Self::Salmon => "salmon",
            Self::SandyBrown => "sandybrown",
            Self::SeaGreen => "seagreen",
            Self::SeaShell => "seashell",
            Self::Sienna => "sienna",
            Self::Silver => "silver",
            Self::SkyBlue => "skyblue",
            Self::SlateBlue => "slateblue",
            Self::SlateGray => "slategray",
            Self::Snow => "snow",
            Self::SpringGreen => "springgreen",
            Self::SteelBlue => "steelblue",
            Self::Tan => "tan",
            Self::Teal => "teal",
            Self::Thistle => "thistle",
            Self::Tomato => "tomato",
            Self::Turquoise => "turquoise",
            Self::Violet => "violet",
            Self::Wheat => "wheat",
            Self::White => "white",
            Self::WhiteSmoke => "whitesmoke",
            Self::Yellow => "yellow",
            Self::YellowGreen => "yellowgreen",
        };
        write!(f, "{}", name)
    }
}

impl SvgColor {
    pub fn from_index(index: u8) -> Self {
        Self::iter()
            .find(|i| i.to_owned() as u8 == index)
            .unwrap_or_default()
    }
}

#[allow(dead_code)]
fn generate_random_color() -> String {
    // Generate random RGB values
    let mut rng = rand::rng();
    let colors_array = SvgColor::iter();
    let random_index = rng.random_range(0..colors_array.len());

    SvgColor::from_index(random_index.try_into().expect("conversion")).to_string()
}

pub fn generate_icon(first_letter: &str) -> Option<WebappIcon> {
    let color = generate_random_color();

    let file_name = format!("{}_{}.svg", first_letter, &color);

    let svg_document = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!-- Created with Inkscape (http://www.inkscape.org/) -->

<svg
   width="512"
   height="512"
   viewBox="0 0 135.46666 135.46667"
   version="1.1"
   id="svg1"
   xmlns="http://www.w3.org/2000/svg"
   xmlns:svg="http://www.w3.org/2000/svg">
  <defs
     id="defs1" />
  <circle
     style="fill:{};stroke-width:1"
     id="path1"
     cx="68.17894"
     cy="-67.73333"
     transform="scale(1,-1)"
     r="64.96875" />
  <text
     xml:space="preserve"
     style="font-style:normal;font-variant:normal;font-weight:normal;font-stretch:normal;font-size:88.1944px;font-family:'Noto Sans';-inkscape-font-specification:'Noto Sans, Normal';font-variant-ligatures:normal;font-variant-caps:normal;font-variant-numeric:normal;font-variant-east-asian:normal;writing-mode:lr-tb;direction:ltr;fill:#ffffff;fill-opacity:1;stroke-width:0.264583"
     x="36.649605"
     y="99.21859"
     id="text1"><tspan
       id="tspan1"
       style="font-style:normal;font-variant:normal;font-weight:normal;font-stretch:normal;font-size:88.1944px;font-family:'Noto Sans';-inkscape-font-specification:'Noto Sans, Normal';font-variant-ligatures:normal;font-variant-caps:normal;font-variant-numeric:normal;font-variant-east-asian:normal;fill:#ffffff;fill-opacity:1;stroke-width:0.264583"
       x="36.649605"
       y="99.21859">{}</tspan></text>
</svg>
"#,
        color, first_letter
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
