use cosmic::cosmic_theme::{self, ThemeBuilder};

#[derive(Debug, Default, Clone)]
pub enum Theme {
    #[default]
    Light,
    Dark,
    Custom((String, Box<cosmic_theme::Theme>)),
}

impl AsRef<str> for Theme {
    fn as_ref(&self) -> &str {
        match self {
            Theme::Light => "COSMIC Light",
            Theme::Dark => "COSMIC Dark",
            Theme::Custom(theme) => &theme.0,
        }
    }
}

impl Theme {
    pub fn build(name: String, value: String) -> Self {
        if let Ok(palette) = ron::from_str::<ThemeBuilder>(&value) {
            return Self::Custom((name, Box::new(palette.build())));
        }

        Self::Light
    }
}
