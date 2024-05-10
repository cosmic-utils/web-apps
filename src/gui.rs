use crate::{
    add_icon_packs_install_script,
    common::{
        self, find_icons, get_icon_name_from_url, image_handle, move_icon, Browser, WebAppLauncher,
    },
    creator, execute_script,
    home_screen::Home,
    icon_pack_installed,
    iconpicker::{self, IconPicker},
    icons_installator::{self, Installator},
    warning::WarnMessages,
};

use cosmic::{
    app::{
        message::{self, app},
        Core, Message as CosmicMessage,
    },
    executor,
    iced::{self, event, window},
    iced_core::Point,
    widget::{self, focus, text},
    Command, Element,
};
use std::{collections::HashMap, process::ExitStatus};

#[derive(Debug, Clone)]
pub enum Buttons {
    SearchFavicon,
    Edit(WebAppLauncher),
    Delete(WebAppLauncher),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Message {
    WindowOpened(window::Id, Option<Point>),
    CloseWindow(window::Id),
    WindowClosed(window::Id),

    OpenHome,
    OpenCreator,
    CloseCreator,
    OpenIconPicker,
    Creator(creator::Message),
    Result,

    Clicked(Buttons),
    // icons
    Favicon(String),
    PerformIconSearch,
    CustomIconsSearch(String),
    FoundIcons(Vec<String>),
    PushIcon(Option<iconpicker::Icon>),
    SetIcon(iconpicker::Icon),
    SelectIcon(iconpicker::Icon),

    // Installator
    InstallScript(String),
    InstallCommand(ExitStatus),
}

#[derive(Debug, Clone)]
pub enum Pages {
    MainWindow,
    AppCreator,
    IconPicker(IconPicker),
    IconInstallator(Installator),
}

pub struct Window {
    core: Core,
    windows: HashMap<window::Id, Home>,
    main_window: Home,
    creator_window: creator::AppCreator,
    current_page: Pages,
}

impl cosmic::Application for Window {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "io.github.elevenhsoft.WebApps";

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
        let manager = Home::new();
        let creator = creator::AppCreator::new();

        let (page, cmd) = if !icon_pack_installed() {
            let cmd = Command::perform(add_icon_packs_install_script(), |file| {
                cosmic::app::message::app(Message::InstallScript(file))
            });

            let installator = icons_installator::Installator::new();
            (Pages::IconInstallator(installator), cmd)
        } else {
            (Pages::MainWindow, Command::none())
        };

        let windows = Window {
            core,
            windows: HashMap::from([(window::Id::MAIN, manager.clone())]),
            main_window: manager,
            creator_window: creator,
            current_page: page,
        };

        (windows, cmd)
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

    fn update(&mut self, message: Self::Message) -> Command<CosmicMessage<Message>> {
        match message {
            Message::CloseWindow(id) => window::close(id),
            Message::WindowClosed(id) => {
                self.windows.remove(&id);
                Command::none()
            }
            Message::WindowOpened(_id, ..) => Command::none(),

            Message::OpenHome => {
                self.current_page = Pages::MainWindow;

                Command::none()
            }

            Message::OpenCreator => {
                self.current_page = Pages::AppCreator;

                Command::none()
            }
            Message::CloseCreator => {
                self.current_page = Pages::MainWindow;
                self.creator_window.edit_mode = false;

                Command::none()
            }
            Message::Creator(message) => {
                let command = self.creator_window.update(message);

                command.map(|mess| app(Message::Creator(mess)))
            }
            Message::OpenIconPicker => {
                let icons_picker = iconpicker::IconPicker::new();
                self.current_page = Pages::IconPicker(icons_picker);

                if let Pages::IconPicker(ref mut picker) = self.current_page {
                    return focus(picker.searching_id.clone());
                };

                Command::none()
            }

            Message::Result => {
                let launcher = if let Some(launcher) = self.main_window.launcher.to_owned() {
                    let _ = launcher.delete();

                    WebAppLauncher::new(
                        self.creator_window.app_title.clone(),
                        Some(launcher.codename),
                        self.creator_window.app_url.clone(),
                        self.creator_window.app_icon.clone(),
                        self.creator_window.app_category.clone(),
                        self.creator_window.app_browser.clone(),
                        self.creator_window.app_parameters.clone(),
                        self.creator_window.app_isolated,
                        self.creator_window.app_navbar,
                        self.creator_window.app_incognito,
                    )
                } else {
                    WebAppLauncher::new(
                        self.creator_window.app_title.clone(),
                        None,
                        self.creator_window.app_url.clone(),
                        self.creator_window.app_icon.clone(),
                        self.creator_window.app_category.clone(),
                        self.creator_window.app_browser.clone(),
                        self.creator_window.app_parameters.clone(),
                        self.creator_window.app_isolated,
                        self.creator_window.app_navbar,
                        self.creator_window.app_incognito,
                    )
                };

                if launcher.is_valid {
                    let _ = launcher.create();
                    self.creator_window.edit_mode = false;
                    self.current_page = Pages::MainWindow;
                } else {
                    self.creator_window.warning.show = true;
                }

                Command::none()
            }

            Message::Clicked(buttons) => match buttons {
                Buttons::Edit(launcher) => {
                    self.creator_window.warning.remove_all_warns();
                    self.main_window.edit_mode = true;
                    self.main_window.launcher = Some(launcher.clone());

                    self.creator_window.app_title = launcher.name;
                    self.creator_window.app_url = launcher.url;
                    self.creator_window.app_icon.clone_from(&launcher.icon);
                    self.creator_window.app_parameters = launcher.custom_parameters;
                    self.creator_window.app_category = launcher.category;
                    self.creator_window.app_browser =
                        Browser::web_browser(launcher.web_browser.name).expect("browser not found");
                    self.creator_window.app_navbar = launcher.navbar;
                    self.creator_window.app_incognito = launcher.is_incognito;
                    self.creator_window.edit_mode = true;

                    Command::perform(image_handle(launcher.icon), |result| {
                        app(Message::SetIcon(result.unwrap()))
                    })
                }
                Buttons::Delete(launcher) => {
                    let _ = launcher.delete();

                    Command::none()
                }
                Buttons::SearchFavicon => {
                    if common::url_valid(&self.creator_window.app_url) {
                        if let Pages::IconPicker(ref mut picker) = self.current_page {
                            picker.icons.clear();
                        }

                        let name = get_icon_name_from_url(&self.creator_window.app_url);
                        let icons = find_icons(name, self.creator_window.app_url.clone());
                        Command::perform(icons, |icons| app(Message::FoundIcons(icons)))
                    } else {
                        Command::none()
                    }
                }
            },

            Message::Favicon(path) => Command::perform(image_handle(path), |result| {
                app(Message::SetIcon(result.unwrap()))
            }),
            Message::PerformIconSearch => {
                if let Pages::IconPicker(ref mut picker) = self.current_page {
                    picker.icons.clear();

                    let icons = find_icons(
                        picker.icon_searching.clone(),
                        self.creator_window.app_url.clone(),
                    );

                    return Command::perform(icons, |icons| app(Message::FoundIcons(icons)));
                };

                Command::none()
            }
            Message::CustomIconsSearch(input) => {
                if let Pages::IconPicker(ref mut picker) = self.current_page {
                    picker.icon_searching = input;
                }

                Command::none()
            }
            Message::FoundIcons(result) => {
                let mut commands: Vec<Command<CosmicMessage<Message>>> = Vec::new();

                result.into_iter().for_each(|path| {
                    commands.push(Command::perform(image_handle(path), |result| {
                        app(Message::PushIcon(result))
                    }));
                });

                Command::batch(commands)
            }
            Message::PushIcon(icon) => {
                if self.creator_window.selected_icon.is_none() && icon.is_some() {
                    let path = icon.as_ref().unwrap().path.clone();
                    let saved = move_icon(path, self.creator_window.app_title.clone());
                    self.creator_window.app_icon.clone_from(&saved);
                    self.creator_window.app_icon = saved;
                    self.creator_window.selected_icon.clone_from(&icon);

                    if self.creator_window.selected_icon.is_some()
                        && !self.creator_window.app_icon.is_empty()
                    {
                        self.creator_window
                            .warning
                            .remove_warn(WarnMessages::AppIcon);
                    } else {
                        self.creator_window.warning.push_warn(WarnMessages::AppIcon);
                    }
                }
                if let Some(ico) = icon {
                    if let Pages::IconPicker(ref mut picker) = self.current_page {
                        picker.icons.push(ico);
                    }
                }

                Command::none()
            }
            Message::SetIcon(icon) => {
                let path = icon.path;

                let saved = move_icon(path, self.creator_window.app_title.clone());
                self.current_page = Pages::AppCreator;
                self.creator_window.app_icon.clone_from(&saved);

                Command::perform(image_handle(saved), |result| {
                    if let Some(res) = result {
                        app(Message::SelectIcon(res))
                    } else {
                        message::none()
                    }
                })
            }
            Message::SelectIcon(ico) => {
                self.creator_window.selected_icon = Some(ico.clone());
                self.creator_window.app_icon = ico.path;

                if self.creator_window.selected_icon.is_some()
                    && !self.creator_window.app_icon.is_empty()
                {
                    self.creator_window
                        .warning
                        .remove_warn(WarnMessages::AppIcon);
                } else {
                    self.creator_window.warning.push_warn(WarnMessages::AppIcon);
                }

                Command::none()
            }

            Message::InstallScript(script) => {
                if !icon_pack_installed() {
                    return Command::perform(execute_script(script), |status| {
                        app(Message::InstallCommand(status))
                    });
                }
                Command::none()
            }
            Message::InstallCommand(exit_status) => {
                if ExitStatus::success(&exit_status) {
                    self.current_page = Pages::MainWindow;
                }

                Command::none()
            }
        }
    }

    fn view_window(&self, _window_id: window::Id) -> Element<Message> {
        match &self.current_page {
            Pages::MainWindow => self.main_window.view(),
            Pages::AppCreator => self.creator_window.view(),
            Pages::IconPicker(picker) => picker.view(),
            Pages::IconInstallator(installator) => installator.view(),
        }
    }
    fn view(&self) -> Element<Message> {
        self.view_window(window::Id::MAIN)
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![
            widget::button::icon(widget::icon::from_name("go-home-symbolic"))
                .on_press(Message::OpenHome)
                .into(),
            widget::button::icon(widget::icon::from_name("document-new-symbolic"))
                .on_press(Message::OpenCreator)
                .into(),
        ]
    }

    fn header_center(&self) -> Vec<Element<Self::Message>> {
        match self.current_page {
            Pages::MainWindow => vec![text("COSMIC Web Apps").into()],
            Pages::AppCreator => {
                let title = if self.creator_window.edit_mode {
                    format!("Edit {}", self.creator_window.app_title)
                } else {
                    "Create new Web App".to_string()
                };
                vec![text(title).into()]
            }
            Pages::IconPicker(_) => vec![text("Icon selector").into()],
            Pages::IconInstallator(_) => vec![text("Papirus Icons installator").into()],
        }
    }
}
