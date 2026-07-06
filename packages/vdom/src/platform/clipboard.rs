pub trait ClipboardOps: Sized + 'static {
    fn copy_to_clipboard(&self, text: &str) -> bool;
    fn read_clipboard(&self) -> Option<String>;
    fn clipboard_write_text_async(
        &self,
        text: &str,
        on_complete: Box<dyn FnOnce(Result<(), String>)>,
    );
    fn clipboard_read_text_async(&self, on_complete: Box<dyn FnOnce(Result<String, String>)>);
}
