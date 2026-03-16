pub mod diff;
pub mod events;
pub mod patch;
pub mod platform;
pub mod portal;
pub mod reactive;
pub mod vnode;

pub use events::{ChangeEvent, EventData, EventWitHandle, FocusEvent, InputEvent, KeyboardEvent, MouseEvent};
pub use patch::Patch;
pub use platform::{ElementHandle, EventHandle, Platform};
pub use portal::{FixedPosition, Portal, PortalManager, PortalMaskMode, PortalPosition};
pub use reactive::{batch, create_effect, EffectHandle, Signal};
pub use vnode::{Classes, Style, VElement, VNode, VText};
