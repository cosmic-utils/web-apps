use crate::{fl, pages::Message};

use cosmic::{
    widget::{self},
    Element,
};

#[derive(Debug, Clone, Default)]
pub struct Installator;

impl Installator {
    pub fn view(&self, output: String) -> Element<Message> {
        widget::container(widget::scrollable(
            widget::column()
                .padding(8)
                .spacing(20)
                .push(widget::text::title1(fl!("icons-installer-header")))
                .push(widget::text::heading(fl!("icons-installer-message")))
                .push(widget::text::monotext(output)),
        ))
        .padding(30)
        .max_width(1000)
        .into()
    }
}
