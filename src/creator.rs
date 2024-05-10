use crate::{
    common::{get_supported_browsers, icon_cache_get, url_valid, Browser, BrowserType},
    gui, iconpicker,
    warning::{WarnMessages, Warning},
};

use cosmic::{
    iced::{id, Alignment, Length},
    style, theme,
    widget::{self, dropdown, focus, toggler, Button, Column, Container, Row, TextInput},
    Command, Element,
};

#[derive(Debug, Clone)]
pub struct AppCreator {
    pub app_codename: Option<String>,
    pub app_title_id: id::Id,
    pub app_title: String,
    pub app_url: String,
    pub app_url_id: id::Id,
    pub app_icon: String,
    pub app_parameters: String,
    pub app_categories: Vec<String>,
    pub app_category: String,
    pub selected_category: usize,
    pub app_browser_name: String,
    pub app_browser: Browser,
    pub app_navbar: bool,
    pub app_incognito: bool,
    pub app_isolated: bool,
    pub selected_icon: Option<iconpicker::Icon>,
    pub app_browsers: Vec<Browser>,
    pub selected_browser: Option<usize>,
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
        let browsers = get_supported_browsers();
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
            app_url_id: id::Id::new("app-url"),
            app_icon: String::new(),
            app_parameters: String::new(),
            app_categories: categories.to_vec(),
            app_category: String::from("Web"),
            selected_category: 0,
            app_browser_name: String::from("Browser"),
            app_browser: browser.clone(),
            app_navbar: false,
            app_incognito: false,
            app_isolated: true,
            selected_icon: None,
            app_browsers: browsers,
            selected_browser: Some(0),
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
                    focus(self.app_title_id.clone())
                } else {
                    self.warning.push_warn(WarnMessages::AppName);
                    focus(self.app_title_id.clone())
                }
            }
            Message::Url(url) => {
                self.app_url = url;

                if url_valid(&self.app_url) {
                    self.warning.remove_warn(WarnMessages::AppUrl);
                    focus(self.app_url_id.clone())
                } else {
                    self.warning.push_warn(WarnMessages::AppUrl);
                    focus(self.app_url_id.clone())
                }
            }
            Message::Arguments(args) => {
                self.app_parameters = args;
                Command::none()
            }
            Message::Browser(idx) => {
                let browser = &self.app_browsers[idx];
                self.selected_browser = Some(idx);
                match browser._type {
                    BrowserType::NoBrowser => self.warning.push_warn(WarnMessages::AppBrowser),
                    _ => self.warning.remove_warn(WarnMessages::AppBrowser),
                };

                self.app_browser = browser.clone();
                Command::none()
            }
            Message::Category(idx) => {
                self.app_category.clone_from(&self.app_categories[idx]);
                self.selected_category = idx;
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
                iconpicker::IconType::Raster(data) => widget::button(cosmic::widget::image(data))
                    .width(Length::Fixed(48.))
                    .height(Length::Fixed(48.))
                    .style(style::Button::Icon),

                iconpicker::IconType::Svg(data) => widget::button(cosmic::widget::svg(data))
                    .width(Length::Fixed(48.))
                    .height(Length::Fixed(48.))
                    .style(style::Button::Icon),
            }
        } else {
            widget::button(icon_cache_get("folder-pictures-symbolic", 16))
                .width(Length::Fixed(48.))
                .height(Length::Fixed(48.))
                .style(style::Button::Icon)
        };

        Container::new(ico).center_x().center_y().into()
    }

    fn download_button(&self) -> Element<gui::Message> {
        Container::new(
            widget::button(icon_cache_get("folder-download-symbolic", 16))
                .width(Length::Fixed(48.))
                .height(Length::Fixed(48.))
                .style(style::Button::Icon),
        )
        .center_x()
        .center_y()
        .into()
    }

    pub fn view(&self) -> Element<gui::Message> {
        let app_title = TextInput::new("Title", &self.app_title)
            .id(self.app_title_id.clone())
            .on_input(|s| gui::Message::Creator(Message::Title(s)))
            .width(Length::Fill);
        let app_url = TextInput::new("URL", &self.app_url)
            .id(self.app_url_id.clone())
            .on_input(|s| gui::Message::Creator(Message::Url(s)))
            .width(Length::Fill);

        let mut col = Column::new().spacing(14);
        col = col.push(app_title);
        col = col.push(app_url);

        let download_button = self.download_button();
        let download_button = widget::button(download_button)
            .width(82.)
            .height(82.)
            .on_press(gui::Message::Clicked(gui::Buttons::SearchFavicon));

        let icon = self.icon_picker_icon(self.selected_icon.clone());
        let icon = widget::button(icon)
            .width(Length::Fixed(82.))
            .height(Length::Fixed(82.))
            .on_press(gui::Message::OpenIconPicker);

        let mut row = Row::new().spacing(12).width(Length::Fill);

        row = row.push(col);
        row = row.push(download_button);
        row = row.push(icon);

        let app_arguments = TextInput::new("Non-standard arguments", &self.app_parameters)
            .on_input(|s| gui::Message::Creator(Message::Arguments(s)))
            .width(Length::Fill);

        let category = dropdown(
            &self.app_categories,
            Some(self.selected_category),
            move |index| gui::Message::Creator(Message::Category(index)),
        )
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

        let app_browsers = dropdown(&self.app_browsers, self.selected_browser, |idx| {
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

        let mut col = Column::new().spacing(20).padding(30);

        if self.warning.show {
            col = col.push(self.warning.view());
        }

        col = col.push(row);
        col = col.push(app_arguments);
        col = col.push(cat_row);
        col = col.push(browsers_row);

        col.into()
    }
}
