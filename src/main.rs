use gui::Wam;
use iced::{window, Application, Settings};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod common;
mod gui;
mod supported_browsers;

fn main() -> iced::Result {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    Wam::run(Settings {
        id: Some("wam-rust".to_string()),
        window: window::Settings {
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
