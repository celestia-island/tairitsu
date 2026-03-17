pub mod callback;
pub mod diff;
pub mod events;
pub mod patch;
pub mod platform;
pub mod portal;
pub mod reactive;
pub mod vnode;

pub use callback::{Callback, EventHandler};
pub use events::{
    ChangeEvent, DataTransfer, DragEvent, Event, EventData, EventWitHandle, FileData, FocusEvent,
    FormData, FormEvent, GenericEvent, InputEvent, Key, KeyboardEvent, MouseData, MouseEvent,
};
pub use patch::Patch;
pub use platform::{ElementHandle, EventHandle, Platform};
pub use portal::{FixedPosition, Portal, PortalManager, PortalMaskMode, PortalPosition};
pub use reactive::{batch, create_effect, EffectHandle, Signal};
pub use vnode::{empty_vnode, Classes, IntoAttrValue, Style, VElement, VNode, VText};
