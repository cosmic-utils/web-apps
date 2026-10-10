pub mod editor;
mod iconpicker;

use ashpd::desktop::file_chooser::{FileFilter, SelectedFiles};
use cosmic::{
    Application, Element,
    app::{Core, Task, context_drawer},
    command::set_theme,
    cosmic_theme,
    iced::{
        Alignment, Length, Subscription,
        alignment::Horizontal,
        futures::{SinkExt as _, channel::mpsc::Sender, future},
    },
    task, theme,
    widget::{
        self, RcElementWrapper,
        about::About,
        button, icon,
        menu::{self, ItemHeight, ItemWidth},
        nav_bar,
    },
};
use editor::AppEditor;
use std::{
    collections::HashMap, fs::read_dir, io::Read, path::Path, process::ExitStatus, sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::oneshot,
};
use tracing::debug;
use webapps::{APP_ICON, APP_ID, AppConfig, Theme, WebappIcon, fl, launcher::installed_webapps};

use crate::pages::iconpicker::IconPicker;

#[derive(Debug, Clone)]
pub enum Message {
    ChangeUserTheme(usize),
    CloseDialog,
    Editor(editor::Message),
    Delete(widget::segmented_button::Entity),
    DeletionDone(widget::segmented_button::Entity),
    DownloaderDone,
    DownloaderStarted,
    DownloaderStream(String),
    DownloaderStreamFinished,
    Close,
    IconPicker(iconpicker::Message),
    IconsResult(Vec<String>),
    ImportThemeFilePicker,
    LaunchUrl(String),
    LoadThemes,
    OpenFileResult(String),
    OpenIconPicker,
    OpenThemeResult(String),
    ConfirmDeletion(widget::segmented_button::Entity),
    PushIcon(WebappIcon),
    ReloadNavbarItems,
    ResetSettings,
    SaveLauncher,
    SetIcon(Option<WebappIcon>),
    DownloaderStop,
    ToggleContextPage(ContextPage),
    UpdateConfig(AppConfig),
    UpdateTheme(Box<Theme>),
}

#[derive(Debug, Clone)]
pub enum Page {
    Editor(AppEditor),
}

#[derive(Debug, Clone)]
pub enum Dialogs {
    IconPicker(IconPicker),
    Confirmation((widget::segmented_button::Entity, String)),
    IconsDownloader,
}

pub struct QuickWebApps {
    core: Core,
    context_page: ContextPage,
    about: About,
    nav: nav_bar::Model,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    config: AppConfig,
    page: Page,
    dialogs: Option<Dialogs>,
    downloader_started: bool,
    downloader_id: usize,
    downloader_output: String,
    themes_list: Vec<Theme>,
    theme_idx: Option<usize>,
}

impl Application for QuickWebApps {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let config = AppConfig::config();
        let add_page = Page::Editor(AppEditor::default());
        let nav = nav_bar::Model::default();

        let themes_list = Vec::new();

        let about = About::default()
            .name(fl!("app"))
            // TODO: Update icon with a svg
            .icon(icon::from_raster_bytes(APP_ICON))
            .version(env!("CARGO_PKG_VERSION"))
            .author("heppen")
            .comments(fl!("comment"))
            .license(env!("CARGO_PKG_LICENSE"))
            .license_url("https://spdx.org/licenses/GPL-3.0-only")
            .developers([("heppen", "piotr@heppen.dev")])
            .links([
                (
                    fl!("repository"),
                    "https://github.com/cosmic-utils/web-apps",
                ),
                (
                    fl!("support"),
                    "https://github.com/cosmic-utils/web-apps/issues",
                ),
            ]);

        let app = QuickWebApps {
            core,
            context_page: ContextPage::default(),
            about,
            nav,
            key_binds: HashMap::new(),
            config,
            page: add_page,
            dialogs: None,
            downloader_started: false,
            downloader_id: 1,
            downloader_output: String::new(),
            themes_list,
            theme_idx: None,
        };

        let tasks = vec![
            task::message(Message::ReloadNavbarItems),
            task::message(Message::LoadThemes),
        ];

        (app, Task::batch(tasks))
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = Vec::new();

        subscriptions.push(
            self.core()
                .watch_config::<AppConfig>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        );

        if self.downloader_started {
            subscriptions.push(Subscription::run_with(self.downloader_id, |_| {
                cosmic::iced::stream::channel(4, move |mut channel: Sender<Message>| async move {
                    let Some(script) = webapps::add_icon_packs_install_script().await else {
                        return;
                    };

                    let mut child = webapps::execute_script(script).await;
                    let stdout = child
                        .stdout
                        .take()
                        .expect("child did not have a handle to stdout");

                    let mut reader = BufReader::new(stdout).lines();
                    let (tx, rx) = oneshot::channel::<ExitStatus>();

                    tokio::spawn(async move {
                        let status = child
                            .wait()
                            .await
                            .expect("child process encountered an error");

                        let _ = tx.send(status);
                    });

                    while let Ok(Some(line)) = reader.next_line().await {
                        _ = channel.send(Message::DownloaderStream(line)).await;
                    }

                    match rx.await {
                        Ok(es) => {
                            if es.success() {
                                let _ = channel.send(Message::DownloaderStreamFinished).await;
                            }
                        }
                        Err(_) => tracing::error!("the sender dropped"),
                    }

                    future::pending().await
                })
            }));
        }

        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Message) -> cosmic::Task<cosmic::Action<Message>> {
        let mut tasks: Vec<cosmic::Task<cosmic::Action<Message>>> = Vec::new();

        match message {
            Message::ChangeUserTheme(idx) => {
                self.theme_idx = Some(idx);
                let selected = self.themes_list[idx].clone();

                return task::message(cosmic::action::app(Message::UpdateTheme(Box::new(
                    selected,
                ))));
            }
            Message::CloseDialog => self.dialogs = None,
            Message::ConfirmDeletion(id) => {
                let data = self.nav.data::<Page>(id);

                if let Some(page) = data {
                    let Page::Editor(app_editor) = page;
                    self.dialogs = Some(Dialogs::Confirmation((id, app_editor.app_title.clone())))
                };
            }
            Message::Editor(msg) => match &mut self.page {
                Page::Editor(app_editor) => tasks.push(app_editor.update(msg)),
            },
            Message::Delete(id) => {
                let data = self.nav.data::<Page>(id);

                if let Some(page) = data {
                    let Page::Editor(app_editor) = page;

                    let app_unique_id = app_editor.app_id.clone();

                    return task::future(async move {
                        let launcher = installed_webapps()
                            .into_iter()
                            .find(|w| w.unique_id == app_unique_id)
                            .map(|l| l);

                        let Some(launcher) = launcher else {
                            return cosmic::action::none();
                        };

                        if launcher.delete() {
                            cosmic::action::app(Message::DeletionDone(id))
                        } else {
                            return cosmic::action::none();
                        }
                    });
                }
            }
            Message::DeletionDone(id) => {
                self.nav.remove(id);
                self.dialogs = None;
                self.page = Page::Editor(AppEditor::default())
            }
            Message::DownloaderDone => {
                self.downloader_started = false;
                return task::message(cosmic::action::app(Message::CloseDialog));
            }
            Message::DownloaderStarted => {
                self.dialogs = None;
                self.downloader_started = true;
                self.dialogs = Some(Dialogs::IconsDownloader)
            }
            Message::DownloaderStream(buffer) => {
                self.downloader_output.push_str(&format!("{buffer:?}\n"));
            }
            Message::DownloaderStop => {
                self.downloader_started = false;
                self.downloader_id += 1;
                self.downloader_output
                    .push_str(&fl!("downloader-canceled").to_string());
            }
            Message::DownloaderStreamFinished => {
                self.downloader_output
                    .push_str(&fl!("icons-installer-finished-waiting").to_string());

                return task::future(async {
                    tokio::time::sleep(Duration::from_secs_f32(3.0)).await;

                    cosmic::action::app(Message::DownloaderDone)
                });
            }
            Message::Close => {
                debug!("should close now...");
                return Task::none();
            }
            Message::IconPicker(msg) => {
                if let Some(Dialogs::IconPicker(icon_picker)) = &mut self.dialogs {
                    tasks.push(icon_picker.update(msg));
                };
            }
            Message::IconsResult(result) => {
                if let Some(Dialogs::IconPicker(_icon_picker)) = &mut self.dialogs {
                    for path in result {
                        tasks.push(Task::future(async move {
                            cosmic::Action::App(Message::PushIcon(WebappIcon::build_from_path(
                                &path,
                            )))
                        }))
                    }
                };
            }
            Message::ImportThemeFilePicker => {
                return task::future(async move {
                    let result = SelectedFiles::open_file()
                        .title("Open Theme")
                        .accept_label("Open")
                        .modal(true)
                        .multiple(false)
                        .filter(FileFilter::new("Ron Theme").glob("*.ron"))
                        .send()
                        .await
                        .unwrap()
                        .response();

                    if let Ok(result) = result {
                        let files = result
                            .uris()
                            .iter()
                            .map(|file| {
                                let mut file_path = file.as_str();
                                println!("file path: {}", file_path);

                                if file_path.starts_with("file://") {
                                    file_path =
                                        file_path.strip_prefix("file://").expect("removing prefix");
                                }

                                file_path.to_string()
                            })
                            .collect::<Vec<String>>();

                        return cosmic::action::app(Message::OpenThemeResult(
                            urlencoding::decode(&files[0])
                                .unwrap_or_default()
                                .to_string(),
                        ));
                    }

                    cosmic::action::none()
                });
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
            Message::LoadThemes => {
                self.themes_list.clear();

                self.themes_list.push(Theme::Dark);
                self.themes_list.push(Theme::Light);

                let Some(folder) = webapps::themes_path("") else {
                    return Task::none();
                };

                let dir = read_dir(folder);

                if let Ok(files) = dir {
                    for path in files {
                        let dir_entry = path.unwrap();
                        let file_name = dir_entry.file_name();
                        let theme_name = file_name.to_str().unwrap().replace(".ron", "");
                        let metadata = std::fs::metadata(dir_entry.path());

                        if let Ok(meta) = metadata {
                            if meta.is_file() {
                                let mut content: String = String::new();

                                let mut file = std::fs::File::open(dir_entry.path()).unwrap();
                                let _ = file.read_to_string(&mut content);

                                let theme = Theme::build(theme_name.to_string(), content);

                                self.themes_list.push(theme);
                            }
                        }
                    }
                }

                self.theme_idx = self.themes_list.iter().position(|c| match c {
                    Theme::Light => self.config.app_theme == "COSMIC Light",
                    Theme::Dark => self.config.app_theme == "COSMIC Dark",
                    Theme::Custom(theme) => self.config.app_theme == theme.0,
                });

                if self.theme_idx.is_none() {
                    self.theme_idx = Some(0);
                }

                return task::message(cosmic::action::app(Message::UpdateTheme(Box::new(
                    self.themes_list[self.theme_idx.unwrap_or_default()].clone(),
                ))));
            }
            Message::OpenFileResult(file_path) => {
                if !file_path.is_empty() {
                    self.dialogs = None;

                    return Task::future(async move {
                        let webapp_icon = WebappIcon::build_from_path(&file_path);

                        cosmic::Action::App(Message::SetIcon(Some(webapp_icon)))
                    });
                }
            }
            Message::OpenIconPicker => {
                self.dialogs = Some(Dialogs::IconPicker(IconPicker::default()));
            }
            Message::OpenThemeResult(theme) => {
                if !theme.is_empty() {
                    let from_path = Path::new(&theme);
                    if let Some(file_name) = from_path.file_name() {
                        let file_name = file_name.to_string_lossy();

                        if let Some(dest) = webapps::themes_path(&file_name) {
                            if !dest.exists() {
                                let _ = std::fs::copy(from_path, dest);
                            }
                        }
                    }
                }

                tasks.push(task::message(Message::LoadThemes));
            }
            Message::PushIcon(icon) => {
                if let Some(Dialogs::IconPicker(icon_picker)) = &mut self.dialogs {
                    icon_picker.push_icon(Some(icon));
                }
            }
            Message::ReloadNavbarItems => {
                self.nav.clear();

                self.nav
                    .insert()
                    .icon(widget::icon::from_name("list-add-symbolic"))
                    .text(fl!("new-app"))
                    .data::<Page>(Page::Editor(AppEditor::default()))
                    .activate();

                webapps::launcher::installed_webapps()
                    .into_iter()
                    .for_each(|app| {
                        let Some(editor) = editor::AppEditor::from_launcher(&app) else {
                            return;
                        };

                        self.nav
                            .insert()
                            .icon(navbar_item_icon(&app.webapp_icon))
                            .text(app.webapp_name.clone())
                            .data::<Page>(Page::Editor(editor))
                            .closable();
                    });

                self.page = Page::Editor(AppEditor::default());
            }
            Message::ResetSettings => {
                if let Some(handler) = AppConfig::config_handler() {
                    let _ = self.config.set_app_theme(&handler, String::new());
                };

                self.theme_idx = Some(0);
                return cosmic::command::set_theme(cosmic::theme::system_dark());
            }
            Message::SaveLauncher => {
                return task::message(Message::ReloadNavbarItems);
            }
            Message::SetIcon(webapp_icon) => {
                let Page::Editor(app_editor) = &mut self.page;
                app_editor.app_icon = webapp_icon;

                if self.dialogs.is_some() && app_editor.app_icon.is_some() {
                    self.dialogs = None;
                }
            }
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::UpdateTheme(theme) => {
                if let Theme::Custom(theme) = *theme {
                    if let Some(handler) = AppConfig::config_handler() {
                        let _ = self.config.set_app_theme(&handler, theme.0);
                    };
                    return set_theme(cosmic::Theme::custom(Arc::new(*theme.1)));
                };

                let theme_selector = match *theme {
                    Theme::Light => {
                        if let Some(handler) = AppConfig::config_handler() {
                            let _ = self.config.set_app_theme(&handler, "COSMIC Light".into());
                        };
                        set_theme(cosmic::theme::system_light())
                    }
                    Theme::Dark => {
                        if let Some(handler) = AppConfig::config_handler() {
                            let _ = self.config.set_app_theme(&handler, "COSMIC Dark".into());
                        };
                        set_theme(cosmic::theme::system_dark())
                    }
                    _ => Task::none(),
                };

                tasks.push(theme_selector);
            }
        };

        Task::batch(tasks)
    }

    fn header_start(&self) -> Vec<Element<'_, Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            RcElementWrapper::new(
                button::icon(icon::from_name("open-menu-symbolic"))
                    .padding([4, 12])
                    .class(theme::Button::MenuRoot)
                    .into(),
            ),
            menu::items(
                &self.key_binds,
                vec![
                    menu::Item::Button(fl!("menu-settings"), None, MenuAction::Settings),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("menu-about"), None, MenuAction::About),
                ],
            ),
        )])
        .item_height(ItemHeight::Dynamic(40))
        .item_width(ItemWidth::Uniform(320))
        .spacing(4.0);

        vec![menu_bar.into()]
    }

    fn nav_bar(&self) -> Option<Element<'_, cosmic::Action<Message>>> {
        if !self.core().nav_bar_active() {
            return None;
        }

        let nav_model = self.nav_model()?;

        let mut nav = widget::nav_bar(nav_model, |id| {
            cosmic::Action::Cosmic(cosmic::app::Action::NavBar(id))
        })
        .on_close(|id| cosmic::action::app(Message::ConfirmDeletion(id)))
        .into_container()
        .width(Length::Shrink)
        .height(Length::Shrink);

        if !self.core().is_condensed() {
            nav = nav.max_width(280);
        }

        Some(Element::from(
            nav.width(Length::Shrink).height(Length::Fill),
        ))
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Message> {
        self.nav.activate(id);
        if let Some(page) = self.nav.data::<Page>(id) {
            self.page = page.clone()
        }
        Task::none()
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            ),
            ContextPage::Settings => context_drawer::context_drawer(
                self.settings(),
                Message::ToggleContextPage(ContextPage::Settings),
            )
            .title(fl!("settings")),
        })
    }

    fn on_escape(&mut self) -> Task<Message> {
        self.dialogs = None;
        self.core.window.show_context = false;

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let Page::Editor(content) = &self.page;

        widget::container(content.view().map(Message::Editor))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .center_x(Length::Fill)
            .into()
    }

    fn dialog(&self) -> Option<Element<'_, Message>> {
        if let Some(dialog) = &self.dialogs {
            let element = match dialog {
                Dialogs::IconPicker(icon_picker) => widget::dialog()
                    .primary_action(
                        widget::button::standard(fl!("close")).on_press(Message::CloseDialog),
                    )
                    .control(icon_picker.view().map(Message::IconPicker)),
                Dialogs::Confirmation((entity, title)) => widget::dialog()
                    .title(fl!("delete"))
                    .primary_action(
                        widget::button::destructive(fl!("yes"))
                            .on_press(Message::Delete(entity.to_owned())),
                    )
                    .secondary_action(
                        widget::button::suggested(fl!("no")).on_press(Message::CloseDialog),
                    )
                    .body(fl!(
                        "confirm-delete",
                        HashMap::from([("app", title.as_str())])
                    )),
                Dialogs::IconsDownloader => widget::dialog()
                    .title(fl!("icons-installer-header"))
                    .body(self.downloader_output.clone())
                    .primary_action(
                        widget::button::destructive(fl!("cancel"))
                            .on_press(Message::DownloaderStop),
                    )
                    .secondary_action(
                        widget::button::suggested(fl!("close")).on_press(Message::CloseDialog),
                    ),
            };

            return Some(element.into());
        };

        None
    }
}

impl QuickWebApps {
    fn settings(&self) -> Element<'_, Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        widget::Column::new()
            .push(
                widget::settings::section()
                    .add(widget::settings::item(
                        fl!("import-theme"),
                        widget::button::standard(fl!("open"))
                            .on_press(Message::ImportThemeFilePicker),
                    ))
                    .add(widget::settings::item(
                        fl!("imported-themes"),
                        widget::dropdown(
                            &self.themes_list,
                            self.theme_idx,
                            Message::ChangeUserTheme,
                        ),
                    ))
                    .add(widget::settings::item(
                        fl!("reset-settings"),
                        widget::button::standard(fl!("reset")).on_press(Message::ResetSettings),
                    )),
            )
            .align_x(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }
}

fn navbar_item_icon(icon: &Option<WebappIcon>) -> widget::icon::Icon {
    let Some(icon) = icon else {
        return widget::icon::from_raster_bytes(APP_ICON).icon();
    };

    match icon.icon {
        webapps::IconType::Raster => widget::icon::from_raster_bytes(icon.buffer.clone()).icon(),
        webapps::IconType::Svg => widget::icon::from_svg_bytes(icon.buffer.clone()).icon(),
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    Settings,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    Settings,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::Settings => Message::ToggleContextPage(ContextPage::Settings),
        }
    }
}
