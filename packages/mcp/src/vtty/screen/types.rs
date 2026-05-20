use super::super::graphics::InlineImageStore;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorKind {
    #[default]
    Default,
    Index(u8),
    Rgb(u8, u8, u8),
}

#[derive(Clone, Copy, Default)]
pub struct CellAttrs {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub fg: ColorKind,
    pub bg: ColorKind,
}

#[derive(Clone)]
pub struct Cell {
    pub ch: char,
    pub attrs: CellAttrs,
    pub wide: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            attrs: CellAttrs::default(),
            wide: false,
        }
    }
}

#[derive(Clone)]
pub struct RenderData {
    pub rows: usize,
    pub cols: usize,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub grid: Vec<Vec<Cell>>,
    pub image_store: InlineImageStore,
}
