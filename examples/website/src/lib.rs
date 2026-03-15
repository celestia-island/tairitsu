use anyhow::Result;
use tairitsu_web::WitPlatform;
use tracing::info;

mod app;
mod components;
mod pages;

pub use app::App;

pub fn run() -> Result<()> {
    info!("Tairitsu Website Demo starting...");

    let _platform = WitPlatform::new()?;

    let _vnode = App.render();

    info!("Tairitsu Website Demo loaded!");
    Ok(())
}
