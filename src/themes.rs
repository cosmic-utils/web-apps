use cosmic::cosmic_theme::{self, ThemeBuilder};

#[derive(Debug, Default, Clone)]
pub enum Theme {
    #[default]
    Default,
    Custom(Box<cosmic_theme::Theme>),
}

impl AsRef<str> for Theme {
    fn as_ref(&self) -> &str {
        match self {
            Theme::Default => "COSMIC",
            Theme::Custom(theme) => &theme.name,
        }
    }
}

impl From<String> for Theme {
    fn from(value: String) -> Self {
        if let Ok(palette) = ron::from_str::<ThemeBuilder>(&value) {
            return Self::Custom(Box::new(palette.build()));
        }

        Self::Default
    }
}
