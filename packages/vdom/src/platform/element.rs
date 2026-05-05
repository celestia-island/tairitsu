use std::any::Any;

pub trait ElementHandle: Clone + 'static {
    fn as_any(&self) -> &dyn Any;
}
