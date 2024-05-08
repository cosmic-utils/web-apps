use crate::gui::Message;

use cosmic::{
    widget::{dialog, text},
    Element,
};

#[derive(Debug, Clone)]
pub struct Installator;

impl Installator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view(&self) -> Element<Message> {
        let info_message = text("Please wait patiently. Currently installing Papirus icon theme in background to your local directory.");

        dialog("Please wait. Installing icons locally...")
            .control(info_message)
            .into()
    }
}
