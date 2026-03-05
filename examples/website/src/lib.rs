use anyhow::Result;
use tairitsu_hooks::{use_signal, use_state};
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;
use tairitsu_web::WebPlatform;
use wasm_bindgen::prelude::*;
use web_sys::console;

mod app;
mod components;
mod pages;

pub use app::App;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    
    #[cfg(feature = "web")]
    {
        tracing_wasm::set_as_global_default();
    }
    
    console::log_1(&"Tairitsu Website Demo starting...".into());
    
    let platform = WebPlatform::new().map_err(|e| JsValue::from_str(&e.to_string()))?;
    let document = web_sys::window()
        .ok_or_else(|| JsValue::from_str("No window"))?
        .document()
        .ok_or_else(|| JsValue::from_str("No document"))?;
    
    let root = document
        .get_element_by_id("app")
        .ok_or_else(|| JsValue::from_str("No #app element"))?;
    
    let app = App::new();
    let vnode = app.render();
    
    if let VNode::Element(element) = vnode {
        let dom_element = platform.create_element(element.tag);
        root.append_child(&dom_element.0)?;
    }
    
    console::log_1(&"Tairitsu Website Demo loaded!".into());
    Ok(())
}
