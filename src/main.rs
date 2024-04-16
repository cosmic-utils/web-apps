use cosmic::{app::Settings, iced::Size};
use gui::Wam;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;
// use xdg::BaseDirectories;

mod common;
mod gui;
mod supported_browsers;

fn main() -> cosmic::iced::Result {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // let base_dir = BaseDirectories::new().expect("cant follow base directories");
    // let local_share = base_dir.get_data_home();
    // let wam_rust_path = local_share.join("wam-rust/icons/wam-icon.png");
    // let path = wam_rust_path.to_str().expect("cant get icon path");
    // let icon = load_icon(path).expect("app icon not found");

    let settings = Settings::default()
        .antialiasing(true)
        .client_decorations(true)
        .debug(false)
        .default_text_size(16.0)
        .size(Size::new(800., 800.));

    cosmic::app::run::<Wam>(settings, ())
}

// fn load_icon(path: &str) -> Result<Icon> {}
