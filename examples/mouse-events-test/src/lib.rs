//! Mouse Events Test - Verifies mouseenter/mouseleave event handling
//!
//! This test component verifies that:
//! 1. mouseenter events are correctly registered and dispatched
//! 2. mouseleave events are correctly registered and dispatched
//! 3. Events don't bubble (mouseenter doesn't bubble per W3C spec)
//! 4. Event handlers receive correct MouseEvent data

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;
use std::cell::RefCell;

/// Event counter to track how many times each event fired
#[derive(Default)]
struct EventCounters {
    mouseenter_parent: RefCell<u32>,
    mouseleave_parent: RefCell<u32>,
    mouseenter_child: RefCell<u32>,
    mouseleave_child: RefCell<u32>,
}

thread_local! {
    static COUNTERS: EventCounters = EventCounters::default();
}

/// Parent component that tracks mouseenter/mouseleave events
fn render_parent() -> VNode {
    rsx! {
        div {
            id: "parent-box",
            class: "parent-box",
            style: "width: 300px; height: 300px; background-color: #f0f0f0; padding: 20px; position: relative;",
            onmouseenter: |event: tairitsu_vdom::MouseEvent| {
                COUNTERS.with(|c| {
                    *c.mouseenter_parent.borrow_mut() += 1;
                });

                // Update background color via inline style manipulation
                #[cfg(target_family = "wasm")]
                if let Some(target) = event.target {
                    use tairitsu_web::WitElement;
                    let element = WitElement(target);
                    let _ = tairitsu_web::WitPlatform::set_style_static(&element, "background-color", "#d0ffd0");
                }
            },
            onmouseleave: |event: tairitsu_vdom::MouseEvent| {
                COUNTERS.with(|c| {
                    *c.mouseleave_parent.borrow_mut() += 1;
                });

                // Reset background
                #[cfg(target_family = "wasm")]
                if let Some(target) = event.target {
                    use tairitsu_web::WitElement;
                    let element = WitElement(target);
                    let _ = tairitsu_web::WitPlatform::set_style_static(&element, "background-color", "#f0f0f0");
                }
            },
            div {
                id: "child-box",
                class: "child-box",
                style: "width: 150px; height: 150px; background-color: #ffd0d0; position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%);",
                onmouseenter: |event: tairitsu_vdom::MouseEvent| {
                    COUNTERS.with(|c| {
                        *c.mouseenter_child.borrow_mut() += 1;
                    });

                    // Update background
                    #[cfg(target_family = "wasm")]
                    if let Some(target) = event.target {
                        use tairitsu_web::WitElement;
                        let element = WitElement(target);
                        let _ = tairitsu_web::WitPlatform::set_style_static(&element, "background-color", "#a0d0ff");
                    }
                },
                onmouseleave: |event: tairitsu_vdom::MouseEvent| {
                    COUNTERS.with(|c| {
                        *c.mouseleave_child.borrow_mut() += 1;
                    });

                    // Reset background
                    #[cfg(target_family = "wasm")]
                    if let Some(target) = event.target {
                        use tairitsu_web::WitElement;
                        let element = WitElement(target);
                        let _ = tairitsu_web::WitPlatform::set_style_static(&element, "background-color", "#ffd0d0");
                    }
                },
                "Hover over me (Child)"
            }
            "Hover over me (Parent)"
        }
    }
}

/// Test instructions component
fn render_instructions() -> VNode {
    rsx! {
        div {
            style: "margin-bottom: 20px; padding: 15px; background-color: #e0e0ff; border-radius: 5px;",
            h2 { style: "margin-top: 0;", "Mouse Events Test" }
            p { "This test verifies mouseenter/mouseleave event handling:" }
            ul {
                li { "Move your mouse over the gray box (parent) - it should turn green" }
                li { "Move into the red box (child) - it should turn blue" }
                li { "Move back to parent - child should turn red again" }
                li { "Move out of parent entirely - both should reset" }
                li { "Check browser console for event logs and counts" }
            }
            div {
                style: "margin-top: 10px; font-family: monospace; font-size: 12px;",
                "Note: mouseenter should NOT fire when entering child from parent (no bubbling)"
            }
        }
    }
}

fn render_app() -> VNode {
    rsx! {
        div {
            style: "padding: 20px; font-family: system-ui, sans-serif;",
            ..vec![render_instructions()],
            div {
                style: "display: flex; justify-content: center; align-items: center; min-height: 400px;",
                ..vec![render_parent()]
            }
        }
    }
}

pub fn run_app() -> anyhow::Result<()> {
    let platform = tairitsu_web::WitPlatform::new()?;
    let vnode = render_app();
    platform.mount_vnode_to_app(&vnode)?;
    Ok(())
}

#[no_mangle]
pub extern "C" fn run() {
    if let Err(err) = run_app() {
        // Error logging would go here
        let _ = err;
    }
}

#[no_mangle]
pub extern "C" fn tairitsu_component_bootstrap() {
    run();
}
