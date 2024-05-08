use crate::{
    common::{get_supported_browsers, url_valid, Browser, BrowserType},
    gui::{self},
    iconpicker,
    warning::{WarnMessages, Warning},
};

use cosmic::{
    iced::{id, Alignment, Length},
    theme,
    widget::{dialog, dropdown, toggler, Button, Column, Container, Row, TextInput},
    Command, Element,
};

#[derive(Debug, Clone)]
pub struct AppCreator {
    pub app_codename: Option<String>,
    pub app_title_id: id::Id,
    pub app_title: String,
    pub app_url: String,
    pub app_icon: String,
    pub app_parameters: String,
    pub app_categories: Vec<String>,
    pub app_category: String,
    pub app_browser_name: String,
    pub app_browser: Browser,
    pub app_navbar: bool,
    pub app_incognito: bool,
    pub app_isolated: bool,
    pub selected_icon: Option<iconpicker::Icon>,
    pub app_browsers: Vec<Browser>,
    pub warning: Warning,
    pub dialog_open: bool,
    pub edit_mode: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Title(String),
    Url(String),
    Arguments(String),
    Browser(usize),
    Category(usize),

    Clicked(Buttons),
}

#[derive(Debug, Clone)]
pub enum Buttons {
    Navbar(bool),
    IsolatedProfile(bool),
    Incognito(bool),
}

impl AppCreator {
    pub fn new() -> Self {
        let mut browsers = get_supported_browsers();
        browsers.insert(
            0,
            Browser::new(
                crate::common::BrowserType::SelectOne,
                "Select browser",
                "",
                "",
            ),
        );

        let browser = &browsers[0];

        let starting_warns = vec![
            WarnMessages::Info,
            WarnMessages::AppName,
            WarnMessages::AppUrl,
            WarnMessages::AppIcon,
            WarnMessages::AppBrowser,
        ];

        let warn_element = Warning::new(starting_warns, true);

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

        AppCreator {
            app_codename: None,
            app_title_id: id::Id::new("app-title"),
            app_title: String::new(),
            app_url: String::new(),
            app_icon: String::new(),
            app_parameters: String::new(),
            app_categories: categories.to_vec(),
            app_category: String::from("Web"),
            app_browser_name: String::from("Browser"),
            app_browser: browser.clone(),
            app_navbar: false,
            app_incognito: false,
            app_isolated: true,
            selected_icon: None,
            app_browsers: browsers,
            warning: warn_element,
            dialog_open: false,
            edit_mode: false,
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Title(title) => {
                self.app_title = title;

                if self.app_title.len() >= 3 {
                    self.warning.remove_warn(WarnMessages::AppName);
                } else {
                    self.warning.push_warn(WarnMessages::AppName);
                }

                Command::none()
            }
            Message::Url(url) => {
                if url_valid(&url) {
                    self.warning.remove_warn(WarnMessages::AppUrl);
                } else {
                    self.warning.push_warn(WarnMessages::AppUrl);
                }

                self.app_url = url;
                Command::none()
            }
            Message::Arguments(args) => {
                self.app_parameters = args;
                Command::none()
            }
            Message::Browser(idx) => {
                let browser = &self.app_browsers[idx];
                match browser._type {
                    BrowserType::SelectOne => self.warning.push_warn(WarnMessages::AppBrowser),
                    _ => self.warning.remove_warn(WarnMessages::AppBrowser),
                };

                self.app_browser = browser.clone();
                Command::none()
            }
            Message::Category(idx) => {
                self.app_category = self.app_categories[idx].clone();
                Command::none()
            }

            Message::Clicked(btns) => match btns {
                Buttons::Navbar(selected) => {
                    self.app_navbar = selected;

                    Command::none()
                }
                Buttons::IsolatedProfile(selected) => {
                    self.app_isolated = selected;

                    Command::none()
                }
                Buttons::Incognito(selected) => {
                    self.app_incognito = selected;

                    Command::none()
                }
            },
        }
    }

    fn icon_picker_icon(&self, icon: Option<iconpicker::Icon>) -> Element<gui::Message> {
        let ico = if let Some(ico) = icon {
            match ico.icon {
                iconpicker::IconType::Raster(data) => Button::new(cosmic::widget::image(data))
                    .on_press(gui::Message::OpenIconPicker)
                    .width(Length::Fixed(48.))
                    .height(Length::Fixed(48.))
                    .style(theme::Button::Transparent),

                iconpicker::IconType::Svg(data) => Button::new(cosmic::widget::svg(data))
                    .on_press(gui::Message::OpenIconPicker)
                    .width(Length::Fixed(48.))
                    .height(Length::Fixed(48.))
                    .style(theme::Button::Transparent),
            }
        } else {
            let default_ico: &'static [u8] = include_bytes!("../assets/icons/moleskine-icon.svg");
            let handler = cosmic::widget::svg::Handle::from_memory(default_ico);
            let default = cosmic::widget::svg(handler);

            Button::new(default)
                .on_press(gui::Message::OpenIconPicker)
                .width(Length::Fixed(48.))
                .height(Length::Fixed(48.))
                .style(theme::Button::Transparent)
        };

        Container::new(ico).center_x().center_y().into()
    }

    pub fn view(&self) -> Element<gui::Message> {
        let app_title = TextInput::new("Title", &self.app_title)
            .id(self.app_title_id.clone())
            .on_input(|s| gui::Message::Creator(Message::Title(s)))
            .width(Length::Fill);
        let app_url = TextInput::new("URL", &self.app_url)
            .on_input(|s| gui::Message::Creator(Message::Url(s)))
            .width(Length::Fill);

        let mut col = Column::new().spacing(14);
        col = col.push(app_title);
        col = col.push(app_url);

        let search_ico: &'static [u8] = include_bytes!("../assets/icons/search.svg");
        let search_ico_handler =
            cosmic::widget::svg(cosmic::widget::svg::Handle::from_memory(search_ico))
                .width(Length::Fixed(48.))
                .height(Length::Fixed(48.));

        let dl_btn = Button::new(
            Container::new(Button::new(search_ico_handler).style(theme::Button::Transparent))
                .center_x()
                .center_y(),
        )
        .on_press(gui::Message::Clicked(gui::Buttons::SearchFavicon))
        .width(Length::Fixed(82.))
        .height(Length::Fixed(82.))
        .style(theme::Button::Suggested);

        let icon = self.icon_picker_icon(self.selected_icon.clone());

        let icon = Button::new(icon)
            .width(Length::Fixed(82.))
            .height(Length::Fixed(82.))
            .on_press(gui::Message::OpenIconPicker)
            .style(theme::Button::Standard);

        let mut row = Row::new().spacing(12).width(Length::Fill);

        row = row.push(col);
        row = row.push(dl_btn);
        row = row.push(icon);

        let app_arguments = TextInput::new("Non-standard arguments", &self.app_parameters)
            .on_input(|s| gui::Message::Creator(Message::Arguments(s)))
            .width(Length::Fill);

        let category = dropdown(&self.app_categories, Some(0), move |index| {
            gui::Message::Creator(Message::Category(index))
        })
        .width(Length::Fixed(200.));

        let browser_specific = match self.app_browser._type {
            crate::common::BrowserType::Firefox => {
                toggler(String::from("Nav Bar"), self.app_navbar, |b| {
                    gui::Message::Creator(Message::Clicked(Buttons::Navbar(b)))
                })
                .width(Length::Fill)
            }
            crate::common::BrowserType::FirefoxFlatpak => {
                toggler(String::from("Nav Bar"), self.app_navbar, |b| {
                    gui::Message::Creator(Message::Clicked(Buttons::Navbar(b)))
                })
                .width(Length::Fill)
            }
            crate::common::BrowserType::Librewolf => {
                toggler(String::from("Nav Bar"), self.app_navbar, |b| {
                    gui::Message::Creator(Message::Clicked(Buttons::Navbar(b)))
                })
                .width(Length::Fill)
            }
            crate::common::BrowserType::WaterfoxFlatpak => {
                toggler(String::from("Nav Bar"), self.app_navbar, |b| {
                    gui::Message::Creator(Message::Clicked(Buttons::Navbar(b)))
                })
                .width(Length::Fill)
            }
            _ => toggler(String::from("Isolated Profile"), self.app_isolated, |b| {
                gui::Message::Creator(Message::Clicked(Buttons::IsolatedProfile(b)))
            })
            .width(Length::Fill),
        };

        let incognito = toggler(String::from("Private Mode"), self.app_incognito, |b| {
            gui::Message::Creator(Message::Clicked(Buttons::Incognito(b)))
        })
        .width(Length::Fill);

        let mut cat_row = Row::new().spacing(20).align_items(Alignment::Center);
        cat_row = cat_row.push(category);
        cat_row = cat_row.push(incognito);
        cat_row = cat_row.push(browser_specific);

        let app_browsers = dropdown(&self.app_browsers, Some(0), |idx| {
            gui::Message::Creator(Message::Browser(idx))
        })
        .width(Length::Fixed(200.));

        let app_done_btn_text = if self.edit_mode { "Edit" } else { "Create" };

        let app_done = Button::new(Container::new(app_done_btn_text).center_x().center_y())
            .on_press(gui::Message::Result)
            .width(Length::Fill)
            .style(theme::Button::Suggested);
        let creator_close = Button::new(Container::new("Close").center_x().center_y())
            .on_press(gui::Message::CloseCreator)
            .width(Length::Fill)
            .style(theme::Button::Destructive);

        let mut browsers_row = Row::new().spacing(20);
        browsers_row = browsers_row.push(app_browsers);
        browsers_row = browsers_row.push(app_done);
        browsers_row = browsers_row.push(creator_close);

        let mut col = Column::new().spacing(20);

        if self.warning.show {
            col = col.push(self.warning.view());
        }

        col = col.push(row);
        col = col.push(app_arguments);
        col = col.push(cat_row);
        col = col.push(browsers_row);

        let dialog_header = if self.edit_mode {
            format!("Edit {}", self.app_title)
        } else {
            "Create new Web App".to_string()
        };
        dialog(dialog_header).control(col).into()
    }
}
