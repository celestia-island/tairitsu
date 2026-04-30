//! Tairitsu Website - Documentation and Demo

mod app;
mod components;
mod i18n;
mod pages;
mod theme;

use anyhow::Result;

pub use app::App;
use tracing::{error, info};

#[cfg(target_family = "wasm")]
use tairitsu_web::WitPlatform;

#[cfg(target_family = "wasm")]
use tairitsu_web::wit_platform::wasm_impl::bindings::tairitsu_browser::full::non_element_parent_node::get_element_by_id;

#[cfg(target_family = "wasm")]
use tairitsu_web::wit_platform::WitElement;

pub fn run_app() -> Result<()> {
    info!("Tairitsu Website starting...");

    #[cfg(target_family = "wasm")]
    {
        let platform = WitPlatform::new()?;
        let vnode = App.render();
        platform.mount_vnode_to_app(vnode.clone())?;

        let root_handle = get_element_by_id(0, "app").expect("#app element not found");
        let root_element = WitElement(root_handle);
        tairitsu_web::init_runtime(root_element);

        let component_id = tairitsu_vdom::use_component(|| App.render());
        tairitsu_vdom::runtime::store_initial_vnode(component_id, vnode);
    }

    #[cfg(not(target_family = "wasm"))]
    {
        let _vnode = App.render();
    }

    info!("Tairitsu Website loaded and rendered!");
    Ok(())
}

#[unsafe(no_mangle)]
pub extern "C" fn run() {
    if let Err(err) = run_app() {
        error!("website run failed: {err}");
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tairitsu_component_bootstrap() {
    run();
}
