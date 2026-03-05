use tairitsu_macros::rsx;
use wasm_bindgen::prelude::*;
use web_sys::window;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    let document = window()
        .expect("no window")
        .document()
        .expect("no document");

    let root = rsx! {
        div {
            class: "container",
            style: "max-width: 800px; margin: 0 auto; padding: 20px;",

            h1 {
                "Tairitsu Web Demo"
            }

            p {
                "WASM module loaded successfully!"
            }
        }
    };

    let body = document.body().expect("no body");
    let message = document
        .create_element("div")
        .expect("create element failed");
    message.set_text_content(Some(&format!("Tairitsu WASM loaded! VNode: {:?}", root)));
    body.append_child(&message).expect("append failed");
}
