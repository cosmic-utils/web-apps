use crate::browser::{Browser, BrowserT};

pub fn supported_browsers() -> Vec<Browser> {
    vec![
        Browser::new(
            "Firefox",
            "org.mozilla.firefox",
            "firefox",
            BrowserT::Firefox,
        ),
        Browser::new(
            "Zen Browser",
            "app.zen_browser.zen",
            "app.zen_browser.zen",
            BrowserT::Zen,
        ),
        Browser::new("Brave", "com.brave.Browser", "brave", BrowserT::Chromium),
        Browser::new(
            "Google Chrome",
            "com.google.Chrome",
            "google-chrome-stable",
            BrowserT::Chromium,
        ),
        Browser::new(
            "Vivaldi",
            "com.vivaldi.Vivaldi",
            "vivaldi-stable",
            BrowserT::Chromium,
        ),
        Browser::new(
            "Epiphany",
            "com.gnome.Epiphany",
            "epiphany",
            BrowserT::Epiphany,
        ),
        Browser::new("Floorp", "one.ablaze.floorp", "floorp", BrowserT::Floorp),
        Browser::new("Falkon", "org.kde.falkon", "falkon", BrowserT::Falkon),
        Browser::new(
            "Chromium",
            "org.chromium.Chromium",
            "chromium",
            BrowserT::Chromium,
        ),
        Browser::new(
            "Microsoft Edge",
            "com.microsoft.Edge",
            "microsoft-edge-stable",
            BrowserT::MsEdge,
        ),
    ]
}
