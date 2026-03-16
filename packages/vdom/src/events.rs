use std::any::Any;

pub trait EventData: Any {
    fn as_any(&self) -> &dyn Any;
}

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
