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
        let info_message = text("This app requires icons to work with. In case we don't have access to your installed icons, we are installing Papirus icon pack to local directory so you can choose one icon for your web app from this pack.");

        dialog("Please wait. Downloading icons...")
            .control(info_message)
            .into()
    }
}
