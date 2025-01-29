pub mod editor;
mod iconpicker;

use crate::common::{find_icon, image_handle, move_icon, qwa_icons_location, Icon};
use crate::config::Config;
use crate::launcher::{installed_webapps, WebAppLauncher};
use crate::{add_icon_packs_install_script, execute_script};
use crate::{fl, pages::iconpicker::IconPicker};
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::Horizontal;
use cosmic::iced::window::Id;
use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::widget::{menu, nav_bar};
use cosmic::{
    app::{command::Task, Core},
    cosmic_theme,
    widget::{self},
    Application, ApplicationExt, Element,
};
use cosmic::{task, theme};
use editor::AppEditor;
use futures_util::SinkExt;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::str::FromStr;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::oneshot;

#[derive(Debug, Clone)]
pub enum Message {
    CloseDialog,
    Editor(editor::Message),
    Delete(widget::segmented_button::Entity),
    DownloaderDone,
    DownloaderStarted,
    DownloaderStream(String),
    DownloaderStreamFinished,
    IconPicker(iconpicker::Message),
    IconsResult(Vec<String>),
    LaunchUrl(String),
    NavBar(widget::segmented_button::Entity),
    OpenFileResult(Vec<String>),
    OpenIconPicker(String),
    OpenRepositoryUrl,
    ConfirmDeletion(widget::segmented_button::Entity),
    ReloadNavbarItems,
    SetIcon(Option<Icon>),
    DownloaderStop,
    ToggleContextPage(ContextPage),
    UpdateConfig(Config),
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
    window_id: Id,
    context_page: ContextPage,
    nav: nav_bar::Model,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    config: Config,
    page: Page,
    dialogs: Option<Dialogs>,
    downloader_started: bool,
    downloader_id: usize,
    downloader_output: String,
}

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] =
    include_bytes!("../../res/icons/hicolor/256x256/apps/io.github.elevenhsoft.WebApps.png");

impl Application for QuickWebApps {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "io.github.hepp3n.WebApps";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Message>) {
        let window_id = if let Some(id) = core.main_window_id() {
            id
        } else {
            Id::unique()
        };

        let add_page = Page::Editor(AppEditor::new());
        let nav = nav_bar::Model::default();

        let mut windows = QuickWebApps {
            window_id,
            core,
            context_page: ContextPage::About,
            nav,
            key_binds: HashMap::new(),
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((errors, config)) => {
                        tracing::error!("error loading app config: {:#?}", errors);
                        config
                    }
                })
                .unwrap_or_default(),
            page: add_page,
            dialogs: None,
            downloader_started: false,
            downloader_id: 1,
            downloader_output: String::new(),
        };

        let tasks = vec![
            windows.update_title(),
            task::message(Message::ReloadNavbarItems),
        ];

        (windows, task::batch(tasks))
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = Vec::new();

        subscriptions.push(
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        );

        if self.downloader_started {
            subscriptions.push(Subscription::run_with_id(
                self.downloader_id,
                cosmic::iced::stream::channel(4, move |mut channel| async move {
                    let script = add_icon_packs_install_script().await;
                    let mut child = execute_script(script).await;
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

                    futures_util::future::pending().await
                }),
            ));
        }

        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CloseDialog => self.dialogs = None,
            Message::ConfirmDeletion(id) => {
                let data = self.nav.data::<Page>(id);

                if let Some(page) = data {
                    let Page::Editor(app_editor) = page;
                    self.dialogs = Some(Dialogs::Confirmation((id, app_editor.app_title.clone())))
                };
            }
            Message::Editor(msg) => match &mut self.page {
                Page::Editor(app_editor) => {
                    return app_editor.update(msg).map(cosmic::app::message::app)
                }
            },
            Message::Delete(id) => {
                let data = self.nav.data::<Page>(id);

                if let Some(page) = data {
                    let Page::Editor(app_editor) = page;

                    if let Some(browser) = &app_editor.app_browser {
                        let launcher = WebAppLauncher {
                            codename: app_editor.app_codename.clone(),
                            browser: browser.clone(),
                            name: app_editor.app_title.clone(),
                            icon: app_editor.app_icon.clone(),
                            category: app_editor.app_category.clone(),
                            url: app_editor.app_url.clone(),
                            custom_parameters: app_editor.app_parameters.clone(),
                            isolate_profile: app_editor.app_isolated,
                            navbar: app_editor.app_navbar,
                            is_incognito: app_editor.app_incognito,
                        };

                        let deleted = launcher.delete();

                        if deleted.is_ok() {
                            self.nav.remove(id);
                            self.dialogs = None;
                            self.page = Page::Editor(AppEditor::new())
                        };
                    };
                }
            }
            Message::DownloaderDone => {
                self.downloader_started = false;
                return task::message(Message::CloseDialog);
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

                    Message::DownloaderDone
                });
            }
            Message::IconPicker(msg) => {
                if let Some(Dialogs::IconPicker(icon_picker)) = &mut self.dialogs {
                    return icon_picker.update(msg).map(cosmic::app::message::app);
                };
            }
            Message::IconsResult(result) => {
                if let Some(Dialogs::IconPicker(icon_picker)) = &mut self.dialogs {
                    for path in result {
                        if let Some(icon) = image_handle(path) {
                            icon_picker.push_icon(icon);
                        }
                    }
                };
            }
            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
            Message::NavBar(id) => {
                let data = self.nav.data::<Page>(id);

                if let Some(page) = data {
                    self.page = page.clone();
                    self.nav.activate(id);
                }
            }
            Message::OpenFileResult(file_paths) => {
                return task::future(async {
                    for path in file_paths {
                        let Ok(buf) = PathBuf::from_str(&path);
                        let icon_name = buf.file_stem();

                        if let Some(file_stem) = icon_name {
                            move_icon(&path, file_stem.to_str().unwrap());
                        };
                    }

                    Message::IconsResult(find_icon(qwa_icons_location(), String::new()).await)
                })
            }
            Message::OpenIconPicker(app_url) => {
                self.dialogs = Some(Dialogs::IconPicker(IconPicker::new(app_url)));
            }
            Message::OpenRepositoryUrl => {
                _ = open::that_detached(REPOSITORY);
            }
            Message::ReloadNavbarItems => {
                self.nav.clear();

                self.nav
                    .insert()
                    .icon(widget::icon::from_name("list-add-symbolic"))
                    .text(fl!("new-app"))
                    .data::<Page>(Page::Editor(AppEditor::new()))
                    .activate();

                installed_webapps().into_iter().for_each(|app| {
                    self.nav
                        .insert()
                        .icon(widget::icon::from_name(app.icon.clone()))
                        .text(app.name.clone())
                        .data::<Page>(Page::Editor(editor::AppEditor::from(app)))
                        .closable();
                });

                self.page = Page::Editor(AppEditor::new());
            }
            Message::SetIcon(icon) => {
                let Page::Editor(app_editor) = &mut self.page;
                app_editor.update_icon(icon);
                self.dialogs = None;
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
            Message::None => (),
        };

        Task::none()
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("help")),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    fn nav_bar(&self) -> Option<Element<cosmic::app::Message<Message>>> {
        if !self.core().nav_bar_active() {
            return None;
        }

        let nav_model = self.nav_model()?;

        let mut nav = widget::nav_bar(nav_model, |id| {
            cosmic::app::message::app(Message::NavBar(id))
        })
        .on_close(|id| cosmic::app::message::app(Message::ConfirmDeletion(id)))
        .into_container()
        .width(Length::Shrink)
        .height(Length::Shrink);

        if !self.core().is_condensed() {
            nav = nav.max_width(280);
        }

        Some(Element::from(nav))
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

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::context_drawer(
                self.about(),
                Message::ToggleContextPage(ContextPage::About),
            )
            .title(fl!("about")),
        })
    }

    fn on_escape(&mut self) -> Task<Message> {
        self.dialogs = None;
        self.core.window.show_context = false;

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let Page::Editor(content) = &self.page;

        widget::container(content.view().map(Message::Editor))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .center_x(Length::Fill)
            .into()
    }

    fn dialog(&self) -> Option<Element<Message>> {
        if let Some(dialog) = &self.dialogs {
            let element = match dialog {
                Dialogs::IconPicker(icon_picker) => widget::dialog()
                    .title(fl!("icon-selector"))
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
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let hash = env!("VERGEN_GIT_SHA");
        let _short_hash: String = hash.chars().take(7).collect();
        let _date = env!("VERGEN_GIT_COMMIT_DATE");

        widget::column()
            .push(widget::image(widget::image::Handle::from_bytes(APP_ICON)))
            .push(widget::text::title3(fl!("app")))
            .push(
                widget::button::link(REPOSITORY)
                    .on_press(Message::OpenRepositoryUrl)
                    .padding(0),
            )
            .push(
                widget::button::link(fl!(
                    "git-description",
                    hash = _short_hash.as_str(),
                    date = _date
                ))
                .on_press(Message::LaunchUrl(format!("{REPOSITORY}/commits/{hash}")))
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

    fn update_title(&mut self) -> Task<Message> {
        self.set_header_title(fl!("app"));
        self.set_window_title(fl!("app"), self.window_id)
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}
