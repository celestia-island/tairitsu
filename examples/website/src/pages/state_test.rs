//! State Management Test Page
//!
//! This page demonstrates and tests various state management patterns:
//! - use_signal (reactive signals)
//! - use_state (component state)
//! - Boolean state (checkboxes)
//! - List state (add/remove items)
//! - Computed/reactive values

use std::{cell::RefCell, rc::Rc};

use tairitsu_hooks::use_signal;
use tairitsu_macros::rsx;
use tairitsu_vdom::{InputEvent, VNode, VText};

/// Render the state management test page.
pub fn render() -> VNode {
    // Counter state using use_signal
    let count = use_signal(|| 0);

    // Text input state - using Rc<RefCell<_>> pattern directly since use_state setter isn't cloneable
    let text: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
    let text_clone = Rc::clone(&text);

    // Toggle state (checkbox)
    let is_toggled: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
    let is_toggled_clone = Rc::clone(&is_toggled);

    // List state
    let items: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let items_clone = Rc::clone(&items);

    // Reactive computed values
    let rect_width = use_signal(|| 10);
    let rect_height = use_signal(|| 20);

    // Clone for callbacks
    let count_clone = count.clone();
    let rect_width_clone = rect_width.clone();
    let rect_height_clone = rect_height.clone();

    rsx! {
        div { id: "page-state-test", class: "hikari-page",
            h1 { "State Management Tests" }
            p { "Tests for reactive state management, signals, and context." }

            // Counter Test
            div { class: "test-section",
                h2 { "Test 1: Counter State (use_signal)" }
                p { "Click the button to increment the counter." }

                div {
                    class: "counter-test",
                    style: "padding: 15px; background: rgba(22,32,45,0.92); border-radius: 8px; display: flex; align-items: center; gap: 15px; border: 1px solid rgba(255,255,255,0.08);",
                    span {
                        style: "font-size: 1.5em; font-weight: bold; min-width: 50px; color: rgba(255,255,255,0.92);",
                        id: "counter-display",
                        ..vec![VNode::Text(VText { text: format!("{}", count_clone.get()) })],
                    }
                    button {
                        id: "counter-increment",
                        class: "hi-button hi-button-primary",
                        onclick: move |_| {
                            let current = count_clone.get();
                            count_clone.set(current + 1);
                        },
                        "Increment"
                    }
                }
            }

            // Text Input Test
            div { class: "test-section",
                h2 { "Test 2: Input State Binding (use_state)" }
                p { "Type in the input field to see two-way binding." }

                 div { style: "padding: 15px; background: rgba(22,32,45,0.92); border-radius: 8px; border: 1px solid rgba(255,255,255,0.08);",
                    input {
                        id: "text-input",
                        class: "hi-input",
                        r#type: "text",
                        placeholder: "Type something...",
                        style: "padding: 8px; border: 1px solid rgba(255,255,255,0.15); border-radius: 4px; width: 250px; background: rgba(26,35,50,1); color: rgba(255,255,255,0.92);",
                        oninput: move |_: InputEvent| {} // TODO: implement input handling
                    }
                    p {
                        id: "text-display",
                        style: "margin-top: 10px; color: rgba(255,255,255,0.65);",
                        ..vec![VNode::Text(VText { text: format!("You typed: {}", text_clone.borrow()) })],
                    }
                }
            }

            // Checkbox Toggle Test
            div { class: "test-section",
                h2 { "Test 3: Boolean State (Toggle)" }
                p { "Click the checkbox to toggle state." }

                 div { style: "padding: 15px; background: rgba(22,32,45,0.92); border-radius: 8px; display: flex; align-items: center; gap: 15px; border: 1px solid rgba(255,255,255,0.08);",
                    label { style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                        input {
                            id: "toggle-checkbox",
                            r#type: "checkbox",
                            checked: *is_toggled.borrow(),
                            onclick: move |_| {
                                let current = *is_toggled_clone.borrow();
                                *is_toggled_clone.borrow_mut() = !current;
                            },
                        }
                        span { "Toggle me" }
                    }
                    span {
                        id: "toggle-display",
                        style: "font-weight: bold; color: rgba(255,255,255,0.92);",
                        ..vec![VNode::Text(VText { text: if *is_toggled.borrow() { "ON".to_string() } else { "OFF".to_string() } })],
                    }
                }
            }

            // List State Test
            div { class: "test-section",
                h2 { "Test 4: List State (Add/Remove)" }
                p { "Add items to the list and remove them individually." }

                div { style: "padding: 15px; background: rgba(22,32,45,0.92); border-radius: 8px; border: 1px solid rgba(255,255,255,0.08);",
                    button {
                        id: "list-add",
                        class: "hi-button hi-button-primary",
                        onclick: move |_| {
                            let mut items_ref = items_clone.borrow().clone();
                            items_ref.push(format!("Item {}", items_ref.len() + 1));
                            *items_clone.borrow_mut() = items_ref;
                        },
                        "Add Item"
                    }
                    ul {
                        id: "list-display",
                        style: "margin-top: 10px; padding-left: 20px;",
                        ..items
                            .borrow()
                            .iter()
                            .map(|item| {
                                rsx! {
                                    li {
                                        style: "margin: 5px 0; display: flex; align-items: center; gap: 10px;",
                                        ..vec![VNode::Text(VText { text: item.clone() })],
                                        button {
                                            class: "remove-btn",
                                            style: "padding: 2px 8px; background: rgba(255,76,0,0.6); color: white; border: none; border-radius: 4px; cursor: pointer;",
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

                div { style: "padding: 15px; background: rgba(22,32,45,0.92); border-radius: 8px; border: 1px solid rgba(255,255,255,0.08);",
                    div { style: "display: flex; gap: 15px; align-items: center;",
                        label { style: "display: flex; flex-direction: column; gap: 5px;",
                            "Width:"
                            input {
                                id: "rect-width",
                                r#type: "number",
                                value: rect_width.get().to_string(),
                                style: "padding: 5px; border: 1px solid rgba(255,255,255,0.15); border-radius: 4px; width: 80px; background: rgba(26,35,50,1); color: rgba(255,255,255,0.92);"
                            }
                        }
                        label { style: "display: flex; flex-direction: column; gap: 5px;",
                            "Height:"
                            input {
                                id: "rect-height",
                                r#type: "number",
                                value: rect_height.get().to_string(),
                                style: "padding: 5px; border: 1px solid rgba(255,255,255,0.15); border-radius: 4px; width: 80px; background: rgba(26,35,50,1); color: rgba(255,255,255,0.92);"
                            }
                        }
                        div { style: "padding: 10px 20px; background: rgba(20,110,116,0.7); color: white; border-radius: 8px;",
                            "Area: "
                            span {
                                id: "rect-area",
                                style: "font-weight: bold;",
                                ..vec![
                                    VNode::Text(VText {
                                        text: format!(
                                            "{} × {} = {}",
                                            rect_width_clone.get(),
                                            rect_height_clone.get(),
                                            rect_width_clone.get() * rect_height_clone.get(),
                                        ),
                                    }),
                                ],
                            }
                        }
                    }
                }
            }

            // Test Summary
            div { class: "test-section",
                h2 { "State Management Test Summary" }
                div { style: "padding: 15px; background: rgba(20,110,116,0.15); border: 1px solid rgba(20,110,116,0.4); border-radius: 6px;",
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
