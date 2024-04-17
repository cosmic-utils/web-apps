use cosmic::{
    iced::Length,
    iced_widget::Scrollable,
    widget::{Button, Column, Container, TextInput},
    Element,
};

use crate::gui::Message;

#[derive(Debug, Clone)]
pub struct IconPicker {
    pub icon_searching: String,
    pub icons_paths: Vec<String>,
    pub icons: Vec<Icon>,
}

impl IconPicker {
    pub fn new() -> Self {
        IconPicker {
            icon_searching: String::new(),
            icons_paths: Vec::new(),
            icons: Vec::new(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let search_field = TextInput::new("Search for icon", &self.icon_searching)
            .on_input(Message::CustomIconsSearch)
            .on_submit(Message::PerformIconSearch)
            .padding(10)
            .width(Length::Fill);

        let mut container = crate::wrap::Wrap::new();

        for ico in self.icons.iter() {
            let btn = match ico.clone().icon {
                IconType::Raster(icon) => Button::new(cosmic::widget::image(icon))
                    .width(Length::Fixed(96.))
                    .height(Length::Fixed(96.))
                    .on_press(Message::Favicon(ico.path.clone())),
                IconType::Svg(icon) => Button::new(cosmic::widget::svg(icon))
                    .width(Length::Fixed(96.))
                    .height(Length::Fixed(96.))
                    .on_press(Message::Favicon(ico.path.clone())),
            };
            container = container.push(btn);
        }

        let scrollable = Scrollable::new(container)
            .width(Length::Fill)
            .height(Length::Fill);

        let col = Column::new().push(search_field).push(scrollable);

        Container::new(col).into()
    }
}

#[derive(Debug, Clone)]
pub enum IconType {
    Raster(cosmic::widget::image::Handle),
    Svg(cosmic::widget::svg::Handle),
}

#[derive(Debug, Clone)]
pub struct Icon {
    pub icon: IconType,
    pub path: String,
}

impl Icon {
    pub fn new(icon: IconType, path: String) -> Self {
        Self { icon, path }
    }
}
