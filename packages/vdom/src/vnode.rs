use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VNode {
    Element(VElement),
    Text(VText),
    Fragment(Vec<VNode>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VElement {
    pub tag: String,
    pub key: Option<String>,
    pub attributes: HashMap<String, String>,
    pub children: Vec<VNode>,
    pub style: Style,
    pub class: Classes,
    pub event_handlers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VText {
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Style {
    pub static_styles: String,
    pub css_variables: Vec<(String, String)>,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(mut self, name: &str, value: &str) -> Self {
        if !self.static_styles.is_empty() {
            self.static_styles.push(';');
        }
        self.static_styles.push_str(&format!("{}:{}", name, value));
        self
    }

    pub fn add_custom(mut self, name: &str, value: &str) -> Self {
        self.css_variables
            .push((name.to_string(), value.to_string()));
        self
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        let mut result = self.static_styles.clone();
        for (name, value) in &self.css_variables {
            if !result.is_empty() {
                result.push(';');
            }
            result.push_str(&format!("{}:{}", name, value));
        }
        result
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Classes {
    pub static_classes: String,
}

impl Classes {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, class: &str) -> Self {
        if !self.static_classes.is_empty() {
            self.static_classes.push(' ');
        }
        self.static_classes.push_str(class);
        self
    }

    pub fn add_if(mut self, class: &str, condition: bool) -> Self {
        if condition {
            self = self.add(class);
        }
        self
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> &str {
        &self.static_classes
    }
}

impl VElement {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
            key: None,
            attributes: HashMap::new(),
            children: Vec::new(),
            style: Style::new(),
            class: Classes::new(),
            event_handlers: Vec::new(),
        }
    }

    pub fn attr(mut self, name: &str, value: &str) -> Self {
        self.attributes.insert(name.to_string(), value.to_string());
        self
    }

    pub fn key(mut self, key: &str) -> Self {
        self.key = Some(key.to_string());
        self
    }

    pub fn child(mut self, child: VNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn class(mut self, class: Classes) -> Self {
        self.class = class;
        self
    }
}

impl VText {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}
