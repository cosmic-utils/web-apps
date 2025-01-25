use crate::{fl, pages::Message};

use cosmic::{
    widget::{self, text, Container},
    Element,
};

#[derive(Debug, Clone, Default)]
pub struct Installator;

impl Installator {
    pub fn view(&self) -> Element<Message> {
        let header = text(fl!("icons-installer-header")).size(20);
        let info_message = text(fl!("icons-installer-message"));

        let column = widget::column().spacing(10).push(header).push(info_message);

        Container::new(column).padding(30).max_width(1000).into()
    }
}
