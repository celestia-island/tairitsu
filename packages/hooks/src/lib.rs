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
    Resource, ResourceState, Suspense, SuspenseBoundary, use_resource, use_suspense,
};

// Re-export Event types from vdom for convenience
pub use tairitsu_vdom::{Event, GenericEvent};
