use clap::Parser;
use i18n_embed::DesktopLanguageRequester;
use std::os::unix::fs::PermissionsExt as _;
use tokio::{fs::File, io::AsyncWriteExt as _, process::Child};
use webapps::{Args, StateFlags};

pub(crate) mod browser;
pub(crate) mod common;
pub(crate) mod config;
pub(crate) mod favicon;
pub(crate) mod launcher;
pub(crate) mod pages;
pub(crate) mod themes;

fn main() -> cosmic::iced::Result {
    init_logging();
    init_localizer();

    let args = Args::parse();

    cosmic::app::run::<crate::pages::QuickWebApps>(
        cosmic::app::Settings::default()
            .antialiasing(true)
            .client_decorations(true),
        StateFlags { args },
    )
}

fn init_localizer() {
    let localizer = webapps::localize::localizer();
    let requested_languages = DesktopLanguageRequester::requested_languages();

    if let Err(why) = localizer.select(&requested_languages) {
        tracing::error!(%why, "error while loading fluent localizations");
    }
}

fn init_logging() {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let filter_layer = EnvFilter::try_from_default_env().unwrap_or(if cfg!(debug_assertions) {
        EnvFilter::new(format!("warn,{}=debug", env!("CARGO_CRATE_NAME")))
    } else {
        EnvFilter::new("warn")
    });

    let fmt_layer = fmt::layer().with_target(false);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .init();
}

pub fn icon_pack_installed() -> bool {
    let packs: Vec<&str> = vec!["Papirus", "Papirus-Dark", "Papirus-Light"];
    let mut directories = 0;

    for theme in packs.iter() {
        let mut icons_dir = common::icons_location();
        icons_dir.push(theme);

        if icons_dir.exists() {
            directories += 1;
        };
    }

    directories > 0
}

pub async fn add_icon_packs_install_script() -> String {
    let install_script = include_bytes!("../../../scripts/icon-installer.sh");
    let temp_file = "/tmp/io.github.elevenhsoft.WebApps.icon-installer.sh";

    // Create a temporary file
    let mut file = File::create(temp_file).await.unwrap();

    file.write_all(install_script).await.unwrap();

    // Make the script executable
    let mut perms = file.metadata().await.unwrap().permissions();
    perms.set_mode(0o755);
    file.set_permissions(perms).await.unwrap();

    temp_file.to_string()
}

pub async fn execute_script(script: String) -> Child {
    tokio::process::Command::new(script)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("cant execute script")
}
