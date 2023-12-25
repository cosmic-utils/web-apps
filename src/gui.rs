use iced::{
    widget::{button, column, pick_list, row, scrollable, text_input, Container},
    Application, Command, Length,
};

#[derive(Debug, Clone, Copy)]
pub enum Buttons {
    Favicon,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    Result,
    Clicked(Buttons),
    Title(String),
    Url(String),
    Arguments(String),
    Browser(&'static str),
}

pub struct Wam {
    pub app_title: String,
    pub app_url: String,
    pub app_arguments: String,
    pub app_browser: &'static str,
}

impl Application for Wam {
    type Executor = iced::executor::Default;

    type Message = AppMessage;

    type Theme = iced::Theme;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Wam {
                app_title: String::new(),
                app_url: String::new(),
                app_arguments: String::new(),
                app_browser: "Browser",
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("WAM Rust - Web app manager written in Rust and Iced rs.")
    }

    fn theme(&self) -> Self::Theme {
        iced::Theme::Dark
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            AppMessage::Title(text) => {
                self.app_title = text;

                println!("{}", self.app_title);

                Command::none()
            }
            AppMessage::Url(url) => {
                self.app_url = url;

                println!("{}", self.app_url);

                Command::none()
            }
            AppMessage::Arguments(args) => {
                self.app_arguments = args;

                println!("{}", self.app_arguments);

                Command::none()
            }
            AppMessage::Clicked(btn) => match btn {
                Buttons::Favicon => Command::none(),
            },
            AppMessage::Browser(browser) => {
                self.app_browser = &browser;

                Command::none()
            }
            AppMessage::Result => Command::none(),
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

        let dl_btn = button("DL FAV")
            .on_press(AppMessage::Clicked(Buttons::Favicon))
            .width(Length::Fixed(96.))
            .height(Length::Fixed(96.));
        let fav_btn = button("FAV BTN")
            .on_press(AppMessage::Clicked(Buttons::Favicon))
            .width(Length::Fixed(96.))
            .height(Length::Fixed(96.));

        let row = row![col, dl_btn, fav_btn].spacing(12).width(Length::Fill);

        let app_arguments = text_input("Non-standard arguments", &self.app_arguments)
            .on_input(AppMessage::Arguments)
            .padding(10)
            .width(Length::Fill);

        let browsers = vec!["Firefox", "Firefox (flatpak)", "Chrome"];

        let app_browsers = pick_list(browsers.clone(), Some(self.app_browser), |browser| {
            AppMessage::Browser(browser)
        })
        .width(Length::Fill)
        .padding(10);

        let app_done = button("Done")
            .on_press(AppMessage::Result)
            .width(Length::Fill)
            .padding(10);

        let mut app_list = column!().spacing(10);

        for _ in 1..20 {
            app_list = app_list.push(button("Appkaaaa").width(Length::Fill).padding(10));
        }

        let scrollable_list = scrollable(app_list).width(Length::Fill);

        let col = column![row, app_arguments, app_browsers, app_done, scrollable_list].spacing(14);

        Container::new(col).padding(30).into()
    }
}
