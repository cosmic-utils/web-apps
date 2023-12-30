use gui::Wam;
use iced::{
    window::{self},
    Application, Settings,
};

use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use xdg::BaseDirectories;

mod common;
mod gui;
mod supported_browsers;

fn main() -> iced::Result {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let base_dir = BaseDirectories::new().expect("cant follow base directories");
    let local_share = base_dir.get_data_home();
    let wam_rust_path = local_share.join("wam-rust/icons/wam-icon.png");
    let path = wam_rust_path.to_str().expect("cant get icon path");
    let icon = load_icon(path).expect("app icon not found");

    Wam::run(Settings {
        id: Some("wam-rust".to_string()),
        window: window::Settings {
            icon: Some(icon),
            size: iced::Size {
                width: 600.,
                height: 740.,
            },
            resizable: false,
            ..Default::default()
        },
        ..Default::default()
    })
}

fn load_icon(path: &str) -> Result<window::Icon, window::icon::Error> {
    window::icon::from_file(path)
}
