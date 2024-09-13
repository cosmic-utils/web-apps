pub mod creator;
pub mod home_screen;
pub mod iconpicker;
pub mod icons_installator;

use std::path::PathBuf;
use std::process::ExitStatus;
use std::str::FromStr;
use std::sync::Arc;

use ashpd::desktop::file_chooser::{FileFilter, SelectedFiles};
use cosmic::app::command::set_theme;
use cosmic::iced::alignment::Horizontal;
use cosmic::iced::Length;
use cosmic::widget::Container;
use cosmic::Theme;
use cosmic::{
    app::{
        message::{self, app},
        Core, Message as CosmicMessage,
    },
    cosmic_theme, executor, style,
    widget::{self},
    Application, ApplicationExt, Command, Element,
};

use crate::{
    add_icon_packs_install_script,
    common::{
        self, find_icon, find_icons, get_icon_name_from_url, get_supported_browsers,
        icon_cache_get, image_handle, move_icon, my_icons_location, Browser, BrowserType,
        WebAppLauncher,
    },
    execute_script, fl, icon_pack_installed,
    pages::home_screen::Home,
    pages::iconpicker::IconPicker,
    pages::icons_installator::Installator,
    warning::WarnMessages,
    warning::{WarnAction, Warning},
};

#[derive(Debug, Clone)]
pub enum Buttons {
    SearchFavicon,
    Edit(WebAppLauncher),
    Delete(WebAppLauncher),
    DoneEdit((Option<String>, Option<String>)),
    DoneCreate,
    AppNameSubmit(WebAppLauncher),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Message {
    OpenHome,
    OpenCreator,
    CloseCreator,
    OpenIconPicker,
    OpenIconPickerDialog,
    OpenFileResult(Vec<String>),
    Creator(creator::Message),
    LoadingDone,

    EditAppName(bool),
    AppNameInput(String),
    Clicked(Buttons),
    // icons
    CustomIconsSearch(String),
    ChangeIcon(iconpicker::Icon),
    MyIcons,
    PerformIconSearch,
    FoundIcons(Vec<String>),
    PushIcon(Option<iconpicker::Icon>),
    SetIcon(iconpicker::Icon),
    SelectIcon(iconpicker::Icon),

    Warning((WarnAction, WarnMessages)),

    // Installator
    DownloadIconsPack,
    InstallScript(String),
    InstallCommand(ExitStatus),

    SystemTheme,
}

#[derive(Debug, Clone)]
pub enum Pages {
    MainWindow,
    AppCreator,
    IconPicker,
    IconInstallator(Installator),
}

pub struct Window {
    core: Core,
    main_window: Home,
    current_page: Pages,
    creator_window: creator::AppCreator,
    icon_selector: IconPicker,
    warning: Warning,
}

impl Application for Window {
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
        let selector = IconPicker::default();

        let warn_element = Warning::new(Vec::new());

        let mut windows = Window {
            core,
            main_window: manager,
            current_page: Pages::MainWindow,
            creator_window: creator,
            icon_selector: selector,
            warning: warn_element,
        };

        let commands = Command::batch(vec![
            windows.update_title(),
            Command::perform(async {}, |_| {
                cosmic::app::message::app(Message::SystemTheme)
            }),
        ]);

        (windows, commands)
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

    fn update(&mut self, message: Self::Message) -> Command<CosmicMessage<Message>> {
        match message {
            Message::OpenHome => {
                self.current_page = Pages::MainWindow;

                self.update_title()
            }

            Message::OpenCreator => {
                self.current_page = Pages::AppCreator;
                self.init_warning_box();

                self.update_title()
            }
            Message::CloseCreator => {
                self.current_page = Pages::MainWindow;
                self.creator_window.edit_mode = false;

                self.update_title()
            }
            Message::Creator(message) => {
                let command = self.creator_window.update(message);

                command.map(|mess| mess)
            }
            Message::Warning((action, message)) => {
                match action {
                    WarnAction::Add => self.warning.push_warn(message),
                    WarnAction::Remove => self.warning.remove_warn(message),
                };
                Command::none()
            }
            Message::OpenIconPicker => {
                self.current_page = Pages::IconPicker;

                self.update_title()
            }
            Message::OpenIconPickerDialog => {
                return Command::perform(
                    async move {
                        let result = SelectedFiles::open_file()
                            .title("Open multiple images")
                            .accept_label("Attach")
                            .modal(true)
                            .multiple(true)
                            .filter(FileFilter::new("PNG Image").glob("*.png"))
                            .filter(FileFilter::new("SVG Images").glob("*.svg"))
                            .send()
                            .await
                            .unwrap()
                            .response();

                        if let Ok(result) = result {
                            result
                                .uris()
                                .iter()
                                .map(|file| file.path().to_string())
                                .collect::<Vec<String>>()
                        } else {
                            Vec::new()
                        }
                    },
                    |files| cosmic::app::message::app(Message::OpenFileResult(files)),
                );
            }
            Message::OpenFileResult(result) => {
                for path in result {
                    if let Ok(buf) = PathBuf::from_str(&path) {
                        let icon_name = buf.file_stem();
                        if let Some(file_stem) = icon_name {
                            move_icon(path.to_string(), file_stem.to_str().unwrap().to_string())
                                .unwrap();

                            return Command::perform(
                                find_icon(my_icons_location(), String::new()),
                                |result| app(Message::FoundIcons(result)),
                            );
                        }
                    }
                }
                Command::none()
            }
            Message::EditAppName(flag) => {
                if !flag {
                    self.main_window.new_app_name.clear()
                }

                self.main_window.edit_appname = flag;
                Command::none()
            }
            Message::AppNameInput(new_name) => {
                self.main_window.new_app_name = new_name;
                Command::none()
            }
            Message::Clicked(buttons) => match buttons {
                Buttons::DoneCreate => {
                    let new_entry = WebAppLauncher::new(
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
                    );

                    if new_entry.is_valid {
                        self.create_valid_launcher(new_entry).unwrap();
                    } else {
                        self.warning.push_warn(WarnMessages::Duplicate);
                    }
                    self.creator_window.edit_mode = false;
                    Command::none()
                }
                Buttons::DoneEdit((new_name, old_icon)) => {
                    if let Some(launcher) = self.main_window.launcher.to_owned() {
                        let _deleted = launcher.delete();
                        let mut edited_entry = WebAppLauncher::new(
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
                        );

                        if new_name.is_some() {
                            edited_entry.name = new_name.unwrap();
                        }

                        if old_icon.is_some() {
                            edited_entry.icon = old_icon.unwrap();
                        }

                        if edited_entry.is_valid {
                            self.create_valid_launcher(edited_entry).unwrap();
                        } else {
                            self.warning.push_warn(WarnMessages::Duplicate);
                        }
                    }
                    self.creator_window.edit_mode = false;
                    Command::none()
                }
                Buttons::AppNameSubmit(mut launcher) => {
                    launcher.name.clone_from(&self.main_window.new_app_name);
                    self.main_window
                        .launcher
                        .clone_from(&Some(launcher.clone()));

                    self.main_window.new_app_name.clear();

                    Command::perform(async {}, |_| {
                        app(Message::Clicked(Buttons::DoneEdit((
                            Some(launcher.name),
                            Some(launcher.icon),
                        ))))
                    })
                }
                Buttons::Edit(launcher) => {
                    let selected_browser = get_supported_browsers()
                        .iter()
                        .position(|b| b.name == launcher.web_browser.name);

                    self.warning.remove_all_warns();
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
                        self.icon_selector.icons.clear();

                        let name = get_icon_name_from_url(&self.creator_window.app_url);
                        let icons = find_icons(name, self.creator_window.app_url.clone());
                        Command::perform(icons, |icons| app(Message::FoundIcons(icons)))
                    } else {
                        Command::none()
                    }
                }
            },
            Message::MyIcons => {
                let icon_name = self.icon_selector.icon_searching.clone();

                self.icon_selector.loading = true;

                Command::perform(find_icon(my_icons_location(), icon_name), |result| {
                    app(Message::FoundIcons(result))
                })
            }
            Message::PerformIconSearch => {
                self.icon_selector.icons.clear();
                self.icon_selector.loading = true;

                let name = if self.icon_selector.icon_searching.is_empty()
                    && !self.creator_window.app_url.is_empty()
                {
                    get_icon_name_from_url(&self.creator_window.app_url)
                } else {
                    self.icon_selector.icon_searching.clone()
                };

                let icons = find_icons(name, self.creator_window.app_url.clone());

                if !self.creator_window.app_url.is_empty()
                    || !self.icon_selector.icon_searching.is_empty()
                {
                    return Command::perform(icons, |icons| app(Message::FoundIcons(icons)));
                }

                Command::none()
            }
            Message::CustomIconsSearch(input) => {
                self.icon_selector.icon_searching = input;
                self.icon_selector.loading = false;

                Command::none()
            }
            Message::FoundIcons(result) => {
                self.icon_selector.icons.clear();
                let mut commands: Vec<Command<CosmicMessage<Message>>> = Vec::new();

                result.into_iter().for_each(|path| {
                    commands.push(Command::perform(image_handle(path), |result| {
                        app(Message::PushIcon(result))
                    }));
                });

                Command::batch(commands)
            }
            Message::PushIcon(icon) => {
                let mut cmd = Command::none();
                if icon.is_some() {
                    cmd = Command::perform(async {}, |_| {
                        app(Message::Warning((
                            WarnAction::Remove,
                            WarnMessages::AppIcon,
                        )))
                    })
                };

                if let Some(ico) = icon {
                    if !self.icon_selector.icons.contains(&ico) {
                        self.icon_selector.icons.push(ico);
                    }
                };

                let done = Command::perform(async {}, |_| app(Message::LoadingDone));

                Command::batch(vec![cmd, done])
            }
            Message::LoadingDone => {
                self.icon_selector.loading = false;

                if !self.icon_selector.icons.is_empty() {
                    let path = self.icon_selector.icons[0].path.clone();
                    if let Some(saved) = move_icon(path, self.creator_window.app_title.clone()) {
                        self.creator_window.app_icon = saved;
                    };
                    self.creator_window.selected_icon = Some(self.icon_selector.icons[0].clone());
                }

                Command::none()
            }
            Message::ChangeIcon(icon) => {
                self.current_page = Pages::AppCreator;

                let path = icon.path.clone();
                self.creator_window.selected_icon = Some(icon.clone());
                if let Some(saved) = move_icon(path, self.creator_window.app_title.clone()) {
                    self.creator_window.app_icon = saved;
                };

                if self.creator_window.selected_icon.is_some()
                    && !self.creator_window.app_icon.is_empty()
                {
                    Command::perform(async {}, |_| {
                        app(Message::Warning((
                            WarnAction::Remove,
                            WarnMessages::AppIcon,
                        )))
                    })
                } else {
                    Command::perform(async {}, |_| {
                        app(Message::Warning((WarnAction::Add, WarnMessages::AppIcon)))
                    })
                }
            }
            Message::SetIcon(icon) => {
                let path = icon.path;

                if let Some(saved) = move_icon(path, self.creator_window.app_title.clone()) {
                    self.current_page = Pages::AppCreator;
                    self.creator_window.app_icon.clone_from(&saved);

                    Command::perform(image_handle(saved), |result| {
                        if let Some(res) = result {
                            app(Message::SelectIcon(res))
                        } else {
                            message::none()
                        }
                    })
                } else {
                    Command::none()
                }
            }
            Message::SelectIcon(ico) => {
                self.creator_window.selected_icon = Some(ico.clone());
                self.creator_window.app_icon = ico.path;

                Command::none()
            }
            Message::DownloadIconsPack => {
                let installator = Installator::new();
                self.current_page = Pages::IconInstallator(installator);

                let update_title = self.update_title();

                Command::batch(vec![
                    update_title,
                    Command::perform(add_icon_packs_install_script(), |file| {
                        app(Message::InstallScript(file))
                    }),
                ])
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

                self.update_title()
            }
            Message::SystemTheme => {
                if std::env::var("XDG_CURRENT_DESKTOP") != Ok("COSMIC".to_string()) {
                    set_theme(Theme::custom(Arc::new(
                        cosmic_theme::Theme::preferred_theme(),
                    )))
                } else {
                    Command::none()
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let view = match &self.current_page {
            Pages::MainWindow => self.main_window.view(),
            Pages::AppCreator => self.creator_window.view(self.warning.messages()),
            Pages::IconPicker => self.icon_selector.view(),
            Pages::IconInstallator(installator) => installator.view(),
        };

        Container::new(view)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .center_x()
            .into()
    }
}

impl Window {
    fn match_title(&self) -> String {
        match self.current_page {
            Pages::MainWindow => fl!("app"),
            Pages::AppCreator => {
                if self.creator_window.edit_mode {
                    format!("{} {}", fl!("edit"), self.creator_window.app_title)
                } else {
                    fl!("create-new-webapp")
                }
            }
            Pages::IconPicker => fl!("icon-selector"),
            Pages::IconInstallator(_) => fl!("icon-installer"),
        }
    }
    fn update_title(&mut self) -> Command<CosmicMessage<Message>> {
        self.set_header_title(self.match_title());
        self.set_window_title(self.match_title())
    }

    fn create_valid_launcher(&mut self, entry: WebAppLauncher) -> anyhow::Result<()> {
        if entry.create().is_ok() {
            self.creator_window.edit_mode = false;
            self.current_page = Pages::MainWindow;
        };

        Ok(())
    }

    fn init_warning_box(&mut self) {
        self.warning.remove_all_warns();

        if self.creator_window.app_title.is_empty() || self.creator_window.app_title.len() <= 3 {
            self.warning.push_warn(WarnMessages::AppName)
        }
        if self.creator_window.app_url.is_empty() {
            self.warning.push_warn(WarnMessages::AppUrl)
        }
        if self.creator_window.app_icon.is_empty() {
            self.warning.push_warn(WarnMessages::AppIcon)
        }
        if self.creator_window.app_browser._type == BrowserType::NoBrowser {
            self.warning.push_warn(WarnMessages::AppBrowser)
        }
    }
}
