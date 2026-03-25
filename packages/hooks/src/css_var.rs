use std::{cell::RefCell, rc::Rc};

#[cfg(feature = "web")]
use wasm_bindgen::JsCast;

/// Creates a reactive CSS variable binding.
///
/// This hook provides a way to read and write CSS custom properties (CSS variables)
/// on the `:root` element. It returns a tuple containing:
/// - A `Rc<RefCell<String>>` containing the current value
/// - A setter function to update the CSS variable
///
/// # Arguments
///
/// * `name` - The name of the CSS variable (without the `--` prefix)
///
/// # Returns
///
/// A tuple of `(value, setter)` where:
/// - `value` is a `Rc<RefCell<String>>` containing the current CSS variable value
/// - `setter` is a function that takes a `String` and updates the CSS variable
///
/// # Platform-specific behavior
///
/// - **Web**: The CSS variable is read from and written to the `:root` element
/// - **Non-web**: The value is stored in memory only (no actual CSS variable is modified)
///
/// # Example
///
/// ```ignore
/// let (color, set_color) = use_css_var("primary-color");
/// set_color("#ff0000".to_string());
/// println!("Current color: {}", color.borrow());
/// ```
#[cfg(feature = "web")]
pub fn use_css_var(name: &str) -> (Rc<RefCell<String>>, impl Fn(String)) {
    let value = Rc::new(RefCell::new(get_css_var(name)));
    let name = name.to_string();
    let value_clone = Rc::clone(&value);

    let setter = move |new_value: String| {
        set_css_var(&name, &new_value);
        *value_clone.borrow_mut() = new_value;
    };

    (value, setter)
}

#[cfg(feature = "web")]
fn get_css_var(name: &str) -> String {
    use web_sys::window;

    if let Some(window) = window() {
        if let Some(document) = window.document() {
            if let Some(root) = document.document_element() {
                if let Some(element) = root.dyn_ref::<web_sys::Element>() {
                    let computed = window.get_computed_style(element).ok().flatten();
                    if let Some(computed) = computed {
                        return computed
                            .get_property_value(&format!("--{}", name))
                            .unwrap_or_default()
                            .trim()
                            .to_string();
                    }
                }
            }
        }
    }
    String::new()
}

#[cfg(feature = "web")]
fn set_css_var(name: &str, value: &str) {
    use web_sys::window;

    if let Some(window) = window() {
        if let Some(document) = window.document() {
            if let Some(root) = document.document_element() {
                if let Some(html_element) = root.dyn_ref::<web_sys::HtmlElement>() {
                    html_element
                        .style()
                        .set_property(&format!("--{}", name), value)
                        .ok();
                }
            }
        }
    }
}

/// Creates a reactive CSS variable binding (non-web implementation).
///
/// This is a fallback implementation for non-web platforms. The value is stored
/// in memory only and does not interact with any actual CSS variables.
///
/// See the web-specific implementation for more details.
#[cfg(not(feature = "web"))]
pub fn use_css_var(_name: &str) -> (Rc<RefCell<String>>, impl Fn(String)) {
    let value = Rc::new(RefCell::new(String::new()));
    let value_clone = Rc::clone(&value);

    let setter = move |new_value: String| {
        *value_clone.borrow_mut() = new_value;
    };

    (value, setter)
}
