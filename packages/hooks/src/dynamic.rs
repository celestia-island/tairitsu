use tairitsu_vdom::{dynamic_text, VNode};

pub fn use_dynamic_text<T>(signal: tairitsu_vdom::Signal<T>) -> VNode
where
    T: Clone + std::string::ToString + 'static,
{
    let initial = signal.get().to_string();
    let signal_clone = signal.clone();
    dynamic_text(initial, move || signal_clone.get().to_string())
}

pub fn use_dynamic_text_fn<F>(initial: String, compute: F) -> VNode
where
    F: FnMut() -> String + 'static,
{
    dynamic_text(initial, compute)
}
