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
    fn default() -> Self { Self { ch: ' ' } }
}

pub struct Vt100Screen {
    cols: usize,
    rows: usize,
    grid: Vec<Vec<Cell>>,
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
            cursor_row: 0,
            cursor_col: 0,
            saved_row: 0,
            saved_col: 0,
            attrs: CellAttrs::default(),
        }
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        let mut ng = vec![vec![Cell::default(); cols]; rows];
        for r in 0..rows.min(self.rows) {
            for c in 0..cols.min(self.cols) { ng[r][c] = self.grid[r][c].clone(); }
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
        let lines: Vec<String> = self.grid.iter().map(|row| {
            row.iter().map(|c| c.ch).collect::<String>().trim_end().to_string()
        }).collect();
        let mut out = lines.clone();
        while out.last().map(|l| l.is_empty()).unwrap_or(false) { out.pop(); }
        out.join("\n")
    }

    pub fn get_line(&self, row: usize) -> String {
        if row >= self.rows { return String::new(); }
        self.grid[row].iter().map(|c| c.ch).collect::<String>().trim_end().to_string()
    }

    pub fn find_text(&self, pattern: &str) -> Vec<(usize, usize)> {
        let mut r = Vec::new();
        for (i, row) in self.grid.iter().enumerate() {
            let line: String = row.iter().map(|c| c.ch).collect();
            if let Some(pos) = line.find(pattern) { r.push((i, pos)); }
        }
        r
    }

    pub fn line_count(&self) -> usize { self.rows }
    pub fn cols_count(&self) -> usize { self.cols }

    fn scroll_up(&mut self, n: usize) {
        for _ in 0..n { self.grid.remove(0); self.grid.push(vec![Cell::default(); self.cols]); }
    }

    fn ensure_cursor_in_bounds(&mut self) {
        if self.cursor_col >= self.cols {
            self.cursor_col = 0; self.cursor_row += 1;
            if self.cursor_row >= self.rows { self.scroll_up(1); self.cursor_row = self.rows - 1; }
        }
        if self.cursor_row >= self.rows {
            self.scroll_up(self.cursor_row - self.rows + 1); self.cursor_row = self.rows - 1;
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
            0x0A => { self.cursor_row += 1; if self.cursor_row >= self.rows { self.scroll_up(1); self.cursor_row = self.rows - 1; } },
            0x08 => { if self.cursor_col > 0 { self.cursor_col -= 1; } },
            0x09 => { self.cursor_col = (self.cursor_col + 8) & !7; if self.cursor_col >= self.cols { self.cursor_col = self.cols - 1; } },
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
        if !intermediates.is_empty() { return; }
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
                    for c in self.cursor_col..self.cols { self.grid[self.cursor_row][c] = Cell::default(); }
                    for r in (self.cursor_row + 1)..self.rows { for c in 0..self.cols { self.grid[r][c] = Cell::default(); } }
                }
                2 => { self.grid.clear(); self.grid.resize(self.rows, vec![Cell::default(); self.cols]); }
                _ => {}
            }
            'K' => match p(0) {
                0 => { for c in self.cursor_col..self.cols { self.grid[self.cursor_row][c] = Cell::default(); } }
                2 => { for c in 0..self.cols { self.grid[self.cursor_row][c] = Cell::default(); } }
                _ => {}
            }
            'm' => { for &v in &pv { match v { 0 => self.attrs = CellAttrs::default(), 1 => self.attrs.bold = true, 3 => self.attrs.italic = true, 4 => self.attrs.underline = true, _ => {} } } }
            's' => { self.saved_row = self.cursor_row; self.saved_col = self.cursor_col; }
            'u' => { self.cursor_row = self.saved_row; self.cursor_col = self.saved_col; }
            'S' => self.scroll_up(p1(0) as usize),
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}
