#![allow(dead_code)]
// z gh
use cosmic::{
    app::{Command, Core},
    executor,
    iced::{self, event, window},
    iced_core::{Alignment, Length, Point},
    iced_widget::{pick_list, toggler, Button, Container},
    iced_widget::{scrollable, text, text_input},
    widget::{Column, Row},
    ApplicationExt, Element,
};
use std::{collections::HashMap, path::PathBuf};
use url::Url;
use xdg::BaseDirectories;

use crate::common::{get_supported_browsers, get_webapps, Browser, WebAppLauncher};

#[derive(Debug, Clone)]
pub enum Buttons {
    SearchFavicon,
    // Favicon(String),
    Edit(Box<WebAppLauncher>),
    Delete(Box<WebAppLauncher>),
    Navbar(bool),
    IsolatedProfile(bool),
    Incognito(bool),
}

#[derive(Debug, Clone)]
pub enum WamMessage {
    Result,
    Clicked(Buttons),
    Title(String),
    Url(String),
    Arguments(String),
    Browser(Browser),
    Category(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    CloseWindow(window::Id),
    WindowOpened(window::Id, Option<Point>),
    WindowClosed(window::Id),
    OpenIconPicker,
    WamStruct(WamMessage),
}

#[derive(Debug, Clone)]
pub enum IconType {
    Raster(cosmic::widget::image::Handle),
    Svg(cosmic::widget::svg::Handle),
}

#[derive(Debug, Clone)]
pub struct Icon {
    pub icon: IconType,
    pub path: String,
}

impl Icon {
    pub fn new(icon: IconType, path: String) -> Self {
        Self { icon, path }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum MultiWindow {
    App(Wam),
    Picker(IconPicker),
}

pub struct Window {
    core: Core,
    windows: HashMap<window::Id, MultiWindow>,
}

#[derive(Debug)]
pub struct Wam {
    pub icons_paths: Vec<String>,
    pub icons: Option<Vec<Icon>>,
    pub app_codename: Option<String>,
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
    _icon_searching: String,
    selected_icon: Option<Icon>,
    app_browsers: Vec<Browser>,
    _edit_mode: bool,
    _launcher: Option<Box<WebAppLauncher>>,
    app_base_dir: PathBuf,
}

#[derive(Debug)]
pub struct IconPicker {
    input: String,
}

impl cosmic::Application for Window {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "org.cosmic.WamRust";

    fn core(&self) -> &cosmic::app::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::app::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::app::Core,
        _flags: Self::Flags,
    ) -> (
        Self,
        cosmic::iced::Command<cosmic::app::Message<Self::Message>>,
    ) {
        let browsers = get_supported_browsers();
        let browser = &browsers[0];

        let base_dir = BaseDirectories::new().expect("cant follow base directories");
        let local_share = base_dir.get_data_home();
        let wam_rust_path = local_share.join("wam-rust");

        let manager = Wam {
            icons_paths: Vec::new(),
            icons: Some(Vec::new()),
            app_codename: None,
            app_title: String::new(),
            app_url: String::new(),
            app_icon: String::new(),
            app_parameters: String::new(),
            app_category: String::from("Web"),
            app_browser_name: String::from("Browser"),
            app_browser: browser.clone(),
            app_navbar: false,
            app_incognito: false,
            app_isolated: true,
            _icon_searching: String::new(),
            selected_icon: None,
            app_browsers: browsers,
            _edit_mode: false,
            _launcher: None,
            app_base_dir: wam_rust_path,
        };

        let windows = Window {
            windows: HashMap::from([(window::Id::MAIN, MultiWindow::App(manager))]),
            core,
        };

        (windows, Command::none())
    }

    fn subscription(&self) -> cosmic::iced_futures::Subscription<Self::Message> {
        event::listen_with(|event, _| {
            if let iced::Event::Window(id, window_event) = event {
                match window_event {
                    window::Event::CloseRequested => Some(Message::CloseWindow(id)),
                    window::Event::Opened { position, .. } => {
                        Some(Message::WindowOpened(id, position))
                    }
                    window::Event::Closed => Some(Message::WindowClosed(id)),
                    _ => None,
                }
            } else {
                None
            }
        })
    }

    fn update(
        &mut self,
        message: Self::Message,
    ) -> iced::Command<cosmic::app::Message<Self::Message>> {
        match message {
            Message::CloseWindow(id) => window::close(id),
            Message::WindowClosed(id) => {
                self.windows.remove(&id);
                Command::none()
            }
            Message::WindowOpened(id, ..) => {
                if let Some(window) = self.windows.get(&id) {
                    println!("{:?}", window);

                    Command::none()
                } else {
                    Command::none()
                }
            }
            Message::OpenIconPicker => {
                let count = self.windows.len() + 1;

                let (id, spawn_window) = window::spawn(window::Settings {
                    position: Default::default(),
                    exit_on_close_request: count % 2 == 0,
                    ..Default::default()
                });

                self.windows.insert(
                    id,
                    MultiWindow::Picker(IconPicker {
                        input: String::from("dupa blada"),
                    }),
                );
                _ = self.set_window_title(String::from("Select an Icon"), id);

                spawn_window
            }
            Message::WamStruct(wam) => Command::none(),
        }
    }
    fn view_window(&self, window_id: window::Id) -> Element<Message> {
        let window = self.windows.get(&window_id).unwrap();

        match window {
            MultiWindow::App(main) => main.view(),
            MultiWindow::Picker(picker) => picker.view(),
        }
    }
    fn view(&self) -> Element<Self::Message> {
        self.view_window(window::Id::MAIN)
    }
}

impl IconPicker {
    fn view(&self) -> cosmic::prelude::Element<Message> {
        text(self.input.to_string()).into()
    }
}

impl Wam {
    fn icon_picker_icon(&self, icon: Option<Icon>) -> Element<Message> {
        let ico = if let Some(ico) = icon {
            match ico.icon {
                IconType::Raster(data) => Button::new(
                    cosmic::widget::image(data)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .width(Length::Fixed(96.))
                .height(Length::Fixed(96.)),
                IconType::Svg(data) => Button::new(
                    cosmic::widget::svg(data)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .width(Length::Fixed(96.))
                .height(Length::Fixed(96.)),
            }
        } else {
            let default_ico = &self.app_base_dir.join("icons/moleskine-icon.svg");
            let default_ico = default_ico.to_str().expect("cant find needed icon");
            let default_icon_path = String::from(default_ico);
            let handler = cosmic::widget::svg::Handle::from_path(default_icon_path);
            let default = cosmic::widget::svg(handler);

            Button::new(default)
                .width(Length::Fill)
                .height(Length::Fill)
                .width(Length::Fixed(96.))
                .height(Length::Fixed(96.))
                .on_press(Message::OpenIconPicker)
        };

        Container::new(ico).into()
    }

    fn view(&self) -> cosmic::prelude::Element<Message> {
        let app_title = text_input("Title", &self.app_title)
            .on_input(|s| Message::WamStruct(WamMessage::Title(s)))
            .padding(10)
            .width(Length::Fixed(340.));
        let app_url = text_input("URL", &self.app_url)
            .on_input(|s| Message::WamStruct(WamMessage::Url(s)))
            .padding(10)
            .width(Length::Fixed(340.));

        let mut col = Column::new().spacing(14);
        col = col.push(app_title);
        col = col.push(app_url);

        let search_ico = &self.app_base_dir.join("icons/search.svg");
        let search_ico = search_ico.to_str().expect("cant find needed search icon");

        let dl_btn = Button::new(
            cosmic::widget::svg(cosmic::widget::svg::Handle::from_path(search_ico))
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Message::WamStruct(WamMessage::Clicked(
            Buttons::SearchFavicon,
        )))
        .width(Length::Fixed(96.))
        .height(Length::Fixed(96.));

        let icons = self.icons.clone().unwrap();

        let icon = if !icons.is_empty() || !self.app_icon.is_empty() {
            self.icon_picker_icon(self.selected_icon.clone())
        } else {
            self.icon_picker_icon(None)
        };
        let mut row = Row::new().spacing(12).width(Length::Fill);
        row = row.push(col);
        row = row.push(dl_btn);
        row = row.push(icon);

        let app_arguments = text_input("Non-standard arguments", &self.app_parameters)
            .on_input(|s| Message::WamStruct(WamMessage::Arguments(s)))
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

        let category = pick_list(categories.to_vec(), Some(self.app_category.clone()), |c| {
            Message::WamStruct(WamMessage::Category(c))
        })
        .width(Length::Fixed(200.))
        .padding(10);

        let browser_specific = match self.app_browser._type {
            crate::common::BrowserType::Firefox => {
                toggler(String::from("Nav Bar"), self.app_navbar, |b| {
                    Message::WamStruct(WamMessage::Clicked(Buttons::Navbar(b)))
                })
                .width(Length::Fill)
            }
            crate::common::BrowserType::FirefoxFlatpak => {
                toggler(String::from("Nav Bar"), self.app_navbar, |b| {
                    Message::WamStruct(WamMessage::Clicked(Buttons::Navbar(b)))
                })
                .width(Length::Fill)
            }
            crate::common::BrowserType::Librewolf => {
                toggler(String::from("Nav Bar"), self.app_navbar, |b| {
                    Message::WamStruct(WamMessage::Clicked(Buttons::Navbar(b)))
                })
                .width(Length::Fill)
            }
            crate::common::BrowserType::WaterfoxFlatpak => {
                toggler(String::from("Nav Bar"), self.app_navbar, |b| {
                    Message::WamStruct(WamMessage::Clicked(Buttons::Navbar(b)))
                })
                .width(Length::Fill)
            }
            _ => toggler(String::from("Isolated Profile"), self.app_isolated, |b| {
                Message::WamStruct(WamMessage::Clicked(Buttons::IsolatedProfile(b)))
            })
            .width(Length::Fill),
        };

        let incognito = toggler(String::from("Private Mode"), self.app_incognito, |b| {
            Message::WamStruct(WamMessage::Clicked(Buttons::Incognito(b)))
        })
        .width(Length::Fill);

        let mut cat_row = Row::new().spacing(20).align_items(Alignment::Center);
        cat_row = cat_row.push(category);
        cat_row = cat_row.push(incognito);
        cat_row = cat_row.push(browser_specific);

        let app_browsers = pick_list(
            self.app_browsers.clone(),
            Some(self.app_browser.clone()),
            |b| Message::WamStruct(WamMessage::Browser(b)),
        )
        .width(Length::Fixed(200.))
        .padding(10);

        let app_done = Button::new("Done")
            .on_press(Message::WamStruct(WamMessage::Result))
            .width(Length::Fill)
            .padding(10);

        let mut browsers_row = Row::new().spacing(20);
        browsers_row = browsers_row.push(app_browsers);
        browsers_row = browsers_row.push(app_done);

        let mut app_list = Column::new().spacing(10);
        let webapps = get_webapps();

        for app in webapps.iter() {
            match app {
                Ok(data) => {
                    let edit = Button::new("Edit")
                        .on_press(Message::WamStruct(WamMessage::Clicked(Buttons::Edit(
                            Box::new(data.clone()),
                        ))))
                        .width(Length::Fixed(90.));
                    let delete = Button::new("Delete")
                        .on_press(Message::WamStruct(WamMessage::Clicked(Buttons::Delete(
                            Box::new(data.clone()),
                        ))))
                        .width(Length::Fixed(90.));

                    let host = Url::parse(&data.url).expect("cant parse url");
                    let host = host.host().unwrap();

                    let name = Button::new(text(data.name.clone())).width(Length::FillPortion(2));
                    let url = Button::new(text(host.to_string())).width(Length::FillPortion(3));

                    let mut row = Row::new().spacing(10).align_items(Alignment::Center);
                    row = row.push(edit);
                    row = row.push(delete);
                    row = row.push(name);
                    row = row.push(url);
                    app_list = app_list.push(row);
                }
                Err(e) => tracing::error!("Error reading web app: {}", e),
            }
        }

        let mut installed = Column::new();

        if !webapps.is_empty() {
            installed = installed.push(text("INSTALLED").size(26.)).spacing(10);

            let scrollable_list = scrollable(app_list).width(Length::Fill);

            installed = installed.push(scrollable_list);
        }

        let mut col = Column::new().spacing(20);
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

// struct CustomButton;
//
// impl button::StyleSheet for CustomButton {
//     type Style = Theme;
//
//     fn active(&self, _style: &Self::Style) -> button::Appearance {
//         button::Appearance {
//             background: Some(iced::Background::Color(Color::TRANSPARENT)),
//             ..Default::default()
//         }
//     }
//
//     fn disabled(&self, style: &Self::Style) -> button::Appearance {
//         let active = self.active(style);
//
//         button::Appearance {
//             shadow_offset: iced::Vector::default(),
//             background: active.background.map(|background| match background {
//                 iced::Background::Color(color) => iced::Background::Color(iced::Color {
//                     a: color.a * 0.5,
//                     ..color
//                 }),
//                 iced::Background::Gradient(gradient) => {
//                     iced::Background::Gradient(gradient.mul_alpha(0.5))
//                 }
//             }),
//             text_color: iced::Color {
//                 a: active.text_color.a * 0.5,
//                 ..active.text_color
//             },
//             ..active
//         }
//     }
//
//     fn hovered(&self, style: &Self::Style) -> button::Appearance {
//         let active = self.active(style);
//
//         button::Appearance { ..active }
//     }
//
//     fn pressed(&self, style: &Self::Style) -> button::Appearance {
//         button::Appearance {
//             shadow_offset: iced::Vector::default(),
//             ..self.active(style)
//         }
//     }
// }
//
// struct InputField;
//
// impl text_input::StyleSheet for InputField {
//     type Style = Theme;
//
//     fn active(&self, style: &Self::Style) -> text_input::Appearance {
//         text_input::Appearance {
//             background: iced::Background::Color(Color::TRANSPARENT),
//             border: Border {
//                 color: style.palette().text,
//                 width: 1.,
//                 radius: Radius::from(4.),
//             },
//             icon_color: style.palette().text,
//         }
//     }
//
//     fn focused(&self, style: &Self::Style) -> text_input::Appearance {
//         let active = self.active(style);
//         text_input::Appearance {
//             border: Border {
//                 color: Color::from_rgba(0.76, 0.76, 0.76, 0.20),
//                 width: 1.,
//                 ..Default::default()
//             },
//             ..active
//         }
//     }
//
//     fn placeholder_color(&self, style: &Self::Style) -> Color {
//         style.palette().text
//     }
//
//     fn value_color(&self, style: &Self::Style) -> Color {
//         style.palette().text
//     }
//
//     fn disabled_color(&self, style: &Self::Style) -> Color {
//         style.palette().text
//     }
//
//     fn selection_color(&self, _style: &Self::Style) -> Color {
//         Color::from_rgb(128., 191., 255.)
//     }
//
//     fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
//         let active = self.active(style);
//
//         text_input::Appearance {
//             background: iced::Background::Color(Color::TRANSPARENT),
//             border: Border {
//                 color: Color::TRANSPARENT,
//                 width: 0.,
//                 ..Default::default()
//             },
//             ..active
//         }
//     }
// }
//
// struct AdaptiveSvg;
//
// impl svg::StyleSheet for AdaptiveSvg {
//     type Style = Theme;
//
//     fn appearance(&self, style: &Self::Style) -> svg::Appearance {
//         svg::Appearance {
//             color: Some(style.palette().text),
//         }
//     }
// }
