use crate::platform::dom::DomOps;

pub trait MediaOps: DomOps {
    fn video_play(&self, element: &Self::Element);
    fn video_pause(&self, element: &Self::Element);
    fn video_get_current_time(&self, element: &Self::Element) -> f64;
    fn video_get_duration(&self, element: &Self::Element) -> f64;
    fn video_seek(&self, element: &Self::Element, time: f64);
    fn video_set_muted(&self, element: &Self::Element, muted: bool);
    fn video_set_volume(&self, element: &Self::Element, volume: f64);
    fn create_audio_context(&self) -> u64;
    fn create_analyser_node(&self, audio_context: u64) -> u64;
    fn create_media_element_source(&self, audio_context: u64, element: u64) -> u64;
    fn analyser_node_get_frequency_data(&self, analyser: u64) -> Vec<f32>;
    fn analyser_node_get_time_domain_data(&self, analyser: u64) -> Vec<f32>;
}
