use clap::Parser;
use cosmic::desktop::fde::get_languages_from_env;
use serde::{Deserialize, Serialize};
use std::{ffi::OsStr, fmt::Display, path::PathBuf, str::FromStr, vec::IntoIter};

pub mod localize;

lazy_static::lazy_static! {
    pub static ref LOCALES: Vec<String> = get_languages_from_env();
}

pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
pub const CONFIG_VERSION: u64 = 1;
pub const APP_ID: &str = "dev.heppen.QuickWebApps.Manager";
pub const APP_ICON: &[u8] =
    include_bytes!("../res/icons/hicolor/256x256/apps/dev.heppen.webapps.png");
pub const WEBVIEW_APP_ID: &str = "dev.heppen.QuickWebApp";
pub const MOBILE_UA: &str = "Mozilla/5.0 (Android 16; Mobile; rv:68.0) Gecko/68.0 Firefox/142.0";

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct WebviewArgs {
    pub app_id: String,
    #[clap(short, long)]
    pub window_title: String,
    #[clap(short, long)]
    pub url: String,
    #[clap(short, long)]
    pub profile: Option<PathBuf>,
    #[clap(short, long)]
    pub window_size: Option<WindowSize>,
    #[clap(short, long)]
    pub window_decorations: Option<bool>,
    #[clap(short, long)]
    pub private_mode: Option<bool>,
    #[clap(short, long)]
    pub try_simulate_mobile: Option<bool>,
}

pub struct WebViewArg(pub String);

impl Display for WebViewArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<OsStr> for WebViewArg {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl IntoIterator for WebviewArgs {
    type Item = WebViewArg;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut args: Vec<WebViewArg> = Vec::new();

        args.push(WebViewArg("--window-title".to_string()));
        args.push(WebViewArg(self.window_title));
        args.push(WebViewArg("--url".to_string()));
        args.push(WebViewArg(self.url));

        if self.profile.is_some() {
            args.push(WebViewArg("--profile".to_string()));
            args.push(WebViewArg(
                self.profile
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            ));
        }

        if self.window_size.is_some() {
            args.push(WebViewArg("--window-size".to_string()));
            args.push(WebViewArg(self.window_size.unwrap_or_default().to_string()));
        }

        if self.window_decorations.is_some() {
            args.push(WebViewArg("--window-decorations".to_string()));
            args.push(WebViewArg(
                self.window_decorations.unwrap_or_default().to_string(),
            ));
        }

        if self.private_mode.is_some() {
            args.push(WebViewArg("--private-mode".to_string()));
            args.push(WebViewArg(
                self.private_mode.unwrap_or_default().to_string(),
            ));
        }

        if self.try_simulate_mobile.is_some() {
            args.push(WebViewArg("--try-simulate-mobile".to_string()));
            args.push(WebViewArg(
                self.try_simulate_mobile.unwrap_or_default().to_string(),
            ));
        }

        args.push(WebViewArg(self.app_id));

        args.into_iter()
    }
}

impl Display for WebviewArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let exec: String = self
            .clone()
            .into_iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        write!(f, "{}", exec)
    }
}

impl WebviewArgs {
    pub fn get_exec(&self, from_flatpak: bool) -> String {
        let mut exec = String::new();

        if from_flatpak {
            exec.push_str("flatpak run --command=quick-webapps-webview dev.heppen.webapps");
        } else {
            exec.push_str("quick-webapps-webview");
        }

        exec.push_str(&format!(" {}", self));

        exec
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebviewArgsBuilder {
    app_id: String,
    window_title: String,
    url: String,
    profile: Option<PathBuf>,
    window_size: Option<WindowSize>,
    window_decorations: Option<bool>,
    private_mode: Option<bool>,
    try_simulate_mobile: Option<bool>,
}

impl WebviewArgsBuilder {
    pub fn new(app_id: String, window_title: String, url: String) -> Self {
        Self {
            app_id,
            window_title,
            url,
            profile: None,
            window_size: None,
            window_decorations: None,
            private_mode: None,
            try_simulate_mobile: None,
        }
    }

    pub fn profile(&mut self, profile: &PathBuf) {
        self.profile = Some(profile.clone());
    }

    pub fn window_size(&mut self, window_size: WindowSize) {
        self.window_size = Some(window_size);
    }

    pub fn window_decorations(&mut self, window_decorations: bool) {
        self.window_decorations = Some(window_decorations);
    }

    pub fn set_incognito(&mut self, private_mode: bool) {
        self.private_mode = Some(private_mode);
    }

    pub fn try_simulate_mobile(&mut self, try_simulate_mobile: bool) {
        self.try_simulate_mobile = Some(try_simulate_mobile);
    }

    pub fn build(self) -> WebviewArgs {
        WebviewArgs {
            app_id: self.app_id,
            window_title: self.window_title,
            url: self.url,
            profile: self.profile,
            window_size: self.window_size,
            window_decorations: self.window_decorations,
            private_mode: self.private_mode,
            try_simulate_mobile: self.try_simulate_mobile,
        }
    }
}

#[derive(Debug)]
pub enum WindowSizeError {
    ParsingError,
    BadArgument,
}

impl Display for WindowSizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowSizeError::ParsingError => write!(f, "Failed to parse window size"),
            WindowSizeError::BadArgument => write!(f, "Invalid window size argument"),
        }
    }
}

impl std::error::Error for WindowSizeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WindowSizeError::ParsingError => None,
            WindowSizeError::BadArgument => None,
        }
    }

    fn description(&self) -> &str {
        match self {
            WindowSizeError::ParsingError => "Failed to parse window size",
            WindowSizeError::BadArgument => "Invalid window size argument",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

pub type WindowWidth = f64;
pub type WindowHeight = f64;

pub const DEFAULT_WINDOW_WIDTH: WindowWidth = 800.0;
pub const DEFAULT_WINDOW_HEIGHT: WindowHeight = 600.0;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowSize(pub WindowWidth, pub WindowHeight);

impl Default for WindowSize {
    fn default() -> Self {
        WindowSize(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
    }
}

impl Display for WindowSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.0, self.1)
    }
}

impl FromStr for WindowSize {
    type Err = WindowSizeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            Err(WindowSizeError::BadArgument)
        } else {
            let width = parts[0]
                .parse()
                .map_err(|_| WindowSizeError::ParsingError)?;
            let height = parts[1]
                .parse()
                .map_err(|_| WindowSizeError::ParsingError)?;
            Ok(WindowSize(width, height))
        }
    }
}

pub fn is_flatpak() -> bool {
    let Ok(path) = PathBuf::from_str("/.flatpak-info");
    path.exists()
}
