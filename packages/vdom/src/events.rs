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
        Self {
            handle: Some(handle),
        }
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
    /// The target element handle that received this event.
    /// This allows event handlers to directly manipulate the element
    /// via Platform methods like set_style and get_bounding_client_rect.
    pub target: Option<u64>,
    pub client_x: i32,
    pub client_y: i32,
    pub screen_x: i32,
    pub screen_y: i32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub page_x: i32,
    pub page_y: i32,
    pub movement_x: i32,
    pub movement_y: i32,
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
            target: None,
            client_x: 0,
            client_y: 0,
            screen_x: 0,
            screen_y: 0,
            offset_x: 0,
            offset_y: 0,
            page_x: 0,
            page_y: 0,
            movement_x: 0,
            movement_y: 0,
            button: 0,
            buttons: 0,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
            event_handle: EventWitHandle::placeholder(),
        }
    }

    pub fn target(mut self, target: u64) -> Self {
        self.target = Some(target);
        self
    }

    pub fn client_x(mut self, x: i32) -> Self {
        self.client_x = x;
        self
    }

    pub fn client_y(mut self, y: i32) -> Self {
        self.client_y = y;
        self
    }

    pub fn screen_x(mut self, x: i32) -> Self {
        self.screen_x = x;
        self
    }

    pub fn screen_y(mut self, y: i32) -> Self {
        self.screen_y = y;
        self
    }

    pub fn offset_x(mut self, x: i32) -> Self {
        self.offset_x = x;
        self
    }

    pub fn offset_y(mut self, y: i32) -> Self {
        self.offset_y = y;
        self
    }

    pub fn page_x(mut self, x: i32) -> Self {
        self.page_x = x;
        self
    }

    pub fn page_y(mut self, y: i32) -> Self {
        self.page_y = y;
        self
    }

    pub fn movement_x(mut self, x: i32) -> Self {
        self.movement_x = x;
        self
    }

    pub fn movement_y(mut self, y: i32) -> Self {
        self.movement_y = y;
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

/// Form data event for form input events.
/// This is a Dioxus-compatible type for handling form submissions and input changes.
#[derive(Debug, Clone)]
pub struct FormData {
    /// The current value of the form element
    pub value: String,
    /// The values of all form elements (for form submit events)
    pub values: Vec<(String, String)>,
    /// Files from file input elements
    pub files: Vec<FileData>,
    event_handle: EventWitHandle,
}

/// File data from file input elements
#[derive(Debug, Clone)]
pub struct FileData {
    pub name: String,
    pub size: u64,
    pub mime_type: String,
}

impl FileData {
    pub fn new(name: impl Into<String>, size: u64, mime_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size,
            mime_type: mime_type.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

impl EventData for FormData {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl FormData {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            values: Vec::new(),
            files: Vec::new(),
            event_handle: EventWitHandle::placeholder(),
        }
    }

    /// Get the current value of the form element
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Get a value from the form data by key
    pub fn get_value(&self, key: &str) -> Option<&str> {
        self.values
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn value_mut(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn values(mut self, values: Vec<(String, String)>) -> Self {
        self.values = values;
        self
    }

    pub fn add_value(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.values.push((key.into(), value.into()));
        self
    }

    pub fn files(mut self, files: Vec<FileData>) -> Self {
        self.files = files;
        self
    }

    pub fn add_file(mut self, file: FileData) -> Self {
        self.files.push(file);
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

impl Default for FormData {
    fn default() -> Self {
        Self::new()
    }
}

/// Alias for FormData, providing Dioxus API compatibility.
/// FormEvent is used for form-related events like onsubmit, oninput, etc.
pub type FormEvent = FormData;

/// Data transfer object for drag and drop events.
/// Contains information about the data being dragged.
#[derive(Debug, Clone, Default)]
pub struct DataTransfer {
    /// The data being transferred, stored as MIME type -> data pairs
    pub data: Vec<(String, String)>,
    /// The drop effect (none, copy, move, link)
    pub drop_effect: String,
    /// The allowed effects
    pub effect_allowed: String,
    /// List of file names being dragged
    pub files: Vec<String>,
}

impl DataTransfer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get data by MIME type
    pub fn get_data(&self, mime_type: &str) -> Option<&str> {
        self.data
            .iter()
            .find(|(t, _)| t == mime_type)
            .map(|(_, v)| v.as_str())
    }

    /// Set data for a MIME type
    pub fn set_data(&mut self, mime_type: impl Into<String>, data: impl Into<String>) {
        self.data.push((mime_type.into(), data.into()));
    }

    pub fn drop_effect(mut self, effect: impl Into<String>) -> Self {
        self.drop_effect = effect.into();
        self
    }

    pub fn effect_allowed(mut self, allowed: impl Into<String>) -> Self {
        self.effect_allowed = allowed.into();
        self
    }

    pub fn add_file(mut self, file: impl Into<String>) -> Self {
        self.files.push(file.into());
        self
    }

    pub fn add_data(mut self, mime_type: impl Into<String>, data: impl Into<String>) -> Self {
        self.data.push((mime_type.into(), data.into()));
        self
    }
}

/// Drag event for drag and drop interactions.
/// This provides Dioxus-compatible API for handling drag events.
#[derive(Debug, Clone)]
pub struct DragEvent {
    /// The data being transferred
    pub data_transfer: Option<DataTransfer>,
    /// Mouse position - client X coordinate
    pub client_x: i32,
    /// Mouse position - client Y coordinate
    pub client_y: i32,
    /// Mouse position - screen X coordinate
    pub screen_x: i32,
    /// Mouse position - screen Y coordinate
    pub screen_y: i32,
    /// Which mouse button was pressed (if any)
    pub button: i16,
    /// Which mouse buttons are pressed
    pub buttons: u16,
    /// Whether ctrl key was pressed
    pub ctrl_key: bool,
    /// Whether shift key was pressed
    pub shift_key: bool,
    /// Whether alt key was pressed
    pub alt_key: bool,
    /// Whether meta key was pressed
    pub meta_key: bool,
    event_handle: EventWitHandle,
}

impl EventData for DragEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl DragEvent {
    pub fn new() -> Self {
        Self {
            data_transfer: None,
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

    /// Get the data transfer object
    pub fn data_transfer(&self) -> Option<&DataTransfer> {
        self.data_transfer.as_ref()
    }

    /// Get mutable access to data transfer, creating it if needed
    pub fn data_transfer_mut(&mut self) -> &mut DataTransfer {
        if self.data_transfer.is_none() {
            self.data_transfer = Some(DataTransfer::new());
        }
        self.data_transfer.as_mut().unwrap()
    }

    /// Builder method to set the data transfer object
    pub fn with_data_transfer(mut self, data_transfer: DataTransfer) -> Self {
        self.data_transfer = Some(data_transfer);
        self
    }

    pub fn client_x(mut self, x: i32) -> Self {
        self.client_x = x;
        self
    }

    pub fn client_y(mut self, y: i32) -> Self {
        self.client_y = y;
        self
    }

    pub fn screen_x(mut self, x: i32) -> Self {
        self.screen_x = x;
        self
    }

    pub fn screen_y(mut self, y: i32) -> Self {
        self.screen_y = y;
        self
    }

    pub fn button(mut self, button: i16) -> Self {
        self.button = button;
        self
    }

    pub fn buttons(mut self, buttons: u16) -> Self {
        self.buttons = buttons;
        self
    }

    pub fn ctrl_key(mut self, ctrl: bool) -> Self {
        self.ctrl_key = ctrl;
        self
    }

    pub fn shift_key(mut self, shift: bool) -> Self {
        self.shift_key = shift;
        self
    }

    pub fn alt_key(mut self, alt: bool) -> Self {
        self.alt_key = alt;
        self
    }

    pub fn meta_key(mut self, meta: bool) -> Self {
        self.meta_key = meta;
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

impl Default for DragEvent {
    fn default() -> Self {
        Self::new()
    }
}

/// Alias for MouseEvent, providing Dioxus API compatibility.
/// MouseData is the preferred type name in Dioxus for mouse event handlers.
pub type MouseData = MouseEvent;
