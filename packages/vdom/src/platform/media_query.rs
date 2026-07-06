pub trait MediaQueryOps: Sized + 'static {
    fn match_media(&self, query: &str) -> u64;
    fn media_query_list_get_media(&self, list: u64) -> String;
    fn media_query_list_get_matches(&self, list: u64) -> bool;
    fn media_query_list_add_listener(&self, list: u64, callback: Box<dyn FnMut(bool)>) -> u64;
    fn media_query_list_remove_listener(&self, list: u64, listener_id: u64);
}
