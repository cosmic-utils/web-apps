use crate::{
    common::icon_cache_get,
    fl,
    launcher::{get_webapps, WebAppLauncher},
    pages::{Buttons, Message},
};

use cosmic::widget::{column, editable_input, inline_input};
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
    pub edit_appname: bool,
    pub new_app_name: String,
    pub launcher: Option<WebAppLauncher>,
}

impl Home {
    pub fn new() -> Self {
        Home {
            edit_mode: false,
            edit_appname: false,
            new_app_name: String::new(),
            launcher: None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut app_list = column().spacing(12).align_x(Alignment::Center);
        let webapps = get_webapps();

        for app in webapps.iter() {
            let used_browser =
                Container::new(inline_input("Web browser", app.browser.name.clone()).width(200));

            let app_name = Container::new(
                editable_input(
                    app.name.clone(),
                    &self.new_app_name,
                    self.edit_appname,
                    Message::EditAppName,
                )
                .on_input(Message::AppNameInput)
                .on_submit(Message::Clicked(Buttons::AppNameSubmit(app.clone()))),
            );

            let edit = widget::button::custom(icon_cache_get("application-menu-symbolic", 16))
                .on_press(Message::Clicked(Buttons::Edit(app.clone())))
                .padding(10)
                .class(style::Button::Icon);

            let delete = widget::button::custom(icon_cache_get("edit-delete-symbolic", 16))
                .on_press(Message::Clicked(Buttons::Delete(app.clone())))
                .padding(10)
                .class(style::Button::Icon);

            let row = cosmic::widget::row()
                .spacing(4)
                .push(used_browser)
                .push(app_name);
            let row2 = cosmic::widget::row().spacing(10).push(edit).push(delete);

            app_list = app_list.push(
                Container::new(cosmic::widget::row().push(row).push(row2))
                    .class(theme::Container::List),
            );
        }

        let mut final_content = column().spacing(20).align_x(Alignment::Center);

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
