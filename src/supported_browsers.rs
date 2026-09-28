use crate::browser::{Browser, Installation};

pub fn supported_browsers() -> Vec<Browser> {
    vec![Browser::new(
        "firefox",
        "Firefox (Snap)",
        "firefox",
        "/snap/bin/firefox",
        Installation::Snap,
    )]
}
