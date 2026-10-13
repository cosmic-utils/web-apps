use std::{collections::HashSet, path::PathBuf, sync::LazyLock};

use cosmic::{
    Element, Task,
    action::Action,
    iced::{Length, alignment::Vertical},
    style,
    widget::{self},
};
use rand::RngExt;
use strum::IntoEnumIterator as _;
use webapps::{
    APP_ICON, Category, WebappIcon,
    browser::{Browser, BrowserConfig, BrowserT, installed_browsers},
    fl, is_profile_path_readonly,
    launcher::WebappLauncher,
    url_valid, webapp_id,
};

use crate::pages;

static INSTALLED_BROWSERS: LazyLock<HashSet<Browser>> = LazyLock::new(|| installed_browsers());

#[derive(Debug, Clone)]
pub struct AppEditor {
    pub app_browser: Option<webapps::browser::Browser>,
    pub app_browser_selection: Option<usize>,
    pub browser_config: BrowserConfig,
    pub app_profile: Option<String>,
    pub app_id: String,
    pub app_num_id: u16,
    pub app_title: String,
    pub app_url: String,
    pub app_icon: Option<WebappIcon>,
    pub app_category: webapps::Category,
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

        let num_id = rand::rng().random_range(1000..10000);

        let mut editor = AppEditor {
            app_browser: None,
            app_browser_selection: None,
            browser_config: BrowserConfig::default(),
            app_profile: None,
            app_id: String::new(),
            app_num_id: num_id,
            app_title: String::new(),
            app_url: String::new(),
            app_icon: None,
            app_category: webapps::Category::default(),
            categories,
            category_idx: webapps::Category::iter().position(|c| c == Category::Utility),
            custom_params: String::new(),
            is_installed: false,
        };

        if !INSTALLED_BROWSERS.is_empty() {
            editor.app_browser_selection = Some(0);
            editor.app_browser = INSTALLED_BROWSERS.iter().next().cloned();
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
    pub fn from_launcher(value: &WebappLauncher) -> Option<Self> {
        let Some(ref browser) = value.browser else {
            return None;
        };

        let categories = webapps::Category::iter()
            .map(|c| c.name())
            .collect::<Vec<String>>();

        let category = webapps::Category::iter()
            .find(|c| c.name() == value.category)
            .map(|c| c)
            .unwrap_or_default();

        let app_browser_selection = INSTALLED_BROWSERS
            .iter()
            .position(|b| b.executable_path == browser.executable_path);

        let app_num_id = {
            let split_pos = value.unique_id.char_indices().nth_back(3).unwrap().0;
            value.unique_id[split_pos..]
                .parse::<u16>()
                .unwrap_or_default()
        };

        Some(Self {
            app_browser_selection,
            app_browser: Some(browser.clone()),
            browser_config: browser.config.clone(),
            app_profile: browser.config.profile_path.clone(),
            app_id: value.unique_id.clone(),
            app_num_id,
            app_title: value.webapp_name.clone(),
            app_url: browser.config.url.clone(),
            app_icon: value.webapp_icon.clone(),
            app_category: category,
            categories,
            category_idx: webapps::Category::iter().position(|c| c.name() == value.category),
            custom_params: browser.config.custom_parameters.clone(),
            is_installed: true,
        })
    }

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
            self.browser_config.url = self.app_url.clone();
        }

        // set classname
        // workaround for chromium browser since they does not respect --class argument
        // on wayland
        self.browser_config.class_name = match browser.browser_t {
            webapps::browser::BrowserT::Brave => {
                webapps::browser::try_chromium_classname("brave", &self.app_url)
            }
            webapps::browser::BrowserT::Chromium | webapps::browser::BrowserT::MsEdge => {
                webapps::browser::try_chromium_classname("chrome", &self.app_url)
            }
            webapps::browser::BrowserT::Vivaldi => {
                webapps::browser::try_chromium_classname("vivaldi", &self.app_url)
            }
            _ => self.app_id.clone(),
        };

        // set profile path
        if let Some(path) =
            install_t.profile_path(&browser.app_id, &browser.executable_name, &self.app_id)
        {
            self.app_profile = Some(path.to_str().unwrap_or_default().to_string())
        }

        self.browser_config
            .set_profile_path(self.app_profile.clone());
        self.browser_config.custom_parameters = self.custom_params.clone();

        browser.config = self.browser_config.clone();
    }

    pub fn update(&mut self, message: Message) -> Task<Action<crate::pages::Message>> {
        match message {
            Message::AppIsolated(flag) => {
                self.browser_config.isolated_profile = flag;
                if !flag {
                    self.app_profile = None;
                }
            }
            Message::AppPrivateMode(flag) => {
                self.browser_config.private_mode = flag;
            }
            Message::Browser(idx) => {
                self.app_browser_selection = Some(idx);
                self.app_browser = INSTALLED_BROWSERS
                    .iter()
                    .enumerate()
                    .find(|(idx, _b)| Some(idx) == self.app_browser_selection.as_ref())
                    .map(|(_, b)| b.clone());

                if let Some(browser) = self.app_browser.as_mut() {
                    browser.config = self.browser_config.clone();
                }
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
                    if let Some(profile) = &self.app_profile {
                        if (browser.browser_t == BrowserT::Firefox
                            || browser.browser_t == BrowserT::Zen)
                            && is_profile_path_readonly(&PathBuf::from(&profile))
                            && browser.config.isolated_profile
                        {
                            return Task::done(Action::App(pages::Message::ShowWarningDialog(
                                profile.clone(),
                            )));
                        }
                    }

                    let webapp_launcher = WebappLauncher {
                        browser: Some(browser.clone()),
                        unique_id: self.app_id.clone(),
                        webapp_name: self.app_title.clone(),
                        webapp_icon: self.app_icon.clone(),
                        category: self.app_category.as_ref().to_owned(),
                    };

                    if let Ok(_) = webapp_launcher.create_desktop_entry() {
                        return Task::done(Action::App(crate::pages::Message::SaveLauncher));
                    }
                }
            }
            Message::GenerateIcon => {
                let webapp_icon = webapps::generate_icon(&self.app_title);
                return Task::done(Action::App(pages::Message::SetIcon(webapp_icon)));
            }
            Message::OpenIconPicker => {
                return Task::done(Action::App(pages::Message::OpenIconPicker));
            }
            Message::ResetIcon => {
                self.app_icon = None;
            }
            Message::Title(title) => {
                if title.len() < 3 {
                    self.app_profile = None;
                    self.app_id.clear();
                }

                self.app_title = title;
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
        let spacing = cosmic::theme::active().cosmic().spacing;

        let browsers: Vec<String> = INSTALLED_BROWSERS
            .iter()
            .map(|b| b.display_name.to_string())
            .collect();

        widget::container(widget::scrollable(
            widget::Column::new()
                .spacing(spacing.space_xs)
                .push(
                    widget::container(
                        widget::Row::new()
                            .spacing(spacing.space_xs)
                            .push(
                                widget::container(icon_iced_element(&self.app_icon))
                                    .width(72.)
                                    .height(72.)
                                    .align_y(Vertical::Center),
                            )
                            .push(
                                widget::container(
                                    widget::Column::new()
                                        .push(widget::text::title4(if self.app_title.is_empty() {
                                            fl!("new-webapp-title")
                                        } else {
                                            self.app_title.clone()
                                        }))
                                        .push(widget::text(self.app_category.name())),
                                )
                                .height(Length::Fixed(72.))
                                .align_y(Vertical::Center),
                            ),
                    )
                    .width(Length::Fill),
                )
                .push(
                    widget::Row::new()
                        .spacing(spacing.space_xs)
                        .push(
                            widget::text_input(fl!("title"), &self.app_title)
                                .on_input(Message::Title),
                        )
                        .push(
                            widget::button::standard(fl!("generate-icon")).on_press_maybe(
                                if !self.app_title.is_empty() {
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
                            if self.app_icon.is_some() {
                                Some(Message::ResetIcon)
                            } else {
                                None
                            },
                        )),
                )
                .push(widget::text_input(fl!("url"), &self.app_url).on_input(Message::Url))
                .push(
                    widget::list_column::list_column()
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
                            widget::toggler(self.browser_config.isolated_profile)
                                .on_toggle(Message::AppIsolated),
                        ))
                        .add(widget::settings::item(
                            fl!("private-mode"),
                            widget::toggler(self.browser_config.private_mode)
                                .on_toggle(Message::AppPrivateMode),
                        ))
                        .add(widget::settings::item(
                            fl!("non-standard-arguments"),
                            widget::text_input("--window-size=800,600", &self.custom_params)
                                .on_input(Message::CustomParams),
                        )),
                )
                .push(widget::settings::item(
                    "",
                    widget::button::suggested(fl!("create")).on_press_maybe(
                        if webapps::launcher::webapplauncher_is_valid(
                            &self.app_title,
                            &Some(self.app_url.clone()),
                        ) {
                            Some(Message::Done)
                        } else {
                            None
                        },
                    ),
                )),
        ))
        .max_width(1000)
        .into()
    }
}

pub fn icon_iced_element<'a>(webapp_icon: &'a Option<WebappIcon>) -> Element<'a, Message> {
    let Some(icon) = webapp_icon else {
        return Element::from(
            widget::button::custom(widget::icon::from_raster_bytes(APP_ICON).icon())
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
