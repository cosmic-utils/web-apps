use std::{io::Read as _, path::PathBuf};

use clap::Parser;
use rand::RngExt;
use serde::{Deserialize, Serialize};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tokio::fs::create_dir_all;
use url::Url;

use crate::launcher::DesktopIcon;

pub mod browser;
pub mod launcher;
pub mod localize;

pub const DEFAULT_WINDOW_WIDTH: WindowWidth = 800;
pub const DEFAULT_WINDOW_HEIGHT: WindowHeight = 600;
pub const ICON_SIZE: u32 = 42;
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
pub const CONFIG_VERSION: u64 = 1;
pub const APP_ID: &str = "dev.heppen.webapps";
pub const WEBVIEW_ID: &str = "dev.heppen.webapps.webview";
pub const APP_ICON: &[u8] =
    include_bytes!("../resources/icons/hicolor/256x256/apps/dev.heppen.webapps.png");
pub const MOBILE_UA: &str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.7727.138 Mobile Safari/537.36";
pub const DESKTOP_UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36";

#[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub id: String,
    pub title: String,
    pub url: String,
    pub icon: Option<Icon>,
    pub window_width: String,
    pub window_height: String,
    pub persistent_profile: bool,
    pub simulate_mobile: bool,
    pub category: Category,
    pub is_installed: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            url: String::new(),
            icon: None,
            window_width: format!("{}", DEFAULT_WINDOW_WIDTH),
            window_height: format!("{}", DEFAULT_WINDOW_HEIGHT),
            persistent_profile: true,
            simulate_mobile: false,
            category: Category::default(),
            is_installed: false,
        }
    }
}

pub fn url_valid(url: &str) -> bool {
    if Url::parse(url).is_ok() {
        return true;
    }
    false
}

pub fn launcher_is_valid(app_id: &str, url: Option<&str>) -> bool {
    let Some(url) = url else {
        return false;
    };

    !app_id.is_empty() && url_valid(url)
}

pub fn browser_path() -> Option<PathBuf> {
    if let Some(xdg_data) = dirs::data_dir() {
        let final_path = xdg_data.join(APP_ID).join("browser");

        if !final_path.exists() {
            if let Err(e) = std::fs::create_dir_all(&final_path) {
                eprintln!("Failed to create icons directory: {}", e);
                return None;
            }
        };
        return Some(final_path);
    }
    None
}

pub async fn icons_location() -> Option<PathBuf> {
    if let Some(xdg_data) = dirs::data_dir() {
        let final_path = xdg_data.join(APP_ID).join("icons");

        if !final_path.exists() {
            if let Err(e) = create_dir_all(&final_path).await {
                eprintln!("Failed to create icons directory: {}", e);
                return None;
            }
        };
        return Some(final_path);
    }
    None
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IconType {
    Raster,
    Vector,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Icon {
    pub icon: IconType,
    pub buffer: Vec<u8>,
    pub path: String,
}

impl Icon {
    pub fn new(icon: IconType, buffer: Vec<u8>, path: String) -> Self {
        Self { icon, buffer, path }
    }

    pub async fn to_launcher_icon(&self) -> Option<DesktopIcon> {
        let mut buffer = Vec::new();

        if let Ok(mut file) = std::fs::File::open(&self.path) {
            let _ = file.read_to_end(&mut buffer).expect("reading icon");

            return Some(DesktopIcon {
                path: self.path.clone().into(),
                buffer: buffer,
            });
        }

        None
    }
}

pub fn handle_icon(path: PathBuf) -> Icon {
    let mut buff = Vec::new();
    let mut file = std::fs::File::open(&path).expect("temp icon not found");
    let _ = file.read_to_end(&mut buff).expect("reading icon data");
    Icon::new(IconType::Vector, buff, path.display().to_string().clone())
}

#[repr(u8)]
#[derive(Debug, Default, Clone, EnumIter, Hash, PartialEq, Eq, Deserialize, Serialize)]
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

fn generate_random_color() -> String {
    // Generate random RGB values
    let mut rng = rand::rng();
    let colors_array = SvgColor::iter();
    let random_index = rng.random_range(0..colors_array.len());
    SvgColor::from_index(random_index.try_into().expect("conversion")).to_string()
}

pub async fn generate_icon(first_letter: String) -> Option<DesktopIcon> {
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
  <g
     id="layer1">
    <circle
       style="fill:{}"
       id="path1"
       cx="67.73333"
       cy="-67.73333"
       r="64.96875"
       transform="scale(1,-1)" />
    <text
       xml:space="preserve"
       style="font-style:normal;font-variant:normal;font-weight:normal;font-stretch:normal;font-size:88.1944px;font-family:'Noto Sans';-inkscape-font-specification:'Noto Sans, Normal';font-variant-ligatures:normal;font-variant-caps:normal;font-variant-numeric:normal;font-variant-east-asian:normal;writing-mode:lr-tb;direction:ltr;fill:#ffffff;fill-opacity:1;stroke-width:0.264583"
       x="39.599369"
       y="99.350899"
       id="text1"><tspan
         id="tspan1"
         style="font-style:normal;font-variant:normal;font-weight:normal;font-stretch:normal;font-size:88.1944px;font-family:'Noto Sans';-inkscape-font-specification:'Noto Sans, Normal';font-variant-ligatures:normal;font-variant-caps:normal;font-variant-numeric:normal;font-variant-east-asian:normal;fill:#ffffff;fill-opacity:1;stroke-width:0.264583"
         x="39.599369"
         y="99.350899">{}</tspan></text>
  </g>
</svg>
"#,
        color, first_letter
    );

    if let Some(loc) = icons_location().await {
        let path = loc.join(file_name);
        tracing::info!("Icon saved to {:?}", path);
        std::fs::write(&path, &svg_document).expect("writing icon");
        return Some(DesktopIcon {
            path,
            buffer: svg_document.as_bytes().to_vec(),
        });
    }

    None
}

pub type WindowWidth = u32;
pub type WindowHeight = u32;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowSize(pub WindowWidth, pub WindowHeight);

impl Default for WindowSize {
    fn default() -> Self {
        WindowSize(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
    }
}

#[derive(Parser, Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true, ignore_errors = true)]
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
        vec![self.id.clone()].into_iter()
    }
}

pub fn webview_bin() -> String {
    let app_id_name = format!("{}.webview", APP_ID);
    let cargo_name = "dev-heppen-webapps-webview";

    if let Ok(mut path) = std::env::current_exe() {
        path.set_file_name(&app_id_name);
        if path.exists() {
            return path.to_str().unwrap().to_string();
        }
        path.set_file_name(cargo_name);
        if path.exists() {
            return path.to_str().unwrap().to_string();
        }
    }

    app_id_name
}

pub fn helper_bin() -> String {
    let app_id_name = format!("{}.webview-helper", APP_ID);
    let cargo_name = "dev-heppen-webapps-webview-helper";

    if let Ok(mut path) = std::env::current_exe() {
        path.set_file_name(&app_id_name);
        if path.exists() {
            return path.to_str().unwrap().to_string();
        }
        path.set_file_name(cargo_name);
        if path.exists() {
            return path.to_str().unwrap().to_string();
        }
    }

    app_id_name
}

pub fn cef_path() -> Option<PathBuf> {
    let Some(xdg_data) = dirs::data_dir() else {
        return None;
    };

    let installed_cef = PathBuf::from(xdg_data).join("cef");

    if installed_cef.exists() {
        return Some(installed_cef);
    }

    None
}
