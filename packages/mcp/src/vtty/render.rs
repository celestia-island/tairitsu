use ab_glyph::{Font, FontVec, GlyphId, PxScale, ScaleFont, point};
use image::{ImageBuffer, Rgba};
use std::io::Cursor;

use super::screen::{Cell, ColorKind, RenderData};

// ─── Color Schemes ──────────────────────────────────────────────────────────

pub struct ColorScheme {
    #[allow(dead_code)]
    pub name: &'static str,
    pub fg: [u8; 3],
    pub bg: [u8; 3],
    pub cursor: [u8; 3],
    pub palette: [[u8; 3]; 16],
}

pub const SOLARIZED_DARK: ColorScheme = ColorScheme {
    name: "solarized-dark",
    fg: [147, 161, 161],
    bg: [0, 43, 54],
    cursor: [220, 50, 47],
    palette: [
        [7, 54, 66],     // 0  Black
        [220, 50, 47],   // 1  Red
        [133, 153, 0],   // 2  Green
        [181, 137, 0],   // 3  Yellow
        [38, 139, 210],  // 4  Blue
        [211, 54, 130],  // 5  Magenta
        [42, 161, 152],  // 6  Cyan
        [238, 232, 213], // 7  White
        [0, 43, 54],     // 8  Bright Black
        [203, 75, 22],   // 9  Bright Red
        [88, 110, 117],  // 10 Bright Green
        [101, 123, 131], // 11 Bright Yellow
        [131, 148, 150], // 12 Bright Blue
        [108, 113, 196], // 13 Bright Magenta
        [147, 161, 161], // 14 Bright Cyan
        [253, 246, 227], // 15 Bright White
    ],
};

pub const SOLARIZED_LIGHT: ColorScheme = ColorScheme {
    name: "solarized-light",
    fg: [101, 123, 131],
    bg: [253, 246, 227],
    cursor: [220, 50, 47],
    palette: [
        [7, 54, 66],
        [220, 50, 47],
        [133, 153, 0],
        [181, 137, 0],
        [38, 139, 210],
        [211, 54, 130],
        [42, 161, 152],
        [238, 232, 213],
        [0, 43, 54],
        [203, 75, 22],
        [88, 110, 117],
        [101, 123, 131],
        [131, 148, 150],
        [108, 113, 196],
        [147, 161, 161],
        [253, 246, 227],
    ],
};

pub const ONE_HALF_DARK: ColorScheme = ColorScheme {
    name: "one-half-dark",
    fg: [220, 223, 228],
    bg: [40, 44, 52],
    cursor: [198, 120, 221],
    palette: [
        [40, 44, 52],
        [224, 108, 117],
        [152, 195, 121],
        [229, 192, 123],
        [97, 175, 239],
        [198, 120, 221],
        [86, 182, 194],
        [220, 223, 228],
        [92, 99, 112],
        [224, 108, 117],
        [152, 195, 121],
        [229, 192, 123],
        [97, 175, 239],
        [198, 120, 221],
        [86, 182, 194],
        [220, 223, 228],
    ],
};

pub const ONE_HALF_LIGHT: ColorScheme = ColorScheme {
    name: "one-half-light",
    fg: [56, 58, 66],
    bg: [250, 250, 250],
    cursor: [152, 52, 115],
    palette: [
        [56, 58, 66],
        [228, 86, 73],
        [80, 161, 79],
        [204, 143, 44],
        [0, 82, 164],
        [152, 52, 115],
        [19, 135, 141],
        [250, 250, 250],
        [92, 99, 112],
        [228, 86, 73],
        [80, 161, 79],
        [204, 143, 44],
        [0, 82, 164],
        [152, 52, 115],
        [19, 135, 141],
        [220, 223, 228],
    ],
};

pub const IBM_5153: ColorScheme = ColorScheme {
    name: "ibm-5153",
    fg: [51, 255, 0],
    bg: [0, 0, 0],
    cursor: [51, 255, 0],
    palette: [
        [0, 0, 0],
        [170, 0, 0],
        [0, 170, 0],
        [170, 85, 0],
        [0, 0, 170],
        [170, 0, 170],
        [0, 170, 170],
        [170, 170, 170],
        [85, 85, 85],
        [255, 85, 85],
        [85, 255, 85],
        [255, 255, 85],
        [85, 85, 255],
        [255, 85, 255],
        [85, 255, 255],
        [255, 255, 255],
    ],
};

pub fn get_scheme(name: &str) -> &'static ColorScheme {
    match name {
        "solarized-dark" => &SOLARIZED_DARK,
        "solarized-light" => &SOLARIZED_LIGHT,
        "one-half-dark" => &ONE_HALF_DARK,
        "one-half-light" => &ONE_HALF_LIGHT,
        "ibm-5153" => &IBM_5153,
        _ => &SOLARIZED_DARK,
    }
}

#[allow(dead_code)]
pub fn scheme_names() -> &'static [&'static str] {
    &[
        "solarized-dark",
        "solarized-light",
        "one-half-dark",
        "one-half-light",
        "ibm-5153",
    ]
}

// ─── Font Loading ───────────────────────────────────────────────────────────

struct Fonts {
    mono: FontVec,
    cjk: Option<FontVec>,
}

static MONO_CANDIDATES: &[&str] = &[
    "FiraCodeNerdFontMono-Regular.ttf",
    "FiraCodeNerdFont-Regular.ttf",
    "JetBrainsMonoNerdFontMono-Regular.ttf",
    "DejaVuSansMono.ttf",
    "NotoSansMono-Regular.ttf",
    "LiberationMono-Regular.ttf",
    "UbuntuMono-Regular.ttf",
    "Consolas.ttf",
];

static CJK_CANDIDATES: &[&str] = &[
    "NotoSansCJKsc-Regular.otf",
    "NotoSansCJK-Regular.ttc",
    "NotoSansSC-Regular.otf",
    "NotoSansSC-Regular.ttf",
    "WenQuanYiMicroHei.ttf",
    "wqy-microhei.ttc",
    "SimHei.ttf",
    "simhei.ttf",
    "SourceHanSansSC-Regular.otf",
    "PingFang.ttc",
    "HiraginoSansGB.ttc",
    "DroidSansFallbackFull.ttf",
];

#[cfg(unix)]
fn font_search_dirs() -> Vec<std::path::PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    let mut dirs = vec![
        std::path::PathBuf::from("/usr/share/fonts"),
        std::path::PathBuf::from("/usr/local/share/fonts"),
    ];
    if !home.is_empty() {
        dirs.push(std::path::PathBuf::from(format!(
            "{}/.local/share/fonts",
            home
        )));
        dirs.push(std::path::PathBuf::from(format!("{}/.fonts", home)));
    }
    dirs.push(std::path::PathBuf::from("/System/Library/Fonts"));
    dirs.push(std::path::PathBuf::from("/Library/Fonts"));
    dirs
}

#[cfg(windows)]
fn font_search_dirs() -> Vec<std::path::PathBuf> {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
    vec![std::path::PathBuf::from(windir).join("Fonts")]
}

fn find_font_file(candidates: &[&str]) -> Option<std::path::PathBuf> {
    let dirs = font_search_dirs();
    for dir in &dirs {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str())
                        && candidates.contains(&name)
                    {
                        return Some(path);
                    }
                } else if path.is_dir()
                    && let Ok(sub) = std::fs::read_dir(&path)
                {
                    for se in sub.flatten() {
                        let sp = se.path();
                        if sp.is_file()
                            && let Some(name) = sp.file_name().and_then(|n| n.to_str())
                            && candidates.contains(&name)
                        {
                            return Some(sp);
                        }
                        if sp.is_dir()
                            && let Ok(l3) = std::fs::read_dir(&sp)
                        {
                            for l3e in l3.flatten() {
                                let l3p = l3e.path();
                                if l3p.is_file()
                                    && let Some(name) = l3p.file_name().and_then(|n| n.to_str())
                                    && candidates.contains(&name)
                                {
                                    return Some(l3p);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn load_font_from(path: &std::path::Path) -> Result<FontVec, String> {
    let data = std::fs::read(path).map_err(|e| format!("read {}: {}", path.display(), e))?;
    FontVec::try_from_vec(data).map_err(|e| format!("parse font {}: {}", path.display(), e))
}

impl Fonts {
    fn load() -> Result<Self, String> {
        let mono_path = find_font_file(MONO_CANDIDATES).ok_or_else(|| {
            format!(
                "No monospace font found. Searched for: {}",
                MONO_CANDIDATES.join(", ")
            )
        })?;
        let mono = load_font_from(&mono_path)?;
        let cjk = find_font_file(CJK_CANDIDATES).and_then(|p| load_font_from(&p).ok());
        Ok(Self { mono, cjk })
    }
}

// ─── 256-Color Lookup ───────────────────────────────────────────────────────

fn index_to_rgb(index: u8, scheme: &ColorScheme) -> [u8; 3] {
    match index {
        0..=15 => scheme.palette[index as usize],
        16..=231 => {
            let i = (index - 16) as usize;
            let cube = [0u8, 95, 135, 175, 215, 255];
            [cube[i / 36], cube[(i % 36) / 6], cube[i % 6]]
        }
        232..=255 => {
            let v = 8 + (index - 232) * 10;
            [v, v, v]
        }
    }
}

fn resolve_color(color: ColorKind, scheme: &ColorScheme, is_fg: bool) -> [u8; 3] {
    match color {
        ColorKind::Default => {
            if is_fg {
                scheme.fg
            } else {
                scheme.bg
            }
        }
        ColorKind::Index(i) => index_to_rgb(i, scheme),
        ColorKind::Rgb(r, g, b) => [r, g, b],
    }
}

// ─── Rendering ──────────────────────────────────────────────────────────────

type ImgBuf = ImageBuffer<Rgba<u8>, Vec<u8>>;

struct RenderCtx<'a> {
    fonts: &'a Fonts,
    scale: PxScale,
    cell_w: f32,
    cell_h: f32,
    ascent: f32,
    padding: u32,
    scheme: &'a ColorScheme,
}

pub fn render_terminal(data: &RenderData, theme: &str) -> Result<Vec<u8>, String> {
    let scheme = get_scheme(theme);
    let fonts = Fonts::load()?;

    let font_size = 15.0;
    let scale = PxScale::from(font_size);
    let scaled = fonts.mono.as_scaled(scale);

    let glyph_m = fonts.mono.glyph_id('M');
    let cell_w = scaled.h_advance(glyph_m);
    let cell_h = scaled.height();
    let ascent = scaled.ascent();

    let padding = 8u32;
    let img_w = (cell_w * data.cols as f32).ceil() as u32 + padding * 2;
    let img_h = (cell_h * data.rows as f32).ceil() as u32 + padding * 2;

    let mut img = ImageBuffer::from_pixel(
        img_w,
        img_h,
        Rgba([scheme.bg[0], scheme.bg[1], scheme.bg[2], 255]),
    );

    let ctx = RenderCtx {
        fonts: &fonts,
        scale,
        cell_w,
        cell_h,
        ascent,
        padding,
        scheme,
    };

    let mut placements_below: Vec<(i32, &super::graphics::ImagePlacement)> = Vec::new();
    let mut placements_above: Vec<(i32, &super::graphics::ImagePlacement)> = Vec::new();

    for p in data.image_store.placements() {
        if p.z_index < 0 {
            placements_below.push((p.z_index, p));
        } else {
            placements_above.push((p.z_index, p));
        }
    }
    placements_below.sort_by_key(|(z, _)| *z);
    placements_above.sort_by_key(|(z, _)| *z);

    for (_z, placement) in &placements_below {
        composite_placement(&mut img, placement, &data.image_store, &ctx);
    }

    for (row_idx, row) in data.grid.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            let x0 = padding as f32 + col_idx as f32 * cell_w;
            let y0 = padding as f32 + row_idx as f32 * cell_h;
            render_cell(&mut img, cell, x0, y0, &ctx);
        }
    }

    for (_z, placement) in &placements_above {
        composite_placement(&mut img, placement, &data.image_store, &ctx);
    }

    render_cursor(&mut img, data.cursor_row, data.cursor_col, &ctx);

    let mut png_data = Vec::new();
    img.write_to(&mut Cursor::new(&mut png_data), image::ImageFormat::Png)
        .map_err(|e| format!("PNG encode error: {}", e))?;
    Ok(png_data)
}

fn render_cell(img: &mut ImgBuf, cell: &Cell, x0: f32, y0: f32, ctx: &RenderCtx) {
    let bg = resolve_color(cell.attrs.bg, ctx.scheme, false);

    let fg_raw = resolve_color(cell.attrs.fg, ctx.scheme, true);
    let fg = if cell.attrs.bold {
        [
            fg_raw[0].saturating_add(30),
            fg_raw[1].saturating_add(30),
            fg_raw[2].saturating_add(30),
        ]
    } else {
        fg_raw
    };

    let px0 = x0.ceil() as u32;
    let py0 = y0.ceil() as u32;
    let px1 = (x0 + ctx.cell_w).ceil() as u32;
    let py1 = (y0 + ctx.cell_h).ceil() as u32;
    let iw = img.width();
    let ih = img.height();

    for y in py0..py1.min(ih) {
        for x in px0..px1.min(iw) {
            img.put_pixel(x, y, Rgba([bg[0], bg[1], bg[2], 255]));
        }
    }

    if cell.attrs.underline {
        let uy = py1.saturating_sub(3);
        for x in px0..px1.min(iw) {
            if uy < ih {
                let p = img.get_pixel(x, uy);
                let a = 200u16;
                let r = (fg[0] as u16 * a + p[0] as u16 * (255 - a)) / 255;
                let g = (fg[1] as u16 * a + p[1] as u16 * (255 - a)) / 255;
                let b = (fg[2] as u16 * a + p[2] as u16 * (255 - a)) / 255;
                img.put_pixel(x, uy, Rgba([r as u8, g as u8, b as u8, 255]));
            }
        }
    }

    if cell.ch == ' ' || cell.ch == '\x00' {
        return;
    }

    let cw = unicode_width::UnicodeWidthChar::width(cell.ch).unwrap_or(1);
    let _actual_cell_w = ctx.cell_w * cw as f32;

    let glyph_id = ctx.fonts.mono.glyph_id(cell.ch);
    let (has_mono, gid) = if glyph_id != GlyphId(0) {
        (true, glyph_id)
    } else {
        (false, glyph_id)
    };

    if has_mono {
        let glyph = ab_glyph::Glyph {
            id: gid,
            scale: ctx.scale,
            position: point(x0, y0 + ctx.ascent),
        };
        draw_glyph_outline(&ctx.fonts.mono, glyph, img, fg);
    } else if let Some(ref cjk) = ctx.fonts.cjk {
        let cjk_id = cjk.glyph_id(cell.ch);
        if cjk_id != GlyphId(0) {
            let cjk_scale = PxScale::from(font_size_for_double(cw, ctx.cell_w, ctx.cell_h));
            let cjk_ascent = cjk.as_scaled(cjk_scale).ascent();
            let glyph = ab_glyph::Glyph {
                id: cjk_id,
                scale: cjk_scale,
                position: point(x0, y0 + cjk_ascent),
            };
            draw_glyph_outline(cjk, glyph, img, fg);
        }
    }
}

fn font_size_for_double(char_width: usize, cell_w: f32, cell_h: f32) -> f32 {
    let target_w = cell_w * char_width as f32;
    let ratio = target_w / cell_h;
    15.0 * ratio.clamp(1.0, 2.0)
}

fn draw_glyph_outline<F: ab_glyph::Font>(
    font: &F,
    glyph: ab_glyph::Glyph,
    img: &mut ImgBuf,
    fg: [u8; 3],
) {
    if let Some(outlined) = font.outline_glyph(glyph) {
        let bounds = outlined.px_bounds();
        let iw = img.width() as i32;
        let ih = img.height() as i32;
        let bx = bounds.min.x as i32;
        let by = bounds.min.y as i32;

        outlined.draw(|gx, gy, coverage| {
            let px = bx + gx as i32;
            let py = by + gy as i32;
            if px >= 0 && py >= 0 && px < iw && py < ih {
                let alpha = (coverage * 255.0) as u16;
                let pixel = img.get_pixel(px as u32, py as u32);
                let inv = 255 - alpha;
                let r = (fg[0] as u16 * alpha + pixel[0] as u16 * inv) / 255;
                let g = (fg[1] as u16 * alpha + pixel[1] as u16 * inv) / 255;
                let b = (fg[2] as u16 * alpha + pixel[2] as u16 * inv) / 255;
                img.put_pixel(px as u32, py as u32, Rgba([r as u8, g as u8, b as u8, 255]));
            }
        });
    }
}

fn render_cursor(img: &mut ImgBuf, cursor_row: usize, cursor_col: usize, ctx: &RenderCtx) {
    let x0 = ctx.padding as f32 + cursor_col as f32 * ctx.cell_w;
    let y0 = ctx.padding as f32 + cursor_row as f32 * ctx.cell_h;
    let px0 = x0.ceil() as u32;
    let py0 = y0.ceil() as u32;
    let px1 = (x0 + ctx.cell_w).ceil() as u32;
    let py1 = (y0 + ctx.cell_h).ceil() as u32;
    let iw = img.width();
    let ih = img.height();

    let cc = ctx.scheme.cursor;
    for y in py0..py1.min(ih) {
        for x in px0..px1.min(iw) {
            let p = img.get_pixel(x, y);
            let a: u16 = 160;
            let inv = 255 - a;
            let r = (cc[0] as u16 * a + p[0] as u16 * inv) / 255;
            let g = (cc[1] as u16 * a + p[1] as u16 * inv) / 255;
            let b = (cc[2] as u16 * a + p[2] as u16 * inv) / 255;
            img.put_pixel(x, y, Rgba([r as u8, g as u8, b as u8, 255]));
        }
    }
}

fn composite_placement(
    img: &mut ImgBuf,
    placement: &super::graphics::ImagePlacement,
    store: &super::graphics::InlineImageStore,
    ctx: &RenderCtx,
) {
    let source = match store.get_image(placement.image_id) {
        Some(img) => img,
        None => return,
    };

    let target_w = if placement.width_cols > 0 {
        (ctx.cell_w * placement.width_cols as f32).ceil() as u32
    } else {
        source.rgba.width()
    };
    let target_h = if placement.height_rows > 0 {
        (ctx.cell_h * placement.height_rows as f32).ceil() as u32
    } else {
        source.rgba.height()
    };

    let x0 = (ctx.padding as f32 + placement.col as f32 * ctx.cell_w).ceil() as u32;
    let y0 = (ctx.padding as f32 + placement.row as f32 * ctx.cell_h).ceil() as u32;

    let src_w = source.rgba.width();
    let src_h = source.rgba.height();

    let iw = img.width();
    let ih = img.height();

    for dy in 0..target_h {
        for dx in 0..target_w {
            let px = x0 + dx;
            let py = y0 + dy;
            if px >= iw || py >= ih {
                continue;
            }
            let sx = (dx as u64 * src_w as u64 / target_w.max(1) as u64) as u32;
            let sy = (dy as u64 * src_h as u64 / target_h.max(1) as u64) as u32;
            if sx >= src_w || sy >= src_h {
                continue;
            }
            let src_pixel = source.rgba.get_pixel(sx, sy);
            let alpha = src_pixel[3] as u16;
            if alpha == 0 {
                continue;
            }
            let dst_pixel = img.get_pixel(px, py);
            if alpha >= 255 {
                img.put_pixel(px, py, *src_pixel);
            } else {
                let inv = 255 - alpha;
                let r = (src_pixel[0] as u16 * alpha + dst_pixel[0] as u16 * inv) / 255;
                let g = (src_pixel[1] as u16 * alpha + dst_pixel[1] as u16 * inv) / 255;
                let b = (src_pixel[2] as u16 * alpha + dst_pixel[2] as u16 * inv) / 255;
                img.put_pixel(px, py, Rgba([r as u8, g as u8, b as u8, 255]));
            }
        }
    }
}

pub fn encode_base64(data: &[u8]) -> String {
    use base64::{Engine, engine::general_purpose::STANDARD};
    STANDARD.encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic_text() {
        let mut screen = super::super::screen::Vt100Screen::new(40, 10);
        screen.process(b"Hello World");
        let data = screen.get_render_data();
        let png = render_terminal(&data, "solarized-dark").unwrap();
        assert!(!png.is_empty());
        assert!(png.len() > 100);
        // PNG magic bytes
        assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    fn test_render_colored_text() {
        let mut screen = super::super::screen::Vt100Screen::new(40, 5);
        screen.process(b"\x1b[31mRed\x1b[0m \x1b[32mGreen\x1b[0m");
        let data = screen.get_render_data();
        let png = render_terminal(&data, "solarized-dark").unwrap();
        assert!(!png.is_empty());
    }

    #[test]
    fn test_render_all_themes() {
        let mut screen = super::super::screen::Vt100Screen::new(20, 5);
        screen.process(b"Test output");
        let data = screen.get_render_data();
        for name in scheme_names() {
            let png = render_terminal(&data, name).unwrap();
            assert!(!png.is_empty(), "Theme '{}' produced empty PNG", name);
        }
    }

    #[test]
    fn test_index_to_rgb() {
        let s = get_scheme("solarized-dark");
        assert_eq!(index_to_rgb(0, s), [7, 54, 66]);
        assert_eq!(index_to_rgb(1, s), [220, 50, 47]);
        // 256-color cube: index 196 = r=5,g=0,b=0 -> [255, 0, 0]
        let c = index_to_rgb(196, s);
        assert_eq!(c, [255, 0, 0]);
        // Grayscale: index 232 = 8
        let g = index_to_rgb(232, s);
        assert_eq!(g, [8, 8, 8]);
        // Grayscale: index 255 = 8 + 23*10 = 238
        let g2 = index_to_rgb(255, s);
        assert_eq!(g2, [238, 238, 238]);
    }

    #[test]
    fn test_resolve_color() {
        let s = get_scheme("solarized-dark");
        assert_eq!(resolve_color(ColorKind::Default, s, true), s.fg);
        assert_eq!(resolve_color(ColorKind::Default, s, false), s.bg);
        assert_eq!(
            resolve_color(ColorKind::Rgb(100, 200, 50), s, true),
            [100, 200, 50]
        );
    }
}
