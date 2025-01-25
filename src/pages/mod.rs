mod editor;
mod iconpicker;
mod icons_installator;

use crate::common::{find_icon, image_handle, move_icon, qwa_icons_location, Icon};
use crate::config::Config;
use crate::launcher::{installed_webapps, WebAppLauncher};
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
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Message {
    CloseIconPicker,
    Editor(editor::Message),
    IconPicker(iconpicker::Message),
    IconsResult(Vec<String>),
    InsertApp(WebAppLauncher),
    LaunchUrl(String),
    OpenFileResult(Vec<String>),
    OpenIconPicker(String),
    OpenRepositoryUrl,
    SetIcon(Option<Icon>),
    SubscriptionChannel,
    ToggleContextPage(ContextPage),
    UpdateConfig(Config),
    // emty message
    None,
}

#[derive(Debug, Clone)]
pub enum Page {
    Editor(AppEditor),
}

pub struct QuickWebApps {
    core: Core,
    window_id: Id,
    context_page: ContextPage,
    nav: nav_bar::Model,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    config: Config,
    page: Page,
    iconpicker: Option<IconPicker>,
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
        let mut nav = nav_bar::Model::default();

        nav.insert()
            .icon(widget::icon::from_name("list-add-symbolic"))
            .text(fl!("new-app"))
            .data::<Page>(add_page.clone())
            .activate();

        installed_webapps().into_iter().for_each(|app| {
            nav.insert()
                .icon(widget::icon::from_name("edit-symbolic"))
                .text(app.name.clone())
                .data::<Page>(Page::Editor(editor::AppEditor::from(app)));
        });

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
            iconpicker: None,
        };

        let command = windows.update_title();

        (windows, command)
    }

    fn subscription(&self) -> Subscription<Message> {
        struct MySubscription;

        Subscription::batch(vec![
            Subscription::run_with_id(
                std::any::TypeId::of::<MySubscription>(),
                cosmic::iced::stream::channel(4, move |mut channel| async move {
                    _ = channel.send(Message::SubscriptionChannel).await;

                    futures_util::future::pending().await
                }),
            ),
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        ])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CloseIconPicker => {
                self.iconpicker = None;
            }
            Message::Editor(msg) => match &mut self.page {
                Page::Editor(app_editor) => {
                    return app_editor.update(msg).map(cosmic::app::message::app)
                }
            },
            Message::IconPicker(msg) => {
                if let Some(picker) = &mut self.iconpicker {
                    return picker.update(msg).map(cosmic::app::message::app);
                };
            }
            Message::IconsResult(result) => {
                if let Some(picker) = &mut self.iconpicker {
                    for path in result {
                        if let Some(icon) = image_handle(path) {
                            picker.push_icon(icon);
                        }
                    }
                }
            }
            Message::InsertApp(launcher) => {
                self.nav
                    .insert()
                    .icon(widget::icon::from_name("edit-symbolic"))
                    .text(launcher.name.clone())
                    .data::<Page>(Page::Editor(editor::AppEditor::from(launcher)));
            }
            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
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
                self.iconpicker = Some(IconPicker::new(app_url));
            }
            Message::OpenRepositoryUrl => {
                _ = open::that_detached(REPOSITORY);
            }
            Message::SetIcon(icon) => {
                let Page::Editor(app_editor) = &mut self.page;
                app_editor.update_icon(icon);
                self.iconpicker = None;
            }
            Message::SubscriptionChannel => {}
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
            menu::root(fl!("view")),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("app"), None, MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
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
            .title(fl!("app")),
        })
    }

    fn on_escape(&mut self) -> Task<Message> {
        self.iconpicker = None;
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
        if let Some(icon_picker) = &self.iconpicker {
            let view = icon_picker.view().map(Message::IconPicker);

            return Some(
                widget::dialog()
                    .title(fl!("icon-selector"))
                    .secondary_action(
                        widget::button::standard(fl!("close")).on_press(Message::CloseIconPicker),
                    )
                    .control(view)
                    .into(),
            );
        };

        None
    }
}

impl QuickWebApps {
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let icon = widget::image(widget::image::Handle::from_bytes(APP_ICON));

        let title = widget::text::title3(fl!("app"));

        let hash = env!("VERGEN_GIT_SHA");
        let _short_hash: String = hash.chars().take(7).collect();
        let _date = env!("VERGEN_GIT_COMMIT_DATE");

        let link = widget::button::link(REPOSITORY)
            .on_press(Message::OpenRepositoryUrl)
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
            .push(link)
            .push(
                widget::button::link(fl!(
                    "git-description",
                    hash = _short_hash.as_str(),
                    date = _date
                ))
                .on_press(Message::LaunchUrl(format!("{REPOSITORY}/commits/{hash}")))
                .padding(0),
            )
            .align_x(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    fn match_title(&self) -> String {
        match &self.page {
            Page::Editor(editor) => format!("{} {}", fl!("edit"), editor.app_title),
        }
    }

    fn update_title(&mut self) -> Task<Message> {
        self.set_header_title(self.match_title());
        self.set_window_title(self.match_title(), self.window_id)
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
