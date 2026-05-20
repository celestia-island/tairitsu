mod types;
mod vte_handler;

pub use types::{Cell, CellAttrs, ColorKind, RenderData};

use crate::vtty::graphics::{
    process_kitty_apc, InlineImageStore, KittyGraphicsState,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum DcsKind {
    None,
    Sixel,
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
    pub image_store: InlineImageStore,
    kitty_state: KittyGraphicsState,
    dcs_kind: DcsKind,
    dcs_buffer: Vec<u8>,
    parser: vte::Parser,
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
            image_store: InlineImageStore::new(),
            kitty_state: KittyGraphicsState::new(),
            dcs_kind: DcsKind::None,
            dcs_buffer: Vec::new(),
            parser: vte::Parser::new(),
        }
    }

    fn char_width(&self, c: char) -> usize {
        if c < '\u{1100}' {
            return 1;
        }
        unicode_width::UnicodeWidthChar::width(c).unwrap_or(1)
    }

    fn row_to_string(row: &[Cell]) -> String {
        let mut line = String::new();
        let mut skip = false;
        for cell in row.iter() {
            if skip {
                skip = false;
                continue;
            }
            if cell.ch == '\u{0}' {
                continue;
            }
            line.push(cell.ch);
            if cell.wide {
                skip = true;
            }
        }
        line.trim_end().to_string()
    }

    #[allow(clippy::needless_range_loop)]
    pub fn resize(&mut self, cols: usize, rows: usize) {
        let mut ng = vec![vec![Cell::default(); cols]; rows];
        for r in 0..rows.min(self.rows) {
            for c in 0..cols.min(self.cols) {
                ng[r][c] = self.grid[r][c].clone();
            }
        }
        if rows < self.rows {
            for r in rows..self.rows {
                self.scrollback.push(self.grid[r].clone());
            }
            while self.scrollback.len() > 1000 {
                self.scrollback.remove(0);
            }
        }
        for r in 0..rows {
            for c in 0..cols {
                if ng[r][c].wide && c + 1 >= cols {
                    ng[r][c] = Cell::default();
                }
                if ng[r][c].ch == '\u{0}' && (c == 0 || !ng[r][c - 1].wide) {
                    ng[r][c] = Cell::default();
                }
            }
        }
        self.grid = ng;
        self.cols = cols;
        self.rows = rows;
        self.cursor_row = self.cursor_row.min(rows.saturating_sub(1));
        self.cursor_col = self.cursor_col.min(cols.saturating_sub(1));
    }

    pub fn get_text(&self) -> String {
        let lines: Vec<String> = self.grid.iter().map(|row| Self::row_to_string(row)).collect();
        let mut out = lines;
        while out.last().map(|l| l.is_empty()).unwrap_or(false) {
            out.pop();
        }
        out.join("\n")
    }

    pub fn get_line(&self, row: usize) -> String {
        if row >= self.rows {
            return String::new();
        }
        Self::row_to_string(&self.grid[row])
    }

    pub fn find_text(&self, pattern: &str) -> Vec<(usize, usize)> {
        let mut r = Vec::new();
        for (i, row) in self.grid.iter().enumerate() {
            let line: String = row.iter().map(|c| c.ch).filter(|&c| c != '\u{0}').collect();
            if let Some(pos) = line.find(pattern) {
                r.push((i, pos));
            }
        }
        r
    }

    pub fn has_output(&self) -> bool {
        self.grid.iter().any(|row| row.iter().any(|c| c.ch != ' '))
    }

    pub fn get_scrollback(&self) -> String {
        self.scrollback
            .iter()
            .map(|row| Self::row_to_string(row))
            .collect::<Vec<_>>()
            .join("\n")
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

    pub fn get_render_data(&self) -> RenderData {
        RenderData {
            rows: self.rows,
            cols: self.cols,
            cursor_row: self.cursor_row,
            cursor_col: self.cursor_col,
            grid: self.grid.clone(),
            image_store: self.image_store.clone(),
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

    fn scroll_down(&mut self, n: usize) {
        for _ in 0..n {
            self.grid.pop();
            self.grid.insert(0, vec![Cell::default(); self.cols]);
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

    fn extract_kitty_apcs(data: &[u8]) -> Vec<(usize, usize, String, Vec<u8>)> {
        let mut results = Vec::new();
        let mut i = 0;
        while i < data.len().saturating_sub(3) {
            if data[i] == 0x1B && data[i + 1] == b'_' && data[i + 2] == b'G' {
                let start = i;
                let payload_start = i + 3;
                let mut j = payload_start;
                while j < data.len().saturating_sub(1) {
                    if data[j] == 0x1B && data[j + 1] == b'\\' {
                        let raw = &data[payload_start..j];
                        let (control, payload) =
                            if let Some(idx) = raw.iter().position(|&b| b == b';') {
                                (
                                    String::from_utf8_lossy(&raw[..idx]).to_string(),
                                    raw[idx + 1..].to_vec(),
                                )
                            } else {
                                (String::from_utf8_lossy(raw).to_string(), Vec::new())
                            };
                        results.push((start, j + 2, control, payload));
                        i = j + 2;
                        break;
                    }
                    j += 1;
                }
                if j >= data.len().saturating_sub(1) {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        results
    }

    pub fn process(&mut self, data: &[u8]) {
        let apcs = Self::extract_kitty_apcs(data);
        for (_, _, control, payload) in &apcs {
            process_kitty_apc(
                &mut self.kitty_state,
                control,
                payload,
                self.cursor_row,
                self.cursor_col,
                &mut self.image_store,
            );
        }
        let mut parser = std::mem::replace(&mut self.parser, vte::Parser::new());
        parser.advance(self, data);
        self.parser = parser;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Basic text ──────────────────────────────────────

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
    fn test_empty_screen() {
        let s = Vt100Screen::new(10, 3);
        assert_eq!(s.get_text(), "");
    }

    #[test]
    fn test_get_line_out_of_bounds() {
        let s = Vt100Screen::new(10, 3);
        assert_eq!(s.get_line(99), "");
    }

    // ── Cursor movement ─────────────────────────────────

    #[test]
    fn test_cursor_movement_csi() {
        let mut s = Vt100Screen::new(20, 3);
        s.process(b"ABC\x1b[2;1HXY");
        assert_eq!(s.get_line(0), "ABC");
        assert_eq!(s.get_line(1), "XY");
    }

    #[test]
    fn test_cursor_up_down_left_right() {
        let mut s = Vt100Screen::new(20, 4);
        s.process(b"row0\r\nrow1\r\nrow2");
        s.process(b"\x1b[A");
        s.process(b"!");
        assert_eq!(s.get_line(1), "row1!");
    }

    #[test]
    fn test_save_restore_cursor() {
        let mut s = Vt100Screen::new(20, 3);
        s.process(b"AAA\x1b[s\x1b[2;1HBBB\x1b[uCC");
        assert_eq!(s.get_line(0), "AAACC");
        assert_eq!(s.get_line(1), "BBB");
    }

    #[test]
    fn test_backspace() {
        let mut s = Vt100Screen::new(20, 2);
        s.process(b"abc\x08X");
        assert_eq!(s.get_line(0), "abX");
    }

    #[test]
    fn test_tab_stop() {
        let mut s = Vt100Screen::new(20, 2);
        s.process(b"A\tB");
        let line = s.get_line(0);
        assert_eq!(line.as_bytes()[0], b'A');
        assert_eq!(line.as_bytes()[8], b'B');
    }

    // ── Clear / scroll ──────────────────────────────────

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
    fn test_resize_preserves_content() {
        let mut s = Vt100Screen::new(10, 3);
        s.process(b"keep me\r\nand this");
        s.resize(15, 5);
        assert_eq!(s.get_line(0), "keep me");
        assert_eq!(s.get_line(1), "and this");
        assert_eq!(s.get_line(4), "");
    }

    #[test]
    fn test_find_text() {
        let mut s = Vt100Screen::new(30, 2);
        s.process(b"hello world\r\nfoo bar baz");
        let hits = s.find_text("bar");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0], (1, 4));
    }

    // ── SGR colors ──────────────────────────────────────

    #[test]
    fn test_sgr_bold_italic_underline() {
        let mut s = Vt100Screen::new(20, 2);
        s.process(b"\x1b[1;3;4mstyled\x1b[0m normal");
        assert_eq!(s.get_line(0), "styled normal");
    }

    #[test]
    fn test_sgr_foreground_ansi_colors() {
        let mut s = Vt100Screen::new(40, 2);
        s.process(b"\x1b[31mred\x1b[0m normal");
        let rd = s.get_render_data();
        assert_eq!(rd.grid[0][0].attrs.fg, ColorKind::Index(1));
        assert_eq!(rd.grid[0][4].attrs.fg, ColorKind::Default);
    }

    #[test]
    fn test_sgr_256_color() {
        let mut s = Vt100Screen::new(40, 2);
        s.process(b"\x1b[38;5;196mX\x1b[0m");
        let rd = s.get_render_data();
        assert_eq!(rd.grid[0][0].attrs.fg, ColorKind::Index(196));
    }

    #[test]
    fn test_sgr_truecolor() {
        let mut s = Vt100Screen::new(40, 2);
        s.process(b"\x1b[38;2;255;100;0mX\x1b[0m");
        let rd = s.get_render_data();
        assert_eq!(rd.grid[0][0].attrs.fg, ColorKind::Rgb(255, 100, 0));
    }

    #[test]
    fn test_sgr_bright_colors() {
        let mut s = Vt100Screen::new(40, 2);
        s.process(b"\x1b[91mX\x1b[0m");
        let rd = s.get_render_data();
        assert_eq!(rd.grid[0][0].attrs.fg, ColorKind::Index(9));
    }

    #[test]
    fn test_sgr_background_colors() {
        let mut s = Vt100Screen::new(40, 2);
        s.process(b"\x1b[48;2;0;0;128mX\x1b[0m");
        let rd = s.get_render_data();
        assert_eq!(rd.grid[0][0].attrs.bg, ColorKind::Rgb(0, 0, 128));
    }

    // ── Split escape sequences ─────────────────────────────

    #[test]
    fn test_sgr_split_across_reads() {
        let mut s = Vt100Screen::new(40, 2);
        s.process(b"\x1b[38;2;255;107;15");
        s.process(b"7mX\x1b[0m");
        assert_eq!(s.get_text(), "X");
        let rd = s.get_render_data();
        assert_eq!(rd.grid[0][0].attrs.fg, ColorKind::Rgb(255, 107, 157));
    }

    #[test]
    fn test_csi_split_across_reads() {
        let mut s = Vt100Screen::new(40, 2);
        s.process(b"AB\x1b[2;");
        s.process(b"5HXY");
        assert_eq!(s.get_line(0), "AB");
        assert_eq!(s.get_line(1), "    XY");
    }

    // ── Wide char / CJK ─────────────────────────────────

    #[test]
    fn test_cjk_basic() {
        let mut s = Vt100Screen::new(40, 5);
        s.process("简体中文".as_bytes());
        assert_eq!(s.get_text(), "简体中文");
    }

    #[test]
    fn test_cjk_with_sgr_color() {
        let mut s = Vt100Screen::new(40, 5);
        s.process(b"\x1b[38;2;255;107;157m\xe7\xae\x80\xe4\xbd\x93\x1b[0m");
        assert_eq!(s.get_text(), "简体");
    }

    #[test]
    fn test_cjk_crossterm_absolute_positioning() {
        let mut s = Vt100Screen::new(80, 10);
        s.process(b"\x1b[8;35H\xe9\x80\x89\x1b[8;37H\xe6\x8b\xa9\x1b[8;39H\xe8\xaf\xad\x1b[8;41H\xe8\xa8\x80");
        assert_eq!(s.get_line(7), "                                  选择语言");
    }

    #[test]
    fn test_cjk_mixed_ascii() {
        let mut s = Vt100Screen::new(40, 5);
        s.process("Hello 简体 World".as_bytes());
        assert_eq!(s.get_text(), "Hello 简体 World");
    }

    #[test]
    fn test_cjk_cursor_advance() {
        let mut s = Vt100Screen::new(20, 5);
        s.process("简体".as_bytes());
        assert_eq!(s.get_text(), "简体");
        s.process(b"X");
        let text = s.get_text();
        assert!(text.ends_with("简体X"), "expected cursor at col 4 after 2 wide chars, got: {}", text);
    }

    #[test]
    fn test_cjk_full_dialog_frame() {
        let mut s = Vt100Screen::new(80, 24);
        s.process(b"\x1b[7;30H\x1b[38;2;255;107;157;48;2;18;18;18m\x1b[7;31H\xe9\x80\x89");
        s.process(b"\x1b[7;33H\xe6\x8b\xa9");
        s.process(b"\x1b[7;35H\xe8\xaf\xad");
        s.process(b"\x1b[7;37H\xe8\xa8\x80");
        s.process(b"\x1b[8;33H\xe7\xae\x80");
        s.process(b"\x1b[8;35H\xe4\xbd\x93");
        s.process(b"\x1b[8;37H\xe4\xb8\xad");
        s.process(b"\x1b[8;39H\xe6\x96\x87");

        let text = s.get_text();
        assert!(text.contains("选择语言"), "title should be readable, got:\n{}", text);
        assert!(text.contains("简体中文"), "menu item should be readable, got:\n{}", text);
    }

    // ── Kitty APC / Sixel ───────────────────────────────

    #[test]
    fn test_apc_hook_put_unhook_chain() {
        let mut s = Vt100Screen::new(40, 10);
        s.process(b"before\x1b_Gf=100,i=1;AAAA\x1b\\after");
        assert_eq!(s.get_text(), "beforeafter");
    }

    #[test]
    fn test_vte_apc_buffer_content() {
        use std::io::Cursor;

        use base64::Engine;
        let mut s = Vt100Screen::new(40, 10);
        let png = image::RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 255]));
        let mut png_bytes = Vec::new();
        png.write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .unwrap();
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
        let apc = format!("\x1b_Gf=100,a=T;{}\x1b\\", b64);
        s.process(apc.as_bytes());

        assert_eq!(s.get_text(), "");
        assert_eq!(
            s.image_store.placements().len(),
            1,
            "should have one placement from Kitty APC"
        );
    }

    #[cfg(feature = "vtty-visual")]
    #[test]
    fn test_kitty_apc_end_to_end() {
        use base64::engine::general_purpose::STANDARD as BASE64;
        use base64::Engine;
        use image::{ImageBuffer, Rgba};

        use crate::vtty::graphics::{process_kitty_apc, InlineImageStore, KittyGraphicsState};

        let mut logo = ImageBuffer::from_pixel(16u32, 8u32, Rgba([0x00u8, 0x2bu8, 0x36u8, 255u8]));
        logo.put_pixel(4, 4, Rgba([255u8, 0u8, 0u8, 255u8]));

        let mut png_bytes = Vec::new();
        use std::io::Cursor;
        logo.write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .unwrap();
        let b64 = BASE64.encode(&png_bytes);

        {
            let mut store = InlineImageStore::new();
            let mut state = KittyGraphicsState::new();
            process_kitty_apc(
                &mut state,
                "a=T,f=100,c=4,r=2",
                b64.as_bytes(),
                3,
                5,
                &mut store,
            );
            assert!(
                !store.placements().is_empty(),
                "direct call with a=T should produce placements"
            );
        }

        {
            let mut store = InlineImageStore::new();
            let mut state = KittyGraphicsState::new();
            process_kitty_apc(&mut state, "f=100,i=42", b64.as_bytes(), 3, 5, &mut store);
            assert!(
                !store.placements().is_empty(),
                "direct call without a=T should produce placements"
            );
        }

        let apc_seq_no_t = format!("\x1b_Gf=100,i=42;{}\x1b\\", b64);

        let mut s = Vt100Screen::new(40, 12);
        s.process(b"\x1b[3;5HT\x1b[0m Logo:\r\n");
        s.process(apc_seq_no_t.as_bytes());
        s.process(b"\r\n\x1b[37mDone.\x1b[0m");

        assert!(s.get_text().contains("Logo:"));
        assert!(s.get_text().contains("Done."));

        let rd = s.get_render_data();
        let pc = rd.image_store.placements().len();
        assert!(
            pc > 0,
            "kitty APC via vte should produce placements (got {})",
            pc,
        );

        let png_data = crate::vtty::render::render_terminal(&rd, "solarized-dark")
            .expect("render should succeed");
        assert!(png_data.len() > 100, "PNG should be non-trivial");
        assert_eq!(&png_data[0..4], &[0x89u8, 0x50, 0x4e, 0x47]);
    }

    #[test]
    fn test_sixel_dcs_end_to_end() {
        let sixel_seq = b"\x1bPq#1;2;100;0;0!6~\x1b\\";

        let mut s = Vt100Screen::new(40, 12);
        s.process(b"Sixel test:\r\n");
        s.process(sixel_seq);
        s.process(b"\r\n");

        let rd = s.get_render_data();
        assert!(
            !rd.image_store.placements().is_empty(),
            "sixel image should be captured"
        );
    }
}

#[cfg(test)]
mod smoke {
    use super::types::ColorKind;
    use super::Vt100Screen;

    fn crossterm_render(cols: u16, rows: u16, draw: impl FnOnce(&mut Vec<u8>)) -> Vt100Screen {
        let mut buf = Vec::new();
        draw(&mut buf);
        assert!(!buf.is_empty(), "crossterm produced no output");
        let mut screen = Vt100Screen::new(cols as usize, rows as usize);
        screen.process(&buf);
        screen
    }

    fn crossterm_render_chunked(
        cols: u16,
        rows: u16,
        chunk_size: usize,
        draw: impl FnOnce(&mut Vec<u8>),
    ) -> Vt100Screen {
        let mut buf = Vec::new();
        draw(&mut buf);
        assert!(!buf.is_empty(), "crossterm produced no output");
        let mut screen = Vt100Screen::new(cols as usize, rows as usize);
        for chunk in buf.chunks(chunk_size) {
            screen.process(chunk);
        }
        screen
    }

    macro_rules! queue_cmd {
        ($buf:expr, $($cmd:expr),+ $(,)?) => {{
            use crossterm::QueueableCommand;
            $( $buf.queue($cmd).unwrap(); )+
        }};
    }

    #[test]
    fn smoke_crossterm_basic_ascii() {
        let s = crossterm_render(40, 5, |buf| {
            queue_cmd!(buf,
                crossterm::cursor::MoveTo(0, 0),
                crossterm::style::Print("Hello World"),
            );
        });
        assert_eq!(s.get_line(0), "Hello World");
        assert_eq!(s.get_text(), "Hello World");
    }

    #[test]
    fn smoke_crossterm_cursor_positioning() {
        let s = crossterm_render(40, 5, |buf| {
            queue_cmd!(buf,
                crossterm::cursor::MoveTo(5, 2),
                crossterm::style::Print("XY"),
                crossterm::cursor::MoveTo(0, 0),
                crossterm::style::Print("AB"),
                crossterm::cursor::MoveTo(10, 1),
                crossterm::style::Print("CD"),
            );
        });
        assert_eq!(s.get_line(0), "AB");
        assert_eq!(s.get_line(1), "          CD");
        assert_eq!(s.get_line(2), "     XY");
    }

    #[test]
    fn smoke_crossterm_move_up_down_left_right() {
        let s = crossterm_render(40, 5, |buf| {
            queue_cmd!(buf,
                crossterm::cursor::MoveTo(10, 3),
                crossterm::style::Print("X"),
                crossterm::cursor::MoveUp(1),
                crossterm::style::Print("Y"),
                crossterm::cursor::MoveLeft(1),
                crossterm::style::Print("Z"),
                crossterm::cursor::MoveDown(2),
                crossterm::style::Print("W"),
                crossterm::cursor::MoveRight(2),
                crossterm::style::Print("V"),
            );
        });
        assert_eq!(s.get_line(2), "           Z");
        assert_eq!(s.get_line(3), "          X");
        assert!(s.get_line(4).contains("W"));
        assert!(s.get_line(4).contains("V"));
    }

    #[test]
    fn smoke_crossterm_truecolor_fg() {
        let s = crossterm_render(40, 2, |buf| {
            queue_cmd!(buf,
                crossterm::style::SetForegroundColor(crossterm::style::Color::Rgb { r: 255, g: 107, b: 157 }),
                crossterm::style::Print("pink"),
                crossterm::style::ResetColor,
                crossterm::style::Print(" normal"),
            );
        });
        let text = s.get_text();
        assert!(text.contains("pink"), "text should contain 'pink', got: {}", text);
        assert!(text.contains("normal"), "text should contain 'normal', got: {}", text);
        assert!(!text.contains(';'), "SGR parameters should not leak into text, got: {}", text);
        assert!(
            !text.contains("\x1b["),
            "escape sequences should not leak into text, got: {:?}",
            text
        );

        let rd = s.get_render_data();
        assert_eq!(rd.grid[0][0].attrs.fg, ColorKind::Rgb(255, 107, 157));
        assert_eq!(rd.grid[0][4].attrs.fg, ColorKind::Default);
    }

    #[test]
    fn smoke_crossterm_truecolor_bg() {
        let s = crossterm_render(40, 2, |buf| {
            queue_cmd!(buf,
                crossterm::style::SetBackgroundColor(crossterm::style::Color::Rgb { r: 18, g: 18, b: 18 }),
                crossterm::style::SetForegroundColor(crossterm::style::Color::Rgb { r: 255, g: 107, b: 157 }),
                crossterm::style::Print("styled"),
                crossterm::style::ResetColor,
            );
        });
        let text = s.get_text();
        assert_eq!(text, "styled");

        let rd = s.get_render_data();
        assert_eq!(rd.grid[0][0].attrs.fg, ColorKind::Rgb(255, 107, 157));
        assert_eq!(rd.grid[0][0].attrs.bg, ColorKind::Rgb(18, 18, 18));
        assert_eq!(rd.grid[0][5].attrs.fg, ColorKind::Rgb(255, 107, 157));
        assert_eq!(rd.grid[0][5].attrs.bg, ColorKind::Rgb(18, 18, 18));
    }

    #[test]
    fn smoke_crossterm_cjk_plain() {
        let s = crossterm_render(40, 5, |buf| {
            queue_cmd!(buf,
                crossterm::cursor::MoveTo(0, 0),
                crossterm::style::Print("简体中文"),
            );
        });
        let text = s.get_text();
        assert_eq!(text, "简体中文");
    }

    #[test]
    fn smoke_crossterm_cjk_absolute_positioned() {
        let s = crossterm_render(80, 10, |buf| {
            queue_cmd!(buf,
                crossterm::cursor::MoveTo(34, 6),
                crossterm::style::Print("选择语言"),
                crossterm::cursor::MoveTo(32, 7),
                crossterm::style::Print("简体中文"),
            );
        });
        let text = s.get_text();
        assert!(text.contains("选择语言"), "should contain title, got:\n{}", text);
        assert!(text.contains("简体中文"), "should contain menu, got:\n{}", text);

        let line6 = s.get_line(6);
        assert!(line6.contains("选择语言"), "line 6 should have title, got: {}", line6);
        let line7 = s.get_line(7);
        assert!(line7.contains("简体中文"), "line 7 should have menu, got: {}", line7);
    }

    #[test]
    fn smoke_crossterm_colored_cjk() {
        let s = crossterm_render(80, 10, |buf| {
            queue_cmd!(buf,
                crossterm::style::SetForegroundColor(crossterm::style::Color::Rgb { r: 255, g: 107, b: 157 }),
                crossterm::style::SetBackgroundColor(crossterm::style::Color::Rgb { r: 18, g: 18, b: 18 }),
                crossterm::cursor::MoveTo(30, 6),
                crossterm::style::Print("选择语言"),
                crossterm::style::ResetColor,
                crossterm::cursor::MoveTo(32, 7),
                crossterm::style::Print("简体中文"),
            );
        });
        let text = s.get_text();
        assert!(text.contains("选择语言"), "should contain colored title, got:\n{}", text);
        assert!(text.contains("简体中文"), "should contain menu, got:\n{}", text);
        assert!(!text.contains(';'), "no SGR leak, got: {}", text);
        assert!(
            !text.contains("38;2"),
            "no raw SGR fragment, got: {}",
            text
        );
        assert!(
            !text.contains("48;2"),
            "no raw SGR fragment, got: {}",
            text
        );

        let rd = s.get_render_data();
        for c in &rd.grid[6][30..34] {
            assert_eq!(c.attrs.fg, ColorKind::Rgb(255, 107, 157));
            assert_eq!(c.attrs.bg, ColorKind::Rgb(18, 18, 18));
        }
        for c in &rd.grid[7][32..36] {
            assert_eq!(c.attrs.fg, ColorKind::Default);
            assert_eq!(c.attrs.bg, ColorKind::Default);
        }
    }

    #[test]
    fn smoke_crossterm_bold_italic_underline() {
        let s = crossterm_render(40, 2, |buf| {
            queue_cmd!(buf,
                crossterm::style::SetAttribute(crossterm::style::Attribute::Bold),
                crossterm::style::SetAttribute(crossterm::style::Attribute::Italic),
                crossterm::style::SetAttribute(crossterm::style::Attribute::Underlined),
                crossterm::style::Print("styled"),
                crossterm::style::SetAttribute(crossterm::style::Attribute::Reset),
                crossterm::style::Print(" plain"),
            );
        });
        let text = s.get_text();
        assert_eq!(text, "styled plain");

        let rd = s.get_render_data();
        assert!(rd.grid[0][0].attrs.bold);
        assert!(rd.grid[0][0].attrs.italic);
        assert!(rd.grid[0][0].attrs.underline);
        assert!(!rd.grid[0][6].attrs.bold);
        assert!(!rd.grid[0][6].attrs.italic);
        assert!(!rd.grid[0][6].attrs.underline);
    }

    #[test]
    fn smoke_crossterm_ansi_8bit_color() {
        let s = crossterm_render(40, 2, |buf| {
            queue_cmd!(buf,
                crossterm::style::SetForegroundColor(crossterm::style::Color::AnsiValue(196)),
                crossterm::style::Print("X"),
                crossterm::style::ResetColor,
            );
        });
        let rd = s.get_render_data();
        assert_eq!(rd.grid[0][0].attrs.fg, ColorKind::Index(196));
    }

    #[test]
    fn smoke_crossterm_clear_screen() {
        let s = crossterm_render(40, 5, |buf| {
            queue_cmd!(buf,
                crossterm::style::Print("should be cleared"),
                crossterm::cursor::MoveTo(0, 0),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
                crossterm::style::Print("after clear"),
            );
        });
        let text = s.get_text();
        assert_eq!(text, "after clear");
    }

    #[test]
    fn smoke_crossterm_clear_to_line_end() {
        let s = crossterm_render(20, 2, |buf| {
            queue_cmd!(buf,
                crossterm::style::Print("ABCDEFGHIJ"),
                crossterm::cursor::MoveTo(3, 0),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine),
                crossterm::style::Print("X"),
            );
        });
        let line = s.get_line(0);
        assert_eq!(&line[..4], "ABCX");
    }

    #[test]
    fn smoke_crossterm_scroll_on_overflow() {
        let s = crossterm_render(20, 3, |buf| {
            for i in 0..5 {
                queue_cmd!(buf,
                    crossterm::style::Print(format!("line{}", i)),
                    crossterm::style::Print("\r\n"),
                );
            }
        });
        assert_eq!(s.get_line(0), "line3");
        assert_eq!(s.get_line(1), "line4");
    }

    #[test]
    fn smoke_crossterm_mixed_cjk_ascii() {
        let s = crossterm_render(40, 3, |buf| {
            queue_cmd!(buf,
                crossterm::cursor::MoveTo(0, 0),
                crossterm::style::Print("Hello 简体 World"),
            );
        });
        assert_eq!(s.get_line(0), "Hello 简体 World");
    }

    #[test]
    fn smoke_crossterm_cjk_cursor_advance() {
        let s = crossterm_render(20, 3, |buf| {
            queue_cmd!(buf,
                crossterm::style::Print("简体"),
                crossterm::style::Print("X"),
            );
        });
        let text = s.get_text();
        assert!(text.ends_with("简体X"), "cursor should be at col 4 after 2 wide chars, got: {}", text);
    }

    #[test]
    fn smoke_crossterm_full_dialog_frame() {
        let s = crossterm_render(80, 24, |buf| {
            queue_cmd!(buf,
                crossterm::style::SetForegroundColor(crossterm::style::Color::Rgb { r: 255, g: 107, b: 157 }),
                crossterm::style::SetBackgroundColor(crossterm::style::Color::Rgb { r: 18, g: 18, b: 18 }),
                crossterm::cursor::MoveTo(30, 6),
                crossterm::style::Print("选择语言"),
                crossterm::style::ResetColor,
                crossterm::style::SetBackgroundColor(crossterm::style::Color::Rgb { r: 18, g: 18, b: 18 }),
                crossterm::cursor::MoveTo(30, 8),
                crossterm::style::Print("┌──────────────┐"),
                crossterm::cursor::MoveTo(30, 9),
                crossterm::style::Print("│  简体中文    │"),
                crossterm::cursor::MoveTo(30, 10),
                crossterm::style::Print("│  English     │"),
                crossterm::cursor::MoveTo(30, 11),
                crossterm::style::Print("└──────────────┘"),
                crossterm::style::ResetColor,
            );
        });
        let text = s.get_text();
        assert!(text.contains("选择语言"), "title present, got:\n{}", text);
        assert!(text.contains("简体中文"), "CJK menu present, got:\n{}", text);
        assert!(text.contains("English"), "ASCII menu present, got:\n{}", text);
        assert!(text.contains("┌"), "box corner present, got:\n{}", text);
        assert!(text.contains("│"), "box side present, got:\n{}", text);
        assert!(text.contains("└"), "box corner present, got:\n{}", text);
        assert!(!text.contains("38;2"), "no SGR fragment leak, got:\n{}", text);
        assert!(!text.contains("48;2"), "no SGR fragment leak, got:\n{}", text);
        assert!(!text.contains(';'), "no SGR number leak, got:\n{}", text);
    }

    #[test]
    fn smoke_crossterm_rapid_color_changes() {
        let s = crossterm_render(80, 5, |buf| {
            let colors = [
                (255, 0, 0),
                (0, 255, 0),
                (0, 0, 255),
                (255, 255, 0),
                (255, 0, 255),
            ];
            for (i, (r, g, b)) in colors.iter().enumerate() {
                queue_cmd!(buf,
                    crossterm::cursor::MoveTo(i as u16 * 3, 0),
                    crossterm::style::SetForegroundColor(crossterm::style::Color::Rgb { r: *r, g: *g, b: *b }),
                    crossterm::style::Print("XYZ"),
                );
            }
            queue_cmd!(buf, crossterm::style::ResetColor);
        });
        let text = s.get_text();
        assert!(
            !text.contains(';'),
            "no SGR parameter leak with rapid color changes, got: {}",
            text
        );

        let rd = s.get_render_data();
        assert_eq!(rd.grid[0][0].attrs.fg, ColorKind::Rgb(255, 0, 0));
        assert_eq!(rd.grid[0][3].attrs.fg, ColorKind::Rgb(0, 255, 0));
        assert_eq!(rd.grid[0][6].attrs.fg, ColorKind::Rgb(0, 0, 255));
        assert_eq!(rd.grid[0][9].attrs.fg, ColorKind::Rgb(255, 255, 0));
        assert_eq!(rd.grid[0][12].attrs.fg, ColorKind::Rgb(255, 0, 255));
    }

    #[test]
    fn smoke_crossterm_chunked_feed_1byte() {
        let s = crossterm_render_chunked(40, 5, 1, |buf| {
            queue_cmd!(buf,
                crossterm::style::SetForegroundColor(crossterm::style::Color::Rgb { r: 255, g: 107, b: 157 }),
                crossterm::style::SetBackgroundColor(crossterm::style::Color::Rgb { r: 18, g: 18, b: 18 }),
                crossterm::cursor::MoveTo(0, 0),
                crossterm::style::Print("简体中文"),
                crossterm::style::ResetColor,
            );
        });
        let text = s.get_text();
        assert_eq!(text, "简体中文", "1-byte chunked feed should parse correctly");
        assert!(!text.contains(';'), "no SGR leak with 1-byte chunks, got: {}", text);
    }

    #[test]
    fn smoke_crossterm_chunked_feed_3bytes() {
        let s = crossterm_render_chunked(40, 5, 3, |buf| {
            queue_cmd!(buf,
                crossterm::style::SetForegroundColor(crossterm::style::Color::Rgb { r: 255, g: 107, b: 157 }),
                crossterm::cursor::MoveTo(0, 0),
                crossterm::style::Print("简体中文"),
                crossterm::style::ResetColor,
            );
        });
        let text = s.get_text();
        assert_eq!(text, "简体中文", "3-byte chunked feed should parse correctly");
    }

    #[test]
    fn smoke_crossterm_chunked_feed_mid_escape() {
        let buf = {
            let mut b = Vec::new();
            queue_cmd!(b,
                crossterm::style::SetForegroundColor(crossterm::style::Color::Rgb { r: 255, g: 107, b: 157 }),
                crossterm::style::Print("简"),
            );
            b
        };
        let esc_start = buf.windows(2).position(|w| w == b"\x1b[").unwrap();
        let split_point = esc_start + 5;

        let mut s = Vt100Screen::new(40, 5);
        s.process(&buf[..split_point]);
        s.process(&buf[split_point..]);

        let text = s.get_text();
        assert_eq!(text, "简", "split mid-escape should still parse correctly");
        assert!(!text.contains(';'), "no SGR leak, got: {}", text);
    }

    #[test]
    fn smoke_crossterm_many_lines_scrolling() {
        let s = crossterm_render(40, 5, |buf| {
            for i in 0..20 {
                queue_cmd!(buf,
                    crossterm::style::Print(format!("Line {:02}", i)),
                    crossterm::style::Print("\r\n"),
                );
            }
        });
        assert_eq!(s.get_line(0), "Line 16");
        assert_eq!(s.get_line(1), "Line 17");
        assert_eq!(s.get_line(2), "Line 18");
        assert_eq!(s.get_line(3), "Line 19");

        let sb = s.get_scrollback();
        assert!(sb.contains("Line 00"), "scrollback should have old content");
    }

    #[test]
    fn smoke_crossterm_save_restore_cursor() {
        let s = crossterm_render(40, 5, |buf| {
            queue_cmd!(buf,
                crossterm::cursor::MoveTo(5, 2),
                crossterm::cursor::SavePosition,
                crossterm::style::Print("A"),
                crossterm::cursor::MoveTo(0, 0),
                crossterm::style::Print("B"),
                crossterm::cursor::RestorePosition,
            );
        });
        assert_eq!(s.get_line(0), "B");
        let line2 = s.get_line(2);
        assert!(
            line2.contains("A"),
            "line 2 should contain A at restored position, got: '{}'",
            line2
        );
        assert_eq!(s.cursor_col, 5);
        assert_eq!(s.cursor_row, 2);
    }

    #[test]
    fn smoke_crossterm_tab_stops() {
        let s = crossterm_render(40, 2, |buf| {
            queue_cmd!(buf,
                crossterm::style::Print("A"),
                crossterm::style::Print("\t"),
                crossterm::style::Print("B"),
            );
        });
        let line = s.get_line(0);
        assert_eq!(line.as_bytes()[0], b'A');
        assert_eq!(line.as_bytes()[8], b'B');
    }

    #[test]
    fn smoke_crossterm_backspace() {
        let s = crossterm_render(40, 2, |buf| {
            queue_cmd!(buf,
                crossterm::style::Print("abc"),
                crossterm::cursor::MoveLeft(1),
                crossterm::style::Print("X"),
            );
        });
        assert_eq!(s.get_line(0), "abX");
    }

    #[test]
    fn smoke_crossterm_hide_cursor_and_size() {
        let s = crossterm_render(60, 15, |buf| {
            queue_cmd!(buf,
                crossterm::cursor::Hide,
                crossterm::cursor::MoveTo(29, 7),
                crossterm::style::Print("centered"),
                crossterm::cursor::Show,
            );
        });
        let line = s.get_line(7);
        assert!(line.contains("centered"), "got: {}", line);
    }

    #[test]
    fn smoke_crossterm_find_text_after_render() {
        let s = crossterm_render(80, 24, |buf| {
            queue_cmd!(buf,
                crossterm::cursor::MoveTo(30, 6),
                crossterm::style::Print("选择语言"),
                crossterm::cursor::MoveTo(32, 7),
                crossterm::style::Print("简体中文"),
            );
        });
        let hits = s.find_text("选择");
        assert!(!hits.is_empty(), "find_text should locate CJK substring");
        assert_eq!(hits[0].0, 6);

        let hits2 = s.find_text("简体");
        assert!(!hits2.is_empty(), "find_text should locate CJK substring");
        assert_eq!(hits2[0].0, 7);
    }

    #[test]
    fn smoke_no_control_leaks_comprehensive() {
        let s = crossterm_render(80, 24, |buf| {
            queue_cmd!(buf,
                crossterm::style::SetForegroundColor(crossterm::style::Color::Rgb { r: 255, g: 107, b: 157 }),
                crossterm::style::SetBackgroundColor(crossterm::style::Color::Rgb { r: 18, g: 18, b: 18 }),
                crossterm::cursor::MoveTo(10, 5),
                crossterm::style::SetAttribute(crossterm::style::Attribute::Bold),
                crossterm::style::Print("Hello"),
                crossterm::style::SetAttribute(crossterm::style::Attribute::Italic),
                crossterm::style::Print(" World"),
                crossterm::style::SetAttribute(crossterm::style::Attribute::Underlined),
                crossterm::style::Print(" 简体"),
                crossterm::style::SetAttribute(crossterm::style::Attribute::Reset),
                crossterm::cursor::MoveTo(10, 7),
                crossterm::style::SetForegroundColor(crossterm::style::Color::AnsiValue(196)),
                crossterm::style::Print("colored text"),
                crossterm::style::ResetColor,
            );
        });
        let text = s.get_text();
        for forbidden in &["38;", "48;", "m\x1b", ";\x1b", "0m\x1b", "107;"] {
            assert!(
                !text.contains(forbidden),
                "text should not contain '{}', got: {}",
                forbidden,
                text
            );
        }
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert!(text.contains("简体"));
        assert!(text.contains("colored text"));
    }

    // ── New: CSI G (CHA), CSI d (VPA) via crossterm ───────

    #[test]
    fn smoke_crossterm_move_to_column() {
        let s = crossterm_render(40, 5, |buf| {
            queue_cmd!(buf,
                crossterm::cursor::MoveToColumn(10),
                crossterm::style::Print("A"),
                crossterm::cursor::MoveToColumn(20),
                crossterm::style::Print("B"),
                crossterm::cursor::MoveToColumn(0),
                crossterm::style::Print("X"),
            );
        });
        assert_eq!(s.get_line(0), "X         A         B");
    }

    #[test]
    fn smoke_crossterm_move_to_row() {
        let s = crossterm_render(40, 5, |buf| {
            queue_cmd!(buf,
                crossterm::cursor::MoveTo(0, 0),
                crossterm::style::Print("R0"),
                crossterm::cursor::MoveTo(0, 4),
                crossterm::style::Print("R4"),
                crossterm::cursor::MoveTo(0, 2),
                crossterm::style::Print("R2"),
            );
        });
        assert_eq!(s.get_line(0), "R0");
        assert_eq!(s.get_line(2), "R2");
        assert_eq!(s.get_line(4), "R4");
    }

    // ── New: CSI 1 J / CSI 1 K via crossterm ────────────

    #[test]
    fn smoke_crossterm_clear_from_cursor_up() {
        let s = crossterm_render(40, 5, |buf| {
            queue_cmd!(buf,
                crossterm::cursor::MoveTo(0, 0),
                crossterm::style::Print("line0"),
                crossterm::cursor::MoveTo(0, 1),
                crossterm::style::Print("line1"),
                crossterm::cursor::MoveTo(0, 2),
                crossterm::style::Print("line2"),
                crossterm::cursor::MoveTo(0, 3),
                crossterm::style::Print("line3"),
                crossterm::cursor::MoveTo(3, 1),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorUp),
            );
        });
        assert_eq!(s.get_line(0), "");
        assert_eq!(s.get_line(1), "    1");
        assert!(s.get_line(2).contains("line2"));
        assert!(s.get_line(3).contains("line3"));
    }

    #[test]
    fn smoke_crossterm_scroll_down() {
        let s = crossterm_render(40, 5, |buf| {
            for i in 0..5 {
                queue_cmd!(buf,
                    crossterm::cursor::MoveTo(0, i as u16),
                    crossterm::style::Print(format!("row{}", i)),
                );
            }
            queue_cmd!(buf,
                crossterm::terminal::ScrollDown(2),
            );
        });
        assert_eq!(s.get_line(0), "");
        assert_eq!(s.get_line(1), "");
        assert_eq!(s.get_line(2), "row0");
        assert_eq!(s.get_line(3), "row1");
    }

    // ── New: wide char edge cases ────────────────────────

    #[test]
    fn test_cjk_wrap_at_right_edge() {
        let mut s = Vt100Screen::new(6, 5);
        s.process(b"ABCDE");
        assert_eq!(s.cursor_col, 5);
        s.process("简".as_bytes());
        assert_eq!(s.cursor_col, 2, "wide char should wrap to next line col=0 then advance by 2");
        assert_eq!(s.get_line(0), "ABCDE");
        assert!(s.get_line(1).starts_with("简"), "line 1 should start with wide char, got: {:?}", s.get_line(1));
    }

    #[test]
    fn test_cjk_wrap_at_right_edge_exact() {
        let mut s = Vt100Screen::new(4, 5);
        s.process(b"ABC");
        s.process("简".as_bytes());
        let text = s.get_text();
        assert!(text.contains("ABC"), "ABC should be intact, got: {}", text);
        assert!(text.contains("简"), "wide char should wrap, got: {}", text);
        assert_eq!(s.get_line(0), "ABC");
        assert!(s.get_line(1).starts_with("简"), "got: {:?}", s.get_line(1));
    }

    #[test]
    fn test_resize_shrink_with_wide_char() {
        let mut s = Vt100Screen::new(10, 3);
        s.process("AB简体CD".as_bytes());
        assert!(s.get_text().contains("简体"));
        s.resize(3, 3);
        let text = s.get_text();
        assert!(!text.contains('\u{0}'), "no null chars after resize, got: {:?}", text);
    }

    // ── New: stress tests ────────────────────────────────

    #[test]
    fn test_massive_output_no_panic() {
        let mut s = Vt100Screen::new(80, 24);
        let mut buf = Vec::with_capacity(512 * 1024);
        for i in 0..10000 {
            buf.extend(format!("\x1b[{};{}Hline{:04}\r\n", (i % 24) + 1, (i % 40) + 1, i).as_bytes());
            if i % 10 == 0 {
                buf.extend(b"\x1b[38;2;255;0;0m");
            }
            if i % 20 == 0 {
                buf.extend(b"\x1b[0m");
            }
        }
        buf.extend("DONE".as_bytes());
        s.process(&buf);
        let text = s.get_text();
        assert!(text.contains("DONE"), "should contain DONE after massive output");
        assert!(!text.contains("38;2"), "no SGR leak after massive output");
    }

    #[test]
    fn test_rapid_resize_cycles() {
        let mut s = Vt100Screen::new(80, 24);
        s.process(b"Hello World");
        for cols in [10, 120, 40, 200, 80, 5, 80] {
            for rows in [5, 50, 24, 1, 100, 24] {
                s.resize(cols, rows);
            }
        }
        s.resize(80, 24);
        s.process(b" AFTER");
        let text = s.get_text();
        assert!(text.contains("AFTER"), "should still work after resize cycles");
        assert!(!text.contains('\u{0}'), "no null chars after resize cycles");
    }

    #[test]
    fn test_single_column_screen() {
        let mut s = Vt100Screen::new(1, 3);
        s.process(b"A");
        assert_eq!(s.get_line(0), "A");
        s.process("简".as_bytes());
        let text = s.get_text();
        assert!(!text.contains('\u{0}'), "no null on 1-col screen");
    }

    #[test]
    fn test_cursor_col_overflow_after_print() {
        let mut s = Vt100Screen::new(5, 3);
        s.process(b"ABCDE");
        assert!(s.cursor_col >= 5, "cursor_col should overflow past cols");
        s.process(b"X");
        assert_eq!(s.get_line(1), "X", "next print after overflow should wrap");
    }
}
