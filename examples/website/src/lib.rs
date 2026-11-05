//! Tairitsu Website - Documentation and Demo

mod app;
mod components;
mod i18n;
mod markdown;
mod pages;
mod theme;

use anyhow::Result;

pub use app::App;
#[cfg(target_family = "wasm")]
#[cfg(target_family = "wasm")]
use tairitsu_web::wit_platform::WitElement;
#[cfg(target_family = "wasm")]
use tairitsu_web::WitPlatform;
use tracing::{error, info};

pub fn run_app() -> Result<()> {
    info!("Tairitsu Website starting...");

    #[cfg(target_family = "wasm")]
    {
        let platform = WitPlatform::new()?;
        let vnode = App.render();
        // mount returns the rendered vnode root element; diff patches are
        // relative to this element, so init_runtime/apply_patches must use
        // the same root (using the mount parent resolves top-level patch
        // indexes one level too high and silently drops updates).
        let root_element = platform.mount_vnode_to_app(vnode.clone())?;
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
