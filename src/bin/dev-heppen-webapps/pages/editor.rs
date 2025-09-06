use cosmic::{
    action::Action,
    iced::{alignment::Vertical, Length},
    style, task,
    widget::{self},
    Element, Task,
};
use rand::{rng, Rng};
use strum::IntoEnumIterator as _;
use webapps::fl;

use crate::pages;

#[derive(Debug, Clone)]
pub struct AppEditor {
    pub app_browser: Option<webapps::browser::Browser>,
    pub app_title: String,
    pub app_url: String,
    pub app_icon: String,
    pub app_category: webapps::Category,
    pub app_persistent: bool,
    pub app_window_width: String,
    pub app_window_height: String,
    pub app_window_size: webapps::WindowSize,
    pub app_window_decorations: bool,
    pub app_private_mode: bool,
    pub app_simulate_mobile: bool,
    pub selected_icon: Option<webapps::Icon>,
    pub categories: Vec<String>,
    pub category_idx: Option<usize>,
    pub is_installed: bool,
}

impl Default for AppEditor {
    fn default() -> Self {
        let categories = webapps::Category::iter()
            .map(|c| c.name())
            .collect::<Vec<String>>();

        AppEditor {
            app_browser: None,
            app_title: String::new(),
            app_url: String::new(),
            app_icon: String::new(),
            app_category: webapps::Category::default(),
            app_persistent: false,
            app_window_width: String::from(webapps::DEFAULT_WINDOW_WIDTH.to_string()),
            app_window_height: String::from(webapps::DEFAULT_WINDOW_HEIGHT.to_string()),
            app_window_size: webapps::WindowSize::default(),
            app_window_decorations: true,
            app_private_mode: false,
            app_simulate_mobile: false,
            selected_icon: None,
            categories,
            category_idx: Some(0),
            is_installed: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Category(usize),
    Done,
    PersistentProfile(bool),
    LaunchApp,
    OpenIconPicker,
    Title(String),
    Url(String),
    WindowWidth(String),
    WindowHeight(String),
    WindowDecorations(bool),
    AppIncognito(bool),
    AppSimulateMobile(bool),
}

impl AppEditor {
    pub fn from(webapp_launcher: webapps::launcher::WebAppLauncher) -> Self {
        let entry = webapps::launcher::installed_webapps()
            .into_iter()
            .find(|webapp| webapp.browser.app_id == webapp_launcher.browser.app_id);

        if let Some(launcher) = entry {
            let window_size = launcher.browser.window_size.clone().unwrap_or_default();
            let window_decorations = launcher.browser.window_decorations.unwrap_or_default();
            let incognito = launcher.browser.private_mode.unwrap_or_default();
            let simulate_mobile = launcher.browser.try_simulate_mobile.unwrap_or_default();

            let mut editor = AppEditor::default();

            editor.app_browser = Some(launcher.browser.clone());
            editor.app_title = launcher.name.clone();
            editor.app_url = launcher.browser.url.clone().unwrap_or_default();
            editor.app_icon = launcher.icon.clone();
            editor.app_category = launcher.category.clone();
            editor.app_persistent = launcher.browser.profile.is_some();
            editor.app_window_width = window_size.0.to_string();
            editor.app_window_height = window_size.1.to_string();
            editor.app_window_size = window_size.clone();
            editor.app_window_decorations = window_decorations;
            editor.app_private_mode = incognito;
            editor.app_simulate_mobile = simulate_mobile;
            editor.category_idx = editor
                .categories
                .iter()
                .position(|c| c == &launcher.category.name());
            editor.is_installed = true;

            editor
        } else {
            AppEditor::default()
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Action<crate::pages::Message>> {
        match message {
            Message::AppIncognito(flag) => {
                self.app_private_mode = flag;
            }
            Message::AppSimulateMobile(flag) => {
                self.app_simulate_mobile = flag;
            }
            Message::Category(idx) => {
                self.app_category = webapps::Category::from_index(idx as u8);
                self.category_idx = Some(idx);
            }
            Message::Done => {
                let browser = if let Some(browser) = &self.app_browser {
                    browser.clone()
                } else {
                    let app_id = self.app_title.replace(' ', "");
                    let app_id = app_id + &rng().random_range(1000..10000).to_string();

                    let mut browser = webapps::browser::Browser::new(&app_id, self.app_persistent);
                    browser.window_title = Some(self.app_title.clone());
                    browser.url = Some(self.app_url.clone());
                    browser.window_size = Some(self.app_window_size.clone());
                    browser.window_decorations = Some(self.app_window_decorations.clone());
                    browser.private_mode = Some(self.app_private_mode);
                    browser.try_simulate_mobile = Some(self.app_simulate_mobile);
                    browser
                };

                if webapps::launcher::webapplauncher_is_valid(
                    &self.app_icon,
                    &self.app_title,
                    &browser.url,
                    &self.app_category,
                ) {
                    let launcher = webapps::launcher::WebAppLauncher {
                        browser: browser.clone(),
                        name: self.app_title.clone(),
                        icon: self.app_icon.clone(),
                        category: self.app_category.clone(),
                    };

                    return task::future(async move {
                        if launcher.create().await.is_ok() {
                            crate::pages::Message::SaveLauncher(launcher)
                        } else {
                            crate::pages::Message::None
                        }
                    });
                } else {
                    return Task::none();
                }
            }
            Message::PersistentProfile(flag) => {
                self.app_persistent = flag;
            }
            Message::LaunchApp => {
                if let Some(browser) = &self.app_browser {
                    let arg_id = browser.app_id.clone();

                    return task::future(async { crate::pages::Message::Launch(arg_id) });
                }
            }
            Message::OpenIconPicker => {
                return task::future(async { pages::Message::OpenIconPicker })
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

    pub fn update_icon(&mut self, icon: Option<webapps::Icon>) {
        if let Some(icon) = icon {
            self.app_icon = icon.path.clone();
            self.selected_icon = Some(icon);
        }
    }

    fn icon_element(&self, icon: Option<webapps::Icon>) -> Element<'_, Message> {
        let ico = if let Some(ico) = icon {
            match ico.icon {
                webapps::IconType::Raster(data) => widget::button::custom(widget::image(data))
                    .width(Length::Fixed(92.0))
                    .height(Length::Fixed(92.0))
                    .class(style::Button::Icon)
                    .on_press(Message::OpenIconPicker),

                webapps::IconType::Svg(data) => widget::button::custom(widget::svg(data))
                    .width(Length::Fixed(92.0))
                    .height(Length::Fixed(92.0))
                    .class(style::Button::Icon)
                    .on_press(Message::OpenIconPicker),
            }
        } else {
            widget::button::custom(widget::icon::from_name("folder-pictures-symbolic"))
                .width(Length::Fixed(92.0))
                .height(Length::Fixed(92.0))
                .class(style::Button::Suggested)
                .on_press(Message::OpenIconPicker)
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
                .push(widget::text_input(fl!("url"), &self.app_url).on_input(Message::Url))
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
                                        format!("{}", webapps::DEFAULT_WINDOW_WIDTH),
                                        &self.app_window_width,
                                    )
                                    .on_input(Message::WindowWidth),
                                )
                                .push(
                                    widget::text_input(
                                        format!("{}", webapps::DEFAULT_WINDOW_HEIGHT),
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
                            if webapps::launcher::webapplauncher_is_valid(
                                &self.app_icon,
                                &self.app_title,
                                &Some(self.app_url.clone()),
                                &self.app_category,
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
