use i18n_embed::DesktopLanguageRequester;

pub(crate) mod config;
pub(crate) mod pages;
pub(crate) mod themes;

fn main() -> cosmic::iced::Result {
    init_logging();
    init_localizer();

    cosmic::app::run::<crate::pages::QuickWebApps>(
        cosmic::app::Settings::default()
            .antialiasing(true)
            .client_decorations(true),
        (),
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
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::INFO)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
