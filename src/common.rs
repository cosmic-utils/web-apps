use std::{
    ffi::OsStr,
    fs::{self, copy, create_dir_all, File},
    io::{Cursor, Read},
    path::PathBuf,
    str::FromStr,
};

use base64::prelude::*;
use bytes::Bytes;
use cosmic::{iced_core, widget};
use freedesktop_desktop_entry::{default_paths, DesktopEntry, Iter};
use image::ImageReader;
use image::{load_from_memory, GenericImageView};
use reqwest::blocking;
use svg::node::element::Image;
use svg::Document;
use url::Url;
use walkdir::WalkDir;

use crate::{favicon, LOCALES};

const ICON_SIZE: u32 = 42;

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
        if filename.is_empty() {
            return xdg_data.join("applications");
        } else {
            return xdg_data
                .join("applications")
                .join(format!("dev.heppen.webapps.{}.desktop", filename));
        }
    }

    PathBuf::new()
}

pub fn icons_location() -> PathBuf {
    if let Some(xdg_data) = dirs::data_dir() {
        return xdg_data.join("icons");
    }

    PathBuf::new()
}

pub fn system_icons() -> PathBuf {
    PathBuf::from_str("/usr/share/icons").unwrap_or_default()
}

pub fn qwa_icons_location() -> PathBuf {
    icons_location().join("QuickWebApps")
}

pub fn is_sandboxed() -> bool {
    PathBuf::from("/.flatpak-info").exists()
}

pub fn fd_entries() -> Vec<DesktopEntry> {
    let mut paths = Vec::new();

    // this is workaround for flatpak sandbox
    if is_sandboxed() {
        paths.push("/var/lib/flatpak/exports/share/applications".into());
        paths.push("/run/host/usr/share/applications".into());
        paths.push("/run/host/usr/local/share/applications".into());
    };

    default_paths().for_each(|path| {
        if !paths.contains(&path) {
            paths.push(path)
        }
    });

    Iter::new(paths.into_iter())
        .entries(Some(&LOCALES))
        .collect::<Vec<DesktopEntry>>()
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
        if let Ok(data) = favicon::download_favicon(&url).await {
            result.extend(data)
        }
    };

    result.extend(find_icon(icons_location(), icon_name.clone()).await);
    result.extend(find_icon(system_icons(), icon_name).await);

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
            .set("href", format!("data:image/png;base64,{}", encoded_img));

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
    qwa_icons_location()
        .join(format!("{}.svg", icon_name))
        .to_str()
        .unwrap()
        .to_string()
}

pub fn move_icon(path: &str, output_name: &str) -> String {
    create_dir_all(qwa_icons_location()).expect("cant create folder for your icons");

    let icon_name = output_name.replace(' ', "");

    let Ok(p) = PathBuf::from_str(&icon_save_path(&icon_name));

    if p.exists() {
        std::fs::remove_file(p).unwrap();
    }

    if url_valid(path) {
        let response = reqwest::blocking::get(path).expect("sending request");

        if response.status().is_success() {
            let content: Bytes = response.bytes().expect("getting image bytes");

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

pub fn image_handle(path: String) -> Option<Icon> {
    if url_valid(&path) {
        if let Ok(response) = blocking::Client::new().get(&path).send() {
            if let Ok(bytes) = response.bytes() {
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
