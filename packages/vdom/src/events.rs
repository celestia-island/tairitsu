use std::any::Any;

pub trait EventData: Any {
    fn as_any(&self) -> &dyn Any;
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

    pub fn prevent_default(&self) {}
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
        }
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = key.into();
        self
    }

    pub fn prevent_default(&self) {}
}

impl Default for KeyboardEvent {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FocusEvent {
    pub related_target: Option<String>,
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
        }
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
        }
    }

    pub fn data(mut self, data: impl Into<String>) -> Self {
        self.data = data.into();
        self
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
