use vte::{Params, Perform};

#[derive(Clone, Copy, Default)]
struct CellAttrs {
    bold: bool,
    italic: bool,
    underline: bool,
}

#[derive(Clone)]
struct Cell {
    ch: char,
}

impl Default for Cell {
    fn default() -> Self {
        Self { ch: ' ' }
    }
}

pub struct Vt100Screen {
    cols: usize,
    rows: usize,
    grid: Vec<Vec<Cell>>,
    scrollback: Vec<Vec<Cell>>,
    cursor_row: usize,
    cursor_col: usize,
    saved_row: usize,
    saved_col: usize,
    attrs: CellAttrs,
}

impl Vt100Screen {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            grid: vec![vec![Cell::default(); cols]; rows],
            scrollback: Vec::new(),
            cursor_row: 0,
            cursor_col: 0,
            saved_row: 0,
            saved_col: 0,
            attrs: CellAttrs::default(),
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn resize(&mut self, cols: usize, rows: usize) {
        let mut ng = vec![vec![Cell::default(); cols]; rows];
        for r in 0..rows.min(self.rows) {
            for c in 0..cols.min(self.cols) {
                ng[r][c] = self.grid[r][c].clone();
            }
        }
        self.grid = ng;
        self.cols = cols;
        self.rows = rows;
        self.cursor_row = self.cursor_row.min(rows.saturating_sub(1));
        self.cursor_col = self.cursor_col.min(cols.saturating_sub(1));
    }

    pub fn process(&mut self, data: &[u8]) {
        let mut parser = vte::Parser::new();
        parser.advance(self, data);
    }

    pub fn get_text(&self) -> String {
        let lines: Vec<String> = self
            .grid
            .iter()
            .map(|row| {
                row.iter()
                    .map(|c| c.ch)
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect();
        let mut out = lines.clone();
        while out.last().map(|l| l.is_empty()).unwrap_or(false) {
            out.pop();
        }
        out.join("\n")
    }

    pub fn get_line(&self, row: usize) -> String {
        if row >= self.rows {
            return String::new();
        }
        self.grid[row]
            .iter()
            .map(|c| c.ch)
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    pub fn find_text(&self, pattern: &str) -> Vec<(usize, usize)> {
        let mut r = Vec::new();
        for (i, row) in self.grid.iter().enumerate() {
            let line: String = row.iter().map(|c| c.ch).collect();
            if let Some(pos) = line.find(pattern) {
                r.push((i, pos));
            }
        }
        r
    }

    pub fn line_count(&self) -> usize {
        self.rows
    }
    pub fn cols_count(&self) -> usize {
        self.cols
    }

    pub fn has_output(&self) -> bool {
        self.grid.iter().any(|row| row.iter().any(|c| c.ch != ' '))
    }

    pub fn get_scrollback(&self) -> String {
        let lines: Vec<String> = self
            .scrollback
            .iter()
            .map(|row| {
                row.iter()
                    .map(|c| c.ch)
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect();
        lines.join("\n")
    }

    pub fn get_scrollback_with_screen(&self) -> String {
        let sb = self.get_scrollback();
        let screen = self.get_text();
        if sb.is_empty() {
            screen
        } else if screen.is_empty() {
            sb
        } else {
            format!("{}\n{}", sb, screen)
        }
    }

    fn scroll_up(&mut self, n: usize) {
        for _ in 0..n {
            self.scrollback.push(self.grid.remove(0));
            self.grid.push(vec![Cell::default(); self.cols]);
        }
        while self.scrollback.len() > 1000 {
            self.scrollback.remove(0);
        }
    }

    fn ensure_cursor_in_bounds(&mut self) {
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.cursor_row += 1;
            if self.cursor_row >= self.rows {
                self.scroll_up(1);
                self.cursor_row = self.rows - 1;
            }
        }
        if self.cursor_row >= self.rows {
            self.scroll_up(self.cursor_row - self.rows + 1);
            self.cursor_row = self.rows - 1;
        }
    }
}

impl Perform for Vt100Screen {
    fn print(&mut self, c: char) {
        self.ensure_cursor_in_bounds();
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            self.grid[self.cursor_row][self.cursor_col] = Cell { ch: c };
        }
        self.cursor_col += 1;
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x0D => self.cursor_col = 0,
            0x0A => {
                self.cursor_row += 1;
                if self.cursor_row >= self.rows {
                    self.scroll_up(1);
                    self.cursor_row = self.rows - 1;
                }
            }
            0x08 if self.cursor_col > 0 => {
                self.cursor_col -= 1;
            }
            0x09 => {
                self.cursor_col = (self.cursor_col + 8) & !7;
                if self.cursor_col >= self.cols {
                    self.cursor_col = self.cols - 1;
                }
            }
            _ => {}
        }
    }

    fn hook(&mut self, _: &Params, _: &[u8], _: bool, _: char) {}
    fn put(&mut self, _: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _: &[&[u8]], _: bool) {}

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, action: char) {
        let pv: Vec<u16> = params.iter().flat_map(|p| p.iter().copied()).collect();
        let p = |i: usize| -> u16 { pv.get(i).copied().unwrap_or(0) };
        let p1 = |i: usize| -> u16 { pv.get(i).copied().unwrap_or(1).max(1) };
        if !intermediates.is_empty() {
            return;
        }
        match action {
            'A' => self.cursor_row = self.cursor_row.saturating_sub(p1(0) as usize),
            'B' => self.cursor_row = (self.cursor_row + p1(0) as usize).min(self.rows - 1),
            'C' => self.cursor_col = (self.cursor_col + p1(0) as usize).min(self.cols - 1),
            'D' => self.cursor_col = self.cursor_col.saturating_sub(p1(0) as usize),
            'H' | 'f' => {
                self.cursor_row = (p1(0) as usize).saturating_sub(1).min(self.rows - 1);
                self.cursor_col = (p1(1) as usize).saturating_sub(1).min(self.cols - 1);
            }
            'J' => match p(0) {
                0 => {
                    for c in self.cursor_col..self.cols {
                        self.grid[self.cursor_row][c] = Cell::default();
                    }
                    for r in (self.cursor_row + 1)..self.rows {
                        for c in 0..self.cols {
                            self.grid[r][c] = Cell::default();
                        }
                    }
                }
                2 => {
                    self.grid.clear();
                    self.grid
                        .resize(self.rows, vec![Cell::default(); self.cols]);
                }
                _ => {}
            },
            'K' => match p(0) {
                0 => {
                    for c in self.cursor_col..self.cols {
                        self.grid[self.cursor_row][c] = Cell::default();
                    }
                }
                2 => {
                    for c in 0..self.cols {
                        self.grid[self.cursor_row][c] = Cell::default();
                    }
                }
                _ => {}
            },
            'm' => {
                for &v in &pv {
                    match v {
                        0 => self.attrs = CellAttrs::default(),
                        1 => self.attrs.bold = true,
                        3 => self.attrs.italic = true,
                        4 => self.attrs.underline = true,
                        _ => {}
                    }
                }
            }
            's' => {
                self.saved_row = self.cursor_row;
                self.saved_col = self.cursor_col;
            }
            'u' => {
                self.cursor_row = self.saved_row;
                self.cursor_col = self.saved_col;
            }
            'S' => self.scroll_up(p1(0) as usize),
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_print() {
        let mut s = Vt100Screen::new(20, 5);
        s.process(b"Hello World");
        assert_eq!(s.get_text(), "Hello World");
    }

    #[test]
    fn test_newline_and_carriage_return() {
        let mut s = Vt100Screen::new(20, 5);
        s.process(b"line1\r\nline2");
        assert_eq!(s.get_line(0), "line1");
        assert_eq!(s.get_line(1), "line2");
    }

    #[test]
    fn test_cursor_movement_csi() {
        let mut s = Vt100Screen::new(20, 3);
        s.process(b"ABC\x1b[2;1HXY");
        assert_eq!(s.get_line(0), "ABC");
        assert_eq!(s.get_line(1), "XY");
    }

    #[test]
    fn test_clear_screen() {
        let mut s = Vt100Screen::new(10, 3);
        s.process(b"fill this line\x1b[2J");
        assert_eq!(s.get_text(), "");
    }

    #[test]
    fn test_clear_to_end_of_line() {
        let mut s = Vt100Screen::new(20, 2);
        s.process(b"ABCDEFGH\x1b[1;3HX");
        assert_eq!(s.get_line(0), "ABXDEFGH");
    }

    #[test]
    fn test_scroll_up() {
        let mut s = Vt100Screen::new(10, 3);
        s.process(b"line0\r\nline1\r\nline2\r\nline3");
        assert_eq!(s.get_line(0), "line1");
        assert_eq!(s.get_line(1), "line2");
        assert_eq!(s.get_line(2), "line3");
    }

    #[test]
    fn test_tab_stop() {
        let mut s = Vt100Screen::new(20, 2);
        s.process(b"A\tB");
        let line = s.get_line(0);
        assert_eq!(line.as_bytes()[0], b'A');
        assert_eq!(line.as_bytes()[8], b'B');
    }

    #[test]
    fn test_backspace() {
        let mut s = Vt100Screen::new(20, 2);
        s.process(b"abc\x08X");
        assert_eq!(s.get_line(0), "abX");
    }

    #[test]
    fn test_save_restore_cursor() {
        let mut s = Vt100Screen::new(20, 3);
        s.process(b"AAA\x1b[s\x1b[2;1HBBB\x1b[uCC");
        assert_eq!(s.get_line(0), "AAACC");
        assert_eq!(s.get_line(1), "BBB");
    }

    #[test]
    fn test_find_text() {
        let mut s = Vt100Screen::new(30, 2);
        s.process(b"hello world\r\nfoo bar baz");
        let hits = s.find_text("bar");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0], (1, 4));
    }

    #[test]
    fn test_resize_preserves_content() {
        let mut s = Vt100Screen::new(10, 3);
        s.process(b"keep me\r\nand this");
        s.resize(15, 5);
        assert_eq!(s.cols_count(), 15);
        assert_eq!(s.line_count(), 5);
        assert_eq!(s.get_line(0), "keep me");
        assert_eq!(s.get_line(1), "and this");
    }

    #[test]
    fn test_sgr_bold_italic_underline() {
        let mut s = Vt100Screen::new(20, 2);
        s.process(b"\x1b[1;3;4mstyled\x1b[0m normal");
        assert_eq!(s.get_line(0), "styled normal");
    }

    #[test]
    fn test_empty_screen() {
        let s = Vt100Screen::new(10, 3);
        assert_eq!(s.get_text(), "");
    }

    #[test]
    fn test_get_line_out_of_bounds() {
        let s = Vt100Screen::new(10, 3);
        assert_eq!(s.get_line(99), "");
    }

    #[test]
    fn test_cursor_up_down_left_right() {
        let mut s = Vt100Screen::new(20, 4);
        s.process(b"row0\r\nrow1\r\nrow2");
        s.process(b"\x1b[A"); // cursor up
        s.process(b"!");
        assert_eq!(s.get_line(1), "row1!");
    }
}
