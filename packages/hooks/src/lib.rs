pub mod animation;
pub mod callback;
pub mod context;
pub mod effect;
pub mod element_ref;
pub mod memo;
pub mod ref_;
pub mod signal;
pub mod state;
pub mod state_machine;
pub mod store;
pub mod suspense;

pub use animation::{
    AnimationCallback, AnimationConfig, AnimationDirection, AnimationHandle, AnimationState,
    EasingFunction, use_animation, use_simple_animation,
};
pub use callback::{Callback, use_callback, use_return_callback, use_void_callback};
pub use context::{Context, consume_context, provide_context, use_context};
pub use element_ref::{ElementRef, use_element_ref};

// Dioxus compatibility alias
pub use effect::use_effect;
pub use memo::{Memo, use_memo, use_memo_with, use_memo_with_deps};
pub use provide_context as use_context_provider;
pub use ref_::{UseRef, use_ref};
pub use signal::{ReactiveSignal, use_signal};
pub use state::use_state;
pub use state_machine::{
    ButtonStateMachine, InteractionCallback, InteractionEvent, InteractionState,
    use_interaction_state,
};
pub use store::{Store, StoreId, register_store};
pub use suspense::{
    Resource, ResourceState, ResourceStatus, Suspense, SuspenseBoundary, resource_state,
    use_resource, use_suspense,
};

// Re-export Event types from vdom for convenience
pub use tairitsu_vdom::{Event, GenericEvent};

#[cfg(target_family = "wasm")]
mod wasm_export {
    use crate::*;

    wit_bindgen::generate!({
        path: "wit",
        world: "hooks",
    });

    pub struct HooksExports;

    impl exports::tairitsu::hooks::version::Guest for HooksExports {
        fn get_version() -> String {
            env!("CARGO_PKG_VERSION").to_string()
        }
    }

    impl exports::tairitsu::hooks::state_machine::Guest for HooksExports {
        fn transition(
            state: exports::tairitsu::hooks::state_machine::InteractionState,
            event: exports::tairitsu::hooks::state_machine::InteractionEvent,
        ) -> Option<exports::tairitsu::hooks::state_machine::InteractionState> {
            let mut machine = crate::state_machine::ButtonStateMachine::new();
            let native_state = match state {
                exports::tairitsu::hooks::state_machine::InteractionState::Idle => {
                    crate::state_machine::InteractionState::Idle
                }
                exports::tairitsu::hooks::state_machine::InteractionState::Hover => {
                    crate::state_machine::InteractionState::Hover
                }
                exports::tairitsu::hooks::state_machine::InteractionState::Active => {
                    crate::state_machine::InteractionState::Active
                }
                exports::tairitsu::hooks::state_machine::InteractionState::Focused => {
                    crate::state_machine::InteractionState::Focused
                }
                exports::tairitsu::hooks::state_machine::InteractionState::Disabled => {
                    crate::state_machine::InteractionState::Disabled
                }
            };
            let native_event = match event {
                exports::tairitsu::hooks::state_machine::InteractionEvent::MouseEnter => {
                    crate::state_machine::InteractionEvent::MouseEnter
                }
                exports::tairitsu::hooks::state_machine::InteractionEvent::MouseLeave => {
                    crate::state_machine::InteractionEvent::MouseLeave
                }
                exports::tairitsu::hooks::state_machine::InteractionEvent::MouseDown => {
                    crate::state_machine::InteractionEvent::MouseDown
                }
                exports::tairitsu::hooks::state_machine::InteractionEvent::MouseUp => {
                    crate::state_machine::InteractionEvent::MouseUp
                }
                exports::tairitsu::hooks::state_machine::InteractionEvent::Focus => {
                    crate::state_machine::InteractionEvent::Focus
                }
                exports::tairitsu::hooks::state_machine::InteractionEvent::Blur => {
                    crate::state_machine::InteractionEvent::Blur
                }
                exports::tairitsu::hooks::state_machine::InteractionEvent::Disable => {
                    crate::state_machine::InteractionEvent::Disable
                }
                exports::tairitsu::hooks::state_machine::InteractionEvent::Enable => {
                    crate::state_machine::InteractionEvent::Enable
                }
            };
            machine.set_state(native_state);
            let result = machine.transition(native_event)?;
            Some(match result {
                crate::state_machine::InteractionState::Idle => {
                    exports::tairitsu::hooks::state_machine::InteractionState::Idle
                }
                crate::state_machine::InteractionState::Hover => {
                    exports::tairitsu::hooks::state_machine::InteractionState::Hover
                }
                crate::state_machine::InteractionState::Active => {
                    exports::tairitsu::hooks::state_machine::InteractionState::Active
                }
                crate::state_machine::InteractionState::Focused => {
                    exports::tairitsu::hooks::state_machine::InteractionState::Focused
                }
                crate::state_machine::InteractionState::Disabled => {
                    exports::tairitsu::hooks::state_machine::InteractionState::Disabled
                }
            })
        }

        fn is_interactive(
            state: exports::tairitsu::hooks::state_machine::InteractionState,
        ) -> bool {
            let native = match state {
                exports::tairitsu::hooks::state_machine::InteractionState::Idle => {
                    crate::state_machine::InteractionState::Idle
                }
                exports::tairitsu::hooks::state_machine::InteractionState::Hover => {
                    crate::state_machine::InteractionState::Hover
                }
                exports::tairitsu::hooks::state_machine::InteractionState::Active => {
                    crate::state_machine::InteractionState::Active
                }
                exports::tairitsu::hooks::state_machine::InteractionState::Focused => {
                    crate::state_machine::InteractionState::Focused
                }
                exports::tairitsu::hooks::state_machine::InteractionState::Disabled => {
                    crate::state_machine::InteractionState::Disabled
                }
            };
            native.is_interactive()
        }
    }

    impl exports::tairitsu::hooks::animation_types::Guest for HooksExports {
        fn evaluate_easing(
            easing: exports::tairitsu::hooks::animation_types::EasingFunction,
            t: f32,
        ) -> f32 {
            let native = match easing {
                exports::tairitsu::hooks::animation_types::EasingFunction::Linear => {
                    crate::animation::EasingFunction::Linear
                }
                exports::tairitsu::hooks::animation_types::EasingFunction::Ease => {
                    crate::animation::EasingFunction::Ease
                }
                exports::tairitsu::hooks::animation_types::EasingFunction::EaseIn => {
                    crate::animation::EasingFunction::EaseIn
                }
                exports::tairitsu::hooks::animation_types::EasingFunction::EaseOut => {
                    crate::animation::EasingFunction::EaseOut
                }
                exports::tairitsu::hooks::animation_types::EasingFunction::EaseInOut => {
                    crate::animation::EasingFunction::EaseInOut
                }
                exports::tairitsu::hooks::animation_types::EasingFunction::CubicBezier(v) => {
                    let (a, b, c, d) = v;
                    crate::animation::EasingFunction::CubicBezier(a, b, c, d)
                }
            };
            native.evaluate(t)
        }

        fn lerp_f32(from: f32, to: f32, t: f32) -> f32 {
            crate::animation::lerp_f32(from, to, t)
        }
    }

    export!(HooksExports);
}
