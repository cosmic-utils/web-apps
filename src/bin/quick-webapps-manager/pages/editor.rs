use cosmic::{
    action::Action,
    iced::{alignment::Vertical, futures::executor::block_on, Length},
    style, task,
    widget::{self},
    Element, Task,
};
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use webapps::{fl, WindowSize, DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, WEBVIEW_APP_ID};

use crate::{
    browser::Browser,
    common::{self, image_handle, move_icon, url_valid, Icon, IconType},
    launcher::{webapplauncher_is_valid, WebAppLauncher},
    pages::{self},
};

#[repr(u8)]
#[derive(Debug, Default, Clone, EnumIter, PartialEq, Eq, Deserialize, Serialize)]
pub enum Category {
    #[default]
    Audio = 0,
    AudioVideo = 1,
    Video = 2,
    Development = 3,
    Education = 4,
    Game = 5,
    Graphics = 6,
    Network = 7,
    Office = 8,
    Science = 9,
    Settings = 10,
    System = 11,
    Utility = 12,
}

impl AsRef<str> for Category {
    fn as_ref(&self) -> &str {
        match self {
            Category::Audio => "Audio",
            Category::AudioVideo => "AudioVideo",
            Category::Video => "Video",
            Category::Development => "Development",
            Category::Education => "Education",
            Category::Game => "Game",
            Category::Graphics => "Graphics",
            Category::Network => "Network",
            Category::Office => "Office",
            Category::Science => "Science",
            Category::Settings => "Settings",
            Category::System => "System",
            Category::Utility => "Utility",
        }
    }
}

impl From<String> for Category {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Audio" => Category::Audio,
            "AudioVideo" => Category::AudioVideo,
            "Video" => Category::Video,
            "Development" => Category::Development,
            "Education" => Category::Education,
            "Game" => Category::Education,
            "Graphics" => Category::Graphics,
            "Network" => Category::Network,
            "Office" => Category::Office,
            "Science" => Category::Science,
            "Settings" => Category::Settings,
            "System" => Category::System,
            "Utility" => Category::Utility,
            _ => Self::default(),
        }
    }
}

impl Category {
    pub fn name(&self) -> String {
        match self {
            Category::Audio => String::from("Audio"),
            Category::AudioVideo => String::from("Audio & Video"),
            Category::Video => String::from("Video"),
            Category::Development => String::from("Development"),
            Category::Education => String::from("Education"),
            Category::Game => String::from("Game"),
            Category::Graphics => String::from("Graphics"),
            Category::Network => String::from("Network"),
            Category::Office => String::from("Office"),
            Category::Science => String::from("Science"),
            Category::Settings => String::from("Settings"),
            Category::System => String::from("System"),
            Category::Utility => String::from("Utility"),
        }
    }

    pub fn from_index(index: u8) -> Self {
        Self::iter()
            .find(|i| i.to_owned() as u8 == index)
            .unwrap_or_default()
    }

    pub fn to_vec() -> Vec<String> {
        Self::iter().map(|c| c.name()).collect()
    }
}

#[derive(Debug, Clone)]
pub struct AppEditor {
    pub app_browser: Browser,
    pub app_title: String,
    pub app_url: String,
    pub app_icon: String,
    pub app_category: Category,
    pub app_persistent: bool,
    pub app_window_width: String,
    pub app_window_height: String,
    pub app_window_size: WindowSize,
    pub app_window_decorations: bool,
    pub app_private_mode: bool,
    pub app_simulate_mobile: bool,
    pub selected_icon: Option<Icon>,
    pub categories: Vec<String>,
    pub category_idx: Option<usize>,
    pub is_installed: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Category(usize),
    Done,
    PersistentProfile(bool),
    LaunchApp,
    OpenIconPicker(String),
    SearchFavicon,
    Title(String),
    Url(String),
    WindowWidth(String),
    WindowHeight(String),
    WindowDecorations(bool),
    AppIncognito(bool),
    AppSimulateMobile(bool),
}

impl AppEditor {
    pub fn new() -> Self {
        let categories = Category::iter().map(|c| c.name()).collect::<Vec<String>>();

        AppEditor {
            app_browser: Browser::new(WEBVIEW_APP_ID, WEBVIEW_APP_ID, "", false),
            app_title: String::new(),
            app_url: String::new(),
            app_icon: String::new(),
            app_category: Category::default(),
            app_persistent: false,
            app_window_width: String::from(DEFAULT_WINDOW_WIDTH.to_string()),
            app_window_height: String::from(DEFAULT_WINDOW_HEIGHT.to_string()),
            app_window_size: WindowSize::default(),
            app_window_decorations: true,
            app_private_mode: false,
            app_simulate_mobile: false,
            selected_icon: None,
            categories,
            category_idx: Some(0),
            is_installed: false,
        }
    }

    pub fn from(webapp_launcher: WebAppLauncher) -> Self {
        let category_idx = Category::iter().position(|c| c == webapp_launcher.category);
        let category = Category::from_index(category_idx.unwrap_or_default() as u8);
        let categories = Category::to_vec();

        let selected_icon = block_on(image_handle(webapp_launcher.icon.clone()));
        let is_installed = webapplauncher_is_valid(
            &webapp_launcher.icon,
            &webapp_launcher.name,
            &webapp_launcher.browser.url,
        );

        let args = webapp_launcher.browser.args.clone().build();
        let persistent = webapp_launcher.browser.with_profile;
        let window_size = args.window_size.unwrap_or_default();
        let window_decorations = args.window_decorations.unwrap_or_default();
        let app_private_mode = args.private_mode.unwrap_or_default();
        let app_simulate_mobile = args.try_simulate_mobile.unwrap_or_default();

        Self {
            app_browser: webapp_launcher.browser.clone(),
            app_title: webapp_launcher.name,
            app_url: webapp_launcher.browser.url,
            app_icon: webapp_launcher.icon,
            app_category: category,
            app_persistent: persistent,
            app_window_width: String::from(window_size.0.to_string()),
            app_window_height: String::from(window_size.1.to_string()),
            app_window_size: window_size.clone(),
            app_window_decorations: window_decorations.clone(),
            app_private_mode,
            app_simulate_mobile,
            selected_icon,
            categories,
            category_idx,
            is_installed,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Action<pages::Message>> {
        match message {
            Message::AppIncognito(flag) => {
                self.app_private_mode = flag;
            }
            Message::AppSimulateMobile(flag) => {
                self.app_simulate_mobile = flag;
            }
            Message::Category(idx) => {
                self.app_category = Category::from_index(idx as u8);
                self.category_idx = Some(idx);
            }
            Message::Done => {
                let app_id = self.app_title.replace(' ', "");
                let app_id = app_id + &rng().random_range(1000..10000).to_string();

                self.app_browser =
                    Browser::new(&app_id, &self.app_title, &self.app_url, self.app_persistent);
                self.app_browser.set_window_size(&self.app_window_size);
                self.app_browser
                    .set_window_decorations(self.app_window_decorations);
                self.app_browser.set_private_mode(self.app_private_mode);
                self.app_browser
                    .set_simulate_mobile(self.app_simulate_mobile);

                let icon_final_path = block_on(move_icon(&self.app_icon, &self.app_browser.app_id));

                if webapplauncher_is_valid(&icon_final_path, &self.app_title, &self.app_url) {
                    let launcher = Arc::new(WebAppLauncher {
                        browser: self.app_browser.clone(),
                        name: self.app_title.clone(),
                        icon: icon_final_path,
                        category: self.app_category.clone(),
                    });

                    let arc_launcher = Arc::clone(&launcher);

                    return task::future(async move {
                        if arc_launcher.create().await.is_ok() {
                            pages::Message::SaveLauncher(arc_launcher)
                        } else {
                            pages::Message::None
                        }
                    });
                }
            }
            Message::PersistentProfile(flag) => {
                self.app_persistent = flag;
            }
            Message::LaunchApp => {
                let args = self.app_browser.args.clone().build();
                return task::future(async { pages::Message::Launch(args) });
            }
            Message::OpenIconPicker(app_url) => {
                return task::future(async { pages::Message::OpenIconPicker(app_url) })
            }
            Message::SearchFavicon => {
                let name = common::get_icon_name_from_url(&self.app_url);
                let url = self.app_url.clone();

                return task::future(async {
                    let paths = common::find_icons(name, url).await;

                    for path in paths {
                        if let Some(icon) = image_handle(path).await {
                            return pages::Message::SetIcon(Some(icon));
                        }
                    }
                    pages::Message::None
                });
            }
            Message::Title(title) => {
                self.app_title = title;
            }
            Message::Url(url) => {
                self.app_url = url;
            }
            Message::WindowDecorations(decorations) => {
                self.app_window_decorations = decorations;
            }
            Message::WindowWidth(width) => {
                self.app_window_width = width;
                self.app_window_size.0 = self.app_window_width.parse().unwrap_or_default();
            }
            Message::WindowHeight(height) => {
                self.app_window_height = height;
                self.app_window_size.1 = self.app_window_height.parse().unwrap_or_default();
            }
        }
        Task::none()
    }

    pub fn update_icon(&mut self, icon: Option<Icon>) {
        if let Some(icon) = icon {
            self.app_icon = icon.path.clone();
            self.selected_icon = Some(icon);
        }
    }

    fn icon_element(&self, icon: Option<Icon>) -> Element<'_, Message> {
        let ico = if let Some(ico) = icon {
            match ico.icon {
                IconType::Raster(data) => widget::button::custom(widget::image(data))
                    .width(Length::Fixed(92.0))
                    .height(Length::Fixed(92.0))
                    .class(style::Button::Icon)
                    .on_press(Message::OpenIconPicker(self.app_url.clone())),

                IconType::Svg(data) => widget::button::custom(widget::svg(data))
                    .width(Length::Fixed(92.0))
                    .height(Length::Fixed(92.0))
                    .class(style::Button::Icon)
                    .on_press(Message::OpenIconPicker(self.app_url.clone())),
            }
        } else {
            widget::button::custom(widget::icon::from_name("folder-pictures-symbolic"))
                .width(Length::Fixed(92.0))
                .height(Length::Fixed(92.0))
                .class(style::Button::Suggested)
                .on_press(Message::OpenIconPicker(self.app_url.clone()))
        };

        widget::container(ico).into()
    }

    pub fn view(&self) -> Element<'_, Message> {
        widget::container(
            widget::column()
                .spacing(24)
                .push(
                    widget::container(
                        widget::row()
                            .spacing(12)
                            .push(
                                widget::container(self.icon_element(self.selected_icon.clone()))
                                    .width(96.)
                                    .height(96.)
                                    .align_y(Vertical::Center),
                            )
                            .push(
                                widget::container(
                                    widget::column()
                                        .spacing(12)
                                        .push(widget::text::title3(format!(
                                            "{}: {}",
                                            fl!("title"),
                                            if self.app_title.is_empty() {
                                                &fl!("new-webapp-title")
                                            } else {
                                                &self.app_title
                                            }
                                        )))
                                        .push(widget::text::title4(format!(
                                            "{}: {}",
                                            fl!("category"),
                                            self.app_category.name()
                                        ))),
                                )
                                .height(Length::Fixed(96.))
                                .align_y(Vertical::Center),
                            ),
                    )
                    .padding(12)
                    .width(Length::Fill)
                    .class(style::Container::Card),
                )
                .push(widget::text_input(fl!("title"), &self.app_title).on_input(Message::Title))
                .push(widget::settings::item_row(vec![
                    widget::text_input(fl!("url"), &self.app_url)
                        .on_input(Message::Url)
                        .into(),
                    widget::button::standard(fl!("download-favicon"))
                        .on_press_maybe(if url_valid(&self.app_url) {
                            Some(Message::SearchFavicon)
                        } else {
                            None
                        })
                        .into(),
                ]))
                .push(
                    widget::settings::section()
                        .add(widget::settings::item(
                            fl!("select-category"),
                            widget::dropdown(
                                &self.categories,
                                self.category_idx,
                                Message::Category,
                            ),
                        ))
                        .add(widget::settings::item(
                            fl!("persistent-profile"),
                            widget::toggler(self.app_persistent)
                                .on_toggle(Message::PersistentProfile),
                        ))
                        .add(widget::settings::item(
                            fl!("window-size"),
                            widget::row()
                                .spacing(8)
                                .push(
                                    widget::text_input(
                                        format!("{}", DEFAULT_WINDOW_WIDTH),
                                        &self.app_window_width,
                                    )
                                    .on_input(Message::WindowWidth),
                                )
                                .push(
                                    widget::text_input(
                                        format!("{}", DEFAULT_WINDOW_HEIGHT),
                                        &self.app_window_height,
                                    )
                                    .on_input(Message::WindowHeight),
                                ),
                        ))
                        .add(widget::settings::item(
                            fl!("decorations"),
                            widget::toggler(self.app_window_decorations)
                                .on_toggle(Message::WindowDecorations),
                        ))
                        .add(widget::settings::item(
                            fl!("private-mode"),
                            widget::toggler(self.app_private_mode).on_toggle(Message::AppIncognito),
                        ))
                        .add(widget::settings::item(
                            fl!("simulate-mobile"),
                            widget::toggler(self.app_simulate_mobile)
                                .on_toggle(Message::AppSimulateMobile),
                        )),
                )
                .push(
                    widget::row()
                        .spacing(8)
                        .push(widget::horizontal_space())
                        .push_maybe(if !self.is_installed {
                            None
                        } else {
                            Some(
                                widget::button::standard(fl!("run-app"))
                                    .on_press(Message::LaunchApp),
                            )
                        })
                        .push(widget::button::suggested(fl!("create")).on_press_maybe(
                            if webapplauncher_is_valid(
                                &self.app_icon,
                                &self.app_title,
                                &self.app_url,
                            ) {
                                Some(Message::Done)
                            } else {
                                None
                            },
                        )),
                ),
        )
        .padding(cosmic::iced::Padding::new(0.).left(30.0).right(30.0))
        .max_width(1000)
        .into()
    }
}
