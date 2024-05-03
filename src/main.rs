mod common;
mod gui;
mod iconpicker;
mod icons_installator;
mod supported_browsers;
mod wam;
mod wrap;

use std::{os::unix::fs::PermissionsExt, process::ExitStatus};

use cosmic::{app::Settings, iced_core::Size};
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use gui::Window;

fn main() -> cosmic::iced::Result {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mut settings = Settings::default();
    settings = settings.size(Size {
        width: 640.,
        height: 680.,
    });

    cosmic::app::run::<Window>(settings, ())
}

pub fn icon_pack_installed() -> bool {
    let packs: Vec<&str> = vec!["Papirus", "Papirus-Dark", "Papirus-Light"];
    let mut directories = 0;

    for theme in packs.iter() {
        let mut icons_dir = common::home_dir();
        icons_dir.push(".local/share");
        icons_dir.push("icons");
        icons_dir.push(theme);

        if icons_dir.exists() {
            directories += 1;
        };
    }

    directories > 2
}

pub async fn add_icon_packs_install_script() -> String {
    let install_script = include_bytes!("../scripts/install.sh");
    let temp_file = "/tmp/org.cosmic.Wam.install-icons.sh";

    // Create a temporary file
    let mut file = File::create(temp_file).await.unwrap();

    file.write_all(install_script).await.unwrap();

    // Make the script executable
    let mut perms = file.metadata().await.unwrap().permissions();
    perms.set_mode(0o755);
    file.set_permissions(perms).await.unwrap();

    temp_file.to_string()
}

pub async fn execute_script(script: String) -> ExitStatus {
    tokio::process::Command::new(script)
        .status()
        .await
        .expect("cant execute script")
}
