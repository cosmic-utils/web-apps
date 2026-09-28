use std::fs;

use cosmic::{
    Element, Task,
    action::Action,
    iced::{Length, alignment::Vertical},
    style, task,
    widget::{self},
};
use strum::IntoEnumIterator as _;
use webapps::{Category, WebappIcon, fl};

use crate::pages;

#[derive(Debug, Clone)]
pub struct AppEditor {
    pub app_browsers: Vec<webapps::browser::Browser>,
    pub app_browser_selection: Option<usize>,
    pub app_title: String,
    pub app_url: String,
    pub app_icon: Option<WebappIcon>,
    pub app_category: webapps::Category,
    pub app_window_width: String,
    pub app_window_height: String,
    pub app_window_size: webapps::WindowSize,
    pub app_isolated: bool,
    pub app_simulate_mobile: bool,
    pub selected_icon: Option<String>,
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
            app_browsers: webapps::browser::Browser::installed_browsers(),
            app_browser_selection: None,
            app_title: String::new(),
            app_url: String::new(),
            app_icon: None,
            app_category: webapps::Category::default(),
            app_window_width: String::from(webapps::DEFAULT_WINDOW_WIDTH.to_string()),
            app_window_height: String::from(webapps::DEFAULT_WINDOW_HEIGHT.to_string()),
            app_window_size: webapps::WindowSize::default(),
            app_isolated: true,
            app_simulate_mobile: false,
            selected_icon: None,
            categories,
            category_idx: webapps::Category::iter().position(|c| c == Category::Utility),
            is_installed: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    AppIsolated(bool),
    Browser(usize),
    Category(usize),
    Done,
    GenerateIcon,
    OpenIconPicker,
    ResetIcon,
    Title(String),
    Url(String),
}

impl AppEditor {
    pub fn update(&mut self, message: Message) -> Task<Action<crate::pages::Message>> {
        match message {
            Message::AppIsolated(flag) => {
                self.app_isolated = flag;
            }
            Message::Browser(idx) => {
                self.app_browser_selection = Some(idx);
            }
            Message::Category(idx) => {
                self.app_category = webapps::Category::from_index(idx as u8);
                self.category_idx = Some(idx);
            }
            Message::Done => {
                // let browser = if let Some(browser) = &self.app_browser {
                //     browser.clone()
                // } else {
                //     let app_id = self.app_title.replace(' ', "");
                //     let app_id = app_id + &rng().random_range(1000..10000).to_string();

                //     let mut browser = webapps::browser::Browser::new(&app_id);
                //     browser.window_title = Some(self.app_title.clone());
                //     browser.url = Some(self.app_url.clone());
                //     browser.window_size = Some(self.app_window_size.clone());
                //     browser.try_simulate_mobile = Some(self.app_simulate_mobile);
                //     browser
                // };

                // if webapps::launcher::webapplauncher_is_valid(&self.app_title, &browser.url) {
                //     if let Some(icon) = &self.app_icon {
                //             browser: browser.clone(),
                //             name: self.app_title.clone(),
                //             icon: icon.clone(),
                //             category: self.app_category.clone(),
                //         };

                //         return task::future(async move {
                //             if let Ok(success) = launcher.create().await {
                //                 if success {
                //                     return crate::pages::Message::SaveLauncher(launcher);
                //                 }
                //             }
                //             crate::pages::Message::None
                //         });
                //     }
                // } else {
                //     return Task::none();
                // }
            }
            Message::GenerateIcon => {
                // if self.app_title.len() > 1 {
                //     let icon = generate_icon(&self.app_title.split_at(1).0);

                //     self.update_icon(icon.clone());

                //     if let Some(icon) = icon {
                //         if webapp_icon_valid(&icon) {
                //             let ico = webapps::handle_icon(icon.path);

                //             return task::future(async {
                //                 Action::App(pages::Message::SetIcon(ico.into()))
                //             });
                //         };
                //     }
                // }
            }
            Message::OpenIconPicker => {
                return task::future(async { pages::Message::OpenIconPicker });
            }
            Message::ResetIcon => {
                self.app_icon = None;
                self.selected_icon = None;
            }
            Message::Title(title) => {
                self.app_title = title;
            }
            Message::Url(url) => {
                self.app_url = url;
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let browsers: Vec<String> = self
            .app_browsers
            .iter()
            .map(|b| b.name.to_string())
            .collect();

        widget::container(
            widget::column()
                .spacing(24)
                .push(
                    widget::container(
                        widget::row()
                            .spacing(12)
                            .push(
                                widget::container(icon_iced_element(&self.selected_icon))
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
                                                fl!("new-webapp-title")
                                            } else {
                                                self.app_title.clone()
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
                .push(
                    widget::row()
                        .spacing(8)
                        .push(
                            widget::text_input(fl!("title"), &self.app_title)
                                .on_input(Message::Title),
                        )
                        .push(
                            widget::button::standard(fl!("generate-icon")).on_press_maybe(
                                if self.app_title.len() > 1 {
                                    Some(Message::GenerateIcon)
                                } else {
                                    None
                                },
                            ),
                        )
                        .push(
                            widget::button::standard(fl!("icon-selector"))
                                .on_press_maybe(Some(Message::OpenIconPicker)),
                        )
                        .push(widget::button::standard(fl!("reset-icon")).on_press_maybe(
                            if self.selected_icon.is_some() {
                                Some(Message::ResetIcon)
                            } else {
                                None
                            },
                        )),
                )
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
                            fl!("select-browser"),
                            widget::dropdown(
                                browsers,
                                self.app_browser_selection,
                                Message::Browser,
                            ),
                        ))
                        .add(widget::settings::item(
                            fl!("isolated-profile"),
                            widget::toggler(self.app_isolated).on_toggle(Message::AppIsolated),
                        )),
                )
                .push(widget::button::suggested(fl!("create")).on_press_maybe(
                    if webapps::launcher::webapplauncher_is_valid(
                        &self.app_title,
                        &Some(self.app_url.clone()),
                    ) {
                        Some(Message::Done)
                    } else {
                        None
                    },
                )),
        )
        .padding(cosmic::iced::Padding::new(0.).left(30.0).right(30.0))
        .max_width(1000)
        .into()
    }
}

pub fn icon_iced_element<'a>(source_path: &'a Option<String>) -> Element<'a, Message> {
    let Some(source) = source_path else {
        let data: &'static [u8] =
            include_bytes!("../../resources/icons/hicolor/128x128/apps/dev.heppen.webapps.png");

        let handle = cosmic::iced_core::image::Handle::from_bytes(data);

        return Element::from(
            widget::button::custom(widget::image(handle))
                .width(Length::Fixed(92.0))
                .height(Length::Fixed(92.0))
                .class(style::Button::Icon),
        );
    };
    if webapps::is_svg(&source) {
        return Element::from(
            widget::button::custom(widget::svg::Svg::from_path(source))
                .width(Length::Fixed(92.0))
                .height(Length::Fixed(92.0))
                .class(style::Button::Icon),
        );
    } else {
        let data = fs::read(&source).unwrap_or_default();

        let handle = cosmic::iced_core::image::Handle::from_bytes(data);

        return Element::from(
            widget::button::custom(widget::image(handle))
                .width(Length::Fixed(92.0))
                .height(Length::Fixed(92.0))
                .class(style::Button::Icon),
        );
    }
}
