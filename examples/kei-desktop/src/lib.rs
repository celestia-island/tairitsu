//! kei vtty — aris-rendered virtual terminal.
//! Dark terminal with green/white ANSI-style text via full Sarasa font.

use anyhow::Result;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;
#[cfg(target_family = "wasm")]
use tairitsu_web::WitPlatform;

pub fn render_desktop() -> VNode {
    rsx! {
        div {
            style: "width:1280px;height:800px;background:#1a1b26;position:relative;font-family:Sarasa Mono SC Nerd;font-size:18px;color:#cdd6f4;overflow:hidden",

            div {
                style: "padding:30px 36px;line-height:1.65",
                "kei 0.1.0 (aarch64)",
            },
            div { style: "padding:0 36px;line-height:1.65", "" },
            div { style: "padding:0 36px;line-height:1.65;color:#89b4fa", "system" },
            div { style: "padding:0 36px;line-height:1.65;color:#89b4fa", "------" },
            div { style: "padding:0 36px;line-height:1.65", "kernel   kei 0.1.0 (aarch64, QEMU virt)" },
            div { style: "padding:0 36px;line-height:1.65", "render   aris-render (Blitz DOM + Vello CPU)" },
            div { style: "padding:0 36px;line-height:1.65", "font     Sarasa Mono SC Nerd" },
            div { style: "padding:0 36px;line-height:1.65", "display  1280x800 /dev/fb0" },
            div { style: "padding:0 36px;line-height:1.65", "" },
            div { style: "padding:0 36px;line-height:1.65;color:#89b4fa", "network" },
            div { style: "padding:0 36px;line-height:1.65;color:#89b4fa", "-------" },
            div { style: "padding:0 36px;line-height:1.65", "webui    kei.celestia.world:8423/ws" },
            div { style: "padding:0 36px;line-height:1.65", "local    10.0.2.15:8423" },
            div { style: "padding:0 36px;line-height:1.65", "proto    ws jsonrpc (arona/_sync)" },
            div { style: "padding:0 36px;line-height:1.65", "" },
            div { style: "padding:0 36px;line-height:1.65;color:#89b4fa", "status   awaiting ws connection" },
            div { style: "padding:0 36px;line-height:1.65", "" },
            div { style: "padding:0 36px;line-height:1.65;color:#cdd6f4", "_" },
        }
    }
}

pub fn run_app() -> Result<()> {
    #[cfg(target_family = "wasm")]
    {
        let platform = WitPlatform::new()?;
        let vnode = render_desktop();
        let _ = platform.mount_vnode_to_app(vnode)?;
    }
    Ok(())
}

#[unsafe(no_mangle)]
pub extern "C" fn tairitsu_component_bootstrap() {
    let _ = run_app();
}

#[unsafe(no_mangle)]
pub extern "C" fn run() {
    let _ = run_app();
}
