pub trait TimerOps: Sized + 'static {
    fn set_timeout(&self, callback: Box<dyn FnOnce()>, ms: i32) -> i32;
    fn clear_timeout(&self, id: i32);
    fn set_interval(&self, callback: Box<dyn FnMut()>, ms: i32) -> i32;
    fn clear_interval(&self, id: i32);
    fn request_animation_frame(&self, callback: Box<dyn FnOnce(f64)>) -> u32;
    fn cancel_animation_frame(&self, id: u32);
}
