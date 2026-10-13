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
        widget::ListColumn::new()
            .add(widget::container(
                widget::Row::new()
                    .push(
                        widget::inline_input(fl!("icon-name-to-find"), &self.icon_searching)
                            .on_input(Message::CustomIconsSearch)
                            .on_submit(|_| Message::IconSearch),
                    )
                    .push_maybe(
                        if !webapps::icon_pack_installed() && !webapps::is_flatpak() {
                            Some(
                                widget::button::link(fl!("download"))
                                    .on_press(Message::DownloadIconsPack),
                            )
                        } else {
                            None
                        },
                    ),
            ))
            .add(if !self.icons.is_empty() {
                Some(
                    widget::container(widget::scrollable(widget::flex_row(
                        self.icons
                            .iter()
                            .map(|icon| match icon.icon {
                                webapps::IconType::Raster => widget::button::custom(
                                    widget::icon::from_raster_bytes(icon.buffer.clone())
                                        .icon()
                                        .size(42),
                                )
                                .on_press(Message::SetIcon(Some(icon.clone())))
                                .class(cosmic::theme::Button::Icon),
                                webapps::IconType::Svg => widget::button::custom(
                                    widget::icon::from_svg_bytes(icon.buffer.clone())
                                        .icon()
                                        .size(42),
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
