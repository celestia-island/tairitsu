#![cfg_attr(not(target_family = "wasm"), allow(dead_code))]

#[cfg(target_family = "wasm")]
mod bindings {
    use super::Plugin;

    wit_bindgen::generate!({
        path: "wit",
        world: "plugin",
    });

    export!(Plugin);
}

#[cfg(target_family = "wasm")]
pub struct Plugin;

#[cfg(target_family = "wasm")]
impl bindings::Guest for Plugin {
    fn greet(name: String) -> String {
        format!("Hello, {}! From WASI plugin.", name)
    }

    fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    fn echo_list(items: Vec<String>) -> Vec<String> {
        items.into_iter().map(|s| format!("echo: {}", s)).collect()
    }
}
