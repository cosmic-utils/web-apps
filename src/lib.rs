use clap::{Parser, Subcommand};
use cosmic::{app::CosmicFlags, desktop::fde::get_languages_from_env};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf, str::FromStr};

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
}

impl IntoIterator for WebviewArgs {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            format!("--window-title {}", self.window_title),
            format!("--url {}", self.url),
            if let Some(profile) = self.profile {
                format!("--profile {}", profile.display())
            } else {
                "".to_string()
            },
            if let Some(window_size) = self.window_size {
                format!("--window-size {}", window_size)
            } else {
                "".to_string()
            },
            if let Some(window_decorations) = self.window_decorations {
                format!("--window-decorations {}", window_decorations)
            } else {
                "".to_string()
            },
            self.app_id,
        ]
        .into_iter()
    }
}

impl Display for WebviewArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "--window-title {} --url {}{}{}{} {}",
            self.window_title,
            self.url,
            if let Some(profile) = &self.profile {
                format!(" --profile {}", profile.display())
            } else {
                "".to_string()
            },
            if let Some(window_size) = &self.window_size {
                format!(" --window-size {}", window_size)
            } else {
                "".to_string()
            },
            if let Some(window_decorations) = &self.window_decorations {
                format!(" --window-decorations {}", window_decorations)
            } else {
                "".to_string()
            },
            self.app_id
        )
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

        exec.push_str(&format!(" --window-title \"{}\"", self.window_title));
        exec.push_str(&format!(" --url \"{}\"", self.url));

        if let Some(profile) = &self.profile {
            exec.push_str(&format!(" --profile \"{}\"", profile.display()));
        }

        if let Some(window_size) = &self.window_size {
            exec.push_str(&format!(" --window-size \"{}\"", window_size));
        }

        if let Some(window_decorations) = &self.window_decorations {
            exec.push_str(&format!(" --window-decorations {}", window_decorations));
        }

        exec.push_str(&format!(" {}", self.app_id));

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

    pub fn build(self) -> WebviewArgs {
        WebviewArgs {
            app_id: self.app_id,
            window_title: self.window_title,
            url: self.url,
            profile: self.profile,
            window_size: self.window_size,
            window_decorations: self.window_decorations,
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

pub struct StateFlags {
    pub args: Args,
}

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    #[clap(subcommand)]
    pub subcommand: Option<ManagerTasks>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Subcommand)]
pub enum ManagerTasks {
    #[clap(about = "Launch webapp by it's args")]
    Launch(WebviewArgs),
}

impl Display for ManagerTasks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::ser::to_string(self).unwrap())
    }
}

impl FromStr for ManagerTasks {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::de::from_str(s)
    }
}

impl CosmicFlags for StateFlags {
    type Args = Vec<String>;
    type SubCommand = ManagerTasks;

    fn action(&self) -> Option<&ManagerTasks> {
        self.args.subcommand.as_ref()
    }
}

pub fn is_flatpak() -> bool {
    let Ok(path) = PathBuf::from_str("/.flatpak-info");
    path.exists()
}
