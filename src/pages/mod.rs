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
use std::{collections::HashMap, path::Path, process::ExitStatus, sync::Arc, time::Duration};
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
    OpenIconPickerDialog,
    OpenThemeResult(String),
    ConfirmDeletion(widget::segmented_button::Entity),
    PushIcon(WebappIcon),
    ReloadNavbarItems,
    ResetSettings,
    SaveLauncher,
    SetIcon(Option<WebappIcon>),
    ShowWarningDialog(String),
    DownloaderStop,
    ToggleContextPage(ContextPage),
    UpdateConfig(AppConfig),
    UpdateTheme(usize),
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
    SandboxWarning(String),
    ThemeError(String),
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
                (fl!("support-me"), "https://github.com/sponsors/hepp3n"),
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
            theme_idx: Some(0),
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
                return task::message(cosmic::action::app(Message::UpdateTheme(idx)));
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
                    let filter = FileFilter::new("COSMIC theme (.ron)").glob("*.ron");

                    match pick_single_file("Import theme", vec![filter]).await {
                        Some(path) => cosmic::action::app(Message::OpenThemeResult(path)),
                        None => cosmic::action::none(),
                    }
                });
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
            Message::LoadThemes => {
                self.themes_list = vec![Theme::Default, Theme::Dark, Theme::Light];

                if let Some(folder) = webapps::themes_path("") {
                    self.themes_list.extend(
                        webapps::custom_theme_names_in(&folder)
                            .into_iter()
                            .map(Theme::Custom),
                    );
                }

                self.theme_idx = self
                    .themes_list
                    .iter()
                    .position(|t| t.to_string() == self.config.app_theme);

                if let Some(theme_idx) = self.theme_idx {
                    return Task::done(cosmic::action::app(Message::UpdateTheme(theme_idx)));
                }
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
            Message::OpenIconPickerDialog => {
                return task::future(async move {
                    let filters = vec![
                        FileFilter::new("PNG Image").glob("*.png"),
                        FileFilter::new("SVG Images").glob("*.svg"),
                    ];

                    match pick_single_file("Open icon", filters).await {
                        Some(path) => cosmic::action::app(Message::OpenFileResult(path)),
                        None => cosmic::action::none(),
                    }
                });
            }
            Message::OpenThemeResult(theme) => {
                let Some(folder) = webapps::themes_path("") else {
                    return Task::none();
                };

                match webapps::import_theme_into(Path::new(&theme), &folder) {
                    Ok(name) => {
                        // Select the theme that was just imported.
                        self.config.app_theme = name;
                        tasks.push(task::message(Message::LoadThemes));
                    }
                    Err(why) => {
                        tracing::error!("theme import failed: {why}");
                        self.dialogs = Some(Dialogs::ThemeError(why.to_string()));
                    }
                }
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
                    let _ = self.config.set_app_theme(&handler, "Default".to_string());
                };

                self.theme_idx = Some(0);
                return cosmic::command::set_theme(cosmic::theme::system_preference());
            }
            Message::SaveLauncher => {
                return task::message(Message::ReloadNavbarItems);
            }
            Message::SetIcon(webapp_icon) => {
                let Page::Editor(app_editor) = &mut self.page;

                let Some(webapp_icon) = webapp_icon else {
                    return Task::none();
                };

                app_editor.app_icon = Some(webapp_icon);

                if self.dialogs.is_some() && app_editor.app_icon.is_some() {
                    self.dialogs = None;
                }
            }
            Message::ShowWarningDialog(profile_path_str) => {
                self.dialogs = Some(Dialogs::SandboxWarning(profile_path_str));
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
            Message::UpdateTheme(idx) => {
                let Some(theme) = self.themes_list.get(idx).cloned() else {
                    return Task::none();
                };

                let cosmic_theme = match &theme {
                    // Follow the system theme (including custom/imported system themes)
                    // and its dark/light mode switches.
                    Theme::Default => cosmic::theme::system_preference(),
                    // Keep the system's dark/light theme even when the system mode flips.
                    // Without `prefer_dark` libcosmic treats these as "follow system".
                    Theme::Dark => {
                        let mut t = cosmic::theme::system_dark();
                        t.theme_type.prefer_dark(Some(true));
                        t
                    }
                    Theme::Light => {
                        let mut t = cosmic::theme::system_light();
                        t.theme_type.prefer_dark(Some(false));
                        t
                    }
                    Theme::Custom(name) => {
                        let loaded = webapps::themes_path("")
                            .ok_or_else(|| anyhow::anyhow!("no data directory"))
                            .and_then(|dir| webapps::load_custom_theme_in(&dir, name));

                        match loaded {
                            Ok(t) => cosmic::theme::Theme::custom(Arc::new(t)),
                            Err(why) => {
                                tracing::error!("cannot load theme {name}: {why}");
                                self.dialogs = Some(Dialogs::ThemeError(why.to_string()));
                                return Task::none();
                            }
                        }
                    }
                };

                self.theme_idx = Some(idx);

                if let Some(handler) = AppConfig::config_handler() {
                    let _ = self.config.set_app_theme(&handler, theme.to_string());
                }

                return set_theme(cosmic_theme);
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
                        widget::button::standard(fl!("open"))
                            .on_press(Message::OpenIconPickerDialog),
                    )
                    .secondary_action(
                        widget::button::destructive(fl!("close")).on_press(Message::CloseDialog),
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
                        widget::button::standard(fl!("cancel")).on_press(Message::DownloaderStop),
                    )
                    .secondary_action(
                        widget::button::destructive(fl!("close")).on_press(Message::CloseDialog),
                    ),
                Dialogs::ThemeError(why) => widget::dialog()
                    .title(fl!("theme-import-error"))
                    .body(why.clone())
                    .primary_action(
                        widget::button::suggested(fl!("close")).on_press(Message::CloseDialog),
                    ),
                Dialogs::SandboxWarning(path) => {
                    let path = std::path::PathBuf::from(path.clone());
                    let path = path.parent().unwrap();

                    widget::dialog()
                        .title(fl!("permissions-warning"))
                        .control(
                            widget::Column::new()
                                .spacing(12)
                                .push(widget::text(fl!(
                                    "permissions-body",
                                    HashMap::from([("path", path.display().to_string())])
                                )))
                                .push(widget::text(format!(
                                    "flatpak override --user --filesystem={} dev.heppen.webapps",
                                    path.display().to_string()
                                )))
                                .push(widget::text(format!(
                                    "flatpak override --filesystem={} dev.heppen.webapps",
                                    path.display().to_string()
                                ))),
                        )
                        .primary_action(
                            widget::button::suggested(fl!("close")).on_press(Message::CloseDialog),
                        )
                }
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

/// Opens the portal file chooser and returns the chosen local path, if any.
async fn pick_single_file(title: &str, filters: Vec<FileFilter>) -> Option<String> {
    let mut request = SelectedFiles::open_file()
        .title(title)
        .accept_label("Open")
        .modal(true)
        .multiple(false);

    for filter in filters {
        request = request.filter(filter);
    }

    let response = match request.send().await {
        Ok(request) => request.response(),
        Err(why) => {
            tracing::error!("file chooser portal failed: {why}");
            return None;
        }
    };

    let selected = response.ok()?;
    let uri = selected.uris().first()?;

    // `to_file_path` also decodes percent-escapes such as `%20`.
    url::Url::parse(uri.as_str())
        .ok()?
        .to_file_path()
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}
