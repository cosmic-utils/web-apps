use std::path::PathBuf;

use cosmic::{
    iced::{Alignment, Length},
    iced_widget::{PickList, Scrollable},
    widget::{text, toggler, Button, Column, Container, Row, TextInput},
    Element,
};
use url::Url;
use xdg::BaseDirectories;

use crate::{
    common::{get_supported_browsers, get_webapps, Browser, WebAppLauncher},
    gui::{Buttons, Message},
    iconpicker,
};

#[derive(Debug, Clone)]
pub struct Wam {
    pub app_codename: Option<String>,
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
    pub selected_icon: Option<iconpicker::Icon>,
    pub app_browsers: Vec<Browser>,
    pub edit_mode: bool,
    pub launcher: Option<WebAppLauncher>,
    pub app_base_dir: PathBuf,
}

impl Wam {
    pub fn new() -> Self {
        let browsers = get_supported_browsers();
        let browser = &browsers[0];

        let base_dir = BaseDirectories::new().expect("cant follow base directories");
        let local_share = base_dir.get_data_home();
        let wam_rust_path = local_share.join("wam-rust");

        Wam {
            app_codename: None,
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
            selected_icon: None,
            app_browsers: browsers,
            edit_mode: false,
            launcher: None,
            app_base_dir: wam_rust_path,
        }
    }

    fn icon_picker_icon(&self, icon: Option<iconpicker::Icon>) -> Element<Message> {
        let ico = if let Some(ico) = icon {
            match ico.icon {
                iconpicker::IconType::Raster(data) => Button::new(
                    cosmic::widget::image(data)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .on_press(Message::OpenIconPicker)
                .width(Length::Fixed(96.))
                .height(Length::Fixed(96.)),
                iconpicker::IconType::Svg(data) => Button::new(
                    cosmic::widget::svg(data)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .on_press(Message::OpenIconPicker)
                .width(Length::Fixed(96.))
                .height(Length::Fixed(96.)),
            }
        } else {
            let default_ico = &self.app_base_dir.join("icons/moleskine-icon.svg");
            let default_ico = default_ico.to_str().expect("cant find needed icon");
            let default_icon_path = String::from(default_ico);
            let handler = cosmic::widget::svg::Handle::from_path(default_icon_path);
            let default = cosmic::widget::svg(handler);

            Button::new(default)
                .width(Length::Fill)
                .height(Length::Fill)
                .on_press(Message::OpenIconPicker)
                .width(Length::Fixed(96.))
                .height(Length::Fixed(96.))
        };

        Container::new(ico).into()
    }

    pub fn view(&self) -> Element<Message> {
        let app_title = TextInput::new("Title", &self.app_title)
            .on_input(Message::Title)
            .padding(10)
            .width(Length::Fixed(340.));
        let app_url = TextInput::new("URL", &self.app_url)
            .on_input(Message::Url)
            .padding(10)
            .width(Length::Fixed(340.));

        let mut col = Column::new().spacing(14);
        col = col.push(app_title);
        col = col.push(app_url);

        let search_ico = &self.app_base_dir.join("icons/search.svg");
        let search_ico = search_ico.to_str().expect("cant find needed search icon");

        let dl_btn = Button::new(
            cosmic::widget::svg(cosmic::widget::svg::Handle::from_path(search_ico))
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Message::Clicked(Buttons::SearchFavicon))
        .width(Length::Fixed(96.))
        .height(Length::Fixed(96.));

        let icon = self.icon_picker_icon(self.selected_icon.clone());

        let icon = Button::new(icon)
            .width(Length::Fill)
            .height(Length::Fill)
            .width(Length::Fixed(96.))
            .height(Length::Fixed(96.))
            .on_press(Message::OpenIconPicker);

        let mut row = Row::new().spacing(12).width(Length::Fill);

        row = row.push(col);
        row = row.push(dl_btn);
        row = row.push(icon);

        let app_arguments = TextInput::new("Non-standard arguments", &self.app_parameters)
            .on_input(Message::Arguments)
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

        let category = PickList::new(
            categories.to_vec(),
            Some(self.app_category.clone()),
            Message::Category,
        )
        .width(Length::Fixed(200.))
        .padding(10);

        let browser_specific = match self.app_browser._type {
            crate::common::BrowserType::Firefox => {
                toggler(String::from("Nav Bar"), self.app_navbar, |b| {
                    Message::Clicked(Buttons::Navbar(b))
                })
                .width(Length::Fill)
            }
            crate::common::BrowserType::FirefoxFlatpak => {
                toggler(String::from("Nav Bar"), self.app_navbar, |b| {
                    Message::Clicked(Buttons::Navbar(b))
                })
                .width(Length::Fill)
            }
            crate::common::BrowserType::Librewolf => {
                toggler(String::from("Nav Bar"), self.app_navbar, |b| {
                    Message::Clicked(Buttons::Navbar(b))
                })
                .width(Length::Fill)
            }
            crate::common::BrowserType::WaterfoxFlatpak => {
                toggler(String::from("Nav Bar"), self.app_navbar, |b| {
                    Message::Clicked(Buttons::Navbar(b))
                })
                .width(Length::Fill)
            }
            _ => toggler(String::from("Isolated Profile"), self.app_isolated, |b| {
                Message::Clicked(Buttons::IsolatedProfile(b))
            })
            .width(Length::Fill),
        };

        let incognito = toggler(String::from("Private Mode"), self.app_incognito, |b| {
            Message::Clicked(Buttons::Incognito(b))
        })
        .width(Length::Fill);

        let mut cat_row = Row::new().spacing(20).align_items(Alignment::Center);
        cat_row = cat_row.push(category);
        cat_row = cat_row.push(incognito);
        cat_row = cat_row.push(browser_specific);

        let app_browsers = PickList::new(
            self.app_browsers.clone(),
            Some(self.app_browser.clone()),
            Message::Browser,
        )
        .width(Length::Fixed(200.))
        .padding(10);

        let app_done = Button::new("Done")
            .on_press(Message::Result)
            .width(Length::Fill)
            .padding(10);

        let mut browsers_row = Row::new().spacing(20);
        browsers_row = browsers_row.push(app_browsers);
        browsers_row = browsers_row.push(app_done);

        let mut app_list = Column::new().spacing(10);
        let webapps = get_webapps();

        for app in webapps.iter() {
            match app {
                Ok(data) => {
                    let edit = Button::new("Edit")
                        .on_press(Message::Clicked(Buttons::Edit(data.clone())))
                        .width(Length::Fixed(90.));
                    let delete = Button::new("Delete")
                        .on_press(Message::Clicked(Buttons::Delete(data.clone())))
                        .width(Length::Fixed(90.));

                    let host = Url::parse(&data.url).expect("cant parse url");
                    let host = host.host().unwrap();

                    let name = Button::new(text(data.name.clone())).width(Length::FillPortion(2));
                    let url = Button::new(text(host.to_string())).width(Length::FillPortion(3));

                    let mut row = Row::new().spacing(10).align_items(Alignment::Center);
                    row = row.push(edit);
                    row = row.push(delete);
                    row = row.push(name);
                    row = row.push(url);
                    app_list = app_list.push(row);
                }
                Err(e) => tracing::error!("Error reading web app: {}", e),
            }
        }

        let mut installed = Column::new();

        if !webapps.is_empty() {
            installed = installed.push(text("INSTALLED").size(26.)).spacing(10);

            let scrollable_list = Scrollable::new(app_list).width(Length::Fill);

            installed = installed.push(scrollable_list);
        }

        let mut col = Column::new().spacing(20);
        col = col.push(row);
        col = col.push(app_arguments);
        col = col.push(cat_row);
        col = col.push(browsers_row);
        let mut col2 = Column::new().spacing(50);
        col2 = col2.push(col);
        col2 = col2.push(installed);

        Container::new(col2).padding(30).into()
    }
}
