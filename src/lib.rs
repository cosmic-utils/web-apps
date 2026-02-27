use clap::Parser;
use cosmic::{iced_core, iced_winit::graphics::image::image_rs::ImageReader, widget};
use rand::RngExt;
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
use svg::node::element::{Circle, Text};
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
pub const MOBILE_UA: &str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.7632.76 Mobile Safari/537.36";
pub const DESKTOP_UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36";

pub fn url_valid(url: &str) -> bool {
    if Url::parse(url).is_ok() {
        return true;
    }
    false
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
        let final_path = xdg_data.join(APP_ID).join("icons");

        if !final_path.exists() {
            if let Err(e) = create_dir_all(&final_path) {
                eprintln!("Failed to create icons directory: {}", e);
                return None;
            }
        };

        return Some(final_path);
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

            return Some(Icon::new(IconType::Svg(handle), path));
        } else {
            let mut data: Vec<_> = Vec::new();

            if let Ok(mut file) = fs::File::open(&result_path) {
                let _ = file.read_to_end(&mut data);
            }

            if let Ok(image_reader) = ImageReader::new(Cursor::new(&data)).with_guessed_format() {
                if let Ok(image) = image_reader.decode() {
                    if image.width() >= ICON_SIZE && image.height() >= ICON_SIZE {
                        let handle = iced_core::image::Handle::from_bytes(data);

                        return Some(Icon::new(IconType::Raster(handle), path));
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
    Raster(widget::image::Handle),
    Svg(widget::svg::Handle),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Icon {
    pub icon: IconType,
    pub path: String,
}

impl Icon {
    pub fn new(icon: IconType, path: String) -> Self {
        Self { icon, path }
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

pub fn generate_icon(first_letter: &str, icon_name: &str) -> PathBuf {
    let Some(path) = icons_location() else {
        return PathBuf::from("/tmp");
    };

    let temp_path = path.join(&format!("{}.svg", icon_name));

    let background = Circle::new()
        .set("cx", 128)
        .set("cy", 128)
        .set("r", 100)
        .set("fill", generate_random_color());

    let text = Text::new(first_letter)
        .set("x", "50%")
        .set("y", "70%")
        .set("text-anchor", "middle")
        .set("font-family", "sans-serif")
        .set("font-size", 150)
        .set("fill", "black")
        .set("font-weight", "bold");

    // Create the document
    let document = svg::Document::new()
        .set("viewBox", "0 0 256 256")
        .set("width", 256)
        .set("height", 256)
        .add(background)
        .add(text);

    // Save the file
    svg::save(&temp_path, &document).expect("Unable to save file");
    println!("Generated {}", temp_path.display());

    temp_path
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
#[command(propagate_version = true, ignore_errors = true)]
pub struct WebviewArgs {
    pub id: String,
    #[arg(long)]
    pub url: Option<String>,
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
        let mut args = vec![self.id.clone()];
        if let Some(url) = &self.url {
            args.push(format!("--url={}", url));
        }
        args.into_iter()
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
    if let Ok(path) = std::env::current_exe() {
        if let Some(parent) = path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
        {
            let cef_dir = parent.join("cef");
            if cef_dir.exists() {
                return Some(cef_dir);
            }
        }
    }

    let is_sandbox = PathBuf::from("/.flatpak-info").exists();
    let prefix = if is_sandbox { "/app" } else { "/usr/local" };
    let installed_cef = PathBuf::from(prefix).join("lib").join(APP_ID);
    // fallback for installed
    if installed_cef.exists() {
        return Some(installed_cef);
    }

    None
}
