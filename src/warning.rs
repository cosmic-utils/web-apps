use crate::fl;

#[derive(Debug, Clone, PartialEq)]
pub enum WarnMessages {
    Info,
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
            WarnMessages::Info => write!(f, "{}", fl!("warning")),
            WarnMessages::AppName => write!(f, "{}", fl!("warning", "app-name")),
            WarnMessages::AppUrl => write!(f, "{}", fl!("warning", "app-url")),
            WarnMessages::AppIcon => write!(f, "{}", fl!("warning", "app-icon")),
            WarnMessages::AppBrowser => {
                write!(f, "{}", fl!("warning", "app-browser"))
            }
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Warning {
    pub messages: Vec<WarnMessages>,
    pub show: bool,
}

impl Warning {
    pub fn new(messages: Vec<WarnMessages>, show: bool) -> Self {
        Self { messages, show }
    }

    pub fn push_warn(&mut self, message: WarnMessages) -> &mut Self {
        self.show = true;

        if !self.messages.contains(&WarnMessages::Info) {
            self.messages.insert(0, WarnMessages::Info);
        }

        if !self.messages.contains(&message) {
            self.messages.push(message);
        }
        self
    }

    pub fn remove_warn(&mut self, message: WarnMessages) -> &mut Self {
        self.messages.retain(|m| *m != message);

        if self.messages.contains(&WarnMessages::Info) && self.messages.len() <= 1 {
            self.show = false;
        };

        self
    }

    pub fn remove_all_warns(&mut self) -> &mut Self {
        self.messages.clear();
        self.show = false;
        self
    }

    pub fn messages(&self) -> String {
        let mut content = String::new();

        for line in &self.messages {
            content.push_str(&format!("{}\n", line));
        }

        content
    }
}
