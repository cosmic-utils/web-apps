use ashpd::desktop::file_chooser::{FileFilter, SelectedFiles};
use cosmic::{
    Element, Task,
    action::Action,
    iced::Length,
    task,
    widget::{self},
};
use webapps::{WebappIcon, fl};

use crate::pages;

#[derive(Debug, Clone)]
pub enum Message {
    CustomIconsSearch(String),
    DownloadIconsPack,
    OpenIconPickerDialog,
    IconSearch,
    SetIcon(Option<WebappIcon>),
}

#[derive(Debug, Clone, Default)]
pub struct IconPicker {
    pub icon_searching: String,
    pub icons: Vec<WebappIcon>,
}

impl IconPicker {
    pub fn push_icon(&mut self, icon: Option<WebappIcon>) {
        if let Some(webapp_icon) = icon {
            self.icons.push(webapp_icon);
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Action<pages::Message>> {
        match message {
            Message::CustomIconsSearch(input) => self.icon_searching = input,
            Message::DownloadIconsPack => return task::message(pages::Message::DownloaderStarted),
            Message::OpenIconPickerDialog => {
                return task::future(async move {
                    let result = SelectedFiles::open_file()
                        .title("Open icon")
                        .accept_label("Open")
                        .modal(true)
                        .multiple(false)
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
                            .map(|file| {
                                let mut file_path = file.as_str();
                                println!("file path: {}", file_path);

                                if file_path.starts_with("file://") {
                                    file_path =
                                        file_path.strip_prefix("file://").expect("removing prefix");
                                }

                                file_path.to_string()
                            })
                            .collect::<Vec<String>>();

                        pages::Message::OpenFileResult(files[0].clone())
                    } else {
                        pages::Message::None
                    }
                });
            }
            Message::IconSearch => {
                self.icons.clear();

                let name = self.icon_searching.clone().to_lowercase();

                return task::future(async {
                    pages::Message::IconsResult(webapps::find_icons(name).await)
                });
            }
            Message::SetIcon(icon) => {
                return Task::done(cosmic::Action::App(pages::Message::SetIcon(icon)));
            }
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let icons_input = widget::text_input(fl!("icon-name-to-find"), &self.icon_searching)
            .on_input(Message::CustomIconsSearch)
            .on_submit(|_| Message::IconSearch);
        let button = widget::button::standard(fl!("open")).on_press(Message::OpenIconPickerDialog);

        widget::Column::new()
            .spacing(30)
            .push(
                widget::container(
                    widget::Row::new()
                        .spacing(8)
                        .push(icons_input)
                        .push(button)
                        .push_maybe(
                            if !webapps::icon_pack_installed() && !webapps::is_flatpak() {
                                Some(
                                    widget::button::standard(fl!("download"))
                                        .on_press(Message::DownloadIconsPack),
                                )
                            } else {
                                None
                            },
                        ),
                )
                .padding(8),
            )
            .push_maybe(if !self.icons.is_empty() {
                Some(
                    widget::container(widget::scrollable(widget::flex_row(
                        self.icons
                            .iter()
                            .map(|icon| match icon.icon {
                                webapps::IconType::Raster => widget::button::custom(
                                    widget::icon::from_raster_bytes(icon.buffer.clone())
                                        .icon()
                                        .size(48),
                                )
                                .on_press(Message::SetIcon(Some(icon.clone())))
                                .class(cosmic::theme::Button::Icon),
                                webapps::IconType::Svg => widget::button::custom(
                                    widget::icon::from_svg_bytes(icon.buffer.clone())
                                        .icon()
                                        .size(48),
                                )
                                .on_press(Message::SetIcon(Some(icon.clone())))
                                .class(cosmic::theme::Button::Icon),
                            })
                            .fold(Vec::new(), |mut v, icon| {
                                v.push(icon.into());
                                v
                            }),
                    )))
                    .height(Length::FillPortion(1)),
                )
            } else {
                None
            })
            .into()
    }
}
