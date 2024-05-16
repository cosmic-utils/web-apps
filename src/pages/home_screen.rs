use crate::{
    common::{get_webapps, icon_cache_get, WebAppLauncher},
    fl,
    pages::{Buttons, Message},
};

use cosmic::widget::inline_input;
use cosmic::{
    iced::Alignment,
    iced_widget::Scrollable,
    style, theme,
    widget::{self, text, Column, Container, Row},
    Element,
};

#[derive(Debug, Clone)]
pub struct Home {
    pub edit_mode: bool,
    pub launcher: Option<WebAppLauncher>,
}

impl Home {
    pub fn new() -> Self {
        Home {
            edit_mode: false,
            launcher: None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut app_list = Column::new().spacing(12).align_items(Alignment::Center);
        let webapps = get_webapps();

        for app in webapps.iter().flatten() {
            let num = Container::new(inline_input(app.web_browser.name.clone()).width(120));

            let app_name = Container::new(inline_input(app.name.clone()));

            let edit = widget::button(icon_cache_get("edit-symbolic", 16))
                .on_press(Message::Clicked(Buttons::Edit(app.clone())))
                .padding(8)
                .style(style::Button::Icon);

            let delete = widget::button(icon_cache_get("edit-delete-symbolic", 16))
                .on_press(Message::Clicked(Buttons::Delete(app.clone())))
                .padding(8)
                .style(style::Button::Icon);

            let mut row = Row::new().spacing(10);
            let mut row2 = Row::new().spacing(10);

            row = row.push(num);
            row = row.push(app_name);

            row2 = row2.push(edit);
            row2 = row2.push(delete);
            app_list = app_list.push(
                Container::new(Row::new().push(row).push(row2)).style(theme::Container::List),
            );
        }

        let mut final_content = Column::new().spacing(20).align_items(Alignment::Center);

        if !webapps.is_empty() {
            let scrollable_list = Container::new(Scrollable::new(app_list));
            final_content = final_content.push(scrollable_list);
        } else {
            final_content = final_content.push(text(fl!("not-installed-header")).size(20));
        };

        Container::new(final_content)
            .padding(30)
            .max_width(1000)
            .into()
    }
}
