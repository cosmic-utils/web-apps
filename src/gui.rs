use crate::{
    common::{find_icons, image_from_memory, move_icon, svg_from_memory, Browser, WebAppLauncher},
    iconpicker, wam,
};

use cosmic::{
    app::Core,
    executor,
    iced::{self, event, window, Command},
    iced_core::Point,
    iced_widget::text_input::focus,
    Element,
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
    CloseIconPicker,
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

pub struct Window {
    core: Core,
    windows: HashMap<window::Id, wam::Wam>,
    main_window: wam::Wam,
    icon_dialog: bool,
    iconpicker: iconpicker::IconPicker,
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
            windows: HashMap::from([(window::Id::MAIN, manager.clone())]),
            main_window: manager,
            icon_dialog: false,
            iconpicker: icons_picker,
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
            Message::WindowOpened(id, ..) => {
                if let Some(window) = self.windows.get(&id) {
                    focus(window.app_title_id.clone())
                } else {
                    Command::none()
                }
            }

            // *** WAM STRUCT *** //
            Message::OpenIconPicker => {
                self.icon_dialog = true;

                focus(self.iconpicker.searching_id.clone())
            }
            Message::CloseIconPicker => {
                self.icon_dialog = false;

                focus(self.main_window.app_title_id.clone())
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
            Message::Favicon(path) => {
                let is_svg = path.ends_with(".svg");

                if is_svg {
                    Command::perform(svg_from_memory(path), |result| {
                        cosmic::app::message::app(Message::SetIcon(result))
                    })
                } else {
                    Command::perform(image_from_memory(path), |result| {
                        cosmic::app::message::app(Message::SetIcon(result))
                    })
                }
            }
            Message::PerformIconSearch => {
                self.iconpicker.icons.clear();

                Command::perform(
                    find_icons(
                        self.iconpicker.icon_searching.clone(),
                        Some(self.main_window.app_url.clone()),
                    ),
                    |icons| cosmic::app::message::app(Message::FoundIcons(icons)),
                )
            }
            Message::CustomIconsSearch(input) => {
                self.iconpicker.icon_searching = input;

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
                self.iconpicker.icons.push(icon);

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

    fn dialog(&self) -> Option<Element<Self::Message>> {
        if self.icon_dialog {
            return Some(self.iconpicker.view());
        };

        None
    }

    fn view_window(&self, _window_id: window::Id) -> Element<Message> {
        self.main_window.view()
    }
    fn view(&self) -> Element<Message> {
        self.view_window(window::Id::MAIN)
    }
}
