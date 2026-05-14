pub mod animation;
pub mod callback;
pub mod context;
pub mod dom_ref;
pub mod effect;
pub mod element_ref;
pub mod interval;
pub mod memo;
pub mod ref_;
pub mod signal;
pub mod state;
pub mod state_machine;
pub mod store;
pub mod suspense;

pub use animation::{
    use_animation, use_simple_animation, AnimationCallback, AnimationConfig, AnimationDirection,
    AnimationHandle, AnimationState, EasingFunction,
};
pub use callback::{use_callback, use_return_callback, use_void_callback, Callback};
pub use context::{consume_context, provide_context, use_context, Context};
pub use dom_ref::{use_dom_ref, DomRef};
pub use element_ref::{use_element_ref, ElementRef};

// Dioxus compatibility alias
pub use effect::use_effect;
pub use interval::{use_interval, IntervalHandle};
pub use memo::{use_memo, use_memo_with, use_memo_with_deps, Memo};
pub use provide_context as use_context_provider;
pub use ref_::{use_ref, UseRef};
pub use signal::{use_signal, use_standalone_signal, ReactiveSignal, StandaloneSignal};
pub use state::use_state;
pub use state_machine::{
    use_interaction_state, ButtonStateMachine, InteractionCallback, InteractionEvent,
    InteractionState,
};
pub use store::{register_store, Store, StoreId};
pub use suspense::{
    resource_state, use_resource, use_suspense, Resource, ResourceState, ResourceStatus, Suspense,
    SuspenseBoundary,
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
            let native_state = match state {
                exports::tairitsu::hooks::state_machine::InteractionState::Idle => {
                    state_machine::InteractionState::Idle
                }
                exports::tairitsu::hooks::state_machine::InteractionState::Hover => {
                    state_machine::InteractionState::Hover
                }
                exports::tairitsu::hooks::state_machine::InteractionState::Active => {
                    state_machine::InteractionState::Active
                }
                exports::tairitsu::hooks::state_machine::InteractionState::Focused => {
                    state_machine::InteractionState::Focused
                }
                exports::tairitsu::hooks::state_machine::InteractionState::Disabled => {
                    state_machine::InteractionState::Disabled
                }
            };
            let native_event = match event {
                exports::tairitsu::hooks::state_machine::InteractionEvent::MouseEnter => {
                    state_machine::InteractionEvent::MouseEnter
                }
                exports::tairitsu::hooks::state_machine::InteractionEvent::MouseLeave => {
                    state_machine::InteractionEvent::MouseLeave
                }
                exports::tairitsu::hooks::state_machine::InteractionEvent::MouseDown => {
                    state_machine::InteractionEvent::MouseDown
                }
                exports::tairitsu::hooks::state_machine::InteractionEvent::MouseUp => {
                    state_machine::InteractionEvent::MouseUp
                }
                exports::tairitsu::hooks::state_machine::InteractionEvent::Focus => {
                    state_machine::InteractionEvent::Focus
                }
                exports::tairitsu::hooks::state_machine::InteractionEvent::Blur => {
                    state_machine::InteractionEvent::Blur
                }
                exports::tairitsu::hooks::state_machine::InteractionEvent::Disable => {
                    state_machine::InteractionEvent::Disable
                }
                exports::tairitsu::hooks::state_machine::InteractionEvent::Enable => {
                    state_machine::InteractionEvent::Enable
                }
            };
            let mut machine = state_machine::ButtonStateMachine::new();
            machine.set_state(native_state);
            let result = machine.transition(native_event)?;
            Some(match result {
                state_machine::InteractionState::Idle => {
                    exports::tairitsu::hooks::state_machine::InteractionState::Idle
                }
                state_machine::InteractionState::Hover => {
                    exports::tairitsu::hooks::state_machine::InteractionState::Hover
                }
                state_machine::InteractionState::Active => {
                    exports::tairitsu::hooks::state_machine::InteractionState::Active
                }
                state_machine::InteractionState::Focused => {
                    exports::tairitsu::hooks::state_machine::InteractionState::Focused
                }
                state_machine::InteractionState::Disabled => {
                    exports::tairitsu::hooks::state_machine::InteractionState::Disabled
                }
            })
        }

        fn is_interactive(
            state: exports::tairitsu::hooks::state_machine::InteractionState,
        ) -> bool {
            let native = match state {
                exports::tairitsu::hooks::state_machine::InteractionState::Idle => {
                    state_machine::InteractionState::Idle
                }
                exports::tairitsu::hooks::state_machine::InteractionState::Hover => {
                    state_machine::InteractionState::Hover
                }
                exports::tairitsu::hooks::state_machine::InteractionState::Active => {
                    state_machine::InteractionState::Active
                }
                exports::tairitsu::hooks::state_machine::InteractionState::Focused => {
                    state_machine::InteractionState::Focused
                }
                exports::tairitsu::hooks::state_machine::InteractionState::Disabled => {
                    state_machine::InteractionState::Disabled
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
                    animation::EasingFunction::Linear
                }
                exports::tairitsu::hooks::animation_types::EasingFunction::Ease => {
                    animation::EasingFunction::Ease
                }
                exports::tairitsu::hooks::animation_types::EasingFunction::EaseIn => {
                    animation::EasingFunction::EaseIn
                }
                exports::tairitsu::hooks::animation_types::EasingFunction::EaseOut => {
                    animation::EasingFunction::EaseOut
                }
                exports::tairitsu::hooks::animation_types::EasingFunction::EaseInOut => {
                    animation::EasingFunction::EaseInOut
                }
                exports::tairitsu::hooks::animation_types::EasingFunction::CubicBezier(v) => {
                    let (a, b, c, d) = v;
                    animation::EasingFunction::CubicBezier(a, b, c, d)
                }
            };
            native.evaluate(t)
        }

        fn lerp_f32(from: f32, to: f32, t: f32) -> f32 {
            animation::lerp_f32(from, to, t)
        }
    }

    impl exports::tairitsu::hooks::animation_builder::Guest for HooksExports {
        fn default_animation_config() -> exports::tairitsu::hooks::animation_builder::AnimationConfig
        {
            let cfg = animation::AnimationConfig::default();
            let easing = convert_easing(&cfg.easing);
            exports::tairitsu::hooks::animation_builder::AnimationConfig {
                duration_ms: cfg.duration.as_millis() as u64,
                delay_ms: cfg.delay.as_millis() as u64,
                iterations: cfg.iterations,
                direction: match cfg.direction {
                    animation::AnimationDirection::Normal => exports::tairitsu::hooks::animation_types::AnimationDirection::Normal,
                    animation::AnimationDirection::Reverse => exports::tairitsu::hooks::animation_types::AnimationDirection::Reverse,
                    animation::AnimationDirection::Alternate => exports::tairitsu::hooks::animation_types::AnimationDirection::Alternate,
                    animation::AnimationDirection::AlternateReverse => exports::tairitsu::hooks::animation_types::AnimationDirection::AlternateReverse,
                },
                easing,
            }
        }
    }

    impl exports::tairitsu::hooks::async_resource::Guest for HooksExports {
        fn resource_status_query(
            resource_id: u64,
        ) -> Option<exports::tairitsu::hooks::async_resource::ResourceStatus> {
            suspense::resource_state(suspense::ResourceId::from(resource_id as usize)).map(|s| {
                match s {
                    suspense::ResourceStatus::Loading => {
                        exports::tairitsu::hooks::async_resource::ResourceStatus::Loading
                    }
                    suspense::ResourceStatus::Ready => {
                        exports::tairitsu::hooks::async_resource::ResourceStatus::Ready
                    }
                    suspense::ResourceStatus::Error => {
                        exports::tairitsu::hooks::async_resource::ResourceStatus::Error
                    }
                }
            })
        }
    }

    fn convert_easing(
        e: &animation::EasingFunction,
    ) -> exports::tairitsu::hooks::animation_types::EasingFunction {
        match e {
            animation::EasingFunction::Linear => {
                exports::tairitsu::hooks::animation_types::EasingFunction::Linear
            }
            animation::EasingFunction::Ease => {
                exports::tairitsu::hooks::animation_types::EasingFunction::Ease
            }
            animation::EasingFunction::EaseIn => {
                exports::tairitsu::hooks::animation_types::EasingFunction::EaseIn
            }
            animation::EasingFunction::EaseOut => {
                exports::tairitsu::hooks::animation_types::EasingFunction::EaseOut
            }
            animation::EasingFunction::EaseInOut => {
                exports::tairitsu::hooks::animation_types::EasingFunction::EaseInOut
            }
            animation::EasingFunction::CubicBezier(a, b, c, d) => {
                exports::tairitsu::hooks::animation_types::EasingFunction::CubicBezier((
                    *a, *b, *c, *d,
                ))
            }
        }
    }

    export!(HooksExports);
}
