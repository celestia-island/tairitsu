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
pub mod limits;
pub mod linker;
#[cfg(feature = "streaming")]
pub mod streaming;
pub mod stubs;
pub mod virtual_dom;

use anyhow::Result;

use bindings::BrowserFull;

// Re-export sign_component for the sign_component_macro! to work
pub use fast_refresh::sign_component;
pub use host_state::{SsrConfig, SsrHostState};
pub use html_render::FullDocumentConfig;
pub use limits::{
    cached_component, component_cache_stats, shared_engine, ComponentCacheStats, RenderOptions,
};
#[cfg(feature = "streaming")]
pub use streaming::{
    hydration_script, render_suspense_boundary, render_to_stream, render_vnode_to_stream,
    HtmlChunk, HtmlStream,
};
pub use virtual_dom::{SsrDom, SsrNode, SsrNodeKind};
use wasmtime::Store;

/// Render a WASM component to HTML
///
/// This is the main entry point for SSR. It loads a WASM component,
/// instantiates it in a Wasmtime runtime with SSR-specific host functions,
/// calls the component's `lifecycle::start()` export, and then extracts
/// the rendered HTML from the in-memory DOM.
///
/// The component is JIT-compiled at most once per process (see
/// [`cached_component`]) and the guest runs under the default execution bounds
/// from [`RenderOptions`], so a component containing `loop {}` or allocating a
/// 4 GiB linear memory fails with an error instead of hanging the host. Use
/// [`render_to_html_with_options`] with [`RenderOptions::trusted`] to opt out.
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
    render_to_html_with_options(wasm_bytes, config, RenderOptions::default())
}

/// Render a WASM component to HTML with an explicit execution policy.
///
/// Identical to [`render_to_html`] except that `options` decides whether the guest
/// is bounded (the default) or trusted to run without fuel, wall-clock or memory
/// limits.
pub fn render_to_html_with_options(
    wasm_bytes: &[u8],
    config: SsrConfig,
    options: RenderOptions,
) -> Result<String> {
    // One engine per process (Winch, fuel/epoch metering enabled) and one compiled
    // component per distinct binary: previously every render built a fresh Engine
    // and re-JIT-compiled the whole component.
    let engine = limits::shared_engine()?;
    let component = limits::cached_component(wasm_bytes)?;

    // Create host state
    let host_state = SsrHostState::with_config(config)?;

    // Create store
    let mut store = Store::new(engine, host_state);

    // Install the guest execution bounds before anything guest-supplied runs.
    limits::apply_limits(&mut store, &options)?;

    // Create linker with allow_shadowing so duplicate func_wrap calls
    // (existing real impls + auto-generated stubs) don't conflict.
    let mut linker = wasmtime::component::Linker::new(engine);
    linker.allow_shadowing(true);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
    register_ssr_imports(&mut linker)?;

    // Instantiate: get the raw instance first (for C exports), then wrap in
    // the bindgen BrowserFull for typed WIT access.
    // Instantiation runs the guest's start functions, so a spinning component is
    // already stopped here when the bounds are in force.
    let instance = linker
        .instantiate(&mut store, &component)
        .map_err(guest_error)?;
    let browser_full = BrowserFull::new(&mut store, &instance)?;

    // Call lifecycle::start using the generated bindings (tolerates absence)
    call_lifecycle_start(&mut store, &browser_full)?;

    // Also try the raw C entry point that tairitsu/hikari components export.
    // Many components use `tairitsu_component_bootstrap()` (a #[no_mangle] C
    // export) rather than the WIT `lifecycle::start` to mount their UI. If the
    // lifecycle call produced no content, calling the bootstrap entry mounts
    // the component's actual UI into the host DOM.
    let pre_html_len = store.data().dom.render_body_html().len();
    if pre_html_len < 50 {
        eprintln!(
            "[ssr] lifecycle produced only {} bytes, trying bootstrap",
            pre_html_len
        );
        // Enumerate the component's top-level exports to find the entry point.
        let comp_ty = component.component_type();
        let exports = comp_ty.exports(engine);
        eprintln!("[ssr] component has {} top-level exports:", exports.len());
        for (name, _) in exports {
            eprintln!("[ssr]   export: '{}'", name);
        }
        // Try each known entry point name
        let entry_names = ["tairitsu_component_bootstrap", "run", "_start", "start"];
        let mut called = false;
        for name in &entry_names {
            if let Some(export_index) = component.get_export_index(None, name) {
                if let Some(func) = instance.get_func(&mut store, export_index) {
                    eprintln!("[ssr] calling '{}' via get_export_index", name);
                    match func.call(&mut store, &[], &mut []) {
                        Ok(_) => {
                            let post_len = store.data().dom.render_body_html().len();
                            eprintln!("[ssr] '{}' OK, DOM {} bytes", name, post_len);
                        }
                        Err(e) => {
                            // A guest killed by one of our own bounds must not be
                            // reported as a successful (truncated) render.
                            if execution_bound_error(&e).is_some() {
                                return Err(guest_error(e));
                            }
                            let post_len = store.data().dom.render_body_html().len();
                            eprintln!("[ssr] '{}' trapped: {} (DOM {} bytes)", name, e, post_len);
                        }
                    }
                    called = true;
                    break;
                }
            }
        }
        if !called {
            eprintln!("[ssr] entry point not found in top-level exports");
        }
    }

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

    // Inject the rendered HTML into the template.
    // Use a single marker with a data attribute to avoid false-positive matches.
    let marker = r#"<div id="app" data-ssr-marker></div>"#;
    let replacement = &format!(r#"<div id="app" data-ssr-marker>{}</div>"#, body_html);

    let full_page = if template.contains(marker) {
        template.replacen(marker, replacement, 1)
    } else {
        // Fallback: look for the conventional marker without data attribute
        let fallback_marker = r#"<div id="app"></div>"#;
        let fallback_replacement = &format!(r#"<div id="app">{}</div>"#, body_html);
        template.replacen(fallback_marker, fallback_replacement, 1)
    };

    Ok(full_page)
}

/// Classify a guest error as "stopped by a host-installed execution bound".
///
/// Returns a human-readable reason when the trap came from the fuel budget or the
/// epoch deadline rather than from a guest-side bug, so callers can tell a runaway
/// component apart from a component that merely traps.
fn execution_bound_error(error: &wasmtime::Error) -> Option<&'static str> {
    match error.downcast_ref::<wasmtime::Trap>()? {
        wasmtime::Trap::OutOfFuel => {
            Some("SSR guest exhausted its fuel budget; the component did not finish rendering")
        }
        wasmtime::Trap::Interrupt => {
            Some("SSR guest exceeded its render deadline; the component did not finish rendering")
        }
        _ => None,
    }
}

/// Convert a guest error into an `anyhow` error, tagging the stops that came from
/// one of our own bounds.
fn guest_error(error: wasmtime::Error) -> anyhow::Error {
    let bound = execution_bound_error(&error);
    let error = anyhow::Error::from(error);
    match bound {
        Some(reason) => error.context(reason),
        None => error,
    }
}

/// Register all SSR WIT implementations with the linker
fn register_ssr_imports(linker: &mut wasmtime::component::Linker<SsrHostState>) -> Result<()> {
    linker::register_ssr_imports_direct(linker)?;
    Ok(())
}

/// Call the lifecycle::start() export on the component
fn call_lifecycle_start(store: &mut Store<SsrHostState>, browser_full: &BrowserFull) -> Result<()> {
    let lifecycle = browser_full.tairitsu_browser_full_lifecycle();

    match lifecycle.call_start(store) {
        Ok(Ok(())) => {
            eprintln!("[ssr] lifecycle::start() returned Ok(Ok(()))");
            Ok(())
        }
        Ok(Err(e)) => {
            eprintln!("[ssr] lifecycle::start() returned Err: {}", e);
            // Don't fail — extract whatever DOM content was written before the error
            Ok(())
        }
        Err(e) => {
            // A guest killed by one of our own bounds (fuel, epoch deadline, memory
            // cap) is a hard error: returning the partially rendered DOM would
            // silently serve a truncated page for a runaway component.
            if execution_bound_error(&e).is_some() {
                return Err(guest_error(e));
            }
            // Print the full error chain to find which import trapped
            let mut chain = vec![format!("{}", e)];
            let mut src = e.source();
            while let Some(s) = src {
                chain.push(format!("{}", s));
                src = s.source();
            }
            eprintln!(
                "[ssr] lifecycle::start() trapped:\n  {}",
                chain.join("\n  ")
            );
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
        assert_eq!(config.current_route, "/");
    }

    #[test]
    fn test_config_new() {
        let config = SsrConfig::new(1280, 720);
        assert_eq!(config.viewport_width, 1280);
        assert_eq!(config.viewport_height, 720);
        assert_eq!(config.current_route, "/");
    }

    #[test]
    fn test_config_with_route() {
        let config = SsrConfig::with_route(1280, 720, "/components/button");
        assert_eq!(config.viewport_width, 1280);
        assert_eq!(config.viewport_height, 720);
        assert_eq!(config.current_route, "/components/button");
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
