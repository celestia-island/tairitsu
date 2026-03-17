//! Tairitsu Website - Documentation and Demo
//!
//! 照抄 Hikari 的 website demo 设计

use anyhow::Result;
use tairitsu_web::WitPlatform;
use tracing::error;
use tracing::info;

mod app;
mod components;
mod pages;

pub use app::App;

pub fn run_app() -> Result<()> {
    info!("Tairitsu Website starting...");

    let platform = WitPlatform::new()?;
    let vnode = App.render();
    platform.mount_vnode_to_app(&vnode)?;

    info!("Tairitsu Website loaded and rendered!");
    Ok(())
}

#[no_mangle]
pub extern "C" fn run() {
    if let Err(err) = run_app() {
        error!("website run failed: {err}");
    }
}

#[no_mangle]
pub extern "C" fn tairitsu_component_bootstrap() {
    run();
}
