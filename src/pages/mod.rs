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
use cosmic::{app, task, Theme};
use cosmic::{
    app::{command::Task, Core},
    cosmic_theme, executor, style,
    widget::{self},
    Application, ApplicationExt, Element,
};

use crate::{
    add_icon_packs_install_script,
    common::{
        self, find_icon, find_icons, get_icon_name_from_url, icon_cache_get, image_handle,
        move_icon, qwa_icons_location,
    },
    execute_script, fl, icon_pack_installed,
    pages::home_screen::Home,
    pages::iconpicker::IconPicker,
    pages::icons_installator::Installator,
    warning::WarnAction,
    warning::WarnMessages,
};
use crate::{browser, launcher};

#[derive(Debug, Clone)]
pub enum Buttons {
    SearchFavicon,
    Edit(launcher::WebAppLauncher),
    Delete(launcher::WebAppLauncher),
    DoneEdit((Option<String>, Option<String>)),
    DoneCreate,
    AppNameSubmit(launcher::WebAppLauncher),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Message {
    None,
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
    MyIcons,
    PerformIconSearch,
    FoundIcons(Vec<String>),
    PushIcon(Option<common::Icon>),
    SetIcon(common::Icon),
    SelectIcon(common::Icon),

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

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Message>) {
        let manager = Home::new();
        let creator = creator::AppCreator::new();
        let selector = IconPicker::default();

        let mut windows = Window {
            core,
            main_window: manager,
            current_page: Pages::MainWindow,
            creator_window: creator,
            icon_selector: selector,
        };

        let commands = vec![
            windows.update_title(),
            task::future(async { cosmic::app::Message::App(Message::SystemTheme) }),
        ];

        (windows, Task::batch(commands))
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let go_home_icon = icon_cache_get("go-home-symbolic", 16);
        let go_creator = icon_cache_get("document-new-symbolic", 16);
        let cosmic_theme::Spacing { space_xxs, .. } = self.core().system_theme().cosmic().spacing;

        vec![
            widget::button::custom(go_home_icon)
                .on_press(Message::OpenHome)
                .padding(space_xxs)
                .class(style::Button::Icon)
                .into(),
            widget::button::custom(go_creator)
                .on_press(Message::OpenCreator)
                .padding(space_xxs)
                .class(style::Button::Icon)
                .into(),
        ]
    }

    fn update(&mut self, message: Self::Message) -> Task<Message> {
        let mut commands: Vec<Task<Message>> = Vec::new();

        match message {
            Message::OpenHome => {
                self.current_page = Pages::MainWindow;
                commands.push(self.update_title());
            }

            Message::OpenCreator => {
                self.current_page = Pages::AppCreator;
                self.init_warning_box();
                commands.push(self.update_title());
            }
            Message::CloseCreator => {
                self.current_page = Pages::MainWindow;
                self.creator_window = creator::AppCreator::new();
                commands.push(self.update_title());
            }
            Message::Creator(message) => {
                commands.push(self.creator_window.update(message).map(|mess| mess));
            }
            Message::Warning((action, message)) => {
                match action {
                    WarnAction::Add => self.creator_window.warning.push_warn(message),
                    WarnAction::Remove => self.creator_window.warning.remove_warn(message),
                };
            }
            Message::OpenIconPicker => {
                self.current_page = Pages::IconPicker;
                commands.push(self.update_title())
            }
            Message::OpenIconPickerDialog => {
                commands.push(task::future(async move {
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
                        let files = result
                            .uris()
                            .iter()
                            .map(|file| file.path().to_string())
                            .collect::<Vec<String>>();

                        cosmic::app::message::app(Message::OpenFileResult(files))
                    } else {
                        cosmic::app::message::none()
                    }
                }));
            }
            Message::OpenFileResult(result) => {
                commands.push(task::future(async {
                    for path in result {
                        let Ok(buf) = PathBuf::from_str(&path);

                        let icon_name = buf.file_stem();
                        if let Some(file_stem) = icon_name {
                            move_icon(path.to_string(), file_stem.to_str().unwrap().to_string());
                        };
                    }

                    Message::FoundIcons(find_icon(qwa_icons_location(), String::new()).await)
                }));
            }
            Message::EditAppName(flag) => {
                if !flag {
                    self.main_window.new_app_name.clear()
                }

                self.main_window.edit_appname = flag;
            }
            Message::AppNameInput(new_name) => {
                self.main_window.new_app_name = new_name;
            }
            Message::Clicked(buttons) => match buttons {
                Buttons::DoneCreate => {
                    let new_entry = launcher::WebAppLauncher::new(
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

                    self.create_valid_launcher(new_entry).unwrap();
                }
                Buttons::DoneEdit((new_name, old_icon)) => {
                    if let Some(launcher) = self.main_window.launcher.to_owned() {
                        let _deleted = launcher.remove_desktop_file();
                        let mut edited_entry = launcher::WebAppLauncher::new(
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

                        self.create_valid_launcher(edited_entry).unwrap();
                    }
                }
                Buttons::AppNameSubmit(mut launcher) => {
                    launcher.name.clone_from(&self.main_window.new_app_name);
                    self.main_window
                        .launcher
                        .clone_from(&Some(launcher.clone()));

                    self.main_window.new_app_name.clear();

                    commands.push(task::future(async {
                        Message::Clicked(Buttons::DoneEdit((
                            Some(launcher.name),
                            Some(launcher.icon),
                        )))
                    }));
                }
                Buttons::Edit(launcher) => {
                    let selected_browser = browser::get_supported_browsers()
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
                        browser::Browser::web_browser(launcher.web_browser.name)
                            .expect("browser not found");
                    self.creator_window.selected_browser = selected_browser;
                    self.creator_window.app_navbar = launcher.navbar;
                    self.creator_window.app_incognito = launcher.is_incognito;
                    self.creator_window.edit_mode = true;

                    commands.push(task::future(async {
                        if let Some(res) = image_handle(launcher.icon).await {
                            return Message::SetIcon(res);
                        }
                        Message::None
                    }));
                }
                Buttons::Delete(launcher) => {
                    let _ = launcher.delete();
                }
                Buttons::SearchFavicon => {
                    if common::url_valid(&self.creator_window.app_url) {
                        self.icon_selector.icons.clear();
                        let url = self.creator_window.app_url.clone();

                        let name = get_icon_name_from_url(&self.creator_window.app_url);
                        commands.push(task::future(async {
                            Message::FoundIcons(find_icons(name, url).await)
                        }))
                    }
                }
            },
            Message::MyIcons => {
                let icon_name = self.icon_selector.icon_searching.clone();

                commands.push(task::future(async {
                    Message::FoundIcons(find_icon(qwa_icons_location(), icon_name).await)
                }))
            }
            Message::PerformIconSearch => {
                self.icon_selector.icons.clear();

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
                    return Task::perform(icons, |icons| {
                        app::message::Message::App(Message::FoundIcons(icons))
                    });
                }
            }
            Message::CustomIconsSearch(input) => {
                self.icon_selector.icon_searching = input;
            }
            Message::FoundIcons(result) => {
                self.icon_selector.icons.clear();
                result.into_iter().for_each(|path| {
                    commands.push(task::future(async {
                        Message::PushIcon(image_handle(path).await)
                    }));
                });
            }
            Message::PushIcon(icon) => {
                if icon.is_some() {
                    commands.push(task::future(async {
                        Message::Warning((WarnAction::Remove, WarnMessages::AppIcon))
                    }));
                };

                if let Some(ico) = icon {
                    if !self.icon_selector.icons.contains(&ico) {
                        self.icon_selector.icons.push(ico);
                    }
                };

                self.icon_selector
                    .icons
                    .sort_by_key(|icon| !icon.is_favicon);

                commands.push(task::future(async { Message::LoadingDone }));
            }
            Message::LoadingDone => {
                if !self.icon_selector.icons.is_empty() {
                    self.creator_window.selected_icon = Some(self.icon_selector.icons[0].clone());

                    if self.creator_window.selected_icon.is_some() {
                        commands.push(task::future(async {
                            Message::Warning((WarnAction::Remove, WarnMessages::AppIcon))
                        }));
                    }
                }
            }
            Message::SetIcon(icon) => {
                self.current_page = Pages::AppCreator;

                self.current_page = Pages::AppCreator;
                self.creator_window.selected_icon = Some(icon.clone());

                commands.push(task::future(async { Message::SelectIcon(icon) }));
            }
            Message::SelectIcon(ico) => {
                self.creator_window.selected_icon = Some(ico.clone());
                self.creator_window.app_icon = ico.path;

                if self.creator_window.selected_icon.is_some() {
                    commands.push(task::future(async {
                        Message::Warning((WarnAction::Remove, WarnMessages::AppIcon))
                    }));
                } else {
                    commands.push(task::future(async {
                        Message::Warning((WarnAction::Add, WarnMessages::AppIcon))
                    }));
                }
            }
            Message::DownloadIconsPack => {
                let installator = Installator::new();
                self.current_page = Pages::IconInstallator(installator);
                commands.push(self.update_title());
                commands.push(task::future(async {
                    Message::InstallScript(add_icon_packs_install_script().await)
                }));
            }
            Message::InstallScript(script) => {
                if !icon_pack_installed() {
                    commands.push(task::future(async {
                        Message::InstallCommand(execute_script(script).await)
                    }));
                }
            }
            Message::InstallCommand(exit_status) => {
                if ExitStatus::success(&exit_status) {
                    self.current_page = Pages::MainWindow;
                }
                commands.push(self.update_title())
            }
            Message::SystemTheme => {
                if std::env::var("XDG_CURRENT_DESKTOP") != Ok("COSMIC".to_string()) {
                    commands.push(set_theme(Theme::custom(Arc::new(
                        cosmic_theme::Theme::preferred_theme(),
                    ))))
                }
            }
            Message::None => {}
        }

        Task::batch(commands)
    }

    fn view(&self) -> Element<Message> {
        let view = match &self.current_page {
            Pages::MainWindow => self.main_window.view(),
            Pages::AppCreator => self
                .creator_window
                .view(self.creator_window.warning.clone()),
            Pages::IconPicker => self.icon_selector.view(),
            Pages::IconInstallator(installator) => installator.view(),
        };

        Container::new(view)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .center_x(Length::Fill)
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
    fn update_title(&mut self) -> Task<Message> {
        self.set_header_title(self.match_title());
        self.set_window_title(self.match_title())
    }

    fn create_valid_launcher(&mut self, mut entry: launcher::WebAppLauncher) -> anyhow::Result<()> {
        if let Some(icon) = &self.creator_window.selected_icon {
            let path = move_icon(icon.path.clone(), self.creator_window.app_title.clone());

            if path.is_empty() {
                self.creator_window
                    .warning
                    .push_warn(WarnMessages::WrongIcon);
                return Ok(());
            }

            self.creator_window
                .warning
                .remove_warn(WarnMessages::WrongIcon);
            entry.icon = path;
        }
        if launcher::webapplauncher_is_valid(
            &entry.web_browser,
            &entry.icon,
            &entry.codename,
            &entry.name,
            &entry.url,
        ) && !self.creator_window.warning.show
        {
            let _ = entry.create().is_ok();
            self.creator_window = creator::AppCreator::new();
            self.current_page = Pages::MainWindow;
            return Ok(());
        };

        self.creator_window
            .warning
            .push_warn(WarnMessages::Duplicate);
        Ok(())
    }

    fn init_warning_box(&mut self) {
        self.creator_window.warning.remove_all_warns();

        if self.creator_window.app_title.is_empty() || self.creator_window.app_title.len() <= 3 {
            self.creator_window.warning.push_warn(WarnMessages::AppName)
        }
        if self.creator_window.app_url.is_empty() {
            self.creator_window.warning.push_warn(WarnMessages::AppUrl)
        }
        if self.creator_window.app_icon.is_empty() {
            self.creator_window.warning.push_warn(WarnMessages::AppIcon)
        }
        if self.creator_window.app_browser._type == browser::BrowserType::NoBrowser {
            self.creator_window
                .warning
                .push_warn(WarnMessages::AppBrowser)
        }
    }
}
