use ashpd::desktop::file_chooser::{FileFilter, SelectedFiles};
use cosmic::{
    action::Action,
    cosmic_theme,
    iced::Length,
    task, theme,
    widget::{self},
    Element, Task,
};
use webapps::fl;

use crate::{
    common::{self, find_icons, get_icon_name_from_url, Icon},
    pages,
};

#[derive(Debug, Clone)]
pub enum Message {
    CustomIconsSearch(String),
    DownloadIconsPack,
    OpenIconPickerDialog,
    IconSearch,
    SetIcon(Option<Icon>),
}

#[derive(Debug, Clone)]
pub struct IconPicker {
    pub app_url: String,
    pub icon_searching: String,
    pub icons: Vec<Icon>,
}

impl IconPicker {
    pub fn new(app_url: String) -> Self {
        Self {
            app_url,
            icon_searching: String::new(),
            icons: Vec::new(),
        }
    }

    pub fn push_icon(&mut self, icon: Icon) {
        self.icons.push(icon);
    }

    pub fn update(&mut self, message: Message) -> Task<Action<pages::Message>> {
        match message {
            Message::CustomIconsSearch(input) => self.icon_searching = input,
            Message::DownloadIconsPack => return task::message(pages::Message::DownloaderStarted),
            Message::OpenIconPickerDialog => {
                return task::future(async move {
                    let result = SelectedFiles::open_file()
                        .title("Open multiple images")
                        .accept_label("Open")
                        .modal(true)
                        .multiple(true)
                        .filter(FileFilter::new("PNG Image").glob("*.png"))
                        .filter(FileFilter::new("SVG Images").glob("*.svg"))
                        .send()
                        .await
                        .unwrap()
                        .response();

                    if let Ok(result) = result {
                        let files = result
                            .uris()
                            .iter()
                            .map(|file| file.path().to_string())
                            .collect::<Vec<String>>();

                        pages::Message::OpenFileResult(files)
                    } else {
                        pages::Message::None
                    }
                })
            }
            Message::IconSearch => {
                self.icons.clear();

                let name = match self.icon_searching.is_empty() {
                    true => get_icon_name_from_url(&self.app_url),
                    false => self.icon_searching.clone(),
                };
                let app_url = self.app_url.clone();

                return task::future(async {
                    pages::Message::IconsResult(find_icons(name, app_url).await)
                });
            }
            Message::SetIcon(icon) => return task::future(async { pages::Message::SetIcon(icon) }),
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let mut icons: Vec<Element<Message>> = Vec::new();

        for ico in self.icons.iter() {
            let btn = match ico.clone().icon {
                common::IconType::Raster(icon) => widget::button::custom(widget::image(icon))
                    .width(Length::Fixed(48.))
                    .height(Length::Fixed(48.))
                    .on_press(Message::SetIcon(Some(ico.clone())))
                    .class(theme::Button::Icon),
                common::IconType::Svg(icon) => widget::button::custom(widget::svg(icon))
                    .width(Length::Fixed(48.))
                    .height(Length::Fixed(48.))
                    .on_press(Message::SetIcon(Some(ico.clone())))
                    .class(theme::Button::Icon),
            };
            icons.push(btn.into());
        }

        let mut elements: Vec<Element<Message>> = vec![
            widget::text_input::inline_input(fl!("icon-name-to-find"), &self.icon_searching)
                .on_input(Message::CustomIconsSearch)
                .on_submit(|_| Message::IconSearch)
                .into(),
            widget::button::standard(fl!("open"))
                .on_press(Message::OpenIconPickerDialog)
                .into(),
        ];
        if !crate::icon_pack_installed() {
            elements.push(
                widget::button::standard(fl!("download"))
                    .on_press(Message::DownloadIconsPack)
                    .into(),
            )
        }
        widget::settings::section()
            .add(widget::settings::item_row(elements))
            .add_maybe(if !icons.is_empty() {
                Some(
                    widget::container(
                        widget::scrollable(widget::settings::flex_item_row(icons))
                            .spacing(space_xxs),
                    )
                    .max_height(600.0),
                )
            } else {
                None
            })
            .into()
    }
}
