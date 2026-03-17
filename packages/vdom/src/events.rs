use std::any::Any;

pub trait EventData: Any {
    fn as_any(&self) -> &dyn Any;
}

/// Type alias for boxed event data, commonly used in event handlers.
/// This provides a simpler type name for `Box<dyn EventData>`.
pub type Event = Box<dyn EventData>;

/// Handle to an event that can be used for prevent_default/stop_propagation.
/// The handle is stored and passed to browser-glue via WIT bindings.
#[derive(Debug, Clone)]
pub struct EventWitHandle {
    handle: Option<u64>,
}

impl EventWitHandle {
    /// Create a new event handle from a WIT handle.
    pub fn from_wit(handle: u64) -> Self {
        Self { handle: Some(handle) }
    }

    /// Create a placeholder handle (for non-WIT builds).
    pub fn placeholder() -> Self {
        Self { handle: None }
    }

    /// Call prevent_default on this event.
    pub fn prevent_default(&self) {
        if let Some(handle) = self.handle {
            // This will be linked via WIT bindings in browser-glue
            unsafe {
                // The actual implementation is provided by browser-glue
                // These are weak symbols that will be resolved at link time
                extern "C" {
                    fn tairitsu_prevent_default(event_handle: u64);
                }
                tairitsu_prevent_default(handle);
            }
        }
    }

    /// Call stop_propagation on this event.
    pub fn stop_propagation(&self) {
        if let Some(handle) = self.handle {
            unsafe {
                extern "C" {
                    fn tairitsu_stop_propagation(event_handle: u64);
                }
                tairitsu_stop_propagation(handle);
            }
        }
    }
}

impl Default for EventWitHandle {
    fn default() -> Self {
        Self::placeholder()
    }
}

#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub client_x: i32,
    pub client_y: i32,
    pub screen_x: i32,
    pub screen_y: i32,
    pub button: i16,
    pub buttons: u16,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
    event_handle: EventWitHandle,
}

impl EventData for MouseEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl MouseEvent {
    pub fn new() -> Self {
        Self {
            client_x: 0,
            client_y: 0,
            screen_x: 0,
            screen_y: 0,
            button: 0,
            buttons: 0,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
            event_handle: EventWitHandle::placeholder(),
        }
    }

    pub fn client_x(mut self, x: i32) -> Self {
        self.client_x = x;
        self
    }

    pub fn client_y(mut self, y: i32) -> Self {
        self.client_y = y;
        self
    }

    pub fn event_handle(mut self, handle: EventWitHandle) -> Self {
        self.event_handle = handle;
        self
    }

    pub fn prevent_default(&self) {
        self.event_handle.prevent_default();
    }

    pub fn stop_propagation(&self) {
        self.event_handle.stop_propagation();
    }
}

impl Default for MouseEvent {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    pub key: String,
    pub code: String,
    pub key_code: u32,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
    pub repeat: bool,
    event_handle: EventWitHandle,
}

impl EventData for KeyboardEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl KeyboardEvent {
    pub fn new() -> Self {
        Self {
            key: String::new(),
            code: String::new(),
            key_code: 0,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
            repeat: false,
            event_handle: EventWitHandle::placeholder(),
        }
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = key.into();
        self
    }

    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.code = code.into();
        self
    }

    pub fn event_handle(mut self, handle: EventWitHandle) -> Self {
        self.event_handle = handle;
        self
    }

    pub fn prevent_default(&self) {
        self.event_handle.prevent_default();
    }

    pub fn stop_propagation(&self) {
        self.event_handle.stop_propagation();
    }
}

impl Default for KeyboardEvent {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FocusEvent {
    pub related_target: Option<String>,
    event_handle: EventWitHandle,
}

impl EventData for FocusEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl FocusEvent {
    pub fn new() -> Self {
        Self {
            related_target: None,
            event_handle: EventWitHandle::placeholder(),
        }
    }

    pub fn event_handle(mut self, handle: EventWitHandle) -> Self {
        self.event_handle = handle;
        self
    }

    pub fn prevent_default(&self) {
        self.event_handle.prevent_default();
    }

    pub fn stop_propagation(&self) {
        self.event_handle.stop_propagation();
    }
}

impl Default for FocusEvent {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct InputEvent {
    pub data: String,
    pub input_type: String,
    event_handle: EventWitHandle,
}

impl EventData for InputEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl InputEvent {
    pub fn new() -> Self {
        Self {
            data: String::new(),
            input_type: String::new(),
            event_handle: EventWitHandle::placeholder(),
        }
    }

    pub fn data(mut self, data: impl Into<String>) -> Self {
        self.data = data.into();
        self
    }

    pub fn event_handle(mut self, handle: EventWitHandle) -> Self {
        self.event_handle = handle;
        self
    }

    pub fn prevent_default(&self) {
        self.event_handle.prevent_default();
    }

    pub fn stop_propagation(&self) {
        self.event_handle.stop_propagation();
    }
}

impl Default for InputEvent {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ChangeEvent {
    pub value: String,
}

impl EventData for ChangeEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ChangeEvent {
    pub fn new() -> Self {
        Self {
            value: String::new(),
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }
}

impl Default for ChangeEvent {
    fn default() -> Self {
        Self::new()
    }
}

/// A generic event type for events that don't have a specific typed representation.
/// This is used for custom events or events where we only need the event type name.
#[derive(Debug, Clone)]
pub struct GenericEvent {
    /// The event type name (e.g., "submit", "scroll", "wheel")
    pub event_type: String,
    event_handle: EventWitHandle,
}

impl EventData for GenericEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl GenericEvent {
    pub fn new() -> Self {
        Self {
            event_type: String::new(),
            event_handle: EventWitHandle::placeholder(),
        }
    }

    pub fn event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = event_type.into();
        self
    }

    pub fn event_handle(mut self, handle: EventWitHandle) -> Self {
        self.event_handle = handle;
        self
    }

    pub fn prevent_default(&self) {
        self.event_handle.prevent_default();
    }

    pub fn stop_propagation(&self) {
        self.event_handle.stop_propagation();
    }
}

impl Default for GenericEvent {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents keyboard keys for keyboard event handling.
/// This enum provides a Dioxus-compatible API for matching keyboard keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    /// Arrow keys
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    /// Special keys
    Enter,
    Escape,
    Tab,
    Backspace,
    Delete,
    Space,
    /// Modifier keys
    Shift,
    Control,
    Alt,
    Meta,
    /// A character key
    Character(String),
    /// Any other key by name
    Other(String),
}

impl Key {
    /// Convert a key string to a Key enum.
    /// This is a convenience method that delegates to [`FromStr`].
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        s.parse().unwrap_or_else(|_| Key::Other(s.to_string()))
    }

    /// Check if this key matches the given string
    pub fn is(&self, s: &str) -> bool {
        match (self, s) {
            (Key::ArrowUp, "ArrowUp") => true,
            (Key::ArrowDown, "ArrowDown") => true,
            (Key::ArrowLeft, "ArrowLeft") => true,
            (Key::ArrowRight, "ArrowRight") => true,
            (Key::Enter, "Enter") => true,
            (Key::Escape, "Escape") => true,
            (Key::Tab, "Tab") => true,
            (Key::Backspace, "Backspace") => true,
            (Key::Delete, "Delete") => true,
            (Key::Space, " " | "Space") => true,
            (Key::Shift, "Shift") => true,
            (Key::Control, "Control") => true,
            (Key::Alt, "Alt") => true,
            (Key::Meta, "Meta") => true,
            (Key::Character(c), s) if s.len() == 1 => c == s,
            (Key::Other(name), s) => name == s,
            _ => false,
        }
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::ArrowUp => write!(f, "ArrowUp"),
            Key::ArrowDown => write!(f, "ArrowDown"),
            Key::ArrowLeft => write!(f, "ArrowLeft"),
            Key::ArrowRight => write!(f, "ArrowRight"),
            Key::Enter => write!(f, "Enter"),
            Key::Escape => write!(f, "Escape"),
            Key::Tab => write!(f, "Tab"),
            Key::Backspace => write!(f, "Backspace"),
            Key::Delete => write!(f, "Delete"),
            Key::Space => write!(f, " "),
            Key::Shift => write!(f, "Shift"),
            Key::Control => write!(f, "Control"),
            Key::Alt => write!(f, "Alt"),
            Key::Meta => write!(f, "Meta"),
            Key::Character(c) => write!(f, "{}", c),
            Key::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for Key {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ArrowUp" => Ok(Key::ArrowUp),
            "ArrowDown" => Ok(Key::ArrowDown),
            "ArrowLeft" => Ok(Key::ArrowLeft),
            "ArrowRight" => Ok(Key::ArrowRight),
            "Enter" => Ok(Key::Enter),
            "Escape" => Ok(Key::Escape),
            "Tab" => Ok(Key::Tab),
            "Backspace" => Ok(Key::Backspace),
            "Delete" => Ok(Key::Delete),
            " " | "Space" => Ok(Key::Space),
            "Shift" => Ok(Key::Shift),
            "Control" => Ok(Key::Control),
            "Alt" => Ok(Key::Alt),
            "Meta" => Ok(Key::Meta),
            s if s.len() == 1 => Ok(Key::Character(s.to_string())),
            other => Ok(Key::Other(other.to_string())),
        }
    }
}

impl KeyboardEvent {
    /// Get the Key enum for this keyboard event
    pub fn key_code(&self) -> Key {
        Key::from_str(&self.key)
    }
}
