pub mod diff;
pub mod patch;
pub mod platform;
pub mod reactive;
pub mod vnode;

pub use patch::Patch;
pub use platform::{ElementHandle, EventHandle, Platform};
pub use reactive::{batch, create_effect, EffectHandle, Signal};
pub use vnode::{Classes, Style, VElement, VNode, VText};
