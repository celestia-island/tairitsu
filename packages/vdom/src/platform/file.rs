pub trait FileOps: Sized + 'static {
    fn file_reader_sync_read_as_text(
        &self,
        blob: u64,
        encoding: Option<&str>,
    ) -> Result<String, String>;
    fn file_reader_sync_read_as_array_buffer(&self, blob: u64) -> Result<Vec<u8>, String>;
    fn file_reader_read_as_text(
        &self,
        blob: u64,
        encoding: Option<&str>,
        on_complete: Box<dyn FnOnce(Result<String, String>)>,
    );
    fn file_reader_read_as_array_buffer(
        &self,
        blob: u64,
        on_complete: Box<dyn FnOnce(Result<Vec<u8>, String>)>,
    );
}
