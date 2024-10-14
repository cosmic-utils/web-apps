use crate::fl;

#[derive(Debug, Clone, PartialEq)]
pub enum WarnMessages {
    Warning,
    Duplicate,
    WrongIcon,
    AppName,
    AppUrl,
    AppIcon,
    AppBrowser,
}

#[derive(Debug, Clone)]
pub enum WarnAction {
    Remove,
    Add,
}

impl std::fmt::Display for WarnMessages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            WarnMessages::Warning => write!(f, "{}", fl!("warning")),
            WarnMessages::Duplicate => write!(f, "{}", fl!("warning", "duplicate")),
            WarnMessages::WrongIcon => write!(f, "{}", fl!("warning", "wrong-icon")),
            WarnMessages::AppName => write!(f, "{}", fl!("warning", "app-name")),
            WarnMessages::AppUrl => write!(f, "{}", fl!("warning", "app-url")),
            WarnMessages::AppIcon => write!(f, "{}", fl!("warning", "app-icon")),
            WarnMessages::AppBrowser => {
                write!(f, "{}", fl!("warning", "app-browser"))
            }
        }
    }
}

impl Default for WarnMessages {
    fn default() -> Self {
        Self::Warning
    }
}

#[derive(Default, Debug, Clone)]
pub struct Warning {
    pub show: bool,
    pub messages: Vec<WarnMessages>,
}

impl Warning {
    pub fn new() -> Self {
        let show = true;
        let messages = vec![
            WarnMessages::Warning,
            WarnMessages::AppName,
            WarnMessages::AppUrl,
            WarnMessages::AppIcon,
            WarnMessages::AppBrowser,
        ];

        Self { show, messages }
    }

    pub fn switch_header(&mut self) {
        self.show = !self.messages.is_empty()
    }

    pub fn push_warn(&mut self, message: WarnMessages) {
        if !self.messages.contains(&message) {
            self.messages.push(message);
        }
        self.switch_header();
    }

    pub fn remove_warn(&mut self, message: WarnMessages) {
        self.messages.retain(|m| *m != message);
        self.switch_header();
    }

    pub fn remove_all_warns(&mut self) {
        self.messages.clear();
        self.switch_header();
    }

    pub fn messages(&self) -> String {
        let mut content = format!("{}\n", WarnMessages::Warning);

        for line in &self.messages {
            content.push_str(&format!("{}\n", line));
        }

        content
    }
}
