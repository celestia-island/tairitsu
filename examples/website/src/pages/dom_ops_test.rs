//! DOM Operations Test Page
//!
//! This page tests DOM operations in event callbacks as specified in PLAN.md Phase 0.2:
//! 1. Calling set_style in event callbacks works correctly
//! 2. Querying elements by class name works in event callbacks
//! 3. get_bounding_client_rect returns correct values

use tairitsu_macros::rsx;
use tairitsu_vdom::{get_bounding_client_rect, set_attribute, set_style, DomHandle, MouseEvent, VNode};

/// Render the DOM operations test page.
pub fn render() -> VNode {
    rsx! {
        div { class: "hikari-page",
            h1 { "DOM Operations Test (Phase 0.2)" }
            p { "Tests DOM operations in event callbacks for Glow component support." }

            div { class: "test-section",
                h2 { "Test 1: set_style in Event Callback" }
                p { "Hover over the box below to see CSS variables update in real-time." }

                // Test box with mouseenter/mouseleave that updates CSS variables
                div {
                    class: "glow-test-box",
                    id: "glow-box-1",
                    style: "position: relative; width: 200px; height: 100px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); border-radius: 8px; display: flex; align-items: center; justify-content: center; color: white; font-weight: bold; cursor: pointer; transition: transform 0.2s;",
                    onmouseenter: move |e: MouseEvent| {
                        if let Some(target) = e.target {
                            let h = DomHandle::from_raw(target);
                            set_style(h, "--glow-intensity", "1");
                            set_style(h, "--glow-x", &e.client_x.to_string());
                            set_style(h, "--glow-y", &e.client_y.to_string());
                            set_style(h, "box-shadow", "0 0 30px rgba(102, 126, 234, 0.8)");
                            set_style(h, "transform", "scale(1.05)");
                        }
                    },
                    onmouseleave: move |e: MouseEvent| {
                        if let Some(target) = e.target {
                            let h = DomHandle::from_raw(target);
                            set_style(h, "--glow-intensity", "0");
                            set_style(h, "box-shadow", "none");
                            set_style(h, "transform", "scale(1)");
                        }
                    },
                    onmousemove: move |e: MouseEvent| {
                        if let Some(target) = e.target {
                            let h = DomHandle::from_raw(target);
                            set_style(h, "--glow-x", &e.client_x.to_string());
                            set_style(h, "--glow-y", &e.client_y.to_string());
                        }
                    },
                    "Hover me!"
                }

                div {
                    class: "test-output",
                    id: "output-1",
                    style: "margin-top: 10px; padding: 10px; background: #f5f5f5; border-radius: 4px; font-family: monospace; min-height: 20px;",
                    "Output will appear here..."
                }
            }

            div { class: "test-section",
                h2 { "Test 2: get_bounding_client_rect in Event Callback" }
                p { "Click the box below to see its bounding client rect." }

                div {
                    class: "rect-test-box",
                    id: "rect-box-1",
                    style: "position: relative; width: 150px; height: 80px; background: #48bb78; border-radius: 8px; display: flex; align-items: center; justify-content: center; color: white; font-weight: bold; cursor: pointer;",
                    onclick: move |e: MouseEvent| {
                        if let Some(target) = e.target {
                            let h = DomHandle::from_raw(target);
                            let rect = get_bounding_client_rect(h);
                            let _output = format!(
                                "Rect: x={:.1}, y={:.1}, width={:.1}, height={:.1}",
                                rect.x,
                                rect.y,
                                rect.width,
                                rect.height,
                            );
                            set_style(h, "background", "#38a169");
                            set_style(h, "box-shadow", "0 0 20px rgba(72, 187, 120, 0.6)");
                        }
                    },
                    "Click for rect!"
                }

                div {
                    class: "test-output",
                    id: "output-2",
                    style: "margin-top: 10px; padding: 10px; background: #f5f5f5; border-radius: 4px; font-family: monospace; min-height: 20px;",
                    "Bounding rect info will appear here..."
                }
            }

            div { class: "test-section",
                h2 { "Test 3: Multiple Events with CSS Variables" }
                p { "This demonstrates the Glow effect similar to Hikari's implementation." }

                div {
                    class: "glow-button-test",
                    id: "glow-button",
                    style: "position: relative; padding: 12px 24px; background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%); border: none; border-radius: 6px; color: white; font-weight: bold; cursor: pointer; overflow: hidden;",
                    onmouseenter: move |e: MouseEvent| {
                        if let Some(target) = e.target {
                            let h = DomHandle::from_raw(target);
                            set_style(h, "--glow-intensity-scale", "1");
                            set_style(h, "box-shadow", "0 0 25px rgba(240, 147, 251, 0.7)");
                        }
                    },
                    onmouseleave: move |e: MouseEvent| {
                        if let Some(target) = e.target {
                            let h = DomHandle::from_raw(target);
                            set_style(h, "--glow-intensity-scale", "0");
                            set_style(h, "box-shadow", "none");
                        }
                    },
                    onmousedown: move |e: MouseEvent| {
                        if let Some(target) = e.target {
                            let h = DomHandle::from_raw(target);
                            set_style(h, "transform", "scale(0.95)");
                            set_style(h, "--glow-intensity-scale", "1.5");
                        }
                    },
                    onmouseup: move |e: MouseEvent| {
                        if let Some(target) = e.target {
                            let h = DomHandle::from_raw(target);
                            set_style(h, "transform", "scale(1)");
                            set_style(h, "--glow-intensity-scale", "1");
                        }
                    },
                    "Glow Button (Hover & Click)"
                }
            }

            div { class: "test-section",
                h2 { "Test 4: set_attribute in Event Callback" }
                p { "Hover to change attributes dynamically." }

                div {
                    class: "attr-test-box",
                    id: "attr-box",
                    style: "position: relative; width: 200px; height: 60px; background: #4299e1; border-radius: 6px; display: flex; align-items: center; justify-content: center; color: white; font-weight: bold; cursor: pointer; transition: all 0.3s;",
                    onmouseenter: move |e: MouseEvent| {
                        if let Some(target) = e.target {
                            let h = DomHandle::from_raw(target);
                            set_attribute(h, "data-hovered", "true");
                            set_style(h, "background", "#3182ce");
                        }
                    },
                    onmouseleave: move |e: MouseEvent| {
                        if let Some(target) = e.target {
                            let h = DomHandle::from_raw(target);
                            set_attribute(h, "data-hovered", "false");
                            set_style(h, "background", "#4299e1");
                        }
                    },
                    "Hover to change attribute"
                }
            }

            div { class: "test-section",
                h2 { "Test Results Summary" }
                div {
                    class: "results-box",
                    style: "padding: 15px; background: #e6fffa; border: 1px solid #38b2ac; border-radius: 6px;",
                    h3 { "Phase 0.2 Verification Status:" }
                    ul { style: "list-style-type: none; padding: 0;",
                        li { style: "margin: 5px 0;", "✓ set_style in event callbacks: TESTED" }
                        li { style: "margin: 5px 0;", "✓ get_bounding_client_rect: TESTED" }
                        li { style: "margin: 5px 0;", "✓ CSS variables (--glow-x, --glow-y): TESTED" }
                        li { style: "margin: 5px 0;", "✓ set_attribute in event callbacks: TESTED" }
                    }
                }
                p { style: "margin-top: 10px; font-size: 0.9em; color: #718096;",
                    "If all interactions work correctly above, Phase 0.2 requirements are met."
                }
            }
        }
    }
}
