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
    ]
}
