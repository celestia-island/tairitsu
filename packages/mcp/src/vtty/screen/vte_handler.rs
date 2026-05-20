use vte::Params;

use super::super::graphics::{process_osc_1337, process_sixel};
use super::types::{Cell, CellAttrs, ColorKind};
use super::{DcsKind, Vt100Screen};

impl vte::Perform for Vt100Screen {
    fn print(&mut self, c: char) {
        self.ensure_cursor_in_bounds();
        let w = self.char_width(c);
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            self.grid[self.cursor_row][self.cursor_col] = Cell {
                ch: c,
                attrs: self.attrs,
                wide: w > 1,
            };
            if w == 2 && self.cursor_col + 1 < self.cols {
                self.grid[self.cursor_row][self.cursor_col + 1] = Cell {
                    ch: '\u{0}',
                    attrs: self.attrs,
                    wide: false,
                };
            }
        }
        self.cursor_col += w;
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

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, action: char) {
        match action {
            'q' => {
                self.dcs_kind = DcsKind::Sixel;
                self.dcs_buffer.clear();
            }
            _ => {
                self.dcs_kind = DcsKind::None;
            }
        }
    }

    fn put(&mut self, byte: u8) {
        if matches!(self.dcs_kind, DcsKind::Sixel) {
            self.dcs_buffer.push(byte);
        }
    }

    fn unhook(&mut self) {
        if matches!(self.dcs_kind, DcsKind::Sixel) {
            let data = std::mem::take(&mut self.dcs_buffer);
            process_sixel(
                &data,
                self.cursor_row,
                self.cursor_col,
                &mut self.image_store,
            );
        }
        self.dcs_kind = DcsKind::None;
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _: bool) {
        if params.is_empty() {
            return;
        }
        let first = params[0];
        if first == b"1337" {
            process_osc_1337(
                params,
                self.cursor_row,
                self.cursor_col,
                &mut self.image_store,
            );
        }
    }

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
                let mut i = 0;
                while i < pv.len() {
                    match pv[i] {
                        0 => self.attrs = CellAttrs::default(),
                        1 => self.attrs.bold = true,
                        3 => self.attrs.italic = true,
                        4 => self.attrs.underline = true,
                        7 => {}
                        22 => self.attrs.bold = false,
                        23 => self.attrs.italic = false,
                        24 => self.attrs.underline = false,
                        27 => {}
                        30..=37 => {
                            self.attrs.fg = ColorKind::Index((pv[i] - 30) as u8);
                        }
                        38 if i + 1 < pv.len() => match pv[i + 1] {
                            5 if i + 2 < pv.len() => {
                                self.attrs.fg = ColorKind::Index(pv[i + 2] as u8);
                                i += 2;
                            }
                            2 if i + 4 < pv.len() => {
                                self.attrs.fg = ColorKind::Rgb(
                                    pv[i + 2] as u8,
                                    pv[i + 3] as u8,
                                    pv[i + 4] as u8,
                                );
                                i += 4;
                            }
                            _ => {}
                        },
                        39 => self.attrs.fg = ColorKind::Default,
                        40..=47 => {
                            self.attrs.bg = ColorKind::Index((pv[i] - 40) as u8);
                        }
                        48 if i + 1 < pv.len() => match pv[i + 1] {
                            5 if i + 2 < pv.len() => {
                                self.attrs.bg = ColorKind::Index(pv[i + 2] as u8);
                                i += 2;
                            }
                            2 if i + 4 < pv.len() => {
                                self.attrs.bg = ColorKind::Rgb(
                                    pv[i + 2] as u8,
                                    pv[i + 3] as u8,
                                    pv[i + 4] as u8,
                                );
                                i += 4;
                            }
                            _ => {}
                        },
                        49 => self.attrs.bg = ColorKind::Default,
                        90..=97 => {
                            self.attrs.fg = ColorKind::Index((pv[i] - 90 + 8) as u8);
                        }
                        100..=107 => {
                            self.attrs.bg = ColorKind::Index((pv[i] - 100 + 8) as u8);
                        }
                        _ => {}
                    }
                    i += 1;
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

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, byte: u8) {
        match byte {
            b'7' => {
                self.saved_row = self.cursor_row;
                self.saved_col = self.cursor_col;
            }
            b'8' => {
                self.cursor_row = self.saved_row;
                self.cursor_col = self.saved_col;
            }
            _ => {}
        }
    }
}
