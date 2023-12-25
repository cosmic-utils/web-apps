use gui::Wam;
use iced::{window, Application, Settings};

mod gui;

fn main() -> iced::Result {
    Wam::run(Settings {
        id: Some("wam_rust".to_string()),
        window: window::Settings {
            size: iced::Size {
                width: 600.,
                height: 600.,
            },
            resizable: false,
            ..Default::default()
        },
        ..Default::default()
    })
}
