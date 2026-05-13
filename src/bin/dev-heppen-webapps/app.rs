// SPDX-License-Identifier: {{ license }}

use cosmic::cosmic_config::ConfigSet;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Length, Subscription};
use cosmic::widget::segmented_button::Entity;
use cosmic::widget::{self, nav_bar};
use cosmic::{Action, prelude::*, style};
use rand::RngExt as _;
use tokio::process::Command;
use webapps::{AppConfig, browser, cef_path, fl, launcher};

use crate::config::{self, Config};

#[derive(Debug, Clone)]
pub enum Page {
    Editor(AppConfig),
}

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Contains items assigned to the nav bar panel.
    nav: nav_bar::Model,
    /// Configuration data that persists between application runs.
    config: Config,
    page: Page,
    /// AppConfiguration
    categories: Vec<String>,
    category_idx: usize,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    App(AppConfigMessage),
    Created((String, AppConfig)),
    Deleted((String, AppConfig)),
    ReloadNavbar,
    SetIcon(Option<launcher::DesktopIcon>),
    UpdateConfig(Config),
}

// AppConfigMessages
#[derive(Debug, Clone)]
pub enum AppConfigMessage {
    AppPersistent(bool),
    AppSimulateMobile(bool),
    Category(usize),
    ConfirmDeletion(Entity),
    CreateLauncher,
    GenerateIcon,
    LaunchApp,
    Title(String),
    Url(String),
    WindowWidth(String),
    WindowHeight(String),
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "dev.heppen.webapps";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let nav = nav_bar::Model::default();

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            nav,
            // Optional configuration file for an application.
            config: config::Config::config(),
            page: Page::Editor(AppConfig::default()),

            categories: webapps::Category::to_vec(),
            category_idx: 12,
        };

        // Create a startup command that sets the window title.
        let update_title = app.update_title();
        let navbar = Task::done(Action::App(Message::ReloadNavbar));

        (app, Task::batch(vec![update_title, navbar]))
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Action<Message>> {
        self.nav.activate(id);

        if let Some(page) = self.nav.data::<Page>(id) {
            self.page = page.clone()
        }

        Task::none()
    }

    fn nav_bar(&self) -> Option<Element<'_, cosmic::Action<Self::Message>>> {
        if !self.core().nav_bar_active() {
            return None;
        }

        let nav_model = self.nav_model()?;

        let mut nav = widget::nav_bar(nav_model, |id| {
            cosmic::Action::Cosmic(cosmic::app::Action::NavBar(id))
        })
        .on_close(|id| cosmic::action::app(Message::App(AppConfigMessage::ConfirmDeletion(id))))
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

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<'_, Self::Message> {
        self.content()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They can be dynamically
    /// stopped and started conditionally based on application state, or persist
    /// indefinitely.
    fn subscription(&self) -> Subscription<Self::Message> {
        // Add subscriptions which are always active.
        let subscriptions = vec![
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ];

        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::App(msg) => match msg {
                AppConfigMessage::AppPersistent(flag) => {
                    let Page::Editor(editor) = &mut self.page;
                    editor.persistent_profile = flag;
                }
                AppConfigMessage::AppSimulateMobile(flag) => {
                    let Page::Editor(editor) = &mut self.page;
                    editor.simulate_mobile = flag;
                }
                AppConfigMessage::Category(idx) => {
                    let Page::Editor(editor) = &mut self.page;
                    editor.category = webapps::Category::from_index(idx as u8);
                    self.category_idx = idx;
                }
                AppConfigMessage::ConfirmDeletion(entity) => {
                    let data = self.nav.data::<Page>(entity);

                    if let Some(page) = data {
                        let Page::Editor(app) = page;
                        let app = app.clone();

                        return Task::future(async move {
                            if let Ok((id, config)) = launcher::uninstall(app).await {
                                return Action::App(Message::Deleted((id, config)));
                            }

                            Action::None
                        });
                    };
                }
                AppConfigMessage::CreateLauncher => {
                    let Page::Editor(ref mut app) = self.page;

                    let app_id = app.title.replace(' ', "");
                    let app_id = format!(
                        "{}{}",
                        app_id,
                        &rand::rng().random_range(1000..10000).to_string()
                    );

                    if !app.is_installed {
                        app.id = app_id;
                    }

                    let id = app.id.clone();
                    let app = app.clone();

                    return Task::future(async move {
                        if let Ok((id, config)) =
                            launcher::create(browser::Browser::new(id.clone()), app).await
                        {
                            return Action::App(Message::Created((id, config)));
                        }

                        Action::None
                    });
                }
                AppConfigMessage::GenerateIcon => {
                    let Page::Editor(ref app) = self.page;
                    let first_letter = &app.title.split_at(1).0;
                    let first_letter = first_letter.to_string();

                    return Task::future(async move {
                        let desktop_icon = webapps::generate_icon(first_letter).await;
                        Action::App(Message::SetIcon(desktop_icon))
                    });
                }
                AppConfigMessage::LaunchApp => {
                    let Page::Editor(editor) = &mut self.page;
                    let id = editor.id.clone();

                    return Task::future(async move {
                        let Some(browser) = browser::Browser::from_appid(&id) else {
                            return Action::None;
                        };

                        let browser = browser.app_id;

                        if let Some(lib) = cef_path() {
                            let mut cmd = Command::new(webapps::WEBVIEW_ID);
                            cmd.env("LD_LIBRARY_PATH", lib);
                            cmd.arg(&browser.id);
                            cmd.spawn().ok();
                        }

                        Action::None
                    });
                }
                AppConfigMessage::Title(title) => {
                    let Page::Editor(editor) = &mut self.page;
                    editor.title = title;
                }
                AppConfigMessage::Url(url) => {
                    let Page::Editor(editor) = &mut self.page;
                    editor.url = url;
                }
                AppConfigMessage::WindowWidth(width) => {
                    let Page::Editor(editor) = &mut self.page;
                    editor.window_width = width;
                }
                AppConfigMessage::WindowHeight(height) => {
                    let Page::Editor(editor) = &mut self.page;
                    editor.window_height = height;
                }
            },
            Message::Created((id, mut cfg)) => {
                if let Some(handler) = Config::config_handler() {
                    let mut apps = self.config.apps.clone();

                    if !cfg.is_installed {
                        cfg.is_installed = true;
                    }

                    apps.insert(id, cfg.to_owned());

                    let _ = handler.set("apps", apps);
                    tracing::info!("{} saved\n{:#?}", cfg.title, cfg);

                    return Task::done(Action::App(Message::ReloadNavbar));
                }
            }
            Message::Deleted((id, cfg)) => {
                if let Some(handler) = Config::config_handler() {
                    let mut apps = self.config.apps.clone();

                    if cfg.is_installed {
                        apps.remove_entry(&id);

                        let _ = handler.set("apps", apps);
                        tracing::info!("{} removed.", cfg.title);
                        return Task::done(Action::App(Message::ReloadNavbar));
                    }
                };
            }
            Message::ReloadNavbar => {
                self.nav.clear();

                self.nav
                    .insert()
                    .icon(widget::icon::from_name("list-add-symbolic"))
                    .text(fl!("new-app"))
                    .data::<Page>(Page::Editor(AppConfig::default()))
                    .activate();

                for (_k, v) in &self.config.apps {
                    self.nav
                        .insert()
                        .icon(navbar_item_icon(
                            &v.icon.as_ref().and_then(|ico| Some(ico.buffer.clone())),
                        ))
                        .text(v.title.clone())
                        .data::<Page>(Page::Editor(AppConfig::from(v.clone())))
                        .closable();
                }

                self.page = Page::Editor(AppConfig::default());
            }
            Message::SetIcon(desktop_icon) => {
                if let Some(icon) = desktop_icon {
                    let icon = icon.to_icon();

                    let Page::Editor(ref mut app) = self.page;
                    app.icon = Some(icon);
                }
            }
            Message::UpdateConfig(config) => {
                self.config = config;
            }
        }

        Task::none()
    }
}

impl AppModel {
    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app");

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }

    /// Crate element from webapps::Icon
    fn icon_element(&self, icon: Option<webapps::Icon>) -> Element<'_, Message> {
        let ico = if let Some(ico) = icon {
            match ico.icon {
                webapps::IconType::Raster => {
                    let data = widget::image::Handle::from_bytes(ico.buffer);
                    widget::button::custom(widget::image(data))
                        .width(Length::Fixed(92.0))
                        .height(Length::Fixed(92.0))
                        .class(style::Button::Icon)
                }

                webapps::IconType::Vector => {
                    let data = widget::svg::Handle::from_memory(ico.buffer);
                    widget::button::custom(widget::svg(data))
                        .width(Length::Fixed(92.0))
                        .height(Length::Fixed(92.0))
                        .class(style::Button::Icon)
                }
            }
        } else {
            widget::button::custom(
                widget::icon::from_raster_bytes(webapps::APP_ICON)
                    .icon()
                    .size(92),
            )
            .class(style::Button::Icon)
        };

        Element::from(ico)
    }

    /// Generate editor's content
    fn content(&self) -> Element<'_, Message> {
        let space_s = cosmic::theme::spacing().space_s;
        let Page::Editor(ref app) = self.page;

        let header_banner = widget::container(
            widget::column().spacing(space_s).push(
                widget::row()
                    .spacing(space_s)
                    .push(
                        widget::container(self.icon_element(app.icon.clone()))
                            .width(96.)
                            .height(96.)
                            .align_y(Vertical::Center),
                    )
                    .push(
                        widget::container(
                            widget::column()
                                .spacing(space_s)
                                .push(widget::text::title3(format!(
                                    "{}: {}",
                                    fl!("title"),
                                    if app.title.is_empty() {
                                        fl!("new-webapp-title")
                                    } else {
                                        app.title.clone()
                                    }
                                )))
                                .push(widget::text::title4(format!(
                                    "{}: {}",
                                    fl!("category"),
                                    app.category.name()
                                ))),
                        )
                        .height(Length::Fixed(96.))
                        .align_y(Vertical::Center),
                    ),
            ),
        )
        .padding(space_s)
        .width(Length::Fill)
        .class(style::Container::Card);

        let app_config = widget::column()
            .spacing(space_s)
            .push(
                widget::row()
                    .spacing(space_s)
                    .push(
                        widget::text_input(fl!("title"), app.title.clone())
                            .on_input(|s| Message::App(AppConfigMessage::Title(s))),
                    )
                    .push(
                        widget::button::standard(fl!("generate-icon")).on_press_maybe(
                            if app.title.len() > 1 {
                                Some(Message::App(AppConfigMessage::GenerateIcon))
                            } else {
                                None
                            },
                        ),
                    ),
            )
            .push(
                widget::text_input(fl!("url"), app.url.clone())
                    .on_input(|s| Message::App(AppConfigMessage::Url(s))),
            )
            .push(
                widget::settings::section()
                    .add(widget::settings::item(
                        fl!("select-category"),
                        widget::dropdown(
                            &self.categories,
                            self.categories
                                .iter()
                                .position(|c| c == app.category.as_ref()),
                            |c| Message::App(AppConfigMessage::Category(c)),
                        ),
                    ))
                    .add(widget::settings::item(
                        fl!("window-size"),
                        widget::row()
                            .spacing(space_s)
                            .push(
                                widget::text_input(
                                    format!("{}", webapps::DEFAULT_WINDOW_WIDTH),
                                    app.window_width.clone(),
                                )
                                .on_input(|w| Message::App(AppConfigMessage::WindowWidth(w))),
                            )
                            .push(
                                widget::text_input(
                                    format!("{}", webapps::DEFAULT_WINDOW_HEIGHT),
                                    app.window_height.clone(),
                                )
                                .on_input(|h| Message::App(AppConfigMessage::WindowHeight(h))),
                            ),
                    ))
                    .add(widget::settings::item(
                        fl!("persistent-profile"),
                        widget::toggler(app.persistent_profile)
                            .on_toggle(|f| Message::App(AppConfigMessage::AppPersistent(f))),
                    ))
                    .add(widget::settings::item(
                        fl!("simulate-mobile"),
                        widget::toggler(app.simulate_mobile)
                            .on_toggle(|f| Message::App(AppConfigMessage::AppSimulateMobile(f))),
                    )),
            )
            .push(
                widget::row()
                    .spacing(space_s)
                    .push(widget::horizontal_space())
                    .push_maybe(if !app.is_installed {
                        None
                    } else {
                        Some(
                            widget::button::standard(fl!("run-app"))
                                .on_press(Message::App(AppConfigMessage::LaunchApp)),
                        )
                    })
                    .push(widget::button::suggested(fl!("create")).on_press_maybe(
                        if webapps::launcher_is_valid(&app.title, Some(app.url.as_str())) {
                            Some(Message::App(AppConfigMessage::CreateLauncher))
                        } else {
                            None
                        },
                    )),
            );

        widget::scrollable(
            widget::column()
                .push(header_banner)
                .push(app_config)
                .spacing(space_s),
        )
        .width(600)
        .height(Length::Fill)
        .apply(widget::container)
        .width(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into()
    }
}

fn navbar_item_icon(icon: &Option<Vec<u8>>) -> widget::icon::Icon {
    if let Some(buf) = icon {
        return widget::icon::from_svg_bytes(buf.clone()).icon();
    }
    return widget::icon::from_raster_bytes(webapps::APP_ICON).icon();
}
