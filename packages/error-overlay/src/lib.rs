//! Error overlay component for Tairitsu development
//!
//! This package provides a development-friendly error overlay that displays
//! compile-time, runtime, and network errors in an accessible and visually
//! appealing way.

mod templates;

use serde::{Deserialize, Serialize};
use tairitsu_vdom::{VElement, VNode};

pub use templates::Templates;

/// Information about an error that occurred in the application
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// The error message
    pub message: String,
    /// Optional stack trace
    pub stack: Option<String>,
    /// Optional location information
    pub location: Option<ErrorLocation>,
    /// The type of error
    pub error_type: ErrorType,
}

impl ErrorInfo {
    /// Create a new error info with just a message
    pub fn new(message: impl Into<String>, error_type: ErrorType) -> Self {
        Self {
            message: message.into(),
            stack: None,
            location: None,
            error_type,
        }
    }

    /// Add a stack trace to the error
    pub fn with_stack(mut self, stack: impl Into<String>) -> Self {
        self.stack = Some(stack.into());
        self
    }

    /// Add location information to the error
    pub fn with_location(mut self, location: ErrorLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Create a compile error
    pub fn compile(message: impl Into<String>) -> Self {
        Self::new(message, ErrorType::CompileError)
    }

    /// Create a runtime error
    pub fn runtime(message: impl Into<String>) -> Self {
        Self::new(message, ErrorType::RuntimeError)
    }

    /// Create a network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::new(message, ErrorType::NetworkError)
    }

    /// Create a type error
    pub fn type_error(message: impl Into<String>) -> Self {
        Self::new(message, ErrorType::TypeError)
    }
}

/// Location information for an error
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorLocation {
    /// The file path where the error occurred
    pub file: String,
    /// The line number
    pub line: u32,
    /// The column number
    pub column: u32,
}

impl ErrorLocation {
    /// Create a new error location
    pub fn new(file: impl Into<String>, line: u32, column: u32) -> Self {
        Self {
            file: file.into(),
            line,
            column,
        }
    }

    /// Format the location as a string
    pub fn format(&self) -> String {
        format!("{}:{}:{}", self.file, self.line, self.column)
    }
}

/// The type of error that occurred
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorType {
    /// A compile-time error (e.g., syntax error, type checking error)
    CompileError,
    /// A runtime error (e.g., null reference, division by zero)
    RuntimeError,
    /// A network error (e.g., failed fetch, timeout)
    NetworkError,
    /// A type error (e.g., type mismatch, invalid cast)
    TypeError,
}

impl ErrorType {
    /// Get the CSS class name for this error type
    pub fn css_class(&self) -> &'static str {
        match self {
            ErrorType::CompileError => "compile",
            ErrorType::RuntimeError => "runtime",
            ErrorType::NetworkError => "network",
            ErrorType::TypeError => "type",
        }
    }

    /// Get the display title for this error type
    pub fn title(&self) -> &'static str {
        match self {
            ErrorType::CompileError => "Compile Error",
            ErrorType::RuntimeError => "Runtime Error",
            ErrorType::NetworkError => "Network Error",
            ErrorType::TypeError => "Type Error",
        }
    }

    /// Get the icon character for this error type
    pub fn icon(&self) -> &'static str {
        match self {
            ErrorType::CompileError => "",
            ErrorType::RuntimeError => "",
            ErrorType::NetworkError => "",
            ErrorType::TypeError => "",
        }
    }
}

/// Generate the error overlay script for injection into HTML
///
/// This function generates the complete JavaScript code needed for
/// client-side error handling, including the overlay HTML template.
///
/// # Example
///
/// ```
/// use tairitsu_error_overlay::overlay_script;
///
/// let script = overlay_script();
/// // Inject into HTML:
/// // <script>{script}</script>
/// ```
pub fn overlay_script() -> String {
    let client_script = Templates::client_script();
    let overlay_html = Templates::overlay_container()
        .replace('\n', "")
        .replace('\\', "\\\\")
        .replace('`', "\\`");

    client_script.replace("__OVERLAY_HTML__", &overlay_html)
}

/// Render an error overlay component as a VNode
///
/// This creates a complete error overlay component that can be
/// rendered directly into the virtual DOM.
///
/// # Example
///
/// ```no_run
/// use tairitsu_error_overlay::{ErrorInfo, ErrorType, render_error_overlay};
///
/// let error = ErrorInfo::runtime("Something went wrong");
/// let vnode = render_error_overlay(error);
/// ```
pub fn render_error_overlay(error: ErrorInfo) -> VNode {
    VNode::Element(
        VElement::new("div")
            .attr("id", "tairitsu-error-overlay")
            .class(format!(
                "tairitsu-error-overlay {}",
                if error.message.is_empty() {
                    "tairitsu-error-hidden"
                } else {
                    ""
                }
            ))
            .child(VNode::Element(
                VElement::new("div")
                    .class("tairitsu-error-container")
                    .child(render_error_header(&error))
                    .child(render_error_body(&error))
                    .child(render_error_footer()),
            )),
    )
}

fn render_error_header(error: &ErrorInfo) -> VNode {
    VNode::Element(
        VElement::new("div")
            .class("tairitsu-error-header")
            .child(VNode::Element(
                VElement::new("div")
                    .class("tairitsu-error-title")
                    .child(VNode::Element(
                        VElement::new("span")
                            .class(format!(
                                "tairitsu-error-icon {}",
                                error.error_type.css_class()
                            ))
                            .child(VNode::Text(tairitsu_vdom::VText {
                                text: error.error_type.icon().to_string(),
                            })),
                    ))
                    .child(VNode::Element(
                        VElement::new("span")
                            .attr("data-title", "")
                            .child(VNode::Text(tairitsu_vdom::VText {
                                text: error.error_type.title().to_string(),
                            })),
                    )),
            ))
            .child(VNode::Element(
                VElement::new("button")
                    .attr("class", "tairitsu-error-close")
                    .attr("data-close", "")
                    .attr("aria-label", "Close")
                    .attr("type", "button")
                    .child(VNode::Text(tairitsu_vdom::VText {
                        text: "\u{d7}".to_string(), // multiplication sign (×)
                    })),
            )),
    )
}

fn render_error_body(error: &ErrorInfo) -> VNode {
    VNode::Element(
        VElement::new("div")
            .class("tairitsu-error-body")
            .child(VNode::Element(
                VElement::new("p")
                    .class("tairitsu-error-message")
                    .attr("data-message", "")
                    .child(VNode::Text(tairitsu_vdom::VText {
                        text: error.message.clone(),
                    })),
            ))
            .child(render_error_location(error))
            .child(render_error_stack(error)),
    )
}

fn render_error_location(error: &ErrorInfo) -> VNode {
    match &error.location {
        Some(loc) => VNode::Element(
            VElement::new("div")
                .class("tairitsu-error-location")
                .attr("data-location", "")
                .child(VNode::Element(
                    VElement::new("span")
                        .class("tairitsu-error-location-icon")
                        .child(VNode::Text(tairitsu_vdom::VText {
                            text: "\u{1f517}".to_string(), // link emoji
                        })),
                ))
                .child(VNode::Element(
                    VElement::new("span")
                        .attr("data-location-text", "")
                        .child(VNode::Text(tairitsu_vdom::VText {
                            text: loc.format(),
                        })),
                )),
        ),
        None => VNode::Element(
            VElement::new("div")
                .class("tairitsu-error-location tairitsu-error-hidden")
                .attr("data-location", ""),
        ),
    }
}

fn render_error_stack(error: &ErrorInfo) -> VNode {
    match &error.stack {
        Some(stack) => VNode::Element(
            VElement::new("div")
                .class("tairitsu-error-stack")
                .attr("data-stack", "")
                .child(VNode::Element(
                    VElement::new("h4")
                        .class("tairitsu-error-stack-title")
                        .child(VNode::Text(tairitsu_vdom::VText {
                            text: "Stack Trace".to_string(),
                        })),
                ))
                .child(VNode::Element(
                    VElement::new("pre")
                        .class("tairitsu-error-stack-content")
                        .attr("data-stack-content", "")
                        .child(VNode::Text(tairitsu_vdom::VText {
                            text: stack.clone(),
                        })),
                )),
        ),
        None => VNode::Element(
            VElement::new("div")
                .class("tairitsu-error-stack tairitsu-error-hidden")
                .attr("data-stack", ""),
        ),
    }
}

fn render_error_footer() -> VNode {
    VNode::Element(
        VElement::new("div")
            .class("tairitsu-error-footer")
            .child(VNode::Element(
                VElement::new("button")
                    .class("tairitsu-error-button secondary")
                    .attr("data-copy", "")
                    .attr("type", "button")
                    .child(VNode::Text(tairitsu_vdom::VText {
                        text: "Copy Error".to_string(),
                    })),
            ))
            .child(VNode::Element(
                VElement::new("button")
                    .class("tairitsu-error-button primary")
                    .attr("data-reload", "")
                    .attr("type", "button")
                    .child(VNode::Text(tairitsu_vdom::VText {
                        text: "Reload".to_string(),
                    })),
            )),
    )
}

/// Display component for rendering different error types
pub struct ErrorDisplay {
    error: ErrorInfo,
    show_overlay: bool,
}

impl ErrorDisplay {
    /// Create a new error display
    pub fn new(error: ErrorInfo) -> Self {
        Self {
            error,
            show_overlay: true,
        }
    }

    /// Set whether to show the overlay
    pub fn show_overlay(mut self, show: bool) -> Self {
        self.show_overlay = show;
        self
    }

    /// Render the error display
    pub fn render(&self) -> VNode {
        if self.show_overlay {
            render_error_overlay(self.error.clone())
        } else {
            // Render a minimal error message inline
            self.render_inline()
        }
    }

    fn render_inline(&self) -> VNode {
        VNode::Element(
            VElement::new("div")
                .class(format!("error-display {}", self.error.error_type.css_class()))
                .child(VNode::Element(
                    VElement::new("strong")
                        .child(VNode::Text(tairitsu_vdom::VText {
                            text: format!("{}:", self.error.error_type.title()),
                        })),
                ))
                .child(VNode::Text(tairitsu_vdom::VText {
                    text: format!(" {}", self.error.message),
                })),
        )
    }
}

/// ErrorOverlay component builder
pub struct ErrorOverlay {
    errors: Vec<ErrorInfo>,
    show_styles: bool,
}

impl Default for ErrorOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorOverlay {
    /// Create a new error overlay
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            show_styles: true,
        }
    }

    /// Add an error to the overlay
    pub fn add_error(mut self, error: ErrorInfo) -> Self {
        self.errors.push(error);
        self
    }

    /// Add multiple errors
    pub fn add_errors(mut self, errors: Vec<ErrorInfo>) -> Self {
        self.errors.extend(errors);
        self
    }

    /// Set whether to include inline styles
    pub fn with_styles(mut self, include: bool) -> Self {
        self.show_styles = include;
        self
    }

    /// Render the overlay
    pub fn render(&self) -> VNode {
        if self.errors.is_empty() {
            return VNode::Element(VElement::new("div").attr("data-empty-overflow", ""));
        }

        // Show the first error (most recent)
        render_error_overlay(self.errors[0].clone())
    }

    /// Render the overlay with styles included
    pub fn render_with_styles(&self) -> VNode {
        if !self.show_styles {
            return self.render();
        }

        VNode::Element(
            VElement::new("div")
                .child(VNode::Element(
                    VElement::new("style")
                        .attr("type", "text/css")
                        .child(VNode::Text(tairitsu_vdom::VText {
                            text: Templates::overlay_styles().to_string(),
                        })),
                ))
                .child(self.render()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_info_creation() {
        let error = ErrorInfo::new("Test error", ErrorType::RuntimeError);
        assert_eq!(error.message, "Test error");
        assert_eq!(error.error_type, ErrorType::RuntimeError);
        assert!(error.stack.is_none());
        assert!(error.location.is_none());
    }

    #[test]
    fn test_error_info_builder() {
        let error = ErrorInfo::new("Test error", ErrorType::CompileError)
            .with_stack("stack trace here")
            .with_location(ErrorLocation::new("file.rs", 10, 5));

        assert_eq!(error.message, "Test error");
        assert_eq!(error.stack, Some("stack trace here".to_string()));
        assert_eq!(error.location.as_ref().unwrap().file, "file.rs");
        assert_eq!(error.location.as_ref().unwrap().line, 10);
        assert_eq!(error.location.as_ref().unwrap().column, 5);
    }

    #[test]
    fn test_error_info_convenience_constructors() {
        let compile_err = ErrorInfo::compile("Syntax error");
        assert_eq!(compile_err.error_type, ErrorType::CompileError);

        let runtime_err = ErrorInfo::runtime("Null pointer");
        assert_eq!(runtime_err.error_type, ErrorType::RuntimeError);

        let network_err = ErrorInfo::network("Connection failed");
        assert_eq!(network_err.error_type, ErrorType::NetworkError);

        let type_err = ErrorInfo::type_error("Type mismatch");
        assert_eq!(type_err.error_type, ErrorType::TypeError);
    }

    #[test]
    fn test_error_location_format() {
        let loc = ErrorLocation::new("src/main.rs", 42, 7);
        assert_eq!(loc.format(), "src/main.rs:42:7");
    }

    #[test]
    fn test_error_type_properties() {
        assert_eq!(ErrorType::CompileError.css_class(), "compile");
        assert_eq!(ErrorType::CompileError.title(), "Compile Error");

        assert_eq!(ErrorType::RuntimeError.css_class(), "runtime");
        assert_eq!(ErrorType::RuntimeError.title(), "Runtime Error");

        assert_eq!(ErrorType::NetworkError.css_class(), "network");
        assert_eq!(ErrorType::NetworkError.title(), "Network Error");

        assert_eq!(ErrorType::TypeError.css_class(), "type");
        assert_eq!(ErrorType::TypeError.title(), "Type Error");
    }

    #[test]
    fn test_error_display() {
        let error = ErrorInfo::runtime("Something went wrong");
        let display = ErrorDisplay::new(error.clone());
        let vnode = display.render();

        // Verify the vnode is created without panicking
        match vnode {
            VNode::Element(el) => {
                assert!(el.attributes.contains_key("id"));
                assert_eq!(el.attributes.get("id").unwrap(), "tairitsu-error-overlay");
            }
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_error_overlay_builder() {
        let overlay = ErrorOverlay::new()
            .add_error(ErrorInfo::compile("Error 1"))
            .add_error(ErrorInfo::runtime("Error 2"))
            .with_styles(true);

        let vnode = overlay.render_with_styles();
        match vnode {
            VNode::Element(el) => {
                // Should have a style child
                assert!(!el.children.is_empty());
            }
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_overlay_script_generation() {
        let script = overlay_script();
        assert!(script.contains("tairitsu-error-overlay"));
        assert!(script.contains("window.addEventListener"));
        assert!(script.contains("unhandledrejection"));
    }

    #[test]
    fn test_error_info_serialization() {
        let error = ErrorInfo::new("Test", ErrorType::RuntimeError)
            .with_location(ErrorLocation::new("file.rs", 1, 2));

        let json = serde_json::to_string(&error).unwrap();
        let deserialized: ErrorInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.message, "Test");
        assert_eq!(deserialized.error_type, ErrorType::RuntimeError);
        assert_eq!(
            deserialized.location.as_ref().unwrap().file,
            "file.rs"
        );
    }

    #[test]
    fn test_error_type_equality() {
        assert_eq!(ErrorType::CompileError, ErrorType::CompileError);
        assert_ne!(ErrorType::CompileError, ErrorType::RuntimeError);
    }

    #[test]
    fn test_templates_provide_content() {
        let styles = Templates::overlay_styles();
        assert!(!styles.is_empty());
        assert!(styles.contains(".tairitsu-error-overlay"));

        let container = Templates::overlay_container();
        assert!(!container.is_empty());
        assert!(container.contains("tairitsu-error-overlay"));

        let script = Templates::client_script();
        assert!(!script.is_empty());
        assert!(script.contains("function"));
    }
}
