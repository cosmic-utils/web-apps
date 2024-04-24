mod common;
mod gui;
mod iconpicker;
mod supported_browsers;
mod wam;
mod wrap;

use cosmic::{app::Settings, iced_core::Size};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
// use xdg::BaseDirectories;

use gui::Window;

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
    let mut settings = Settings::default();
    settings = settings.size(Size {
        width: 640.,
        height: 680.,
    });

    cosmic::app::run::<Window>(settings, ())
}

// fn load_icon(path: &str) -> Result<Icon> {}
