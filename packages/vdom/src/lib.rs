pub mod callback;
pub mod diff;
pub mod dom_ops;
pub mod events;
pub mod patch;
pub mod platform;
pub mod portal;
pub mod reactive;
pub mod runtime;
pub mod scheduler;
pub mod svg;
pub mod vnode;

pub use callback::{Callback, EventHandler};
pub use dom_ops::{
    DomHandle, DomFuncs, get_bounding_client_rect, register_dom_functions,
    register_wit_functions, set_attribute, set_style,
};
pub use events::{
    AnimationEvent, ChangeEvent, DataTransfer, DragEvent, Event, EventData, EventWitHandle,
    FileData, FocusEvent, FormData, FormEvent, GenericEvent, InputEvent, Key, KeyboardEvent,
    MouseData, MouseEvent, PointerEvent, PointerType, TouchEvent, TouchPoint, TransitionEvent,
    WheelEvent,
};
pub use patch::Patch;
pub use platform::{
    CanvasContext, ContentEditableState, DomRect, ElementHandle, EventHandle, GeoPosition,
    GeoPositionError, MutationObserverInit, MutationRecord, Platform, ResizeObserverEntry,
    ResizeObserverSize,
};
pub use portal::{FixedPosition, Portal, PortalManager, PortalMaskMode, PortalPosition};
pub use reactive::{EffectHandle, Signal, batch, create_effect};
pub use runtime::{
    ComponentId, cleanup_component, flush_render, mark_dirty, notify_signal, request_rerender,
    rerender, subscribe_component, use_component, with_component,
};
pub use scheduler::Scheduler;
pub use svg::SafeSvg;
pub use vnode::{Classes, IntoAttrValue, Style, VElement, VNode, VText, empty_vnode};

#[cfg(target_family = "wasm")]
mod wasm_export {
    wit_bindgen::generate!({
        path: "wit",
        world: "vdom",
    });

    pub struct VdomExports;

    impl exports::tairitsu::vdom::version::Guest for VdomExports {
        fn get_version() -> String {
            env!("CARGO_PKG_VERSION").to_string()
        }
    }

    impl exports::tairitsu::vdom::svg::Guest for VdomExports {
        fn sanitize_svg(content: String) -> String {
            crate::svg::SafeSvg::new(&content).into_content()
        }
    }

    export!(VdomExports);
}
