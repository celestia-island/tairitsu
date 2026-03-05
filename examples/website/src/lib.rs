use anyhow::Result;
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

    console::log_1(&"Tairitsu Website Demo starting...".into());

    let _platform = WebPlatform::new().map_err(|e| JsValue::from_str(&e.to_string()))?;
    let document = web_sys::window()
        .ok_or_else(|| JsValue::from_str("No window"))?
        .document()
        .ok_or_else(|| JsValue::from_str("No document"))?;

    let _root = document
        .get_element_by_id("app")
        .ok_or_else(|| JsValue::from_str("No #app element"))?;

    let _vnode = App.render();

    console::log_1(&"Tairitsu Website Demo loaded!".into());
    Ok(())
}
