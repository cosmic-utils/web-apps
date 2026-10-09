use std::fs;

use cosmic::{
    Element, Task,
    action::Action,
    iced::{Length, alignment::Vertical},
    style,
    widget::{self},
};
use rand::RngExt;
use strum::IntoEnumIterator as _;
use webapps::{Category, WebappIcon, fl, launcher::create_desktop_entry, url_valid, webapp_id};

use crate::pages;

#[derive(Debug, Clone)]
pub struct AppEditor {
    pub app_browsers: Vec<webapps::browser::Browser>,
    pub app_browser: Option<webapps::browser::Browser>,
    pub app_browser_selection: Option<usize>,
    pub app_profile: Option<String>,
    pub app_id: String,
    pub app_num_id: u16,
    pub app_title: String,
    pub app_url: String,
    pub app_icon: Option<WebappIcon>,
    pub app_category: webapps::Category,
    pub selected_icon: Option<String>,
    pub categories: Vec<String>,
    pub category_idx: Option<usize>,
    pub custom_params: String,
    pub is_installed: bool,
}

impl Default for AppEditor {
    fn default() -> Self {
        let categories = webapps::Category::iter()
            .map(|c| c.name())
            .collect::<Vec<String>>();
        let installed_browsers = webapps::browser::installed_browsers();

        let num_id = rand::rng().random_range(1000..10000);

        let mut editor = AppEditor {
            app_browser_selection: None,
            app_browser: None,
            app_browsers: installed_browsers.clone(),
            app_profile: None,
            app_id: String::new(),
            app_num_id: num_id,
            app_title: String::new(),
            app_url: String::new(),
            app_icon: None,
            app_category: webapps::Category::default(),
            selected_icon: None,
            categories,
            category_idx: webapps::Category::iter().position(|c| c == Category::Utility),
            custom_params: String::new(),
            is_installed: false,
        };

        if !installed_browsers.is_empty() {
            editor.app_browser_selection = Some(0);
            editor.app_browser = Some(installed_browsers[0].clone());
        }

        editor
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    AppIsolated(bool),
    AppPrivateMode(bool),
    Browser(usize),
    Category(usize),
    CustomParams(String),
    Done,
    GenerateIcon,
    OpenIconPicker,
    ResetIcon,
    Title(String),
    Url(String),
}

impl AppEditor {
    pub fn update_browser_config(&mut self) {
        if self.app_title.len() >= 3 {
            self.app_id = webapp_id(self.app_title.clone(), self.app_num_id);
        }
        let Some(browser) = self.app_browser.as_mut() else {
            return;
        };

        let Some(install_t) = browser.install_t.as_ref() else {
            return;
        };

        if url_valid(&self.app_url) {
            browser.config.url = self.app_url.clone();
        }

        // set classname
        browser.config.class_name = self.app_id.clone();

        // set profile path
        if let Some(path) =
            install_t.profile_path(&browser.app_id, &browser.executable_name, &self.app_id)
        {
            self.app_profile = Some(path.to_str().unwrap_or_default().to_string())
        }

        browser.config.set_profile_path(self.app_profile.clone());

        browser.config.custom_parameters = self.custom_params.clone();
    }

    pub fn update(&mut self, message: Message) -> Task<Action<crate::pages::Message>> {
        match message {
            Message::AppIsolated(flag) => {
                if let Some(browser) = self.app_browser.as_mut() {
                    browser.config.isolated_profile = flag;
                }
            }
            Message::AppPrivateMode(flag) => {
                if let Some(browser) = self.app_browser.as_mut() {
                    browser.config.private_mode = flag;
                }
            }
            Message::Browser(idx) => {
                self.app_browser_selection = Some(idx);
                self.app_browser = Some(self.app_browsers[idx].clone());
            }
            Message::Category(idx) => {
                self.app_category = webapps::Category::from_index(idx as u8);
                self.category_idx = Some(idx);
            }
            Message::CustomParams(s) => {
                self.custom_params = s;
            }
            Message::Done => {
                if let Some(browser) = &self.app_browser {
                    if let Ok(_) = create_desktop_entry(
                        browser,
                        &self.app_id,
                        &self.app_title,
                        &self.app_icon,
                        self.app_category.as_ref(),
                    ) {
                        return Task::done(Action::App(crate::pages::Message::SaveLauncher));
                    }
                }
            }
            Message::GenerateIcon => {
                let first_letter = &self.app_title.split_at(1).0;
                if !self.app_title.is_empty() {
                    let webapp_icon = webapps::generate_icon(&first_letter);

                    return Task::done(Action::App(pages::Message::SetIcon(webapp_icon)));
                }
            }
            Message::OpenIconPicker => {
                return Task::done(Action::App(pages::Message::OpenIconPicker));
            }
            Message::ResetIcon => {
                self.app_icon = None;
                self.selected_icon = None;
            }
            Message::Title(title) => {
                if title.len() < 3 {
                    self.app_profile = None;
                    self.app_id.clear();
                }

                self.app_title = title.clone();

                if !title.is_empty() {
                    return Task::done(Action::App(pages::Message::Editor(Message::GenerateIcon)));
                }
            }
            Message::Url(url) => {
                self.app_url = url;

                if url_valid(&self.app_url) {
                    let Some(browser) = self.app_browser.as_mut() else {
                        return Task::none();
                    };

                    browser.config.url = self.app_url.clone();
                }
            }
        }
        self.update_browser_config();
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let browsers: Vec<String> = self
            .app_browsers
            .iter()
            .map(|b| b.display_name.to_string())
            .collect();

        widget::scrollable(
            widget::container(
                widget::column()
                    .spacing(24)
                    .push(
                        widget::container(
                            widget::row()
                                .spacing(12)
                                .push(
                                    widget::container(icon_iced_element(&self.app_icon))
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
                                widget::toggler(if let Some(browser) = &self.app_browser {
                                    browser.config.isolated_profile
                                } else {
                                    true
                                })
                                .on_toggle(Message::AppIsolated),
                            ))
                            .add(widget::settings::item(
                                fl!("private-mode"),
                                widget::toggler(if let Some(browser) = &self.app_browser {
                                    browser.config.private_mode
                                } else {
                                    false
                                })
                                .on_toggle(Message::AppPrivateMode),
                            ))
                            .add(widget::settings::item(
                                fl!("non-standard-arguments"),
                                widget::text_input("--window-size=800,600", &self.custom_params)
                                    .on_input(Message::CustomParams),
                            ))
                            .add_maybe(if self.app_browser.is_some() {
                                Some(widget::settings::item_row(vec![
                                    widget::text(if let Some(browser) = &self.app_browser {
                                        browser.get_exec_string()
                                    } else {
                                        "".into()
                                    })
                                    .into(),
                                ]))
                            } else {
                                None
                            }),
                    )
                    .push(widget::button::standard(fl!("create")).on_press_maybe(
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
            .max_width(1000),
        )
        .into()
    }
}

pub fn icon_iced_element<'a>(webapp_icon: &'a Option<WebappIcon>) -> Element<'a, Message> {
    let Some(icon) = webapp_icon else {
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

    return match icon.icon {
        webapps::IconType::Raster => Element::from(
            widget::icon::from_raster_bytes(icon.buffer.clone())
                .icon()
                .size(92),
        ),
        webapps::IconType::Svg => Element::from(
            widget::icon::from_svg_bytes(icon.buffer.clone())
                .icon()
                .size(92),
        ),
    };
}
