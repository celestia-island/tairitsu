use anyhow::Result;

#[cfg(feature = "web")]
pub struct WebPlatform {
    document: web_sys::Document,
}

#[cfg(feature = "web")]
impl WebPlatform {
    pub fn new() -> Result<Self> {
        let document = web_sys::window()
            .ok_or_else(|| anyhow::anyhow!("No window object"))?
            .document()
            .ok_or_else(|| anyhow::anyhow!("No document object"))?;

        Ok(Self { document })
    }
}

#[cfg(feature = "web")]
impl Platform for WebPlatform {
    type Element = web_sys::Element;
    type Event = web_sys::Event;

    fn create_element(&self, tag: &str) -> Self::Element {
        self.document.create_element(tag).unwrap()
    }

    fn create_text_node(&self, text: &str) -> Self::Element {
        self.document.create_text_node(text).unwrap().into()
    }

    fn append_child(&self, parent: &Self::Element, child: &Self::Element) {
        parent.append_child(child).unwrap();
    }

    fn remove_child(&self, parent: &Self::Element, child: &Self::Element) {
        parent.remove_child(child).unwrap();
    }

    fn set_attribute(&self, element: &Self::Element, name: &str, value: &str) {
        element.set_attribute(name, value).unwrap();
    }

    fn remove_attribute(&self, element: &Self::Element, name: &str) {
        element.remove_attribute(name).unwrap();
    }

    fn set_style(&self, element: &Self::Element, name: &str, value: &str) {
        use wasm_bindgen::JsCast;
        element
            .dyn_ref::<web_sys::HtmlElement>()
            .unwrap()
            .style()
            .set_property(name, value)
            .unwrap();
    }

    fn set_class(&self, element: &Self::Element, class: &str) {
        element.set_attribute("class", class).unwrap();
    }

    fn add_event_listener(&self, element: &Self::Element, event: &str, handler: Box<dyn FnMut()>) {
        use std::cell::RefCell;
        use std::rc::Rc;
        use wasm_bindgen::closure::Closure;

        let handler = Rc::new(RefCell::new(handler));
        let closure = Closure::wrap(Box::new(move || {
            handler.borrow_mut()();
        }) as Box<dyn FnMut()>);

        element
            .add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())
            .unwrap();

        closure.forget();
    }

    fn remove_event_listener(&self, element: &Self::Element, event: &str) {
        element
            .set_attribute(&format!("data-remove-{}", event), "")
            .ok();
    }
}

#[cfg(not(feature = "web"))]
pub struct WebPlatform;

#[cfg(not(feature = "web"))]
impl WebPlatform {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

#[cfg(feature = "web")]
impl ElementHandle for web_sys::Element {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
