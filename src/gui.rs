use iced::{
    widget::{button, column, pick_list, row, scrollable, text, text_input, Container},
    Application, Command, Length,
};

use crate::common::{get_supported_browsers, get_webapps, WebAppLauncher};

#[derive(Debug, Clone)]
pub enum Buttons {
    Favicon,
    Edit(Box<WebAppLauncher>),
    Delete(Box<WebAppLauncher>),
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    Result,
    Clicked(Buttons),
    Title(String),
    Url(String),
    Arguments(String),
    Browser(String),
}

pub struct Wam {
    pub app_title: String,
    pub app_url: String,
    pub app_parameters: String,
    pub app_browser: String,
}

impl Application for Wam {
    type Executor = iced::executor::Default;

    type Message = AppMessage;

    type Theme = iced::Theme;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Wam {
                app_title: String::new(),
                app_url: String::new(),
                app_parameters: String::new(),
                app_browser: String::from("Browser"),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("WAM Rust - Web app manager written in Rust and Iced rs.")
    }

    fn theme(&self) -> Self::Theme {
        iced::Theme::Dark
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            AppMessage::Title(text) => {
                self.app_title = text;

                println!("{}", self.app_title);

                Command::none()
            }
            AppMessage::Url(url) => {
                self.app_url = url;

                println!("{}", self.app_url);

                Command::none()
            }
            AppMessage::Arguments(args) => {
                self.app_parameters = args;

                println!("{}", self.app_parameters);

                Command::none()
            }
            AppMessage::Clicked(btn) => match btn {
                Buttons::Favicon => Command::none(),
                Buttons::Edit(launcher) => {
                    self.app_title = launcher.name;
                    self.app_url = launcher.url;
                    self.app_parameters = launcher.custom_parameters;
                    self.app_browser = launcher.web_browser;

                    Command::none()
                }
                Buttons::Delete(launcher) => {
                    tracing::info!("{:?}", launcher);
                    Command::none()
                }
            },
            AppMessage::Browser(browser) => {
                self.app_browser = browser;

                Command::none()
            }
            AppMessage::Result => Command::none(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let app_title = text_input("Title", &self.app_title)
            .on_input(AppMessage::Title)
            .padding(10)
            .width(Length::Fixed(340.));
        let app_url = text_input("URL", &self.app_url)
            .on_input(AppMessage::Url)
            .padding(10)
            .width(Length::Fixed(340.));

        let col = column![app_title, app_url].spacing(14);

        let dl_btn = button("DL FAV")
            .on_press(AppMessage::Clicked(Buttons::Favicon))
            .width(Length::Fixed(96.))
            .height(Length::Fixed(96.));
        let fav_btn = button("FAV BTN")
            .on_press(AppMessage::Clicked(Buttons::Favicon))
            .width(Length::Fixed(96.))
            .height(Length::Fixed(96.));

        let row = row![col, dl_btn, fav_btn].spacing(12).width(Length::Fill);

        let app_arguments = text_input("Non-standard arguments", &self.app_parameters)
            .on_input(AppMessage::Arguments)
            .padding(10)
            .width(Length::Fill);

        let mut browsers = Vec::new();

        for browser in get_supported_browsers() {
            let name = browser.name.clone();
            browsers.push(name)
        }

        let app_browsers = pick_list(browsers, Some(self.app_browser.clone()), |browser| {
            AppMessage::Browser(browser)
        })
        .width(Length::Fill)
        .padding(10);

        let app_done = button("Done")
            .on_press(AppMessage::Result)
            .width(Length::Fill)
            .padding(10);

        let heading = text("Installed").size(32.);

        let mut app_list = column!().spacing(10);

        for app in get_webapps() {
            match app {
                Ok(data) => {
                    let name = text(&data.name);
                    let edit = button("Edit")
                        .on_press(AppMessage::Clicked(Buttons::Edit(Box::new(data.clone()))));
                    let delete = button("Delete")
                        .on_press(AppMessage::Clicked(Buttons::Delete(Box::new(data.clone()))));

                    let row = row!(name, edit, delete);
                    app_list = app_list.push(row);
                }
                Err(e) => tracing::error!("Error reading web app: {}", e),
            }
        }

        let scrollable_list = scrollable(app_list).width(Length::Fill);

        let col = column![
            row,
            app_arguments,
            app_browsers,
            app_done,
            heading,
            scrollable_list
        ]
        .spacing(24);

        Container::new(col).padding(30).into()
    }
}
