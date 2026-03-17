pub mod animation;
pub mod callback;
pub mod context;
pub mod effect;
pub mod memo;
pub mod ref_;
pub mod signal;
pub mod state;

pub use animation::{
    use_animation, use_simple_animation, AnimationConfig, AnimationDirection, AnimationState,
    EasingFunction,
};
pub use callback::{use_callback, use_return_callback, use_void_callback, Callback};
pub use context::{consume_context, provide_context, use_context, Context};
// Dioxus compatibility alias
pub use provide_context as use_context_provider;
pub use effect::use_effect;
pub use memo::{use_memo, use_memo_with, Memo};
pub use ref_::{use_ref, UseRef};
pub use signal::use_signal;
pub use state::use_state;

// Re-export Event type from vdom for convenience
pub use tairitsu_vdom::Event;
