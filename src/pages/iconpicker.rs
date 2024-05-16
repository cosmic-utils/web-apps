use crate::fl;
use cosmic::widget::text;
use cosmic::{
    iced::Length,
    iced_widget::Scrollable,
    theme,
    widget::{self, flex_row, Button, Column, Container, Row, TextInput},
    Element,
};

use crate::gui::Message;

#[derive(Debug, Clone, Default)]
pub struct IconPicker {
    pub icon_searching: String,
    pub icons_paths: Vec<String>,
    pub icons: Vec<Icon>,
    pub loading: bool,
}

impl IconPicker {
    pub fn view(&self) -> Element<Message> {
        let search_field = TextInput::new(fl!("icon-name-to-find"), &self.icon_searching)
            .on_input(Message::CustomIconsSearch)
            .on_submit(Message::PerformIconSearch)
            .width(Length::FillPortion(3));

        let loading_state_text = if !self.loading {
            text(fl!("my-icons"))
        } else {
            text(fl!("loading"))
        };

        let my_icons_btn = widget::button(loading_state_text)
            .on_press(Message::MyIcons)
            .padding(8)
            .width(Length::FillPortion(1));

        let custom_icon_btn = widget::button(text(fl!("open")))
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

        let container = {
            let content = Container::new(flex_row(items)).center_x();

            Column::new().push(
                Scrollable::new(content)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
        };

        Column::new()
            .push(controls)
            .push(container)
            .spacing(10)
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
