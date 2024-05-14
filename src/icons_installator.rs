use crate::gui::Message;

use crate::fl;
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
        let header = text(fl!("icons-installer-header")).size(20);
        let info_message = text(fl!("icons-installer-message"));

        let mut column = Column::new().spacing(10);

        column = column.push(header);
        column = column.push(info_message);

        Container::new(column).into()
    }
}
