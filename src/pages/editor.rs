use cosmic::{
    iced::{alignment::Vertical, Length},
    style, task,
    widget::{self},
    Element, Task,
};
use rand::{thread_rng, Rng};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    browser::{installed_browsers, Browser, BrowserModel},
    common::{self, image_handle, move_icon, url_valid, Icon, IconType},
    fl,
    launcher::{webapplauncher_is_valid, WebAppLauncher},
    pages,
};

use super::REPOSITORY;

#[derive(Debug, Clone, EnumIter)]
pub enum Categories {
    AudioVideo,
    Audio,
    Video,
    Development,
    Education,
    Game,
    Graphics,
    Network,
    Office,
    Science,
    Settings,
    System,
    Utility,
}

impl AsRef<str> for Categories {
    fn as_ref(&self) -> &str {
        match self {
            Categories::AudioVideo => "Audio & Video",
            Categories::Audio => "Audio",
            Categories::Video => "Video",
            Categories::Development => "Development",
            Categories::Education => "Education",
            Categories::Game => "Game",
            Categories::Graphics => "Graphics",
            Categories::Network => "Network",
            Categories::Office => "Office",
            Categories::Science => "Science",
            Categories::Settings => "Settings",
            Categories::System => "System",
            Categories::Utility => "Utility",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppEditor {
    pub app_codename: String,
    pub app_title: String,
    pub app_url: String,
    pub app_icon: String,
    pub app_parameters: String,
    pub app_categories: Vec<String>,
    pub category_idx: Option<usize>,
    pub app_browser: Option<Browser>,
    pub app_navbar: bool,
    pub app_incognito: bool,
    pub app_isolated: bool,
    pub selected_icon: Option<Icon>,
    pub app_browsers: Vec<Browser>,
    pub browser_idx: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Arguments(String),
    Browser(usize),
    Category(usize),
    Done,
    Incognito(bool),
    IsolatedProfile(bool),
    Navbar(bool),
    OpenIconPicker(String),
    SearchFavicon,
    Title(String),
    Url(String),
}

impl AppEditor {
    pub fn new() -> Self {
        let browsers = installed_browsers();
        let browser = if !browsers.is_empty() {
            Some(browsers[0].clone())
        } else {
            None
        };

        let categories = Categories::iter().map(|c| c.as_ref().to_string()).collect();

        AppEditor {
            app_codename: String::new(),
            app_title: fl!("new-webapp-title"),
            app_url: String::from(REPOSITORY),
            app_icon: String::new(),
            app_parameters: String::new(),
            app_categories: categories,
            category_idx: Some(0),
            app_browser: browser,
            app_navbar: false,
            app_incognito: false,
            app_isolated: true,
            selected_icon: None,
            app_browsers: browsers,
            browser_idx: Some(0),
        }
    }

    pub fn from(webapp_launcher: WebAppLauncher) -> Self {
        let categories: Vec<String> = Categories::iter().map(|c| c.as_ref().to_string()).collect();
        let category = categories
            .iter()
            .position(|c| c == &webapp_launcher.category);
        let app_browsers = installed_browsers();
        let browser_idx = app_browsers
            .iter()
            .position(|b| b.model == webapp_launcher.browser.model);

        Self {
            app_codename: webapp_launcher.codename,
            app_title: webapp_launcher.name,
            app_url: webapp_launcher.url,
            app_icon: webapp_launcher.icon,
            app_parameters: webapp_launcher.custom_parameters,
            app_categories: categories,
            category_idx: category,
            app_browser: Some(webapp_launcher.browser),
            app_navbar: webapp_launcher.navbar,
            app_incognito: webapp_launcher.is_incognito,
            app_isolated: webapp_launcher.isolate_profile,
            selected_icon: None,
            app_browsers,
            browser_idx,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<pages::Message> {
        match message {
            Message::Arguments(args) => {
                self.app_parameters = args;
            }
            Message::Browser(idx) => {
                self.browser_idx = Some(idx);
                self.app_browser = Some(self.app_browsers[idx].clone());
            }
            Message::Category(idx) => {
                self.category_idx = Some(idx);
            }
            Message::Done => {
                self.app_codename = format!(
                    "{}{}",
                    &self.app_title.replace(' ', ""),
                    thread_rng().gen_range(1000..10000)
                );
                let icon_final_path = move_icon(&self.app_icon, &self.app_title);

                if webapplauncher_is_valid(
                    &icon_final_path,
                    &self.app_codename,
                    &self.app_title,
                    &self.app_url,
                ) {
                    if let Some(browser) = &self.app_browser {
                        let launcher = WebAppLauncher {
                            codename: self.app_codename.clone(),
                            browser: browser.clone(),
                            name: self.app_title.clone(),
                            icon: icon_final_path,
                            category: self.app_categories[self.category_idx.unwrap_or_default()]
                                .clone(),
                            url: self.app_url.clone(),
                            custom_parameters: self.app_parameters.clone(),
                            isolate_profile: self.app_isolated,
                            navbar: self.app_navbar,
                            is_incognito: self.app_incognito,
                        };

                        let _ = launcher.create().is_ok();

                        return task::future(async { pages::Message::InsertApp(launcher) });
                    };
                }
            }
            Message::Navbar(flag) => {
                self.app_navbar = flag;
            }
            Message::Incognito(flag) => {
                self.app_incognito = flag;
            }
            Message::IsolatedProfile(flag) => {
                self.app_isolated = flag;
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
                        if let Some(icon) = image_handle(path) {
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
        }
        Task::none()
    }

    pub fn update_icon(&mut self, icon: Option<Icon>) {
        if let Some(icon) = icon {
            self.app_icon = icon.path.clone();
            self.selected_icon = Some(icon);
        }
    }

    fn icon_element(&self, icon: Option<Icon>) -> Element<Message> {
        let ico = if let Some(ico) = icon {
            match ico.icon {
                IconType::Raster(data) => widget::button::custom(widget::image(data))
                    .width(Length::Fixed(64.))
                    .height(Length::Fixed(64.))
                    .class(style::Button::Icon)
                    .on_press(Message::OpenIconPicker(self.app_url.clone())),

                IconType::Svg(data) => widget::button::custom(widget::svg(data))
                    .width(Length::Fixed(64.))
                    .height(Length::Fixed(64.))
                    .class(style::Button::Icon)
                    .on_press(Message::OpenIconPicker(self.app_url.clone())),
            }
        } else {
            widget::button::custom(widget::icon::from_name("folder-pictures-symbolic"))
                .width(Length::Fixed(64.))
                .height(Length::Fixed(64.))
                .class(style::Button::Icon)
                .on_press(Message::OpenIconPicker(self.app_url.clone()))
        };

        widget::container(ico).into()
    }

    pub fn view(&self) -> Element<Message> {
        widget::container(
            widget::column()
                .push(
                    widget::column()
                        .spacing(8)
                        .push(
                            widget::row()
                                .push(
                                    widget::container(
                                        self.icon_element(self.selected_icon.clone()),
                                    )
                                    .width(96.)
                                    .height(96.)
                                    .align_y(Vertical::Center),
                                )
                                .push(
                                    widget::container(
                                        widget::column()
                                            .push(widget::text::title1(&self.app_title))
                                            .push(widget::text::title4(format!(
                                                "{}: {}",
                                                self.app_categories
                                                    [self.category_idx.unwrap_or_default()],
                                                self.app_browsers
                                                    [self.browser_idx.unwrap_or_default()]
                                                .name
                                            ))),
                                    )
                                    .height(Length::Fixed(96.))
                                    .align_y(Vertical::Center),
                                ),
                        )
                        .push(
                            widget::column().push(widget::text(fl!("title"))).push(
                                widget::text_input("", &self.app_title)
                                    .on_input(Message::Title)
                                    .width(Length::Fill),
                            ),
                        )
                        .push(
                            widget::column().push(widget::text(fl!("url"))).push(
                                widget::row()
                                    .spacing(8)
                                    .push(
                                        widget::text_input("", &self.app_url)
                                            .on_input(Message::Url)
                                            .width(Length::Fill),
                                    )
                                    .push(
                                        widget::button::suggested(fl!("download-favicon"))
                                            .on_press_maybe(if url_valid(&self.app_url) {
                                                Some(Message::SearchFavicon)
                                            } else {
                                                None
                                            }),
                                    ),
                            ),
                        )
                        .push(
                            widget::column()
                                .push(widget::text(fl!("non-standard-arguments")))
                                .push(
                                    widget::text_input("", &self.app_parameters)
                                        .on_input(Message::Arguments)
                                        .width(Length::Fill),
                                ),
                        ),
                )
                .push(
                    widget::row()
                        .push(
                            widget::dropdown(
                                &self.app_categories,
                                self.category_idx,
                                Message::Category,
                            )
                            .width(Length::Fixed(200.)),
                        )
                        .push_maybe(if let Some(browser) = &self.app_browser {
                            match browser.model {
                                Some(BrowserModel::Firefox) | Some(BrowserModel::Zen) => Some(
                                    widget::toggler(self.app_navbar)
                                        .label(fl!("navbar"))
                                        .on_toggle(Message::Navbar)
                                        .spacing(12),
                                ),

                                _ => Some(
                                    widget::toggler(self.app_isolated)
                                        .label(fl!("isolated-profile"))
                                        .on_toggle(Message::IsolatedProfile)
                                        .spacing(12),
                                ),
                            }
                        } else {
                            None
                        })
                        .spacing(12),
                )
                .push(
                    widget::row()
                        .push(
                            widget::dropdown(&self.app_browsers, self.browser_idx, |idx| {
                                Message::Browser(idx)
                            })
                            .width(Length::Fixed(200.)),
                        )
                        .push(
                            widget::toggler(self.app_incognito)
                                .label(fl!("private-mode"))
                                .on_toggle(Message::Incognito)
                                .spacing(12),
                        )
                        .push(widget::horizontal_space())
                        .push(widget::button::suggested(fl!("create")).on_press(Message::Done))
                        .spacing(12),
                )
                .spacing(12),
        )
        .max_width(1000)
        .into()
    }
}
