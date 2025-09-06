use ashpd::desktop::file_chooser::{FileFilter, SelectedFiles};
use cosmic::{
    action::Action,
    iced::Length,
    task, theme,
    widget::{self},
    Element, Task,
};
use webapps::fl;

use crate::pages;

#[derive(Debug, Clone)]
pub enum Message {
    CustomIconsSearch(String),
    DownloadIconsPack,
    OpenIconPickerDialog,
    IconSearch,
    SetIcon(Option<webapps::Icon>),
}

#[derive(Debug, Clone, Default)]
pub struct IconPicker {
    pub icon_searching: String,
    pub icons: Vec<webapps::Icon>,
}

impl IconPicker {
    pub fn push_icon(&mut self, icon: webapps::Icon) {
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

                let name = self.icon_searching.clone().to_lowercase();

                return task::future(async {
                    pages::Message::IconsResult(webapps::find_icons(name).await)
                });
            }
            Message::SetIcon(icon) => return task::future(async { pages::Message::SetIcon(icon) }),
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let mut icons: Vec<Element<Message>> = Vec::new();

        for ico in self.icons.iter() {
            let btn = match ico.clone().icon {
                webapps::IconType::Raster(icon) => widget::button::custom(widget::image(icon))
                    .width(Length::Fixed(48.))
                    .height(Length::Fixed(48.))
                    .on_press(Message::SetIcon(Some(ico.clone())))
                    .class(theme::Button::Icon),
                webapps::IconType::Svg(icon) => widget::button::custom(widget::svg(icon))
                    .width(Length::Fixed(48.))
                    .height(Length::Fixed(48.))
                    .on_press(Message::SetIcon(Some(ico.clone())))
                    .class(theme::Button::Icon),
            };
            icons.push(btn.into());
        }

        let icons_input = widget::text_input(fl!("icon-name-to-find"), &self.icon_searching)
            .on_input(Message::CustomIconsSearch)
            .on_submit(|_| Message::IconSearch);
        let button = widget::button::standard(fl!("open")).on_press(Message::OpenIconPickerDialog);

        widget::column()
            .spacing(30)
            .push(
                widget::container(
                    widget::row()
                        .spacing(8)
                        .push(icons_input)
                        .push(button)
                        .push_maybe(if !webapps::icon_pack_installed() {
                            Some(
                                widget::button::standard(fl!("download"))
                                    .on_press(Message::DownloadIconsPack),
                            )
                        } else {
                            None
                        }),
                )
                .padding(8),
            )
            .push_maybe(if !icons.is_empty() {
                Some(
                    widget::container(widget::scrollable(widget::flex_row(icons)))
                        .height(Length::FillPortion(1)),
                )
            } else {
                None
            })
            .into()
    }
}
