use crate::{
    common::{get_webapps, WebAppLauncher},
    gui::{Buttons, Message},
};

use cosmic::{
    iced::{Alignment, Length},
    iced_widget::Scrollable,
    widget::{self, text, Button, Column, Container, Row},
    Element,
};

#[derive(Debug, Clone)]
pub struct Wam {
    pub edit_mode: bool,
    pub launcher: Option<WebAppLauncher>,
}

impl Wam {
    pub fn new() -> Self {
        Wam {
            edit_mode: false,
            launcher: None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut app_list = Column::new().spacing(10);
        let webapps = get_webapps();

        for app in webapps.iter() {
            match app {
                Ok(data) => {
                    let num = Button::new(
                        Container::new(text(data.web_browser.name.clone()))
                            .center_x()
                            .center_y(),
                    )
                    .width(Length::FillPortion(1));

                    let app_name = Button::new(
                        Container::new(text(data.name.clone()))
                            .center_x()
                            .center_y(),
                    )
                    .width(Length::FillPortion(4))
                    .style(cosmic::theme::Button::Suggested);

                    let edit = widget::button::icon(widget::icon::from_name("edit-symbolic"))
                        .on_press(Message::Clicked(Buttons::Edit(data.clone())));

                    let delete =
                        widget::button::icon(widget::icon::from_name("edit-delete-symbolic"))
                            .on_press(Message::Clicked(Buttons::Delete(data.clone())));

                    let mut row = Row::new().spacing(10).height(Length::Fixed(50.));
                    let mut row2 = Row::new().spacing(10).height(Length::Fixed(50.));

                    row = row.push(num);
                    row = row.push(app_name);

                    row2 = row2.push(edit);
                    row2 = row2.push(delete);
                    app_list = app_list.push(
                        Row::new()
                            .push(row)
                            .push(row2)
                            .width(Length::Fill)
                            .align_items(Alignment::Center)
                            .spacing(30),
                    );
                }
                Err(e) => tracing::error!("Error reading web app: {}", e),
            }
        }

        let mut installed = Column::new().spacing(30.);

        if !webapps.is_empty() {
            installed = installed.push(text(format!("Installed #{}", webapps.len())).size(22.));

            let scrollable_list = Scrollable::new(app_list).width(Length::Fill);

            installed = installed.push(scrollable_list);
        }

        Container::new(installed).padding(30).into()
    }
}
