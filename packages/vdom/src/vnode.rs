use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::EventData;

/// Trait for attribute values that can be optionally rendered.
/// This allows `attr` to accept both `T` and `Option<T>` values.
pub trait IntoAttrValue {
    fn into_attr_value(self) -> Option<String>;
}

// Implement for common non-Option types
impl IntoAttrValue for &str {
    fn into_attr_value(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl IntoAttrValue for String {
    fn into_attr_value(self) -> Option<String> {
        Some(self)
    }
}

impl IntoAttrValue for i32 {
    fn into_attr_value(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl IntoAttrValue for i64 {
    fn into_attr_value(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl IntoAttrValue for u32 {
    fn into_attr_value(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl IntoAttrValue for u64 {
    fn into_attr_value(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl IntoAttrValue for usize {
    fn into_attr_value(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl IntoAttrValue for bool {
    fn into_attr_value(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl IntoAttrValue for f64 {
    fn into_attr_value(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl IntoAttrValue for f32 {
    fn into_attr_value(self) -> Option<String> {
        Some(self.to_string())
    }
}

// Implement for Option types
impl<T: ToString> IntoAttrValue for Option<T> {
    fn into_attr_value(self) -> Option<String> {
        self.map(|v| v.to_string())
    }
}

// Blanket implementation for references
impl<T: ToString + Clone> IntoAttrValue for &T {
    fn into_attr_value(self) -> Option<String> {
        Some(self.to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum VNode {
    Element(VElement),
    Text(VText),
    Fragment(Vec<VNode>),
}

pub type EventHandler = Rc<RefCell<dyn FnMut(Box<dyn EventData>)>>;

pub struct VElement {
    pub tag: String,
    pub key: Option<String>,
    pub attributes: HashMap<String, String>,
    pub children: Vec<VNode>,
    pub style: Style,
    pub class: Classes,
    pub event_handlers: HashMap<String, EventHandler>,
    /// Raw HTML to be set as inner_html (for dangerouslySetInnerHTML equivalent)
    pub inner_html: Option<String>,
}

impl PartialEq for VElement {
    fn eq(&self, other: &Self) -> bool {
        // Compare all fields except event_handlers (which can't be compared)
        self.tag == other.tag
            && self.key == other.key
            && self.attributes == other.attributes
            && self.children == other.children
            && self.style == other.style
            && self.class == other.class
            && self.inner_html == other.inner_html
        // Note: event_handlers are intentionally not compared
    }
}

impl fmt::Debug for VElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VElement")
            .field("tag", &self.tag)
            .field("key", &self.key)
            .field("attributes", &self.attributes)
            .field("children", &self.children)
            .field("style", &self.style)
            .field("class", &self.class)
            .field(
                "event_handlers",
                &self.event_handlers.keys().collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl Clone for VElement {
    fn clone(&self) -> Self {
        Self {
            tag: self.tag.clone(),
            key: self.key.clone(),
            attributes: self.attributes.clone(),
            children: self.children.clone(),
            style: self.style.clone(),
            class: self.class.clone(),
            event_handlers: self.event_handlers.clone(),
            inner_html: self.inner_html.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VText {
    pub text: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
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

#[derive(Debug, Clone, Default, PartialEq)]
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
            event_handlers: HashMap::new(),
            inner_html: None,
        }
    }

    /// Add an attribute to the element.
    /// Accepts both direct values and `Option<T>` values.
    /// When the value is `Some(v)`, the attribute is added with value `v`.
    /// When the value is `None`, the attribute is not added (this allows conditional attributes).
    pub fn attr(mut self, name: &str, value: impl IntoAttrValue) -> Self {
        if let Some(v) = value.into_attr_value() {
            self.attributes.insert(name.to_string(), v);
        }
        self
    }

    /// Add an optional attribute to the element.
    /// This is a convenience method that is equivalent to calling `attr` with an `Option<T>` value.
    /// When the value is `Some(v)`, the attribute is added with value `v`.
    /// When the value is `None`, the attribute is not added.
    pub fn attr_opt<T: ToString>(mut self, name: &str, value: Option<T>) -> Self {
        if let Some(v) = value {
            self.attributes.insert(name.to_string(), v.to_string());
        }
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

    pub fn children(mut self, children: Vec<VNode>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.class = class.into();
        self
    }

    pub fn on_event(
        mut self,
        event: &str,
        handler: impl FnMut(Box<dyn EventData>) + 'static,
    ) -> Self {
        self.event_handlers
            .insert(event.to_string(), Rc::new(RefCell::new(handler)));
        self
    }

    /// Set inner HTML directly (dangerously, equivalent to dangerouslySetInnerHTML)
    pub fn inner_html(mut self, html: impl Into<String>) -> Self {
        self.inner_html = Some(html.into());
        self
    }
}

impl From<&str> for Classes {
    fn from(s: &str) -> Self {
        Classes::new().add(s)
    }
}

impl From<String> for Classes {
    fn from(s: String) -> Self {
        Classes::new().add(&s)
    }
}

impl From<&str> for Style {
    fn from(s: &str) -> Self {
        let mut style = Style::new();
        for part in s.split(';') {
            let part = part.trim();
            if !part.is_empty() {
                if let Some((name, value)) = part.split_once(':') {
                    style = style.add(name.trim(), value.trim());
                }
            }
        }
        style
    }
}

impl From<String> for Style {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl VText {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

impl VNode {
    /// Creates an empty text node
    pub fn empty() -> Self {
        VNode::Text(VText::new(""))
    }
}
