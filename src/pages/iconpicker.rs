use cosmic::{
    iced::Length,
    iced_widget::Scrollable,
    theme,
    widget::{self, flex_row, text, text_input, Container},
    Element,
};

use crate::{common, fl, icon_pack_installed, pages::Message};

#[derive(Debug, Clone, Default)]
pub struct IconPicker {
    pub icon_searching: String,
    pub icons: Vec<common::Icon>,
}

impl IconPicker {
    pub fn view(&self) -> Element<Message> {
        let search_field = text_input(fl!("icon-name-to-find"), &self.icon_searching)
            .on_input(Message::CustomIconsSearch)
            .on_submit(Message::PerformIconSearch)
            .width(Length::FillPortion(3));

        let my_icons_btn = widget::button::custom(text(fl!("my-icons")))
            .on_press(Message::MyIcons)
            .padding(8)
            .width(Length::FillPortion(1));

        let custom_icon_btn = widget::button::custom(text(fl!("open")))
            .on_press(Message::OpenIconPickerDialog)
            .padding(8)
            .width(Length::FillPortion(1));

        let mut controls = widget::row()
            .spacing(10)
            .push(search_field)
            .push(my_icons_btn)
            .push(custom_icon_btn);

        if !icon_pack_installed() {
            controls = controls.push(
                widget::button::custom(text(fl!("download")))
                    .on_press(Message::DownloadIconsPack)
                    .padding(8)
                    .width(Length::FillPortion(1)),
            );
        }

        let mut items: Vec<Element<Message>> = Vec::new();

        for ico in self.icons.iter() {
            let btn = match ico.clone().icon {
                common::IconType::Raster(icon) => widget::button::custom(widget::image(icon))
                    .width(Length::Fixed(64.))
                    .height(Length::Fixed(64.))
                    .on_press(Message::ChangeIcon(ico.clone()))
                    .style(theme::Button::Icon),
                common::IconType::Svg(icon) => widget::button::custom(widget::svg(icon))
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
