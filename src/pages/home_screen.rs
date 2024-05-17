use crate::{
    common::{get_webapps, icon_cache_get, WebAppLauncher},
    fl,
    pages::{Buttons, Message},
};

use cosmic::widget::{column, inline_input};
use cosmic::{
    iced::Alignment,
    iced_widget::Scrollable,
    style, theme,
    widget::{self, text, Container},
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
        let mut app_list = column().spacing(12).align_items(Alignment::Center);
        let webapps = get_webapps();

        for app in webapps.iter().flatten() {
            let used_browser =
                Container::new(inline_input(app.web_browser.name.clone()).width(120));

            let app_name = Container::new(inline_input(app.name.clone()));

            let edit = widget::button(icon_cache_get("edit-symbolic", 16))
                .on_press(Message::Clicked(Buttons::Edit(app.clone())))
                .padding(8)
                .style(style::Button::Icon);

            let delete = widget::button(icon_cache_get("edit-delete-symbolic", 16))
                .on_press(Message::Clicked(Buttons::Delete(app.clone())))
                .padding(8)
                .style(style::Button::Icon);

            let row = cosmic::widget::row()
                .spacing(10)
                .push(used_browser)
                .push(app_name);
            let row2 = cosmic::widget::row().spacing(10).push(edit).push(delete);

            app_list = app_list.push(
                Container::new(cosmic::widget::row().push(row).push(row2))
                    .style(theme::Container::List),
            );
        }

        let mut final_content = column().spacing(20).align_items(Alignment::Center);

        if !webapps.is_empty() {
            final_content = final_content.push(Scrollable::new(app_list));
        } else {
            final_content = final_content.push(text(fl!("not-installed-header")).size(20));
        };

        Container::new(final_content)
            .padding(30)
            .max_width(1000)
            .into()
    }
}
