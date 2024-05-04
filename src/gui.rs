use crate::{
    add_icon_packs_install_script,
    common::{
        self, find_icons, get_icon_name_from_url, image_handle, move_icon, Browser, WebAppLauncher,
    },
    execute_script, icon_pack_installed,
    iconpicker::{self, IconPicker},
    icons_installator::{self, Installator},
    wam,
};

use cosmic::{
    app::Core,
    executor,
    iced::{self, event, window, Command},
    iced_core::Point,
    iced_widget::text_input::focus,
    Element,
};
use std::{collections::HashMap, process::ExitStatus};

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
    PushIcon(Option<iconpicker::Icon>),
    SetIcon(iconpicker::Icon),
    SelectIcon(iconpicker::Icon),

    // Installator
    InstallScript(String),
    InstallCommand(ExitStatus),
}

#[derive(Debug)]
pub enum Dialogs {
    IconPicker(IconPicker),
    IconInstallator(Installator),
}

pub struct Window {
    core: Core,
    windows: HashMap<window::Id, wam::Wam>,
    main_window: wam::Wam,
    dialog_open: bool,
    dialog_window: Dialogs,
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
        let manager = wam::Wam::new();

        let mut icon_dialog = false;

        let (dialog, cmd) = if !icon_pack_installed() {
            icon_dialog = true;

            let cmd = Command::perform(add_icon_packs_install_script(), |file| {
                cosmic::app::message::app(Message::InstallScript(file))
            });

            let installator = icons_installator::Installator::new();
            (Dialogs::IconInstallator(installator), cmd)
        } else {
            let icons_picker = iconpicker::IconPicker::new();
            (Dialogs::IconPicker(icons_picker), Command::none())
        };

        let windows = Window {
            core,
            windows: HashMap::from([(window::Id::MAIN, manager.clone())]),
            main_window: manager,
            dialog_open: icon_dialog,
            dialog_window: dialog,
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
                self.dialog_open = true;

                let icons_picker = iconpicker::IconPicker::new();
                self.dialog_window = Dialogs::IconPicker(icons_picker);

                if let Dialogs::IconPicker(picker) = &self.dialog_window {
                    return focus(picker.searching_id.clone());
                };

                Command::none()
            }
            Message::CloseIconPicker => {
                self.dialog_open = false;

                focus(self.main_window.app_title_id.clone())
            }
            Message::Result => {
                let launcher = if let Some(launcher) = self.main_window.launcher.to_owned() {
                    let _ = launcher.delete();

                    WebAppLauncher::new(
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
                    )
                } else {
                    WebAppLauncher::new(
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
                    )
                };

                if launcher.is_valid {
                    let _ = launcher.create();
                }

                Command::none()
            }
            Message::Clicked(buttons) => match buttons {
                Buttons::SearchFavicon => {
                    if common::url_valid(&self.main_window.app_url) {
                        if let Dialogs::IconPicker(ref mut picker) = self.dialog_window {
                            picker.icons.clear();
                        }

                        let name = get_icon_name_from_url(&self.main_window.app_url);
                        let icons = find_icons(name, self.main_window.app_url.clone());
                        Command::perform(icons, |icons| {
                            cosmic::app::message::app(Message::FoundIcons(icons))
                        })
                    } else {
                        Command::none()
                    }
                }
                Buttons::Edit(launcher) => {
                    self.main_window.edit_mode = true;
                    self.main_window.launcher = Some(launcher.clone());

                    self.main_window.app_title = launcher.name;
                    self.main_window.app_url = launcher.url;
                    self.main_window.app_icon = launcher.icon.clone();
                    self.main_window.app_parameters = launcher.custom_parameters;
                    self.main_window.app_category = launcher.category;
                    self.main_window.app_browser =
                        Browser::web_browser(launcher.web_browser.name).expect("browser not found");
                    self.main_window.app_navbar = launcher.navbar;
                    self.main_window.app_incognito = launcher.is_incognito;

                    Command::perform(image_handle(launcher.icon), |result| {
                        cosmic::app::message::app(Message::SetIcon(result.unwrap()))
                    })
                }
                Buttons::Delete(launcher) => {
                    let _ = launcher.delete();

                    Command::none()
                }
                Buttons::Navbar(selected) => {
                    self.main_window.app_navbar = selected;

                    Command::none()
                }
                Buttons::IsolatedProfile(selected) => {
                    self.main_window.app_isolated = selected;

                    Command::none()
                }
                Buttons::Incognito(selected) => {
                    self.main_window.app_incognito = selected;

                    Command::none()
                }
            },
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
            Message::Favicon(path) => Command::perform(image_handle(path), |result| {
                cosmic::app::message::app(Message::SetIcon(result.unwrap()))
            }),
            Message::PerformIconSearch => {
                if let Dialogs::IconPicker(ref mut picker) = self.dialog_window {
                    picker.icons.clear();

                    let icons = find_icons(
                        picker.icon_searching.clone(),
                        self.main_window.app_url.clone(),
                    );

                    return Command::perform(icons, |icons| {
                        cosmic::app::message::app(Message::FoundIcons(icons))
                    });
                };

                Command::none()
            }
            Message::CustomIconsSearch(input) => {
                if let Dialogs::IconPicker(ref mut picker) = self.dialog_window {
                    picker.icon_searching = input;
                }

                Command::none()
            }
            Message::FoundIcons(result) => {
                let mut commands: Vec<iced::Command<cosmic::app::Message<Message>>> = Vec::new();

                result.into_iter().for_each(|path| {
                    commands.push(Command::perform(image_handle(path), |result| {
                        cosmic::app::message::app(Message::PushIcon(result))
                    }));
                });

                Command::batch(commands)
            }
            Message::PushIcon(icon) => {
                if self.main_window.selected_icon.is_none() && icon.is_some() {
                    let path = icon.as_ref().unwrap().path.clone();
                    let saved = move_icon(path, self.main_window.app_title.clone());
                    self.main_window.app_icon = saved.clone();
                    self.main_window.app_icon = saved;
                    self.main_window.selected_icon = icon.clone();
                }
                if let Some(ico) = icon {
                    if let Dialogs::IconPicker(ref mut picker) = self.dialog_window {
                        picker.icons.push(ico);
                    }
                }

                Command::none()
            }
            Message::SetIcon(icon) => {
                let path = icon.path;

                let saved = move_icon(path, self.main_window.app_title.clone());
                self.dialog_open = false;
                self.main_window.app_icon = saved.clone();

                Command::perform(image_handle(saved), |result| {
                    cosmic::app::message::app(Message::SelectIcon(result.unwrap()))
                })
            }
            Message::SelectIcon(ico) => {
                self.main_window.selected_icon = Some(ico.clone());
                self.main_window.app_icon = ico.path;
                Command::none()
            }

            // *** INSTALLATOR **** //
            Message::InstallScript(script) => {
                if !icon_pack_installed() {
                    return Command::perform(execute_script(script), |status| {
                        cosmic::app::message::app(Message::InstallCommand(status))
                    });
                }
                Command::none()
            }
            Message::InstallCommand(exit_status) => {
                if ExitStatus::success(&exit_status) {
                    self.dialog_open = false;
                }

                tracing::info!("{}", exit_status);

                Command::none()
            }
        }
    }

    fn dialog(&self) -> Option<Element<Self::Message>> {
        if self.dialog_open {
            return match &self.dialog_window {
                Dialogs::IconPicker(picker) => Some(picker.view()),
                Dialogs::IconInstallator(installator) => Some(installator.view()),
            };
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
