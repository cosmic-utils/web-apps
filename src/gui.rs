use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, image, pick_list, row, scrollable, svg, text, text_input, toggler,
        Container, Row,
    },
    Alignment, Application, Command, Length,
};
use iced_aw::{modal, Card, Wrap};

use crate::common::{
    find_icons, get_icon_name_from_url, get_supported_browsers, get_webapps, image_from_memory,
    move_icon, svg_from_memory, Browser, WebAppLauncher,
};

#[derive(Debug, Clone)]
pub enum Buttons {
    SearchFavicon,
    Favicon(String),
    Edit(Box<WebAppLauncher>),
    Delete(Box<WebAppLauncher>),
    Navbar(bool),
    Incognito(bool),
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    PushIcon(Icon),
    FoundIcons(Vec<String>),
    SetIcon(Icon),
    // modal
    OpenModal,
    CloseModal,
    CancelButtonPressed,
    // common
    Result,
    Clicked(Buttons),
    Title(String),
    Url(String),
    Arguments(String),
    Browser(Browser),
    Category(String),
    ErrorLoadingIcon,
    SelectIcon(Icon),
}

#[derive(Debug, Clone)]
pub enum IconType {
    Raster(image::Handle),
    Svg(svg::Handle),
}

#[derive(Debug, Clone)]
pub struct Icon {
    pub icon: IconType,
    pub path: String,
}

impl Icon {
    pub fn new(icon: IconType, path: String) -> Self {
        Self { icon, path }
    }
}

pub struct Wam {
    pub icons_paths: Vec<String>,
    pub icons: Option<Vec<Icon>>,
    pub app_title: String,
    pub app_url: String,
    pub app_icon: String,
    pub app_parameters: String,
    pub app_category: String,
    pub app_browser_name: String,
    pub app_browser: Browser,
    pub app_navbar: bool,
    pub app_incognito: bool,
    pub app_isolated: bool,
    show_modal: bool,
    selected_icon: Option<Icon>,
    app_browsers: Vec<Browser>,
    edit_mode: bool,
    launcher: Option<Box<WebAppLauncher>>,
}

impl Application for Wam {
    type Executor = iced::executor::Default;

    type Message = AppMessage;

    type Theme = iced::Theme;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let browsers = get_supported_browsers();
        let browser = &browsers[0];

        (
            Wam {
                icons_paths: Vec::new(),
                icons: Some(Vec::new()),
                app_title: String::new(),
                app_url: String::new(),
                app_icon: String::new(),
                app_parameters: String::new(),
                app_category: String::from("Web"),
                app_browser_name: String::from("Browser"),
                app_browser: browser.clone(),
                app_navbar: false,
                app_incognito: false,
                app_isolated: true,
                show_modal: false,
                selected_icon: None,
                app_browsers: browsers,
                edit_mode: false,
                launcher: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("WAM Rust - Web App Manager written in Rust and Iced rs.")
    }

    fn theme(&self) -> Self::Theme {
        iced::Theme::Dark
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            AppMessage::Title(text) => {
                self.app_title = text;

                Command::none()
            }
            AppMessage::Url(url) => {
                self.app_url = url;

                Command::none()
            }
            AppMessage::Arguments(args) => {
                self.app_parameters = args;

                Command::none()
            }
            AppMessage::PushIcon(icon) => {
                if let Some(vec) = self.icons.as_mut() {
                    vec.push(icon)
                }

                Command::none()
            }
            AppMessage::FoundIcons(result) => {
                if !result.is_empty() {
                    let commands: Vec<Command<AppMessage>> = result
                        .into_iter()
                        .map(|path| {
                            let is_svg = path.ends_with(".svg");

                            match is_svg {
                                true => {
                                    Command::perform(svg_from_memory(path), |result| match result {
                                        Ok(icon) => AppMessage::PushIcon(icon),
                                        Err(_) => AppMessage::ErrorLoadingIcon,
                                    })
                                }
                                false => {
                                    Command::perform(image_from_memory(path), |result| match result
                                    {
                                        Ok(icon) => AppMessage::PushIcon(icon),
                                        Err(_) => AppMessage::ErrorLoadingIcon,
                                    })
                                }
                            }
                        })
                        .collect();

                    Command::batch(commands)
                } else {
                    Command::none()
                }
            }
            AppMessage::Clicked(btn) => {
                match btn {
                    Buttons::SearchFavicon => {
                        if let Some(icons) = self.icons.as_mut() {
                            icons.clear()
                        };

                        if !self.app_url.is_empty() {
                            let url = self.app_url.clone();
                            let to_find = get_icon_name_from_url(url.clone());

                            Command::perform(find_icons(to_find, url), |icons| {
                                AppMessage::FoundIcons(icons)
                            })
                        } else {
                            Command::none()
                        }
                    }
                    Buttons::Favicon(path) => {
                        let is_svg = path.ends_with(".svg");

                        match is_svg {
                            true => {
                                Command::perform(svg_from_memory(path), |result| match result {
                                    Ok(icon) => AppMessage::SetIcon(icon),
                                    Err(_) => AppMessage::ErrorLoadingIcon,
                                })
                            }
                            false => {
                                Command::perform(image_from_memory(path), |result| match result {
                                    Ok(icon) => AppMessage::SetIcon(icon),
                                    Err(_) => AppMessage::ErrorLoadingIcon,
                                })
                            }
                        }
                    }
                    Buttons::Edit(launcher) => {
                        self.edit_mode = true;
                        self.launcher = Some(launcher.clone());

                        self.app_title = launcher.name;
                        self.app_url = launcher.url;
                        self.app_icon = launcher.icon.clone();
                        self.app_parameters = launcher.custom_parameters;
                        self.app_category = launcher.category;
                        self.app_browser = Browser::web_browser(launcher.web_browser.name)
                            .expect("browser not found");
                        self.app_navbar = launcher.navbar;
                        self.app_incognito = launcher.is_incognito;

                        let is_svg = launcher.icon.ends_with(".svg");

                        match is_svg {
                            true => Command::perform(svg_from_memory(launcher.icon), |result| {
                                match result {
                                    Ok(icon) => AppMessage::SetIcon(icon),
                                    Err(_) => AppMessage::ErrorLoadingIcon,
                                }
                            }),
                            false => Command::perform(image_from_memory(launcher.icon), |result| {
                                match result {
                                    Ok(icon) => AppMessage::SetIcon(icon),
                                    Err(_) => AppMessage::ErrorLoadingIcon,
                                }
                            }),
                        }
                    }
                    Buttons::Delete(launcher) => {
                        let _ = launcher.delete();

                        Command::none()
                    }
                    Buttons::Navbar(selected) => {
                        self.app_navbar = selected;

                        Command::none()
                    }
                    Buttons::Incognito(selected) => {
                        self.app_incognito = selected;

                        Command::none()
                    }
                }
            }
            AppMessage::Browser(browser) => {
                self.app_browser = browser;

                Command::none()
            }
            AppMessage::Result => {
                let launcher = if let Some(launcher) = self.launcher.to_owned() {
                    let _ = launcher.delete();
                    Box::new(WebAppLauncher::new(
                        self.app_title.clone(),
                        self.app_url.clone(),
                        self.app_icon.clone(),
                        self.app_category.clone(),
                        self.app_browser.clone(),
                        self.app_parameters.clone(),
                        self.app_isolated,
                        self.app_navbar,
                        self.app_incognito,
                    ))
                } else {
                    Box::new(WebAppLauncher::new(
                        self.app_title.clone(),
                        self.app_url.clone(),
                        self.app_icon.clone(),
                        self.app_category.clone(),
                        self.app_browser.clone(),
                        self.app_parameters.clone(),
                        self.app_isolated,
                        self.app_navbar,
                        self.app_incognito,
                    ))
                };

                if launcher.is_valid {
                    let _ = launcher.create();
                }

                Command::none()
            }
            AppMessage::Category(category) => {
                self.app_category = category;

                Command::none()
            }
            AppMessage::OpenModal => {
                self.show_modal = true;

                Command::none()
            }
            AppMessage::CloseModal => {
                self.show_modal = false;

                Command::none()
            }
            AppMessage::CancelButtonPressed => {
                self.show_modal = false;

                Command::none()
            }
            AppMessage::ErrorLoadingIcon => Command::none(),
            AppMessage::SetIcon(icon) => {
                self.show_modal = false;

                let path = icon.path;

                if let Ok(saved) = move_icon(path, self.app_title.clone()) {
                    self.app_icon = saved.clone();

                    if saved.ends_with(".svg") {
                        Command::perform(svg_from_memory(saved), |result| {
                            AppMessage::SelectIcon(result.unwrap())
                        })
                    } else {
                        Command::perform(image_from_memory(saved), |result| {
                            AppMessage::SelectIcon(result.unwrap())
                        })
                    }
                } else {
                    Command::none()
                }
            }
            AppMessage::SelectIcon(ico) => {
                self.selected_icon = Some(ico);

                Command::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let app_title = text_input("Title", &self.app_title)
            .on_input(AppMessage::Title)
            .padding(10)
            .width(Length::Fixed(340.));
        let app_url = text_input("URL", &self.app_url)
            .on_input(AppMessage::Url)
            .padding(10)
            .width(Length::Fixed(340.));

        let col = column![app_title, app_url].spacing(14);

        let search_ico = include_bytes!("../assets/icons/search.svg");
        let dl_btn = button(svg(svg::Handle::from_memory(search_ico.to_vec())))
            .on_press(AppMessage::Clicked(Buttons::SearchFavicon))
            .width(Length::Fixed(96.))
            .height(Length::Fixed(96.));

        let icons = self.icons.clone().unwrap();

        let icon = if !icons.is_empty() || !self.app_icon.is_empty() {
            match self.selected_icon.clone() {
                Some(data) => match data.icon {
                    IconType::Raster(data) => button(image(data))
                        .on_press(AppMessage::OpenModal)
                        .width(Length::Fixed(96.))
                        .height(Length::Fixed(96.)),
                    IconType::Svg(data) => button(svg(data))
                        .on_press(AppMessage::OpenModal)
                        .width(Length::Fixed(96.))
                        .height(Length::Fixed(96.)),
                },
                None => button("")
                    .on_press(AppMessage::OpenModal)
                    .width(Length::Fixed(96.))
                    .height(Length::Fixed(96.)),
            }
        } else {
            button("")
                .width(Length::Fixed(96.))
                .height(Length::Fixed(96.))
        };
        let row = row![col, dl_btn, icon].spacing(12).width(Length::Fill);

        let app_arguments = text_input("Non-standard arguments", &self.app_parameters)
            .on_input(AppMessage::Arguments)
            .padding(10)
            .width(Length::Fill);

        let categories = [
            String::from("Web"),
            String::from("Accesories"),
            String::from("Education"),
            String::from("Games"),
            String::from("Graphics"),
            String::from("Internet"),
            String::from("Office"),
            String::from("Programming"),
            String::from("Sound & Video"),
        ];

        let category = pick_list(
            categories.to_vec(),
            Some(self.app_category.clone()),
            AppMessage::Category,
        )
        .width(Length::Fill)
        .padding(10);

        let navbar = toggler(String::from("Nav Bar"), self.app_navbar, |b| {
            AppMessage::Clicked(Buttons::Navbar(b))
        })
        .width(Length::Fill);

        let incognito = toggler(String::from("Private Mode"), self.app_incognito, |b| {
            AppMessage::Clicked(Buttons::Incognito(b))
        })
        .width(Length::Fill);

        let cat_row = row![category, incognito, navbar]
            .align_items(Alignment::Center)
            .spacing(20);

        let app_browsers = pick_list(
            self.app_browsers.clone(),
            Some(self.app_browser.clone()),
            AppMessage::Browser,
        )
        .width(Length::Fill)
        .padding(10);

        let app_done = button("Done")
            .on_press(AppMessage::Result)
            .width(Length::Fill)
            .padding(10);

        let mut app_list = column!().spacing(10);
        let webapps = get_webapps();

        for app in webapps.iter() {
            match app {
                Ok(data) => {
                    let edit = button("Edit")
                        .on_press(AppMessage::Clicked(Buttons::Edit(Box::new(data.clone()))))
                        .padding(10);
                    let delete = button("Delete")
                        .on_press(AppMessage::Clicked(Buttons::Delete(Box::new(data.clone()))))
                        .padding(10);
                    let name = text(&data.name).size(26.);

                    let row = row![edit, delete, name]
                        .spacing(10)
                        .align_items(Alignment::Center);
                    app_list = app_list.push(row);
                }
                Err(e) => tracing::error!("Error reading web app: {}", e),
            }
        }

        let mut installed = column![];

        if !webapps.is_empty() {
            installed = installed.push(text("Installed").size(32.));

            let scrollable_list = scrollable(app_list).width(Length::Fill);

            installed = installed.push(scrollable_list);
        }

        let col = column![
            row,
            app_arguments,
            cat_row,
            app_browsers,
            app_done,
            installed,
        ]
        .spacing(24);

        let underlay = Container::new(col).padding(30);

        let overlay = if self.show_modal {
            Some(
                Card::new(text("Icon Picker"), icons_container(self.icons.clone()))
                    .foot(
                        Row::new().spacing(10).padding(5).width(Length::Fill).push(
                            button(text("Cancel").horizontal_alignment(Horizontal::Center))
                                .width(Length::Fill)
                                .on_press(AppMessage::CancelButtonPressed),
                        ),
                    )
                    .max_width(500.0)
                    .max_height(600.0)
                    .height(Length::Shrink)
                    .on_close(AppMessage::CloseModal),
            )
        } else {
            None
        };

        modal(underlay, overlay)
            .backdrop(AppMessage::CloseModal)
            .on_esc(AppMessage::CloseModal)
            .align_y(Vertical::Center)
            .into()
    }
}

fn icons_container(icons: Option<Vec<Icon>>) -> iced::Element<'static, AppMessage> {
    let mut container = Wrap::new().max_width(500.);

    if icons.is_some() {
        for ico in icons.unwrap().iter() {
            let btn = match ico.clone().icon {
                IconType::Raster(icon) => button(image(icon))
                    .width(Length::Fixed(96.))
                    .height(Length::Fixed(96.))
                    .on_press(AppMessage::Clicked(Buttons::Favicon(ico.path.clone()))),
                IconType::Svg(icon) => button(svg(icon))
                    .width(Length::Fixed(96.))
                    .height(Length::Fixed(96.))
                    .on_press(AppMessage::Clicked(Buttons::Favicon(ico.path.clone()))),
            };
            container = container.push(btn);
        }
    }

    scrollable(container).into()
}
