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
                unsafe extern "C" {
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
                unsafe extern "C" {
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

    pub fn button(mut self, button: i16) -> Self {
        self.button = button;
        self
    }

    pub fn buttons(mut self, buttons: u16) -> Self {
        self.buttons = buttons;
        self
    }

    pub fn ctrl_key(mut self, ctrl_key: bool) -> Self {
        self.ctrl_key = ctrl_key;
        self
    }

    pub fn shift_key(mut self, shift_key: bool) -> Self {
        self.shift_key = shift_key;
        self
    }

    pub fn alt_key(mut self, alt_key: bool) -> Self {
        self.alt_key = alt_key;
        self
    }

    pub fn meta_key(mut self, meta_key: bool) -> Self {
        self.meta_key = meta_key;
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

    pub fn key_code(mut self, key_code: u32) -> Self {
        self.key_code = key_code;
        self
    }

    pub fn ctrl_key(mut self, ctrl_key: bool) -> Self {
        self.ctrl_key = ctrl_key;
        self
    }

    pub fn shift_key(mut self, shift_key: bool) -> Self {
        self.shift_key = shift_key;
        self
    }

    pub fn alt_key(mut self, alt_key: bool) -> Self {
        self.alt_key = alt_key;
        self
    }

    pub fn meta_key(mut self, meta_key: bool) -> Self {
        self.meta_key = meta_key;
        self
    }

    pub fn repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
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
    pub fn get_key(&self) -> Key {
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

// ============================================================================
// Wheel Event - Phase 4
// ============================================================================

/// Wheel event for mouse wheel interactions (zooming, scrolling, etc.).
///
/// This event provides information about wheel/scroll interactions, including:
/// - Scroll deltas (pixel, line, page)
/// - Modifier keys
/// - Event target for prevent_default/stop_propagation
///
/// # Example
///
/// ```ignore
/// div {
///     onwheel: |e: WheelEvent| {
///         if e.delta_y > 0 {
///             // Scrolling down
///         }
///         e.prevent_default(); // Prevent default scrolling
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct WheelEvent {
    /// The target element handle that received this event.
    pub target: Option<u64>,
    /// Horizontal scroll amount (pixels)
    pub delta_x: f64,
    /// Vertical scroll amount (pixels)
    pub delta_y: f64,
    /// Z-axis scroll amount (pixels)
    pub delta_z: f64,
    /// Unit of delta values: 0=pixel, 1=line, 2=page, 3=viewport
    pub delta_mode: u32,
    /// Mouse position - client X coordinate
    pub client_x: i32,
    /// Mouse position - client Y coordinate
    pub client_y: i32,
    /// Mouse position - screen X coordinate
    pub screen_x: i32,
    /// Mouse position - screen Y coordinate
    pub screen_y: i32,
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

impl EventData for WheelEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl WheelEvent {
    pub fn new() -> Self {
        Self {
            target: None,
            delta_x: 0.0,
            delta_y: 0.0,
            delta_z: 0.0,
            delta_mode: 0,
            client_x: 0,
            client_y: 0,
            screen_x: 0,
            screen_y: 0,
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

    pub fn delta_x(mut self, x: f64) -> Self {
        self.delta_x = x;
        self
    }

    pub fn delta_y(mut self, y: f64) -> Self {
        self.delta_y = y;
        self
    }

    pub fn delta_z(mut self, z: f64) -> Self {
        self.delta_z = z;
        self
    }

    pub fn delta_mode(mut self, mode: u32) -> Self {
        self.delta_mode = mode;
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

impl Default for WheelEvent {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Touch Event - Phase 4
// ============================================================================

/// A single touch point.
#[derive(Debug, Clone)]
pub struct TouchPoint {
    /// Unique identifier for this touch point
    pub identifier: i32,
    /// X coordinate relative to the viewport
    pub client_x: i32,
    /// Y coordinate relative to the viewport
    pub client_y: i32,
    /// X coordinate relative to the screen
    pub screen_x: i32,
    /// Y coordinate relative to the screen
    pub screen_y: i32,
    /// X coordinate relative to the target element
    pub page_x: i32,
    /// Y coordinate relative to the target element
    pub page_y: i32,
    /// The target element handle
    pub target: Option<u64>,
    /// The force of the touch (0.0 to 1.0)
    pub force: f32,
    /// Radius of the touch area
    pub radius_x: f32,
    /// Radius of the touch area
    pub radius_y: f32,
    /// Rotation angle of the touch ellipse
    pub rotation_angle: f32,
}

impl Default for TouchPoint {
    fn default() -> Self {
        Self {
            identifier: 0,
            client_x: 0,
            client_y: 0,
            screen_x: 0,
            screen_y: 0,
            page_x: 0,
            page_y: 0,
            target: None,
            force: 0.0,
            radius_x: 0.0,
            radius_y: 0.0,
            rotation_angle: 0.0,
        }
    }
}

/// Touch event for multi-touch interactions on mobile devices.
///
/// This event provides information about touch interactions, including:
/// - All active touch points
/// - Changed touch points (since last event)
/// - Touch target information
///
/// # Example
///
/// ```ignore
/// div {
///     ontouchstart: |e: TouchEvent| {
///         for touch in &e.touches {
///             println!("Touch at: ({}, {})", touch.client_x, touch.client_y);
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TouchEvent {
    /// The target element handle that received this event.
    pub target: Option<u64>,
    /// All active touch points
    pub touches: Vec<TouchPoint>,
    /// Touch points that have changed since the last event
    pub changed_touches: Vec<TouchPoint>,
    /// Touch points that were in the target element
    pub target_touches: Vec<TouchPoint>,
    /// Timestamp of the event
    pub timestamp: f64,
    event_handle: EventWitHandle,
}

impl EventData for TouchEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TouchEvent {
    pub fn new() -> Self {
        Self {
            target: None,
            touches: Vec::new(),
            changed_touches: Vec::new(),
            target_touches: Vec::new(),
            timestamp: 0.0,
            event_handle: EventWitHandle::placeholder(),
        }
    }

    pub fn target(mut self, target: u64) -> Self {
        self.target = Some(target);
        self
    }

    pub fn touches(mut self, touches: Vec<TouchPoint>) -> Self {
        self.touches = touches;
        self
    }

    pub fn changed_touches(mut self, touches: Vec<TouchPoint>) -> Self {
        self.changed_touches = touches;
        self
    }

    pub fn target_touches(mut self, touches: Vec<TouchPoint>) -> Self {
        self.target_touches = touches;
        self
    }

    pub fn timestamp(mut self, timestamp: f64) -> Self {
        self.timestamp = timestamp;
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

impl Default for TouchEvent {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Pointer Event - Phase 4
// ============================================================================

/// Pointer type for unified pointer events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerType {
    /// Mouse pointer
    Mouse,
    /// Pen/stylus pointer
    Pen,
    /// Touch pointer
    Touch,
}

impl PointerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PointerType::Mouse => "mouse",
            PointerType::Pen => "pen",
            PointerType::Touch => "touch",
        }
    }
}

impl std::str::FromStr for PointerType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mouse" => Ok(PointerType::Mouse),
            "pen" => Ok(PointerType::Pen),
            "touch" => Ok(PointerType::Touch),
            _ => Ok(PointerType::Mouse),
        }
    }
}

/// Pointer event for unified pointer interactions (mouse, touch, pen).
///
/// This event provides a unified abstraction for all pointer input devices,
/// making it easier to handle interactions across different input methods.
///
/// # Example
///
/// ```ignore
/// div {
///     onpointerdown: |e: PointerEvent| {
///         match e.pointer_type {
///             PointerType::Touch => println!("Touch at: ({}, {})", e.client_x, e.client_y),
///             PointerType::Mouse => println!("Mouse click at: ({}, {})", e.client_x, e.client_y),
///             PointerType::Pen => println!("Pen at: ({}, {})", e.client_x, e.client_y),
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct PointerEvent {
    /// The target element handle that received this event.
    pub target: Option<u64>,
    /// Unique pointer ID for this active pointer
    pub pointer_id: i32,
    /// Pointer type (mouse, pen, touch)
    pub pointer_type: PointerType,
    /// Whether the pointer is the primary pointer
    pub is_primary: bool,
    /// Mouse position - client X coordinate
    pub client_x: i32,
    /// Mouse position - client Y coordinate
    pub client_y: i32,
    /// Mouse position - screen X coordinate
    pub screen_x: i32,
    /// Mouse position - screen Y coordinate
    pub screen_y: i32,
    /// Mouse position - offset X coordinate
    pub offset_x: i32,
    /// Mouse position - offset Y coordinate
    pub offset_y: i32,
    /// Mouse position - page X coordinate
    pub page_x: i32,
    /// Mouse position - page Y coordinate
    pub page_y: i32,
    /// Movement since last event - X
    pub movement_x: i32,
    /// Movement since last event - Y
    pub movement_y: i32,
    /// Width of the pointer contact area
    pub width: f32,
    /// Height of the pointer contact area
    pub height: f32,
    /// Pressure of the pointer (0.0 to 1.0)
    pub pressure: f32,
    /// Tangential pressure (0.0 to 1.0)
    pub tangential_pressure: f32,
    /// Tilt X angle (-90 to 90)
    pub tilt_x: i32,
    /// Tilt Y angle (-90 to 90)
    pub tilt_y: i32,
    /// Twist angle (0 to 359)
    pub twist: i32,
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

impl EventData for PointerEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PointerEvent {
    pub fn new() -> Self {
        Self {
            target: None,
            pointer_id: 0,
            pointer_type: PointerType::Mouse,
            is_primary: false,
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
            width: 0.0,
            height: 0.0,
            pressure: 0.0,
            tangential_pressure: 0.0,
            tilt_x: 0,
            tilt_y: 0,
            twist: 0,
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

    pub fn pointer_id(mut self, id: i32) -> Self {
        self.pointer_id = id;
        self
    }

    pub fn pointer_type(mut self, pointer_type: PointerType) -> Self {
        self.pointer_type = pointer_type;
        self
    }

    pub fn is_primary(mut self, is_primary: bool) -> Self {
        self.is_primary = is_primary;
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

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn pressure(mut self, pressure: f32) -> Self {
        self.pressure = pressure;
        self
    }

    pub fn tangential_pressure(mut self, pressure: f32) -> Self {
        self.tangential_pressure = pressure;
        self
    }

    pub fn tilt_x(mut self, tilt: i32) -> Self {
        self.tilt_x = tilt;
        self
    }

    pub fn tilt_y(mut self, tilt: i32) -> Self {
        self.tilt_y = tilt;
        self
    }

    pub fn twist(mut self, twist: i32) -> Self {
        self.twist = twist;
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

impl Default for PointerEvent {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Transition Event - Phase 4
// ============================================================================

/// Transition event for CSS transition completion callbacks.
///
/// This event fires when a CSS transition completes, allowing you to
/// synchronize animations with application logic.
///
/// # Example
///
/// ```ignore
/// div {
///     style: "transition: opacity 0.3s;",
///     ontransitionend: |e: TransitionEvent| {
///         if e.property_name == "opacity" {
///             println!("Opacity transition complete!");
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TransitionEvent {
    /// The target element handle that received this event.
    pub target: Option<u64>,
    /// The name of the CSS property that completed transitioning
    pub property_name: String,
    /// The number of seconds the transition took
    pub elapsed_time: f32,
    /// The name of the pseudo-element that was transitioning
    pub pseudo_element: String,
    event_handle: EventWitHandle,
}

impl EventData for TransitionEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TransitionEvent {
    pub fn new() -> Self {
        Self {
            target: None,
            property_name: String::new(),
            elapsed_time: 0.0,
            pseudo_element: String::new(),
            event_handle: EventWitHandle::placeholder(),
        }
    }

    pub fn target(mut self, target: u64) -> Self {
        self.target = Some(target);
        self
    }

    pub fn property_name(mut self, name: impl Into<String>) -> Self {
        self.property_name = name.into();
        self
    }

    pub fn elapsed_time(mut self, time: f32) -> Self {
        self.elapsed_time = time;
        self
    }

    pub fn pseudo_element(mut self, pseudo: impl Into<String>) -> Self {
        self.pseudo_element = pseudo.into();
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

impl Default for TransitionEvent {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Animation Event - Phase 4
// ============================================================================

/// Animation event for CSS animation lifecycle callbacks.
///
/// This event fires at various points during a CSS animation's lifecycle,
/// allowing you to synchronize animations with application logic.
///
/// # Example
///
/// ```ignore
/// div {
///     style: "animation: fadeIn 0.5s;",
///     onanimationend: |e: AnimationEvent| {
///         println!("Animation '{}' complete!", e.animation_name);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AnimationEvent {
    /// The target element handle that received this event.
    pub target: Option<u64>,
    /// The name of the animation
    pub animation_name: String,
    /// The name of the pseudo-element that was animating
    pub pseudo_element: String,
    /// The elapsed time since the animation started
    pub elapsed_time: f32,
    /// The current iteration of the animation
    pub iteration: f32,
    event_handle: EventWitHandle,
}

impl EventData for AnimationEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl AnimationEvent {
    pub fn new() -> Self {
        Self {
            target: None,
            animation_name: String::new(),
            pseudo_element: String::new(),
            elapsed_time: 0.0,
            iteration: 0.0,
            event_handle: EventWitHandle::placeholder(),
        }
    }

    pub fn target(mut self, target: u64) -> Self {
        self.target = Some(target);
        self
    }

    pub fn animation_name(mut self, name: impl Into<String>) -> Self {
        self.animation_name = name.into();
        self
    }

    pub fn pseudo_element(mut self, pseudo: impl Into<String>) -> Self {
        self.pseudo_element = pseudo.into();
        self
    }

    pub fn elapsed_time(mut self, time: f32) -> Self {
        self.elapsed_time = time;
        self
    }

    pub fn iteration(mut self, iteration: f32) -> Self {
        self.iteration = iteration;
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

impl Default for AnimationEvent {
    fn default() -> Self {
        Self::new()
    }
}
