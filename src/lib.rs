use clap::{Parser, Subcommand};
use cosmic::app::CosmicFlags;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf, str::FromStr};

pub mod localize;

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
