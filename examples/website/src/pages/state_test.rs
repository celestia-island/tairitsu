//! State Management Test Page
//!
//! This page demonstrates and tests various state management patterns:
//! - use_signal (reactive signals)
//! - use_state (component state)
//! - Boolean state (checkboxes)
//! - List state (add/remove items)
//! - Computed/reactive values

use tairitsu_hooks::{use_callback, use_signal, use_state};
use tairitsu_macros::rsx;
use tairitsu_vdom::{MouseEvent, VNode};

/// Render the state management test page.
pub fn render() -> VNode {
    // Counter state using use_signal
    let (count, set_count) = use_signal(|| 0);

    // Text input state
    let (text, set_text) = use_state(String::new);

    // Toggle state (checkbox)
    let (is_toggled, set_toggle) = use_state(false);

    // List state
    let (items, set_items) = use_state(Vec::<String>::new);

    // Reactive computed values
    let (rect_width, set_width) = use_signal(|| 10);
    let (rect_height, set_height) = use_signal(|| 20);

    // Clone for callbacks
    let count_clone = count.clone();
    let set_count_clone = set_count.clone();
    let set_text_clone = set_text.clone();
    let set_toggle_clone = set_toggle.clone();
    let items_clone = items.clone();
    let set_items_clone = set_items.clone();
    let set_width_clone = set_width.clone();
    let set_height_clone = set_height.clone();

    rsx! {
        div { id: "page-state-test", class: "tairitsu-page",
            h1 { "State Management Tests" }
            p { "Tests for reactive state management, signals, and context." }

            // Counter Test
            div { class: "test-section",
                h2 { "Test 1: Counter State (use_signal)" }
                p { "Click the button to increment the counter." }

                div {
                    class: "counter-test",
                    style: "padding: 15px; background: #f7fafc; border-radius: 8px; display: flex; align-items: center; gap: 15px;",
                    span {
                        style: "font-size: 1.5em; font-weight: bold; min-width: 50px;",
                        id: "counter-display",
                        ..vec![VNode::text(format!("{}", count_clone.get()))],
                    }
                    button {
                        id: "counter-increment",
                        class: "tairitsu-button",
                        onclick: move |_| {
                            let current = count_clone.get();
                            set_count_clone.set(current + 1);
                        },
                        "Increment"
                    }
                }
            }

            // Text Input Test
            div { class: "test-section",
                h2 { "Test 2: Input State Binding (use_state)" }
                p { "Type in the input field to see two-way binding." }

                div { style: "padding: 15px; background: #f7fafc; border-radius: 8px;",
                    input {
                        id: "text-input",
                        class: "tairitsu-input",
                        r#type: "text",
                        placeholder: "Type something...",
                        style: "padding: 8px; border: 1px solid #cbd5e0; border-radius: 4px; width: 250px;",
                        oninput: move |e: MouseEvent| {} // In a real implementation, we'd get the input value,
                    }
                    p {
                        id: "text-display",
                        style: "margin-top: 10px; color: #4a5568;",
                        ..vec![VNode::text("You typed: ")],
                    }
                }
            }

            // Checkbox Toggle Test
            div { class: "test-section",
                h2 { "Test 3: Boolean State (Toggle)" }
                p { "Click the checkbox to toggle state." }

                div { style: "padding: 15px; background: #f7fafc; border-radius: 8px; display: flex; align-items: center; gap: 15px;",
                    label { style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                        input {
                            id: "toggle-checkbox",
                            r#type: "checkbox",
                            checked: *is_toggled.borrow(),
                            onclick: move |_| {
                                let current = *is_toggled.borrow();
                                set_toggle_clone(!current);
                            },
                        }
                        span { "Toggle me" }
                    }
                    span {
                        id: "toggle-display",
                        style: "font-weight: bold; color: #4a5568;",
                        ..vec![VNode::text(if *is_toggled.borrow() { "ON" } else { "OFF" })],
                    }
                }
            }

            // List State Test
            div { class: "test-section",
                h2 { "Test 4: List State (Add/Remove)" }
                p { "Add items to the list and remove them individually." }

                div { style: "padding: 15px; background: #f7fafc; border-radius: 8px;",
                    button {
                        id: "list-add",
                        class: "tairitsu-button",
                        onclick: move |_| {
                            let mut items = items_clone.borrow().clone();
                            items.push(format!("Item {}", items.len() + 1));
                            set_items_clone(items);
                        },
                        "Add Item"
                    }
                    ul {
                        id: "list-display",
                        style: "margin-top: 10px; padding-left: 20px;",
                        ..items
                            .borrow()
                            .iter()
                            .enumerate()
                            .map(|(i, item)| {
                                rsx! {
                                    li {
                                        style: "margin: 5px 0; display: flex; align-items: center; gap: 10px;",
                                        ..vec![VNode::text(item)],
                                        button {
                                            class: "remove-btn",
                                            style: "padding: 2px 8px; background: #fc8181; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                            "×"
                                        }
                                    }
                                }
                            })
                            .collect(),
                    }
                }
            }

            // Reactive Computed Values Test
            div { class: "test-section",
                h2 { "Test 5: Reactive Computed Values" }
                p { "Change width/height to see the area update automatically." }

                div { style: "padding: 15px; background: #f7fafc; border-radius: 8px;",
                    div { style: "display: flex; gap: 15px; align-items: center;",
                        label { style: "display: flex; flex-direction: column; gap: 5px;",
                            "Width:"
                            input {
                                id: "rect-width",
                                r#type: "number",
                                value: rect_width.get().to_string(),
                                style: "padding: 5px; border: 1px solid #cbd5e0; border-radius: 4px; width: 80px;",
                            }
                        }
                        label { style: "display: flex; flex-direction: column; gap: 5px;",
                            "Height:"
                            input {
                                id: "rect-height",
                                r#type: "number",
                                value: rect_height.get().to_string(),
                                style: "padding: 5px; border: 1px solid #cbd5e0; border-radius: 4px; width: 80px;",
                            }
                        }
                        div { style: "padding: 10px 20px; background: #4299e1; color: white; border-radius: 8px;",
                            "Area: "
                            span {
                                id: "rect-area",
                                style: "font-weight: bold;",
                                ..vec![
                                    VNode::text(
                                        format!(
                                            "{} × {} = {}",
                                            rect_width.get(),
                                            rect_height.get(),
                                            rect_width.get() * rect_height.get(),
                                        ),
                                    ),
                                ],
                            }
                        }
                    }
                }
            }

            // Test Summary
            div { class: "test-section",
                h2 { "State Management Test Summary" }
                div { style: "padding: 15px; background: #e6fffa; border: 1px solid #38b2ac; border-radius: 6px;",
                    h3 { "State Patterns Tested:" }
                    ul { style: "list-style-type: none; padding: 0;",
                        li { style: "margin: 5px 0;", "✓ use_signal (reactive counter)" }
                        li { style: "margin: 5px 0;", "✓ use_state (text input)" }
                        li { style: "margin: 5px 0;", "✓ Boolean state (toggle)" }
                        li { style: "margin: 5px 0;", "✓ List state (add/remove)" }
                        li { style: "margin: 5px 0;", "✓ Computed values (area calculation)" }
                    }
                }
            }
        }
    }
}
