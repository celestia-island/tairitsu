//! Tairitsu SSR - Server-Side Rendering for Tairitsu components
//!
//! This crate provides the ability to render Tairitsu components on the server
//! using Wasmtime, producing HTML that can be sent to the client.

pub mod bindings;
pub mod data_fetcher;
pub mod error_overlay;
pub mod fast_refresh;
pub mod hmr;
pub mod host_state;
pub mod html_render;
pub mod linker;
pub mod stubs;
pub mod virtual_dom;

#[cfg(feature = "streaming")]
pub mod streaming;

pub use host_state::{SsrConfig, SsrHostState};
pub use html_render::FullDocumentConfig;
pub use virtual_dom::{SsrDom, SsrNode, SsrNodeKind};

#[cfg(feature = "streaming")]
pub use streaming::{HtmlChunk, HtmlStream, hydration_script, render_suspense_boundary, render_to_stream, render_vnode_to_stream};

use anyhow::Result;
use bindings::BrowserFull;
use wasmtime::{Engine, Store};

/// Render a WASM component to HTML
///
/// This is the main entry point for SSR. It loads a WASM component,
/// instantiates it in a Wasmtime runtime with SSR-specific host functions,
/// calls the component's `lifecycle::start()` export, and then extracts
/// the rendered HTML from the in-memory DOM.
///
/// # Arguments
/// * `wasm_bytes` - The compiled WASM component bytes
/// * `config` - SSR configuration (viewport dimensions, etc.)
///
/// # Returns
/// The rendered HTML as a string
///
/// # Example
/// ```no_run
/// use tairitsu_ssr::{render_to_html, SsrConfig};
///
/// # fn main() -> anyhow::Result<()> {
/// let wasm_bytes = std::fs::read("my_component.wasm")?;
/// let html = render_to_html(&wasm_bytes, SsrConfig::default())?;
/// println!("{}", html);
/// # Ok(())
/// # }
/// ```
pub fn render_to_html(wasm_bytes: &[u8], config: SsrConfig) -> Result<String> {
    // Create engine
    let mut engine_config = wasmtime::Config::new();
    engine_config.wasm_component_model(true);
    let engine = Engine::new(&engine_config)?;

    // Create component from bytes
    let component = wasmtime::component::Component::from_binary(&engine, wasm_bytes)?;

    // Create host state
    let host_state = SsrHostState::with_config(config)?;

    // Create store
    let mut store = Store::new(&engine, host_state);

    // Create linker and register imports
    let mut linker = wasmtime::component::Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

    // Register SSR-specific imports
    register_ssr_imports(&mut linker)?;

    // Instantiate the component using the bindgen-generated bindings
    // This gives us type-safe access to all exports including lifecycle::start
    let browser_full = BrowserFull::instantiate(&mut store, &component, &linker)?;

    // Call lifecycle::start using the generated bindings
    call_lifecycle_start(&mut store, &browser_full)?;

    // Extract HTML from the DOM
    let html = store.data().dom.render_body_html();

    Ok(html)
}

/// Render a WASM component to a full HTML page
///
/// This function renders the component and injects the result into
/// an HTML template. The template should contain a `<div id="app"></div>`
/// element that will be replaced with the rendered content.
///
/// # Arguments
/// * `wasm_bytes` - The compiled WASM component bytes
/// * `config` - SSR configuration
/// * `template` - The HTML template (typically an index.html)
///
/// # Returns
/// The complete HTML page with rendered content
pub fn render_full_page(wasm_bytes: &[u8], config: SsrConfig, template: &str) -> Result<String> {
    let body_html = render_to_html(wasm_bytes, config)?;

    // Inject the rendered HTML into the template
    let full_page = template.replace(
        "<div id=\"app\"></div>",
        &format!("<div id=\"app\">{}</div>", body_html),
    );

    Ok(full_page)
}

/// Register all SSR WIT implementations with the linker
fn register_ssr_imports(linker: &mut wasmtime::component::Linker<SsrHostState>) -> Result<()> {
    linker::register_ssr_imports_direct(linker)?;
    Ok(())
}

/// Call the lifecycle::start() export on the component
fn call_lifecycle_start(store: &mut Store<SsrHostState>, browser_full: &BrowserFull) -> Result<()> {
    // Use the bindgen-generated bindings to call lifecycle::start
    // The bindings provide type-safe access to all exported interfaces
    let lifecycle = browser_full.tairitsu_browser_full_lifecycle();

    // Call the start function
    match lifecycle.call_start(store) {
        Ok(Ok(())) => {
            println!("lifecycle::start() called successfully");
            Ok(())
        }
        Ok(Err(e)) => {
            // The start function returned an error
            let error_msg = e.to_string();
            Err(anyhow::anyhow!(
                "lifecycle::start returned error: {}",
                error_msg
            ))
        }
        Err(e) => {
            // Failed to call the function (e.g., the export doesn't exist)
            // This is acceptable for components that don't have lifecycle::start
            println!("lifecycle::start not available or failed to call: {}", e);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = SsrConfig::default();
        assert_eq!(config.viewport_width, 1920);
        assert_eq!(config.viewport_height, 1080);
    }

    #[test]
    fn test_config_new() {
        let config = SsrConfig::new(1280, 720);
        assert_eq!(config.viewport_width, 1280);
        assert_eq!(config.viewport_height, 720);
    }

    #[test]
    fn test_dom_creation() {
        let dom = SsrDom::new();
        assert_ne!(dom.body_handle(), 0);
        assert_ne!(dom.head_handle(), 0);
    }

    #[test]
    fn test_dom_html_render() {
        let mut dom = SsrDom::new();
        let div = dom.create_element("div", None);
        dom.get_node_mut(div)
            .unwrap()
            .set_attribute("class", "test");
        dom.append_child(dom.body_handle(), div).unwrap();

        let html = dom.render_body_html();
        assert!(html.contains("<div"));
        assert!(html.contains("class=\"test\""));
        assert!(html.contains("</div>"));
    }
}
