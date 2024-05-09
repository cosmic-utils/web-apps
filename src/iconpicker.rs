use crate::gui::{self, Message};

use cosmic::{
    iced::{id, Length},
    iced_widget::Scrollable,
    theme,
    widget::{Button, Column, Container, TextInput},
    Element,
};

#[derive(Debug, Clone)]
pub struct IconPicker {
    pub searching_id: id::Id,
    pub icon_searching: String,
    pub icons_paths: Vec<String>,
    pub icons: Vec<Icon>,
}

impl IconPicker {
    pub fn new() -> Self {
        IconPicker {
            searching_id: id::Id::new("searching"),
            icon_searching: String::new(),
            icons_paths: Vec::new(),
            icons: Vec::new(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let search_field = TextInput::new("Search for icon", &self.icon_searching)
            .id(self.searching_id.clone())
            .on_input(gui::Message::CustomIconsSearch)
            .on_submit(gui::Message::PerformIconSearch)
            .width(Length::FillPortion(3));

        let mut wrapper = crate::wrap::Wrap::new().spacing(8.);

        for ico in self.icons.iter() {
            let btn = match ico.clone().icon {
                IconType::Raster(icon) => Button::new(cosmic::widget::image(icon))
                    .width(Length::Fixed(64.))
                    .height(Length::Fixed(64.))
                    .on_press(Message::Favicon(ico.path.clone()))
                    .style(theme::Button::Icon),
                IconType::Svg(icon) => Button::new(cosmic::widget::svg(icon))
                    .width(Length::Fixed(64.))
                    .height(Length::Fixed(64.))
                    .on_press(Message::Favicon(ico.path.clone()))
                    .style(theme::Button::Icon),
            };
            wrapper = wrapper.push(btn);
        }

        let container = Container::new(wrapper).center_x();

        let scrollable = Scrollable::new(container)
            .width(Length::Fill)
            .height(Length::Fill);

        Column::new()
            .push(search_field)
            .push(scrollable)
            .spacing(16)
            .into()
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
