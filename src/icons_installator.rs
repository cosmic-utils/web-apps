use crate::gui::Message;

use cosmic::{
    widget::{text, Column, Container},
    Element,
};

#[derive(Debug, Clone)]
pub struct Installator;

impl Installator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view(&self) -> Element<Message> {
        let header = text("Please wait. Downloading icons...").size(20);
        let info_message = text("This app requires icons to work with. In case we don't have access to your installed icons, we are installing Papirus icon pack to local directory so you can choose one icon for your web app from this pack.");

        let mut column = Column::new().spacing(10);

        column = column.push(header);
        column = column.push(info_message);

        Container::new(column).into()
    }
}
