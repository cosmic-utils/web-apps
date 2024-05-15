use crate::fl;

#[derive(Debug, Clone, PartialEq)]
pub enum WarnMessages {
    Warning,
    Sucess,
    Duplicate,
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
            WarnMessages::Sucess => write!(f, "{}", fl!("warning", "success")),
            WarnMessages::Duplicate => write!(f, "{}", fl!("warning", "duplicate")),
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
    pub header: WarnMessages,
    pub messages: Vec<WarnMessages>,
}

impl Warning {
    pub fn new(messages: Vec<WarnMessages>) -> Self {
        let header = if !messages.is_empty() {
            WarnMessages::Warning
        } else {
            WarnMessages::Sucess
        };

        Self { header, messages }
    }

    pub fn switch_header(&mut self) {
        if self.messages.is_empty() {
            self.header = WarnMessages::Sucess
        } else {
            self.header = WarnMessages::Warning
        }
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
        let mut content = format!("{}\n", self.header);

        for line in &self.messages {
            content.push_str(&format!("{}\n", line));
        }

        content
    }
}
