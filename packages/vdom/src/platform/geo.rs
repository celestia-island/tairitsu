use super::r#trait::{GeoPosition, GeoPositionError};

pub trait GeoOps: Sized + 'static {
    fn get_current_position(
        &self,
        on_success: Box<dyn FnOnce(GeoPosition)>,
        on_error: Box<dyn FnOnce(GeoPositionError)>,
        enable_high_accuracy: bool,
        timeout: u32,
        maximum_age: u32,
    );
}
