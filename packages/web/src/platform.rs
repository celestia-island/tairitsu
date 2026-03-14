use anyhow::Result;

#[cfg(feature = "web")]
use std::cell::RefCell;
#[cfg(feature = "web")]
use std::collections::HashMap;
#[cfg(feature = "web")]
use std::rc::Rc;

#[cfg(feature = "web")]
use tairitsu_vdom::{ElementHandle, EventData, EventHandle, MouseEvent, Platform};
#[cfg(feature = "web")]
use wasm_bindgen::JsCast;

#[cfg(feature = "web")]
thread_local! {
    static EVENT_LISTENERS: RefCell<HashMap<usize, HashMap<String, wasm_bindgen::JsValue>>> = RefCell::new(HashMap::new());
}

#[cfg(feature = "web")]
fn get_element_id(element: &web_sys::Element) -> usize {
    let element_ptr = element as &wasm_bindgen::JsValue;
    element_ptr.as_f64().unwrap_or(0.0) as usize
}

#[cfg(feature = "web")]
#[derive(Clone)]
pub struct WebElement(pub web_sys::Element);

#[cfg(feature = "web")]
impl ElementHandle for WebElement {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(feature = "web")]
#[derive(Clone)]
pub struct WebEvent(pub web_sys::Event);

#[cfg(feature = "web")]
impl EventHandle for WebEvent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

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
    type Element = WebElement;
    type Event = WebEvent;

    fn create_element(&self, tag: &str) -> Self::Element {
        WebElement(self.document.create_element(tag).unwrap())
    }

    fn create_text_node(&self, text: &str) -> Self::Element {
        let text_node = self.document.create_text_node(text);
        WebElement(web_sys::Element::from(
            text_node.unchecked_into::<wasm_bindgen::JsValue>(),
        ))
    }

    fn append_child(&self, parent: &Self::Element, child: &Self::Element) {
        parent.0.append_child(&child.0).unwrap();
    }

    fn remove_child(&self, parent: &Self::Element, child: &Self::Element) {
        parent.0.remove_child(&child.0).unwrap();
    }

    fn set_attribute(&self, element: &Self::Element, name: &str, value: &str) {
        element.0.set_attribute(name, value).unwrap();
    }

    fn remove_attribute(&self, element: &Self::Element, name: &str) {
        element.0.remove_attribute(name).unwrap();
    }

    fn set_style(&self, element: &Self::Element, name: &str, value: &str) {
        element
            .0
            .dyn_ref::<web_sys::HtmlElement>()
            .unwrap()
            .style()
            .set_property(name, value)
            .unwrap();
    }

    fn set_class(&self, element: &Self::Element, class: &str) {
        element.0.set_attribute("class", class).unwrap();
    }

    fn add_event_listener(
        &self,
        element: &Self::Element,
        event: &str,
        handler: Box<dyn FnMut(Box<dyn EventData>)>,
    ) {
        use wasm_bindgen::closure::Closure;

        let handler = Rc::new(RefCell::new(handler));
        let event_name = event.to_string();
        let event_name_for_handler = event_name.clone();
        let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
            let event_data: Box<dyn EventData> = match event_name_for_handler.as_str() {
                "click" | "mousedown" | "mouseup" | "mousemove" => {
                    if let Some(mouse_event) = e.dyn_ref::<web_sys::MouseEvent>() {
                        Box::new(
                            MouseEvent::new()
                                .client_x(mouse_event.client_x())
                                .client_y(mouse_event.client_y()),
                        )
                    } else {
                        Box::new(MouseEvent::new())
                    }
                }
                _ => Box::new(MouseEvent::new()),
            };
            handler.borrow_mut()(event_data);
        }) as Box<dyn FnMut(web_sys::Event)>);

        let closure_js = closure.as_ref().clone();

        element
            .0
            .add_event_listener_with_callback(&event_name, closure.as_ref().unchecked_ref())
            .unwrap();

        let element_id = get_element_id(&element.0);
        EVENT_LISTENERS.with(|listeners| {
            let mut listeners = listeners.borrow_mut();
            listeners
                .entry(element_id)
                .or_insert_with(HashMap::new)
                .insert(event_name, closure_js);
        });

        closure.forget();
    }

    fn remove_event_listener(&self, element: &Self::Element, event: &str) {
        let element_id = get_element_id(&element.0);

        let closure_js = EVENT_LISTENERS.with(|listeners| {
            let mut listeners = listeners.borrow_mut();
            if let Some(element_listeners) = listeners.get_mut(&element_id) {
                element_listeners.remove(event)
            } else {
                None
            }
        });

        if let Some(closure) = closure_js {
            let callback = wasm_bindgen::JsCast::unchecked_ref::<js_sys::Function>(&closure);
            element
                .0
                .remove_event_listener_with_callback(event, callback)
                .ok();
        }
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
