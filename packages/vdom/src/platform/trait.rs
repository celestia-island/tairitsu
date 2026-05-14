use super::canvas::CanvasOps;
use super::clipboard::ClipboardOps;
use super::content_editable::ContentEditableOps;
use super::dom::DomOps;
use super::file::FileOps;
use super::geo::GeoOps;
use super::idb::IdbOps;
use super::layout::LayoutOps;
use super::media::MediaOps;
use super::media_query::MediaQueryOps;
use super::observer::ObserverOps;
use super::query::QueryOps;
use super::scroll::ScrollOps;
use super::timer::TimerOps;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DomRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub type CanvasContext = u64;

pub trait Platform:
    DomOps
    + TimerOps
    + LayoutOps
    + ObserverOps
    + MediaQueryOps
    + ClipboardOps
    + ContentEditableOps
    + ScrollOps
    + QueryOps
    + CanvasOps
    + MediaOps
    + GeoOps
    + FileOps
    + IdbOps
{
}

impl<T> Platform for T where
    T: DomOps
        + TimerOps
        + LayoutOps
        + ObserverOps
        + MediaQueryOps
        + ClipboardOps
        + ContentEditableOps
        + ScrollOps
        + QueryOps
        + CanvasOps
        + MediaOps
        + GeoOps
        + FileOps
        + IdbOps
{
}

pub struct ResizeObserverEntry {
    pub target: u64,
    pub content_rect: DomRect,
    pub border_box_size: Vec<ResizeObserverSize>,
    pub content_box_size: Vec<ResizeObserverSize>,
}

pub struct ResizeObserverSize {
    pub inline_size: f64,
    pub block_size: f64,
}

pub struct MutationObserverInit {
    pub child_list: bool,
    pub attributes: bool,
    pub character_data: bool,
    pub subtree: bool,
    pub attribute_old_value: bool,
    pub character_data_old_value: bool,
}

pub struct MutationRecord {
    pub record_type: String,
    pub target: u64,
    pub added_nodes: Vec<u64>,
    pub removed_nodes: Vec<u64>,
    pub previous_sibling: Option<u64>,
    pub next_sibling: Option<u64>,
    pub attribute_name: Option<String>,
    pub attribute_namespace: Option<String>,
    pub old_value: Option<String>,
}

pub struct ContentEditableState {
    pub editable: bool,
    pub focused: bool,
}

#[derive(Clone, Debug)]
pub struct GeoPosition {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub accuracy: f64,
    pub altitude_accuracy: Option<f64>,
    pub heading: Option<f64>,
    pub speed: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct GeoPositionError {
    pub code: u8,
    pub message: String,
}
