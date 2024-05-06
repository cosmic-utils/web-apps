use crate::{
    common::{get_supported_browsers, get_webapps, Browser, WebAppLauncher},
    gui::{Buttons, Message},
    iconpicker,
};

use cosmic::{
    iced::{id, Alignment, Length},
    iced_widget::{horizontal_rule, PickList, Scrollable},
    theme,
    widget::{text, toggler, warning, Button, Column, Container, Row, TextInput},
    Element,
};

#[derive(Debug, Clone)]
pub struct Wam {
    pub app_codename: Option<String>,
    pub app_title_id: id::Id,
    pub app_title: String,
    pub app_url: String,
    pub app_icon: String,
    pub app_parameters: String,
    pub app_category: String,
    pub app_browser_name: String,
    pub app_browser: Browser,
    pub app_navbar: bool,
    pub app_incognito: bool,
    pub app_isolated: bool,
    pub selected_icon: Option<iconpicker::Icon>,
    pub app_browsers: Vec<Browser>,
    pub edit_mode: bool,
    pub warning: Warning,
    pub launcher: Option<WebAppLauncher>,
}

impl Wam {
    pub fn new() -> Self {
        let browsers = get_supported_browsers();
        let browser = if !browsers.is_empty() {
            browsers[0].clone()
        } else {
            Browser::new(
                crate::common::BrowserType::NotInstalled,
                "Install some browser",
                "",
                "",
            )
        };

        let starting_warns = vec![
            WarnMessages::Info,
            WarnMessages::AppName,
            WarnMessages::AppUrl,
            WarnMessages::AppIcon,
            WarnMessages::AppBrowser,
        ];

        let warn_element = Warning::new(starting_warns, true);

        Wam {
            app_codename: None,
            app_title_id: id::Id::new("app-title"),
            app_title: String::new(),
            app_url: String::new(),
            app_icon: String::new(),
            app_parameters: String::new(),
            app_category: String::from("Web"),
            app_browser_name: String::from("Browser"),
            app_browser: browser,
            app_navbar: false,
            app_incognito: false,
            app_isolated: true,
            selected_icon: None,
            app_browsers: browsers,
            edit_mode: false,
            warning: warn_element,
            launcher: None,
        }
    }

    fn icon_picker_icon(&self, icon: Option<iconpicker::Icon>) -> Element<Message> {
        let ico = if let Some(ico) = icon {
            match ico.icon {
                iconpicker::IconType::Raster(data) => Button::new(cosmic::widget::image(data))
                    .on_press(Message::OpenIconPicker)
                    .width(Length::Fixed(64.))
                    .height(Length::Fixed(64.))
                    .style(theme::Button::Transparent),

                iconpicker::IconType::Svg(data) => Button::new(cosmic::widget::svg(data))
                    .on_press(Message::OpenIconPicker)
                    .width(Length::Fixed(64.))
                    .height(Length::Fixed(64.))
                    .style(theme::Button::Transparent),
            }
        } else {
            let default_ico = include_bytes!("../assets/icons/moleskine-icon.svg");
            let handler = cosmic::widget::svg::Handle::from_memory(default_ico);
            let default = cosmic::widget::svg(handler);

            Button::new(default)
                .on_press(Message::OpenIconPicker)
                .width(Length::Fixed(64.))
                .height(Length::Fixed(64.))
                .style(theme::Button::Transparent)
        };

        Container::new(ico).center_x().center_y().into()
    }

    pub fn view(&self) -> Element<Message> {
        let app_title = TextInput::new("Title", &self.app_title)
            .id(self.app_title_id.clone())
            .on_input(Message::Title)
            .padding(10)
            .width(Length::Fill);
        let app_url = TextInput::new("URL", &self.app_url)
            .on_input(Message::Url)
            .padding(10)
            .width(Length::Fill);

        let mut col = Column::new().spacing(14);
        col = col.push(app_title);
        col = col.push(app_url);

        let search_ico = include_bytes!("../assets/icons/search.svg");
        let search_ico_handler =
            cosmic::widget::svg(cosmic::widget::svg::Handle::from_memory(search_ico))
                .width(Length::Fixed(64.))
                .height(Length::Fixed(64.));

        let dl_btn = Button::new(
            Container::new(Button::new(search_ico_handler).style(theme::Button::Transparent))
                .center_x()
                .center_y(),
        )
        .on_press(Message::Clicked(Buttons::SearchFavicon))
        .width(Length::Fixed(96.))
        .height(Length::Fixed(96.))
        .style(theme::Button::Suggested);

        let icon = self.icon_picker_icon(self.selected_icon.clone());

        let icon = Button::new(icon)
            .width(Length::Fixed(96.))
            .height(Length::Fixed(96.))
            .on_press(Message::OpenIconPicker)
            .style(theme::Button::Standard);

        let mut row = Row::new().spacing(12).width(Length::Fill);

        row = row.push(col);
        row = row.push(dl_btn);
        row = row.push(icon);

        let app_arguments = TextInput::new("Non-standard arguments", &self.app_parameters)
            .on_input(Message::Arguments)
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

        let category = PickList::new(
            categories.to_vec(),
            Some(self.app_category.clone()),
            Message::Category,
        )
        .width(Length::Fixed(200.))
        .padding(10);

        let browser_specific = match self.app_browser._type {
            crate::common::BrowserType::Firefox => {
                toggler(String::from("Nav Bar"), self.app_navbar, |b| {
                    Message::Clicked(Buttons::Navbar(b))
                })
                .width(Length::Fill)
            }
            crate::common::BrowserType::FirefoxFlatpak => {
                toggler(String::from("Nav Bar"), self.app_navbar, |b| {
                    Message::Clicked(Buttons::Navbar(b))
                })
                .width(Length::Fill)
            }
            crate::common::BrowserType::Librewolf => {
                toggler(String::from("Nav Bar"), self.app_navbar, |b| {
                    Message::Clicked(Buttons::Navbar(b))
                })
                .width(Length::Fill)
            }
            crate::common::BrowserType::WaterfoxFlatpak => {
                toggler(String::from("Nav Bar"), self.app_navbar, |b| {
                    Message::Clicked(Buttons::Navbar(b))
                })
                .width(Length::Fill)
            }
            _ => toggler(String::from("Isolated Profile"), self.app_isolated, |b| {
                Message::Clicked(Buttons::IsolatedProfile(b))
            })
            .width(Length::Fill),
        };

        let incognito = toggler(String::from("Private Mode"), self.app_incognito, |b| {
            Message::Clicked(Buttons::Incognito(b))
        })
        .width(Length::Fill);

        let mut cat_row = Row::new().spacing(20).align_items(Alignment::Center);
        cat_row = cat_row.push(category);
        cat_row = cat_row.push(incognito);
        cat_row = cat_row.push(browser_specific);

        let app_browsers = PickList::new(
            self.app_browsers.clone(),
            Some(self.app_browser.clone()),
            Message::Browser,
        )
        .width(Length::Fixed(200.))
        .padding(10);

        let app_done = Button::new(Container::new("Create").center_x().center_y())
            .on_press(Message::Result)
            .width(Length::Fill)
            .padding(10)
            .style(theme::Button::Suggested);

        let mut browsers_row = Row::new().spacing(20);
        browsers_row = browsers_row.push(app_browsers);
        browsers_row = browsers_row.push(app_done);

        let mut app_list = Column::new().spacing(10);
        let webapps = get_webapps();

        for app in webapps.iter() {
            match app {
                Ok(data) => {
                    let num = Button::new(
                        Container::new(text(data.web_browser.name.clone()).font(
                            cosmic::iced::Font {
                                weight: cosmic::iced::font::Weight::Semibold,
                                ..Default::default()
                            },
                        ))
                        .center_x()
                        .center_y(),
                    )
                    .width(Length::FillPortion(1))
                    .height(Length::Fixed(50.))
                    .padding(10);

                    let run = Button::new(
                        Container::new(text(data.name.clone()).font(cosmic::iced::Font {
                            weight: cosmic::iced::font::Weight::Semibold,
                            ..Default::default()
                        }))
                        .center_x()
                        .center_y(),
                    )
                    .width(Length::FillPortion(3))
                    .height(Length::Fixed(50.))
                    .padding(10);

                    let edit = Button::new(Container::new("Edit").center_x().center_y())
                        .on_press(Message::Clicked(Buttons::Edit(data.clone())))
                        .width(Length::Fixed(90.))
                        .height(Length::Fixed(50.))
                        .padding(10)
                        .style(theme::Button::Standard);
                    let delete = Button::new(Container::new("Delete").center_x().center_y())
                        .on_press(Message::Clicked(Buttons::Delete(data.clone())))
                        .width(Length::Fixed(90.))
                        .height(Length::Fixed(50.))
                        .padding(10)
                        .style(theme::Button::Destructive);

                    let mut row = Row::new().spacing(10).height(Length::Fixed(50.));
                    let mut row2 = Row::new().spacing(10).height(Length::Fixed(50.));

                    row = row.push(num);
                    row = row.push(run);

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

        let mut installed = Column::new();

        if !webapps.is_empty() {
            installed = installed.push(text(format!("Installed #{}", webapps.len())).size(26.));

            installed = installed.push(horizontal_rule(3)).spacing(30);

            let scrollable_list = Scrollable::new(app_list).width(Length::Fill);

            installed = installed.push(scrollable_list);
        }

        let mut col = Column::new().spacing(20);

        // if self.warning {
        //     let warn = warning(
        //         "You don't meet requirements
        //     - App must have name.\n
        //     - App must have valid URL starting with http:// or https://\n
        //     - App must have selected icon.\n
        //     - You must have installed browser via flatpak.",
        //     );
        //     col = col.push(warn);
        // }
        if self.warning.show {
            col = col.push(self.warning.view());
        }

        col = col.push(row);
        col = col.push(app_arguments);
        col = col.push(cat_row);
        col = col.push(browsers_row);
        let mut col2 = Column::new().spacing(50);
        col2 = col2.push(col);
        col2 = col2.push(installed);

        Container::new(col2).padding(30).into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WarnMessages {
    Info,
    AppName,
    AppUrl,
    AppIcon,
    AppBrowser,
}

impl std::fmt::Display for WarnMessages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            WarnMessages::Info => write!(f, "You don't meet requirements"),
            WarnMessages::AppName => write!(f, "  - App name must be longer than 3 characters"),
            WarnMessages::AppUrl => write!(
                f,
                "  - You must provide valid URL starting with http:// or https://"
            ),
            WarnMessages::AppIcon => write!(f, "  - You must select an Icon for your launcher"),
            WarnMessages::AppBrowser => {
                write!(f, "  - Please select a browser installed via Flatpak")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Warning {
    pub messages: Vec<WarnMessages>,
    pub show: bool,
}

impl Default for Warning {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            show: false,
        }
    }
}

impl Warning {
    pub fn new(messages: Vec<WarnMessages>, show: bool) -> Self {
        Self { messages, show }
    }

    pub fn push_warn(&mut self, message: WarnMessages) -> &mut Self {
        if !self.messages.contains(&message) {
            self.messages.push(message);
        }
        self
    }

    pub fn remove_warn(&mut self, message: WarnMessages) -> &mut Self {
        self.messages.retain(|m| *m != message);
        self
    }

    pub fn view(&self) -> Element<Message> {
        let mut content = String::new();

        for line in &self.messages {
            content.push_str(&format!("{}\n", line));
        }

        let warn = warning(content);

        if self.show {
            Container::new(warn).into()
        } else {
            Container::new(Column::new()).into()
        }
    }
}
