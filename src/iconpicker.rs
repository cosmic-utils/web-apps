use cosmic::widget::flex_row;
use cosmic::{
    iced::Length,
    iced_widget::Scrollable,
    theme,
    widget::{self, Button, Column, Container, Row, TextInput},
    Element,
};

use crate::gui::Message;

#[derive(Debug, Clone, Default)]
pub struct IconPicker {
    pub icon_searching: String,
    pub icons_paths: Vec<String>,
    pub icons: Vec<Icon>,
}

impl IconPicker {
    pub fn view(&self) -> Element<Message> {
        let search_field = TextInput::new("Icon name to find", &self.icon_searching)
            .on_input(Message::CustomIconsSearch)
            .on_submit(Message::PerformIconSearch)
            .width(Length::FillPortion(3));

        let my_icons_btn = widget::button("My Icons")
            .on_press(Message::MyIcons)
            .padding(8)
            .width(Length::FillPortion(1));

        let custom_icon_btn = widget::button("Open")
            .on_press(Message::OpenIconPickerDialog)
            .padding(8)
            .width(Length::FillPortion(1));

        let mut controls = Row::new().spacing(10);
        controls = controls.push(search_field);
        controls = controls.push(my_icons_btn);
        controls = controls.push(custom_icon_btn);

        let mut items: Vec<Element<Message>> = Vec::new();

        for ico in self.icons.iter() {
            let btn = match ico.clone().icon {
                IconType::Raster(icon) => Button::new(cosmic::widget::image(icon))
                    .width(Length::Fixed(64.))
                    .height(Length::Fixed(64.))
                    .on_press(Message::ChangeIcon(ico.clone()))
                    .style(theme::Button::Icon),
                IconType::Svg(icon) => Button::new(cosmic::widget::svg(icon))
                    .width(Length::Fixed(64.))
                    .height(Length::Fixed(64.))
                    .on_press(Message::ChangeIcon(ico.clone()))
                    .style(theme::Button::Icon),
            };
            items.push(btn.into());
        }

        let container = Container::new(flex_row(items)).center_x();

        let scrollable = Scrollable::new(container)
            .width(Length::Fill)
            .height(Length::Fill);

        Column::new()
            .push(controls)
            .push(scrollable)
            .spacing(16)
            .into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IconType {
    Raster(widget::image::Handle),
    Svg(widget::svg::Handle),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Icon {
    pub icon: IconType,
    pub path: String,
}

impl Icon {
    pub fn new(icon: IconType, path: String) -> Self {
        Self { icon, path }
    }
}
