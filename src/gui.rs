use std::process::ExitStatus;

use cosmic::{
    app::{
        Core,
        message::{self, app}, Message as CosmicMessage,
    },
    Command, cosmic_theme,
    Element,
    executor,
    iced::window,
    style, widget::{self, text},
};
use cosmic_files::dialog::{Dialog, DialogKind, DialogMessage, DialogResult};

use crate::{
    add_icon_packs_install_script,
    common::{
        self, Browser, find_icons, get_icon_name_from_url, get_supported_browsers,
        icon_cache_get, image_handle, move_icon, search_user_icons, WebAppLauncher,
    },
    creator, execute_script,
    home_screen::Home,
    icon_pack_installed,
    iconpicker::{self, IconPicker},
    icons_installator::Installator,
    warning::WarnMessages,
};

#[derive(Debug, Clone)]
pub enum Buttons {
    SearchFavicon,
    Edit(WebAppLauncher),
    Delete(WebAppLauncher),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Message {
    OpenHome,
    OpenCreator,
    CloseCreator,
    OpenIconPicker,
    OpenIconPickerDialog,
    DialogMessage(DialogMessage),
    OpenFileResult(DialogResult),
    Creator(creator::Message),
    Result,

    Clicked(Buttons),
    // icons
    PerformIconSearch,
    CustomIconsSearch(String),
    FoundIcons(Vec<String>),
    PushIcon(Option<iconpicker::Icon>),
    ChangeIcon(iconpicker::Icon),
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
    main_window: Home,
    current_page: Pages,
    creator_window: creator::AppCreator,
    dialog_opt: Option<Dialog<Message>>,
}

impl cosmic::Application for Window {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "io.github.elevenhsoft.WebApps";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(
        core: Core,
        _flags: Self::Flags,
    ) -> (Self, Command<cosmic::app::Message<Self::Message>>) {
        let manager = Home::new();
        let creator = creator::AppCreator::new();

        let (page, cmd) = if !icon_pack_installed() {
            let cmd = Command::perform(add_icon_packs_install_script(), |file| {
                app(Message::InstallScript(file))
            });

            let installator = Installator::new();
            (Pages::IconInstallator(installator), cmd)
        } else {
            (Pages::MainWindow, Command::none())
        };

        let windows = Window {
            core,
            main_window: manager,
            current_page: page,
            creator_window: creator,
            dialog_opt: None,
        };

        (windows, cmd)
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let go_home_icon = icon_cache_get("go-home-symbolic", 16);
        let go_creator = icon_cache_get("document-new-symbolic", 16);
        let cosmic_theme::Spacing { space_xxs, .. } = self.core().system_theme().cosmic().spacing;

        vec![
            widget::button(go_home_icon)
                .on_press(Message::OpenHome)
                .padding(space_xxs)
                .style(style::Button::Icon)
                .into(),
            widget::button(go_creator)
                .on_press(Message::OpenCreator)
                .padding(space_xxs)
                .style(style::Button::Icon)
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

    fn update(&mut self, message: Self::Message) -> Command<CosmicMessage<Message>> {
        match message {
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
                let icons_picker = IconPicker::new();
                self.current_page = Pages::IconPicker(icons_picker);

                Command::perform(async {}, |_| app(Message::PerformIconSearch))
            }
            Message::OpenIconPickerDialog => {
                if self.dialog_opt.is_none() {
                    let (dialog, command) = Dialog::new(
                        DialogKind::OpenMultipleFiles,
                        None,
                        Message::DialogMessage,
                        Message::OpenFileResult,
                    );
                    self.dialog_opt = Some(dialog);
                    return command;
                }
                Command::none()
            }
            Message::DialogMessage(message) => {
                if let Some(dialog) = &mut self.dialog_opt {
                    return dialog.update(message);
                }
                Command::none()
            }
            Message::OpenFileResult(result) => {
                self.dialog_opt = None;
                match result {
                    DialogResult::Cancel => {}
                    DialogResult::Open(paths) => {
                        for path in paths {
                            if let Some(str) = path.to_str() {
                                let icon_name = path.file_stem();
                                if let Some(file_stem) = icon_name {
                                    move_icon(
                                        str.to_string(),
                                        file_stem.to_str().unwrap().to_string(),
                                    );

                                    return Command::perform(search_user_icons(), |result| {
                                        app(Message::FoundIcons(result))
                                    });
                                }
                            }
                        }
                    }
                }
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
                    let _ = move_icon(
                        self.creator_window.app_icon.clone(),
                        self.creator_window.app_title.clone(),
                    );

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
                    let selected_browser = get_supported_browsers()
                        .iter()
                        .position(|b| b.name == launcher.web_browser.name);

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
                    self.creator_window.selected_browser = selected_browser;
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

            Message::PerformIconSearch => {
                if let Pages::IconPicker(ref mut picker) = self.current_page {
                    picker.icons.clear();

                    let name = if picker.icon_searching.is_empty() {
                        get_icon_name_from_url(&self.creator_window.app_url)
                    } else {
                        picker.icon_searching.clone()
                    };

                    let icons = find_icons(name, self.creator_window.app_url.clone());

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

                    self.creator_window
                        .warning
                        .remove_warn(WarnMessages::AppIcon);
                }
                if let Some(ico) = icon {
                    if let Pages::IconPicker(ref mut picker) = self.current_page {
                        picker.icons.push(ico);
                    }
                }

                Command::none()
            }
            Message::ChangeIcon(icon) => {
                let path = icon.path.clone();
                let saved = move_icon(path, self.creator_window.app_title.clone());
                self.creator_window.selected_icon = Some(icon.clone());
                self.creator_window.app_icon = saved;
                self.current_page = Pages::AppCreator;

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

    fn view(&self) -> Element<Message> {
        match &self.current_page {
            Pages::MainWindow => self.main_window.view(),
            Pages::AppCreator => self.creator_window.view(),
            Pages::IconPicker(picker) => picker.view(),
            Pages::IconInstallator(installator) => installator.view(),
        }
    }

    fn view_window(&self, window_id: window::Id) -> Element<Message> {
        match &self.dialog_opt {
            Some(dialog) => dialog.view(window_id),
            None => widget::text("Unknown window ID").into(),
        }
    }
}
