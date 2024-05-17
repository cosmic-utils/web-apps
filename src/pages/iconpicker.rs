use cosmic::{
    iced::Length,
    iced_widget::Scrollable,
    theme,
    widget::{self, flex_row, text, text_input, Container},
    Element,
};

use crate::{fl, pages::Message};

#[derive(Debug, Clone, Default)]
pub struct IconPicker {
    pub icon_searching: String,
    pub icons_paths: Vec<String>,
    pub icons: Vec<Icon>,
    pub loading: bool,
}

impl IconPicker {
    pub fn view(&self) -> Element<Message> {
        let search_field = text_input(fl!("icon-name-to-find"), &self.icon_searching)
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

        let controls = widget::row()
            .spacing(10)
            .push(search_field)
            .push(my_icons_btn)
            .push(custom_icon_btn);

        let mut items: Vec<Element<Message>> = Vec::new();

        for ico in self.icons.iter() {
            let btn = match ico.clone().icon {
                IconType::Raster(icon) => widget::button(widget::image(icon))
                    .width(Length::Fixed(64.))
                    .height(Length::Fixed(64.))
                    .on_press(Message::ChangeIcon(ico.clone()))
                    .style(theme::Button::Icon),
                IconType::Svg(icon) => widget::button(widget::svg(icon))
                    .width(Length::Fixed(64.))
                    .height(Length::Fixed(64.))
                    .on_press(Message::ChangeIcon(ico.clone()))
                    .style(theme::Button::Icon),
            };
            items.push(btn.into());
        }

        let container = {
            let content = Container::new(flex_row(items));

            Scrollable::new(content)
                .width(Length::Fill)
                .height(Length::Fill)
        };

        let final_column = widget::column().push(controls).push(container).spacing(10);

        Container::new(final_column)
            .padding(30)
            .max_width(1000)
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
