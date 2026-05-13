use super::r#trait::CanvasContext;
use crate::platform::dom::DomOps;

pub trait CanvasOps: DomOps {
    fn get_canvas_context(
        &self,
        element: &Self::Element,
        context_type: &str,
    ) -> Option<CanvasContext>;
    fn canvas_set_fill_style(&self, ctx: CanvasContext, color: &str);
    fn canvas_fill_rect(&self, ctx: CanvasContext, x: f64, y: f64, w: f64, h: f64);
    fn canvas_clear_rect(&self, ctx: CanvasContext, x: f64, y: f64, w: f64, h: f64);
    fn draw_qrcode_on_canvas_by_id(
        &self,
        canvas_id: &str,
        matrix: &[Vec<bool>],
        modules: u64,
        color: &str,
        background: &str,
    ) -> bool;
}
