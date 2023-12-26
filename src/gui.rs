use iced::{
    widget::{button, column, pick_list, row, scrollable, text, text_input, toggler, Container},
    Alignment, Application, Command, Length,
};

use crate::common::{get_supported_browsers, get_webapps, Browser, WebAppLauncher};

#[derive(Debug, Clone)]
pub enum Buttons {
    Favicon,
    Edit(Box<WebAppLauncher>),
    Delete(Box<WebAppLauncher>),
    Navbar(bool),
    Incognito(bool),
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    Result,
    Clicked(Buttons),
    Title(String),
    Url(String),
    Arguments(String),
    Browser(Browser),
    Category(String),
}

pub struct Wam {
    pub app_title: String,
    pub app_url: String,
    pub app_icon: String,
    pub app_parameters: String,
    pub app_category: String,
    pub app_browser_name: String,
    pub app_browser: Browser,
    app_browsers: Vec<Browser>,
    pub app_navbar: bool,
    pub app_incognito: bool,
    pub app_isolated: bool,
}

impl Application for Wam {
    type Executor = iced::executor::Default;

    type Message = AppMessage;

    type Theme = iced::Theme;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let browsers = get_supported_browsers();
        let browser = &browsers[0];

        (
            Wam {
                app_title: String::new(),
                app_url: String::new(),
                app_icon: String::new(),
                app_parameters: String::new(),
                app_category: String::from("Web"),
                app_browser_name: String::from("Browser"),
                app_browser: browser.clone(),
                app_browsers: browsers,
                app_navbar: false,
                app_incognito: false,
                app_isolated: true,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("WAM Rust - Web App Manager written in Rust and Iced rs.")
    }

    fn theme(&self) -> Self::Theme {
        iced::Theme::Dark
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            AppMessage::Title(text) => {
                self.app_title = text;

                Command::none()
            }
            AppMessage::Url(url) => {
                self.app_url = url;

                Command::none()
            }
            AppMessage::Arguments(args) => {
                self.app_parameters = args;

                Command::none()
            }
            AppMessage::Clicked(btn) => match btn {
                Buttons::Favicon => Command::none(),
                Buttons::Edit(launcher) => {
                    self.app_title = launcher.name;
                    self.app_url = launcher.url;
                    self.app_parameters = launcher.custom_parameters;
                    self.app_category = launcher.category;
                    self.app_browser =
                        Browser::web_browser(launcher.web_browser.name).expect("browser not found");
                    self.app_navbar = launcher.navbar;
                    self.app_incognito = launcher.is_incognito;

                    Command::none()
                }
                Buttons::Delete(launcher) => {
                    let _ = launcher.delete();

                    Command::none()
                }
                Buttons::Navbar(selected) => {
                    self.app_navbar = selected;

                    Command::none()
                }
                Buttons::Incognito(selected) => {
                    self.app_incognito = selected;

                    Command::none()
                }
            },
            AppMessage::Browser(browser) => {
                self.app_browser = browser;

                Command::none()
            }
            AppMessage::Result => {
                let launcher = WebAppLauncher::new(
                    self.app_title.clone(),
                    self.app_url.clone(),
                    self.app_icon.clone(),
                    self.app_category.clone(),
                    self.app_browser.clone(),
                    self.app_parameters.clone(),
                    self.app_isolated,
                    self.app_navbar,
                    self.app_incognito,
                );

                if launcher.is_valid {
                    let _ = launcher.create();
                }

                Command::none()
            }
            AppMessage::Category(category) => {
                self.app_category = category;

                Command::none()
            }
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

        let categories = [
            String::from("Web"),
            String::from("Accesories"),
            String::from("Education"),
            String::from("Games"),
            String::from("Graphics"),
            String::from("Internet"),
            String::from("Office"),
            String::from("Programming"),
            String::from("Sound & Video"),
        ];

        let category = pick_list(
            categories.to_vec(),
            Some(self.app_category.clone()),
            AppMessage::Category,
        )
        .width(Length::Fill)
        .padding(10);

        let navbar = toggler(String::from("Nav Bar"), self.app_navbar, |b| {
            AppMessage::Clicked(Buttons::Navbar(b))
        })
        .width(Length::Fill);

        let incognito = toggler(String::from("Private Mode"), self.app_incognito, |b| {
            AppMessage::Clicked(Buttons::Incognito(b))
        })
        .width(Length::Fill);

        let cat_row = row![category, incognito, navbar]
            .align_items(Alignment::Center)
            .spacing(20);

        let app_browsers = pick_list(
            self.app_browsers.clone(),
            Some(self.app_browser.clone()),
            AppMessage::Browser,
        )
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
                    let edit = button("Edit")
                        .on_press(AppMessage::Clicked(Buttons::Edit(Box::new(data.clone()))))
                        .padding(10);
                    let delete = button("Delete")
                        .on_press(AppMessage::Clicked(Buttons::Delete(Box::new(data.clone()))))
                        .padding(10);
                    let name = text(&data.name).size(26.);

                    let row = row![edit, delete, name]
                        .spacing(10)
                        .align_items(Alignment::Center);
                    app_list = app_list.push(row);
                }
                Err(e) => tracing::error!("Error reading web app: {}", e),
            }
        }

        let scrollable_list = scrollable(app_list).width(Length::Fill);

        let col = column![
            row,
            app_arguments,
            cat_row,
            app_browsers,
            app_done,
            heading,
            scrollable_list
        ]
        .spacing(24);

        Container::new(col).padding(30).into()
    }
}
