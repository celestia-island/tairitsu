use std::any::Any;

pub trait EventHandle: 'static {
    fn as_any(&self) -> &dyn Any;
}
