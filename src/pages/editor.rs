use std::sync::Arc;

use cosmic::{
    iced::{alignment::Vertical, futures::executor::block_on, Length},
    action::Action,
    style, task,
    widget::{self},
    Element, Task,
};
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    browser::{installed_browsers, Browser, BrowserModel},
    common::{self, image_handle, move_icon, url_valid, Icon, IconType},
    fl,
    launcher::{webapplauncher_is_valid, WebAppLauncher},
    pages,
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
    pub app_codename: String,
    pub app_title: String,
    pub app_url: String,
    pub app_icon: String,
    pub app_parameters: String,
    pub app_category: Category,
    pub app_browser: Option<Browser>,
    pub app_navbar: bool,
    pub app_incognito: bool,
    pub app_isolated: bool,
    pub selected_icon: Option<Icon>,
    pub browsers: Vec<Browser>,
    pub browser_idx: Option<usize>,
    pub categories: Vec<String>,
    pub category_idx: Option<usize>,
    //pub is_installed: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Arguments(String),
    Browser(usize),
    Category(usize),
    Done,
    Incognito(bool),
    IsolatedProfile(bool),
    //LaunchApp,
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

        let categories = Category::iter().map(|c| c.name()).collect::<Vec<String>>();

        AppEditor {
            app_codename: String::new(),
            app_title: String::new(),
            app_url: String::new(),
            app_icon: String::new(),
            app_parameters: String::new(),
            app_category: Category::default(),
            app_browser: browser,
            app_navbar: false,
            app_incognito: false,
            app_isolated: true,
            selected_icon: None,
            browsers,
            browser_idx: Some(0),
            categories,
            category_idx: Some(0),
            //is_installed: false,
        }
    }

    pub fn from(webapp_launcher: WebAppLauncher) -> Self {
        let category_idx = Category::iter().position(|c| c == webapp_launcher.category);
        let category = Category::from_index(category_idx.unwrap_or_default() as u8);
        let categories = Category::to_vec();

        let selected_icon = block_on(image_handle(webapp_launcher.icon.clone()));
        let browsers = installed_browsers();
        let browser_idx = browsers
            .iter()
            .position(|b| b.model == webapp_launcher.browser.model);

        Self {
            app_codename: webapp_launcher.codename,
            app_title: webapp_launcher.name,
            app_url: webapp_launcher.url,
            app_icon: webapp_launcher.icon,
            app_parameters: webapp_launcher.custom_parameters,
            app_category: category,
            app_browser: Some(webapp_launcher.browser),
            app_navbar: webapp_launcher.navbar,
            app_incognito: webapp_launcher.is_incognito,
            app_isolated: webapp_launcher.isolate_profile,
            selected_icon,
            browsers,
            browser_idx,
            categories,
            category_idx,
            //is_installed: installed,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Action<pages::Message>> {
        match message {
            Message::Arguments(args) => {
                self.app_parameters = args;
            }
            Message::Browser(idx) => {
                self.browser_idx = Some(idx);
                self.app_browser = Some(self.browsers[idx].clone());
            }
            Message::Category(idx) => {
                self.app_category = Category::from_index(idx as u8);
                self.category_idx = Some(idx);
            }
            Message::Done => {
                if self.app_codename.is_empty() {
                    self.app_codename = format!(
                        "{}{}",
                        &self.app_title.replace(' ', ""),
                        rng().random_range(1000..10000)
                    );
                }
                let icon_final_path = block_on(move_icon(&self.app_icon, &self.app_codename));

                if webapplauncher_is_valid(&icon_final_path, &self.app_title, &self.app_url) {
                    if let Some(browser) = &self.app_browser {
                        if let Some(entry) = &browser.entry {
                            let launcher = Arc::new(WebAppLauncher {
                                appid: entry.appid.clone(),
                                codename: self.app_codename.clone(),
                                browser: browser.clone(),
                                name: self.app_title.clone(),
                                icon: icon_final_path,
                                category: self.app_category.clone(),
                                url: self.app_url.clone(),
                                custom_parameters: self.app_parameters.clone(),
                                isolate_profile: self.app_isolated,
                                navbar: self.app_navbar,
                                is_incognito: self.app_incognito,
                            });

                            let arc_launcher = Arc::clone(&launcher);

                            return task::future(async move {
                                if arc_launcher.create().await.is_ok() {
                                    pages::Message::SaveLauncher(arc_launcher)
                                } else {
                                    pages::Message::None
                                }
                            });
                        };
                    }
                }
            }
            Message::Incognito(flag) => {
                self.app_incognito = flag;
            }
            Message::IsolatedProfile(flag) => {
                self.app_isolated = flag;
            }
            //Message::LaunchApp => {
            //let app_id = Arc::new(self.app_codename.clone());
            //let cloned_id = Arc::clone(&app_id);
            //return task::future(async move {
            //    launch_webapp(cloned_id).await.unwrap();
            //
            //    pages::Message::None
            //});
            //}
            Message::Navbar(flag) => {
                self.app_navbar = flag;
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
                .class(style::Button::Icon)
                .on_press(Message::OpenIconPicker(self.app_url.clone()))
        };

        widget::container(ico).into()
    }

    pub fn view(&self) -> Element<Message> {
        widget::container(
            widget::column()
                .spacing(24)
                .push(
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
                                    .push(widget::text::title1(&self.app_title))
                                    .push(widget::text::title4(format!(
                                        "{}: {}",
                                        self.app_category.name(),
                                        self.browsers[self.browser_idx.unwrap_or_default()].name
                                    ))),
                            )
                            .height(Length::Fixed(96.))
                            .align_y(Vertical::Center),
                        ),
                )
                .push(
                    widget::settings::section()
                        .add(
                            widget::text_input::inline_input(fl!("title"), &self.app_title)
                                .on_input(Message::Title),
                        )
                        .add(widget::settings::item_row(vec![
                            widget::text_input::inline_input(fl!("url"), &self.app_url)
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
                            widget::dropdown(&self.browsers, self.browser_idx, Message::Browser),
                        ))
                        .add(
                            widget::text_input::inline_input(
                                fl!("non-standard-arguments"),
                                &self.app_parameters,
                            )
                            .on_input(Message::Arguments),
                        )
                        .add_maybe(if let Some(browser) = &self.app_browser {
                            match browser.model {
                                Some(BrowserModel::Firefox)
                                | Some(BrowserModel::Zen)
                                | Some(BrowserModel::Librewolf)
                                | Some(BrowserModel::Waterfox) => widget::settings::item(
                                    fl!("navbar"),
                                    widget::toggler(self.app_navbar).on_toggle(Message::Navbar),
                                )
                                .into(),
                                _ => None,
                            }
                        } else {
                            None
                        })
                        .add(widget::settings::item(
                            fl!("isolated-profile"),
                            widget::toggler(self.app_isolated).on_toggle_maybe(
                                if let Some(browser) = &self.app_browser {
                                    match browser.model {
                                        Some(BrowserModel::Firefox)
                                        | Some(BrowserModel::Zen)
                                        | Some(BrowserModel::Librewolf)
                                        | Some(BrowserModel::Waterfox) => {
                                            if self.app_navbar {
                                                Message::IsolatedProfile.into()
                                            } else {
                                                None
                                            }
                                        }
                                        _ => Message::IsolatedProfile.into(),
                                    }
                                } else {
                                    None
                                },
                            ),
                        ))
                        .add(widget::settings::item(
                            fl!("private-mode"),
                            widget::toggler(self.app_incognito).on_toggle(Message::Incognito),
                        )),
                )
                .push(
                    widget::row()
                        .spacing(8)
                        .push(widget::horizontal_space())
                        //.push_maybe(if self.is_installed {
                        //    Some(
                        //        widget::button::standard(fl!("run-app"))
                        //            .on_press(Message::LaunchApp),
                        //    )
                        //} else {
                        //    None
                        //})
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
        .max_width(1000)
        .into()
    }
}
