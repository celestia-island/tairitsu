pub trait IdbOps: Sized + 'static {
    fn idb_open(
        &self,
        name: &str,
        version: Option<u64>,
        on_complete: Box<dyn FnOnce(Result<u64, String>)>,
    ) -> u64;
    fn idb_put(
        &self,
        db: u64,
        store_name: &str,
        value: &str,
        key: Option<&str>,
        on_complete: Box<dyn FnOnce(Result<(), String>)>,
    );
    fn idb_get(
        &self,
        db: u64,
        store_name: &str,
        key: &str,
        on_complete: Box<dyn FnOnce(Result<Option<String>, String>)>,
    );
    fn idb_delete(
        &self,
        db: u64,
        store_name: &str,
        key: &str,
        on_complete: Box<dyn FnOnce(Result<(), String>)>,
    );
    fn idb_get_all(
        &self,
        db: u64,
        store_name: &str,
        on_complete: Box<dyn FnOnce(Result<Vec<String>, String>)>,
    );
    fn idb_clear(
        &self,
        db: u64,
        store_name: &str,
        on_complete: Box<dyn FnOnce(Result<(), String>)>,
    );
}
