use std::{cell::RefCell, rc::Rc};

#[cfg(feature = "web")]
use wasm_bindgen::JsCast;

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
                if let Some(style) = root.dyn_ref::<web_sys::Element>() {
                    let computed = window.get_computed_style(style).ok().flatten();
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
                root.style()
                    .set_property(&format!("--{}", name), value)
                    .ok();
            }
        }
    }
}

#[cfg(not(feature = "web"))]
pub fn use_css_var(name: &str) -> (Rc<RefCell<String>>, impl Fn(String)) {
    let value = Rc::new(RefCell::new(String::new()));
    let value_clone = Rc::clone(&value);

    let setter = move |new_value: String| {
        *value_clone.borrow_mut() = new_value;
    };

    (value, setter)
}
