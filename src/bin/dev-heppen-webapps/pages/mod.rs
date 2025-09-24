pub mod editor;
mod iconpicker;

use crate::{config::AppConfig, pages::iconpicker::IconPicker, themes::Theme};
use ashpd::desktop::file_chooser::{FileFilter, SelectedFiles};
use cosmic::{
    app::{context_drawer, Core, Task},
    command::set_theme,
    cosmic_theme,
    iced::{
        alignment::Horizontal,
        futures::{future, SinkExt as _},
        Alignment, Length, Subscription,
    },
    surface, task, theme,
    widget::{
        self,
        menu::{self, ItemHeight, ItemWidth},
        nav_bar, responsive_menu_bar,
    },
    Application, Element,
};
use editor::AppEditor;
use ron::ser::to_string_pretty;
use std::{
    collections::HashMap,
    fs::read_dir,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::ExitStatus,
    str::FromStr as _,
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::oneshot,
};
use tracing::debug;
use webapps::{fl, APP_ICON, APP_ID, REPOSITORY};

static MENU_ID: LazyLock<cosmic::widget::Id> =
    LazyLock::new(|| cosmic::widget::Id::new("responsive-menu"));

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
    Launch(webapps::WebviewArgs),
    LaunchUrl(String),
    LoadThemes,
    OpenFileResult(Vec<String>),
    OpenIconPicker,
    OpenRepositoryUrl,
    OpenThemeResult(String),
    ConfirmDeletion(widget::segmented_button::Entity),
    PushIcon(webapps::Icon),
    ReloadNavbarItems,
    ResetSettings,
    SaveLauncher(webapps::launcher::WebAppLauncher),
    SetIcon(Option<webapps::Icon>),
    Surface(surface::Action),
    DownloaderStop,
    ToggleContextPage(ContextPage),
    UpdateConfig(AppConfig),
    UpdateTheme(Box<Theme>),
    // emty message
    None,
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

        let windows = QuickWebApps {
            core,
            context_page: ContextPage::About,
            nav,
            key_binds: HashMap::new(),
            config,
            page: add_page,
            dialogs: None,
            downloader_started: false,
            downloader_id: 1,
            downloader_output: String::new(),
            themes_list,
            theme_idx: Some(0),
        };

        let tasks = vec![
            task::message(Message::ReloadNavbarItems),
            task::message(Message::LoadThemes),
            task::message(Message::UpdateTheme(Box::new(Theme::Light))),
        ];

        (windows, Task::batch(tasks))
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = Vec::new();

        subscriptions.push(
            self.core()
                .watch_config::<AppConfig>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        );

        if self.downloader_started {
            subscriptions.push(Subscription::run_with_id(
                self.downloader_id,
                cosmic::iced::stream::channel(4, move |mut channel| async move {
                    let script = webapps::add_icon_packs_install_script().await;
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
                }),
            ));
        }

        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Message) -> cosmic::Task<cosmic::Action<Message>> {
        let mut tasks: Vec<cosmic::Task<cosmic::Action<Message>>> = Vec::new();

        match message {
            Message::ChangeUserTheme(idx) => {
                self.theme_idx = Some(idx);
                let selected = self.themes_list[idx].clone();

                tasks.push(task::message(cosmic::action::app(Message::UpdateTheme(
                    Box::new(selected),
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

                    if let Some(browser) = &app_editor.app_browser {
                        let launcher = webapps::launcher::WebAppLauncher {
                            browser: browser.clone(),
                            name: app_editor.app_title.clone(),
                            icon: app_editor.app_icon.clone(),
                            category: app_editor.app_category.clone(),
                        };

                        return task::future(async move {
                            launcher.delete().await.unwrap();
                            cosmic::action::app(Message::DeletionDone(id))
                        });
                    }
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
                        tasks.push(Task::perform(webapps::image_handle(path), |icon| {
                            if let Some(icon) = icon {
                                cosmic::Action::App(Message::PushIcon(icon))
                            } else {
                                cosmic::Action::None
                            }
                        }))
                    }
                };
            }
            Message::ImportThemeFilePicker => {
                return task::future(async {
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
                            .map(|file| file.path().to_string())
                            .collect::<Vec<String>>();

                        if !files.is_empty() {
                            return cosmic::action::app(Message::OpenThemeResult(
                                urlencoding::decode(&files[0])
                                    .unwrap_or_default()
                                    .to_string(),
                            ));
                        }
                        cosmic::action::none()
                    } else {
                        cosmic::action::none()
                    }
                })
            }
            Message::Launch(args) => {
                return Task::perform(
                    async move {
                        Command::new("dev.heppen.webapps.webview")
                            .args(args)
                            .spawn()
                            .expect("Failed to spawn webview");
                    },
                    |_| cosmic::Action::App(Message::Close),
                );
            }
            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
            Message::LoadThemes => {
                self.themes_list.clear();

                if std::env::var("XDG_CURRENT_DESKTOP") != Ok("COSMIC".to_string()) {
                    self.themes_list.push(Theme::Light);
                    self.themes_list.push(Theme::Dark);
                }

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

                                if theme_name == self.config.app_theme {
                                    tasks.push(task::message(cosmic::action::app(
                                        Message::UpdateTheme(Box::new(theme.clone())),
                                    )));
                                }

                                self.themes_list.push(theme);
                            }
                        }
                    }
                }

                self.theme_idx = self.themes_list.iter().position(|c| match c {
                    Theme::Light => self.config.app_theme == "COSMIC Light",
                    Theme::Dark => self.config.app_theme == "COSMIC Dark",
                    Theme::Custom(theme) => self.config.app_theme == theme.0,
                })
            }
            Message::OpenFileResult(file_paths) => {
                let mut moved: Vec<String> = Vec::new();

                for path in file_paths {
                    let Ok(buf) = PathBuf::from_str(&path);
                    let icon_name = buf.file_stem();

                    if let Some(file_stem) = icon_name {
                        if let Some(final_path) = webapps::move_icon(
                            &path,
                            file_stem.to_str().unwrap(),
                            buf.extension().unwrap_or_default().to_str().unwrap(),
                        ) {
                            moved.push(final_path.display().to_string());
                        }
                    };
                }

                return task::message(Message::IconsResult(moved));
            }
            Message::OpenIconPicker => {
                self.dialogs = Some(Dialogs::IconPicker(IconPicker::default()));
            }
            Message::OpenRepositoryUrl => {
                _ = open::that_detached(REPOSITORY);
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
                    icon_picker.push_icon(icon);
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
                        self.nav
                            .insert()
                            .icon(widget::icon::from_name(app.icon.clone()))
                            .text(app.name.clone())
                            .data::<Page>(Page::Editor(editor::AppEditor::from(app)))
                            .closable();
                    });

                self.page = Page::Editor(AppEditor::default());
            }
            Message::ResetSettings => {
                if let Some(handler) = AppConfig::config_handler() {
                    let _ = self.config.set_app_theme(&handler, String::new());
                };

                return cosmic::command::set_theme(cosmic::Theme::light());
            }
            Message::SaveLauncher(launcher) => {
                if let Some(location) =
                    webapps::database_path(&format!("{}.ron", launcher.browser.app_id.as_ref()))
                {
                    let content = to_string_pretty(&launcher, ron::ser::PrettyConfig::default());

                    if let Ok(content) = content {
                        let file = std::fs::File::create(location);

                        if let Ok(mut f) = file {
                            let _ = f.write_all(content.as_bytes());
                        }
                    }

                    return task::message(Message::ReloadNavbarItems);
                }
            }
            Message::SetIcon(icon) => {
                let Page::Editor(app_editor) = &mut self.page;
                app_editor.update_icon(icon);
                self.dialogs = None;
            }
            Message::Surface(a) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(a),
                ));
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
                        set_theme(cosmic::theme::Theme::light())
                    }
                    Theme::Dark => {
                        if let Some(handler) = AppConfig::config_handler() {
                            let _ = self.config.set_app_theme(&handler, "COSMIC Dark".into());
                        };
                        set_theme(cosmic::theme::Theme::dark())
                    }
                    _ => Task::none(),
                };

                if std::env::var("XDG_CURRENT_DESKTOP") != Ok("COSMIC".to_string()) {
                    tasks.push(theme_selector);
                }
            }
            Message::None => (),
        };

        Task::batch(tasks)
    }

    fn header_start(&self) -> Vec<Element<'_, Message>> {
        vec![responsive_menu_bar()
            .item_height(ItemHeight::Dynamic(40))
            .item_width(ItemWidth::Uniform(240))
            .spacing(4.0)
            .into_element(
                &self.core,
                &self.key_binds,
                MENU_ID.clone(),
                Message::Surface,
                vec![(
                    fl!("help"),
                    vec![
                        menu::Item::Button(fl!("settings"), None, MenuAction::Settings),
                        menu::Item::Button(fl!("about"), None, MenuAction::About),
                    ],
                )],
            )]
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
            nav.width(Length::Shrink).height(Length::Shrink),
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
            ContextPage::About => context_drawer::context_drawer(
                self.about(),
                Message::ToggleContextPage(ContextPage::About),
            )
            .title(fl!("about")),
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
    fn about(&self) -> Element<'_, Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        widget::column()
            .push(widget::image(widget::image::Handle::from_bytes(APP_ICON)))
            .push(widget::text::title3(fl!("app")))
            .push(
                widget::button::link(REPOSITORY)
                    .on_press(Message::OpenRepositoryUrl)
                    .padding(0),
            )
            .push(
                widget::column()
                    .push(widget::text::title3(fl!("support-me")))
                    .push(widget::text::body(fl!("support-body")))
                    .push(widget::button::link("github.com/sponsors/hepp3n").on_press(
                        Message::LaunchUrl("https://github.com/sponsors/hepp3n".to_string()),
                    ))
                    .push(widget::button::link("paypal.me/elevenhsoft").on_press(
                        Message::LaunchUrl("https://paypal.me/elevenhsoft".to_string()),
                    ))
                    .push(widget::button::link("ko-fi.com/elevenhsoft").on_press(
                        Message::LaunchUrl("https://ko-fi.com/elevenhsoft".to_string()),
                    ))
                    .align_x(Alignment::Center)
                    .spacing(space_xxs),
            )
            .align_x(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    fn settings(&self) -> Element<'_, Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        widget::column()
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
