use super::r#trait::{MutationObserverInit, MutationRecord, ResizeObserverEntry};
use crate::platform::dom::DomOps;

pub trait ObserverOps: DomOps {
    fn create_resize_observer(&self, callback: Box<dyn FnMut(Vec<ResizeObserverEntry>)>) -> u64;
    fn observe_resize(&self, observer: u64, element: &Self::Element);
    fn unobserve_resize(&self, observer: u64, element: &Self::Element);
    fn disconnect_resize(&self, observer: u64);
    fn create_mutation_observer(&self, callback: Box<dyn FnMut(Vec<MutationRecord>)>) -> u64;
    fn observe_mutations(
        &self,
        observer: u64,
        element: &Self::Element,
        options: Option<MutationObserverInit>,
    );
    fn disconnect_mutation(&self, observer: u64);
}
