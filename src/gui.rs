use cosmic::{
    app::Core,
    executor,
    iced::{self, event, window, Command},
    iced_core::Point,
    Element,
};

use crate::{
    common::{find_icons, image_from_memory, move_icon, svg_from_memory, Browser, WebAppLauncher},
    iconpicker, wam,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Buttons {
    SearchFavicon,
    Edit(WebAppLauncher),
    Delete(WebAppLauncher),
    Navbar(bool),
    IsolatedProfile(bool),
    Incognito(bool),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Message {
    WindowOpened(window::Id, Option<Point>),
    CloseWindow(window::Id),
    WindowClosed(window::Id),

    // wam
    OpenIconPicker,
    Result,
    Clicked(Buttons),
    Title(String),
    Url(String),
    Arguments(String),
    Browser(Browser),
    Category(String),

    // icons
    Favicon(String),
    PerformIconSearch,
    CustomIconsSearch(String),
    FoundIcons(Vec<String>),
    PushIcon(iconpicker::Icon),
    SetIcon(iconpicker::Icon),
    SelectIcon(iconpicker::Icon),
}

#[derive(Debug)]
pub enum MultiWindow {
    App,
    Picker,
}

pub struct Window {
    core: Core,
    windows: HashMap<window::Id, MultiWindow>,
    main_window: wam::Wam,
    icons_window: iconpicker::IconPicker,
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
        let manager = wam::Wam::new();
        let icons_picker = iconpicker::IconPicker::new();
        let windows = Window {
            core,
            windows: HashMap::from([(window::Id::MAIN, MultiWindow::App)]),
            main_window: manager,
            icons_window: icons_picker,
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

    fn update(&mut self, message: Self::Message) -> iced::Command<cosmic::app::Message<Message>> {
        match message {
            Message::CloseWindow(id) => window::close(id),
            Message::WindowClosed(id) => {
                self.windows.remove(&id);
                Command::none()
            }
            Message::WindowOpened(_id, ..) => Command::none(),

            // *** WAM STRUCT *** //
            Message::OpenIconPicker => {
                let count = self.windows.len() + 1;

                let (id, spawn_window) = window::spawn(window::Settings {
                    position: Default::default(),
                    exit_on_close_request: count % 2 == 0,
                    ..Default::default()
                });

                self.windows.insert(id, MultiWindow::Picker);

                spawn_window
            }
            Message::Result => {
                let launcher = if let Some(launcher) = self.main_window.launcher.to_owned() {
                    let _ = launcher.delete();

                    Box::new(WebAppLauncher::new(
                        self.main_window.app_title.clone(),
                        Some(launcher.codename),
                        self.main_window.app_url.clone(),
                        self.main_window.app_icon.clone(),
                        self.main_window.app_category.clone(),
                        self.main_window.app_browser.clone(),
                        self.main_window.app_parameters.clone(),
                        self.main_window.app_isolated,
                        self.main_window.app_navbar,
                        self.main_window.app_incognito,
                    ))
                } else {
                    Box::new(WebAppLauncher::new(
                        self.main_window.app_title.clone(),
                        None,
                        self.main_window.app_url.clone(),
                        self.main_window.app_icon.clone(),
                        self.main_window.app_category.clone(),
                        self.main_window.app_browser.clone(),
                        self.main_window.app_parameters.clone(),
                        self.main_window.app_isolated,
                        self.main_window.app_navbar,
                        self.main_window.app_incognito,
                    ))
                };

                if launcher.is_valid {
                    let _ = launcher.create();
                }

                Command::none()
            }
            Message::Clicked(_btns) => {
                // match btns {
                //     Buttons::SearchFavicon => todo!(),
                //     Buttons::Edit(_) => todo!(),
                //     Buttons::Delete(_) => todo!(),
                //     Buttons::Navbar(_) => todo!(),
                //     Buttons::IsolatedProfile(_) => todo!(),
                //     Buttons::Incognito(_) => todo!(),
                // };
                Command::none()
            }
            Message::Title(title) => {
                self.main_window.app_title = title;
                Command::none()
            }
            Message::Url(url) => {
                self.main_window.app_url = url;
                Command::none()
            }
            Message::Arguments(args) => {
                self.main_window.app_parameters = args;
                Command::none()
            }
            Message::Browser(browser) => {
                self.main_window.app_browser = browser;
                Command::none()
            }
            Message::Category(cat) => {
                self.main_window.app_category = cat;
                Command::none()
            }

            // *** ICON PICKER **** //
            Message::Favicon(_) => todo!(),
            Message::PerformIconSearch => {
                self.icons_window.icons.clear();

                Command::perform(
                    find_icons(
                        self.icons_window.icon_searching.clone(),
                        Some(self.main_window.app_url.clone()),
                    ),
                    |icons| cosmic::app::message::app(Message::FoundIcons(icons)),
                )
            }
            Message::CustomIconsSearch(input) => {
                self.icons_window.icon_searching = input;

                Command::none()
            }
            Message::FoundIcons(result) => {
                let mut commands: Vec<iced::Command<cosmic::app::Message<Message>>> = Vec::new();

                result.into_iter().for_each(|path| {
                    let is_svg = path.ends_with(".svg");
                    if is_svg {
                        commands.push(Command::perform(svg_from_memory(path), |result| {
                            cosmic::app::message::app(Message::PushIcon(result))
                        }));
                    } else {
                        commands.push(Command::perform(image_from_memory(path), |result| {
                            cosmic::app::message::app(Message::PushIcon(result))
                        }));
                    };
                });

                Command::batch(commands)
            }
            Message::PushIcon(icon) => {
                // self.main_window.selected_icon = Some(icon.clone());
                // if !&icon.path.starts_with("http") {
                //     self.main_window.app_icon = icon.path.clone()
                // } else {
                //     self.main_window.app_icon = move_icon(
                //         icon.path.clone(),
                //         self.main_window.app_title.replace(' ', ""),
                //     )
                //     .expect("cant download icon")
                // }

                println!("{:?}", icon);
                self.icons_window.icons.push(icon);

                Command::none()
            }
            Message::SetIcon(icon) => {
                let path = icon.path;

                if let Ok(saved) = move_icon(path, self.main_window.app_title.clone()) {
                    self.main_window.app_icon = saved.clone();

                    if saved.ends_with(".svg") {
                        Command::perform(svg_from_memory(saved), |result| {
                            cosmic::app::message::app(Message::SelectIcon(result))
                        })
                    } else {
                        Command::perform(image_from_memory(saved), |result| {
                            cosmic::app::message::app(Message::SelectIcon(result))
                        })
                    }
                } else {
                    Command::none()
                }
            }
            Message::SelectIcon(ico) => {
                self.main_window.selected_icon = Some(ico);
                Command::none()
            }
        }
    }

    fn view_window(&self, window_id: window::Id) -> Element<Message> {
        let window = self.windows.get(&window_id).unwrap();

        match window {
            MultiWindow::App => self.main_window.view(),
            MultiWindow::Picker => self.icons_window.view(),
        }
    }
    fn view(&self) -> Element<Self::Message> {
        self.view_window(window::Id::MAIN)
    }
}
