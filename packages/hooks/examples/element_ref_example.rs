//! Example demonstrating the use of `use_element_ref` hook.
//!
//! This example shows how to:
//! 1. Create an element ref using `use_element_ref()`
//! 2. Attach it to an element using the `ref_:` attribute in rsx!
//! 3. Access the element handle after mounting

use tairitsu_hooks::use_element_ref;
use tairitsu_vdom::{VElement, VNode};

/// Component demonstrating basic element ref usage.
///
/// The ref will be populated with the element handle when
/// this component is mounted to the DOM.
pub fn ref_example() -> VNode {
    // Create an element ref for u64-based element handles (e.g., WitElement)
    let div_ref = use_element_ref::<u64>();

    // Initially, the ref is None
    assert!(div_ref.get().is_none());

    // Create a VElement with the ref attached
    let element = VElement::new("div")
        .ref_(div_ref.as_any_ref())
        .child(VNode::Text(tairitsu_vdom::VText::new("Hello with ref!")));

    // Simulate what happens during mounting:
    // The platform would call div_ref.set(element_handle) when the element is created
    // For demonstration:
    div_ref.set(42u64);

    // Now the ref has the element handle
    if let Some(handle) = div_ref.get() {
        println!("Element handle: {}", handle);
    }

    VNode::Element(element)
}

/// Example showing how refs can be cloned and shared.
///
/// Multiple refs can point to the same underlying element.
pub fn ref_clone_example() -> VNode {
    let ref1 = use_element_ref::<u64>();
    let ref2 = ref1.clone(); // Both share the same underlying storage

    let element = VElement::new("div")
        .ref_(ref1.as_any_ref())
        .child(VNode::Text(tairitsu_vdom::VText::new("Shared ref")));

    // After mounting, both refs will have the same value
    ref1.set(100u64);

    assert_eq!(ref1.get(), Some(100u64));
    assert_eq!(ref2.get(), Some(100u64));

    VNode::Element(element)
}

/// Example showing ref lifecycle.
pub fn ref_lifecycle_example() -> VNode {
    let ref_handle = use_element_ref::<u64>();

    // Initially None
    assert!(ref_handle.get().is_none());

    // Set during mount
    ref_handle.set(42);
    assert_eq!(ref_handle.get(), Some(42));

    // Clear during unmount/reconciliation
    ref_handle.clear();
    assert!(ref_handle.get().is_none());

    // Take transfers ownership
    ref_handle.set(100);
    let taken = ref_handle.take();
    assert_eq!(taken, Some(100));
    assert!(ref_handle.get().is_none());

    VNode::Element(VElement::new("div").child(VNode::Text(tairitsu_vdom::VText::new(
        "Lifecycle demo",
    ))))
}

fn main() {
    println!("Running element ref examples...");

    println!("\n=== Basic Ref Example ===");
    let vnode = ref_example();
    println!("Created VNode: {:?}", vnode);

    println!("\n=== Clone Ref Example ===");
    let vnode = ref_clone_example();
    println!("Created VNode: {:?}", vnode);

    println!("\n=== Lifecycle Ref Example ===");
    let vnode = ref_lifecycle_example();
    println!("Created VNode: {:?}", vnode);

    println!("\nAll examples completed successfully!");
}
