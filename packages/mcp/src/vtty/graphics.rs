use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::collections::HashMap;

use image::{ImageBuffer, Rgba};

type ImgBuf = ImageBuffer<Rgba<u8>, Vec<u8>>;

#[derive(Clone)]
pub struct InlineImage {
    pub rgba: ImgBuf,
}

#[derive(Clone)]
pub struct ImagePlacement {
    pub image_id: u32,
    pub row: usize,
    pub col: usize,
    pub width_cols: usize,
    pub height_rows: usize,
    pub z_index: i32,
}

#[derive(Default, Clone)]
pub struct InlineImageStore {
    images: HashMap<u32, InlineImage>,
    placements: Vec<ImagePlacement>,
    next_auto_id: u32,
}

impl InlineImageStore {
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
            placements: Vec::new(),
            next_auto_id: 1,
        }
    }

    pub fn store(&mut self, id: u32, img: InlineImage) {
        if id != 0 {
            self.images.insert(id, img);
        } else {
            let auto_id = self.next_auto_id;
            self.next_auto_id += 1;
            self.images.insert(auto_id, img);
        }
    }

    pub fn remove(&mut self, id: u32) {
        self.images.remove(&id);
        self.placements.retain(|p| p.image_id != id);
    }

    pub fn add_placement(&mut self, placement: ImagePlacement) {
        if let Some(existing) = self.placements.iter_mut().find(|p| {
            p.image_id == placement.image_id && p.row == placement.row && p.col == placement.col
        }) {
            *existing = placement;
        } else {
            self.placements.push(placement);
        }
    }

    pub fn placements(&self) -> &[ImagePlacement] {
        &self.placements
    }

    pub fn get_image(&self, id: u32) -> Option<&InlineImage> {
        self.images.get(&id)
    }

    pub fn clear(&mut self) {
        self.images.clear();
        self.placements.clear();
    }
}

pub struct KittyGraphicsState {
    control: String,
    payload: Vec<u8>,
    image_id: u32,
    chunked: bool,
    first_chunk: bool,
}

impl KittyGraphicsState {
    pub fn new() -> Self {
        Self {
            control: String::new(),
            payload: Vec::new(),
            image_id: 0,
            chunked: false,
            first_chunk: true,
        }
    }

    pub fn reset(&mut self) {
        self.control.clear();
        self.payload.clear();
        self.image_id = 0;
        self.chunked = false;
        self.first_chunk = true;
    }
}

fn parse_kitty_control(control: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in control.split(',') {
        if let Some((k, v)) = pair.split_once('=') {
            map.insert(k.to_string(), v.to_string());
        }
    }
    map
}

pub fn process_kitty_apc(
    state: &mut KittyGraphicsState,
    control: &str,
    payload: &[u8],
    cursor_row: usize,
    cursor_col: usize,
    store: &mut InlineImageStore,
) {
    let params = parse_kitty_control(control);

    let action = params.get("a").map(|s| s.as_str()).unwrap_or("");
    let m_val = params.get("m").map(|s| s.as_str()).unwrap_or("");

    match action {
        "q" | "" | "T" | "t" => {}
        "d" => {
            if let Some(Ok(id)) = params.get("i").map(|v| v.parse::<u32>()) {
                store.remove(id);
            } else {
                store.clear();
            }
            return;
        }
        "p" => {
            if let Some(Ok(id)) = params.get("i").map(|v| v.parse::<u32>()) {
                let cols = params
                    .get("c")
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(1);
                let rows = params
                    .get("r")
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(1);
                let z = params
                    .get("z")
                    .and_then(|v| v.parse::<i32>().ok())
                    .unwrap_or(0);
                if store.get_image(id).is_some() {
                    store.add_placement(ImagePlacement {
                        image_id: id,
                        row: cursor_row,
                        col: cursor_col,
                        width_cols: cols,
                        height_rows: rows,
                        z_index: z,
                    });
                }
            }
            return;
        }
        _ => return,
    }

    if m_val == "1" {
        if state.first_chunk {
            state.control = control.to_string();
            state.payload = payload.to_vec();
            if let Some(Ok(id)) = params.get("i").map(|v| v.parse::<u32>()) {
                state.image_id = id;
            }
            state.first_chunk = false;
        } else {
            state.payload.extend_from_slice(payload);
        }
        state.chunked = true;
        return;
    }

    let (final_control, final_payload) = if state.chunked && !state.first_chunk {
        state.payload.extend_from_slice(payload);
        let c = std::mem::take(&mut state.control);
        let p = std::mem::take(&mut state.payload);
        state.reset();
        (c, p)
    } else {
        (control.to_string(), payload.to_vec())
    };

    let params = parse_kitty_control(&final_control);

    let is_transmit = matches!(action, "T" | "t" | "q" | "");

    if !is_transmit {
        return;
    }

    let decoded = match BASE64.decode(&final_payload) {
        Ok(d) => d,
        Err(_) => return,
    };

    let format = params
        .get("f")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(32);
    let transmission = params.get("t").map(|s| s.as_str()).unwrap_or("d");

    if transmission != "d" && transmission != "f" && transmission != "t" && transmission != "s" {
        return;
    }

    let image_id = params
        .get("i")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    let img_buf = decode_kitty_image(format, &decoded, &params);

    if let Some(rgba) = img_buf {
        let inline = InlineImage { rgba };

        let effective_id = if image_id != 0 {
            image_id
        } else {
            let id = store.next_auto_id;
            store.next_auto_id += 1;
            id
        };

        store.store(effective_id, inline);

        let cols = params.get("c").and_then(|v| v.parse::<usize>().ok());
        let rows = params.get("r").and_then(|v| v.parse::<usize>().ok());
        let z = params
            .get("z")
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(0);
        let cursor_movement = params
            .get("C")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        let w_cols = cols.unwrap_or(1);
        let h_rows = rows.unwrap_or(1);

        if action == "T" || action.is_empty() {
            store.add_placement(ImagePlacement {
                image_id: effective_id,
                row: cursor_row,
                col: cursor_col,
                width_cols: if cols.is_some() { w_cols } else { 1 },
                height_rows: if rows.is_some() { h_rows } else { 1 },
                z_index: z,
            });

            let _ = cursor_movement;
        }
    }
}

fn decode_kitty_image(
    format: u32,
    data: &[u8],
    params: &HashMap<String, String>,
) -> Option<ImgBuf> {
    match format {
        100 => decode_png(data),
        32 => {
            let w = params.get("s").and_then(|v| v.parse::<u32>().ok())?;
            let h = params.get("v").and_then(|v| v.parse::<u32>().ok())?;
            decode_rgba(data, w, h)
        }
        24 => {
            let w = params.get("s").and_then(|v| v.parse::<u32>().ok())?;
            let h = params.get("v").and_then(|v| v.parse::<u32>().ok())?;
            decode_rgb(data, w, h)
        }
        _ => None,
    }
}

fn decode_png(data: &[u8]) -> Option<ImgBuf> {
    let img = image::load_from_memory(data).ok()?;
    Some(img.to_rgba8())
}

fn decode_rgba(data: &[u8], w: u32, h: u32) -> Option<ImgBuf> {
    if data.len() < (w * h * 4) as usize {
        return None;
    }
    let mut buf = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let offset = ((y * w + x) * 4) as usize;
            buf.put_pixel(
                x,
                y,
                Rgba([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]),
            );
        }
    }
    Some(buf)
}

fn decode_rgb(data: &[u8], w: u32, h: u32) -> Option<ImgBuf> {
    if data.len() < (w * h * 3) as usize {
        return None;
    }
    let mut buf = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let offset = ((y * w + x) * 3) as usize;
            buf.put_pixel(
                x,
                y,
                Rgba([data[offset], data[offset + 1], data[offset + 2], 255]),
            );
        }
    }
    Some(buf)
}

pub fn process_sixel(
    data: &[u8],
    cursor_row: usize,
    cursor_col: usize,
    store: &mut InlineImageStore,
) {
    let img = decode_sixel(data);
    if let Some(rgba) = img {
        let id = store.next_auto_id;
        store.next_auto_id += 1;
        store.store(id, InlineImage { rgba });
        store.add_placement(ImagePlacement {
            image_id: id,
            row: cursor_row,
            col: cursor_col,
            width_cols: 0,
            height_rows: 0,
            z_index: 0,
        });
    }
}

fn decode_sixel(data: &[u8]) -> Option<ImgBuf> {
    let mut colors: Vec<[u8; 3]> = vec![[255, 255, 255]; 256];
    let mut current_color_idx: usize = 0;
    let mut repeat_count: usize = 1;

    struct Band {
        cols: Vec<[u8; 6]>,
    }
    impl Band {
        fn new() -> Self {
            Self { cols: Vec::new() }
        }
        fn draw(&mut self, col: usize, color: u8, bits: u8) {
            if col >= self.cols.len() {
                self.cols.resize(col + 1, [0; 6]);
            }
            for bit in 0..6u8 {
                if bits & (1 << bit) != 0 {
                    self.cols[col][bit as usize] = color;
                }
            }
        }
    }

    let mut bands: Vec<Band> = Vec::new();
    let mut current_band = Band::new();
    let mut col_pos: usize = 0;
    let mut pos = 0;

    while pos < data.len() {
        let ch = data[pos];
        match ch {
            b'"' => {
                pos += 1;
                while pos < data.len()
                    && data[pos] != b'$'
                    && data[pos] != b'-'
                    && data[pos] != b'#'
                    && data[pos] != b'!'
                    && !(63..=126).contains(&data[pos])
                {
                    pos += 1;
                }
            }
            b'!' => {
                pos += 1;
                let mut num = 0usize;
                while pos < data.len() && data[pos].is_ascii_digit() {
                    num = num * 10 + (data[pos] - b'0') as usize;
                    pos += 1;
                }
                repeat_count = num.max(1);
            }
            b'#' => {
                pos += 1;
                let mut num = 0usize;
                while pos < data.len() && data[pos].is_ascii_digit() {
                    num = num * 10 + (data[pos] - b'0') as usize;
                    pos += 1;
                }
                current_color_idx = num;

                if pos < data.len() && data[pos] == b';' {
                    pos += 1;
                    let mut color_parts: Vec<i64> = Vec::new();
                    let mut current_num = String::new();
                    while pos < data.len() {
                        let c = data[pos];
                        if c.is_ascii_digit() {
                            current_num.push(c as char);
                            pos += 1;
                        } else if c == b';' {
                            color_parts.push(current_num.parse().unwrap_or(0));
                            current_num.clear();
                            pos += 1;
                            if color_parts.len() >= 4 {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    if !current_num.is_empty() {
                        color_parts.push(current_num.parse().unwrap_or(0));
                    }

                    if color_parts.len() >= 4 {
                        let _colorspace = color_parts[0];
                        let r = color_parts[1];
                        let g = color_parts[2];
                        let b = color_parts[3];
                        if current_color_idx < 256 {
                            colors[current_color_idx] = [
                                (r * 255 / 100).clamp(0, 255) as u8,
                                (g * 255 / 100).clamp(0, 255) as u8,
                                (b * 255 / 100).clamp(0, 255) as u8,
                            ];
                        }
                    }
                }
            }
            b'$' => {
                col_pos = 0;
                pos += 1;
            }
            b'-' => {
                bands.push(std::mem::replace(&mut current_band, Band::new()));
                col_pos = 0;
                pos += 1;
            }
            63..=126 => {
                let sixel_bits = ch - 63;
                for _ in 0..repeat_count {
                    current_band.draw(col_pos, current_color_idx as u8, sixel_bits);
                    col_pos += 1;
                }
                repeat_count = 1;
                pos += 1;
            }
            _ => {
                pos += 1;
            }
        }
    }
    if !current_band.cols.is_empty() {
        bands.push(current_band);
    }

    if bands.is_empty() {
        return None;
    }

    let width = bands.iter().map(|b| b.cols.len()).max().unwrap_or(0);
    let height = bands.len() * 6;

    if width == 0 || height == 0 {
        return None;
    }

    let mut buf = ImageBuffer::from_pixel(width as u32, height as u32, Rgba([0, 0, 0, 0]));

    for (band_idx, band) in bands.iter().enumerate() {
        let y_base = (band_idx * 6) as u32;
        for (col, color_indices) in band.cols.iter().enumerate() {
            let x = col as u32;
            for bit in 0..6u8 {
                let cidx = color_indices[bit as usize] as usize;
                if cidx > 0 || band_idx == 0 {
                    let y = y_base + bit as u32;
                    if x < buf.width() && y < buf.height() && cidx > 0 {
                        let color = colors.get(cidx).copied().unwrap_or([255, 255, 255]);
                        buf.put_pixel(x, y, Rgba([color[0], color[1], color[2], 255]));
                    }
                }
            }
        }
    }

    Some(buf)
}

pub fn process_osc_1337(
    parts: &[&[u8]],
    cursor_row: usize,
    cursor_col: usize,
    store: &mut InlineImageStore,
) {
    if parts.len() < 2 {
        return;
    }
    let payload_str = String::from_utf8_lossy(parts[1]);
    if !payload_str.starts_with("File=") {
        return;
    }
    let kv_str = &payload_str[5..];
    let mut kvs: HashMap<String, String> = HashMap::new();
    for kv in kv_str.split(';') {
        if let Some((k, v)) = kv.split_once('=') {
            kvs.insert(k.to_string(), v.to_string());
        }
    }

    let inline = kvs.keys().any(|k| k == "inline");
    if !inline {
        return;
    }

    let raw_data: Option<Vec<u8>> = {
        let s = String::from_utf8_lossy(parts[1]);
        let after_file = &s[5..];
        let data_part = if let Some(idx) = after_file.find(":") {
            &after_file[idx + 1..]
        } else {
            ""
        };
        if data_part.is_empty() {
            None
        } else {
            BASE64.decode(data_part).ok()
        }
    };

    if let Some(rgba) = raw_data.and_then(|d| decode_png(&d)) {
        let id = store.next_auto_id;
        store.next_auto_id += 1;
        store.store(id, InlineImage { rgba });
        store.add_placement(ImagePlacement {
            image_id: id,
            row: cursor_row,
            col: cursor_col,
            width_cols: 0,
            height_rows: 0,
            z_index: 0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sixel_simple_red_pixel() {
        let sixel = b"#1;2;100;0;0!6~";
        let img = decode_sixel(sixel);
        assert!(img.is_some());
        let img = img.unwrap();
        assert_eq!(img.width(), 6);
        assert_eq!(img.height(), 6);
        assert_eq!(img.get_pixel(0, 0), &Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_sixel_two_bands() {
        let sixel = b"#0;2;0;100;0~$-~";
        let img = decode_sixel(sixel);
        assert!(img.is_some());
        let img = img.unwrap();
        assert_eq!(img.width(), 1);
        assert_eq!(img.height(), 12);
    }

    #[test]
    fn test_kitty_png_direct() {
        let mut store = InlineImageStore::new();
        let png = image::RgbaImage::from_pixel(4, 4, Rgba([255, 0, 0, 255]));
        let mut png_bytes = Vec::new();
        use std::io::Cursor;
        png.write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .unwrap();
        let b64 = BASE64.encode(&png_bytes);

        let control = "f=100,a=T";
        let mut state = KittyGraphicsState::new();
        process_kitty_apc(&mut state, control, b64.as_bytes(), 0, 0, &mut store);

        assert!(!store.images.is_empty());
        assert!(!store.placements.is_empty());
    }

    #[test]
    fn test_kitty_chunked() {
        let mut store = InlineImageStore::new();
        let png = image::RgbaImage::from_pixel(4, 4, Rgba([0, 255, 0, 255]));
        let mut png_bytes = Vec::new();
        use std::io::Cursor;
        png.write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .unwrap();
        let b64 = BASE64.encode(&png_bytes);

        let mid = b64.len() / 2;
        let (chunk1, chunk2) = (&b64[..mid], &b64[mid..]);

        let mut state = KittyGraphicsState::new();
        process_kitty_apc(
            &mut state,
            "f=100,a=T,m=1",
            chunk1.as_bytes(),
            0,
            0,
            &mut store,
        );
        assert!(store.images.is_empty());

        process_kitty_apc(&mut state, "m=0", chunk2.as_bytes(), 0, 0, &mut store);
        assert!(!store.images.is_empty());
    }

    #[test]
    fn test_kitty_delete() {
        let mut store = InlineImageStore::new();
        let png = image::RgbaImage::from_pixel(4, 4, Rgba([0, 0, 255, 255]));
        let mut png_bytes = Vec::new();
        use std::io::Cursor;
        png.write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .unwrap();
        let b64 = BASE64.encode(&png_bytes);

        let mut state = KittyGraphicsState::new();
        process_kitty_apc(
            &mut state,
            "f=100,a=T,i=42",
            b64.as_bytes(),
            0,
            0,
            &mut store,
        );
        assert!(store.get_image(42).is_some());

        process_kitty_apc(&mut state, "a=d,i=42", b"", 0, 0, &mut store);
        assert!(store.get_image(42).is_none());
    }

    #[test]
    fn test_full_kitty_pipeline_logo_render() {
        let logo_w = 48u32;
        let logo_h = 24u32;
        let mut logo_img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_pixel(logo_w, logo_h, Rgba([0x00, 0x2b, 0x36, 255]));
        let accent = Rgba([0xdc, 0x32, 0x2f, 255]);
        for x in 4..44 {
            logo_img.put_pixel(x, 3, accent);
            logo_img.put_pixel(x, 4, accent);
        }
        let stem_x = 23u32;
        for y in 5..20u32 {
            logo_img.put_pixel(stem_x, y, Rgba([0xeb, 0xdb, 0xe2, 255]));
            logo_img.put_pixel(stem_x + 1, y, Rgba([0xeb, 0xdb, 0xe2, 255]));
            logo_img.put_pixel(stem_x + 2, y, Rgba([0xeb, 0xdb, 0xe2, 255]));
        }

        let mut png_bytes = Vec::new();
        use std::io::Cursor;
        logo_img
            .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .unwrap();
        let b64 = BASE64.encode(&png_bytes);

        let control = "f=100,a=T,c=16,r=12";
        let mut state = KittyGraphicsState::new();
        let mut store = InlineImageStore::new();

        if b64.len() > 4096 {
            let mid = b64.len() / 2;
            process_kitty_apc(
                &mut state,
                "f=100,a=T,m=1,c=16,r=12",
                &b64.as_bytes()[..mid],
                6,
                2,
                &mut store,
            );
            process_kitty_apc(&mut state, "m=0", &b64.as_bytes()[mid..], 6, 2, &mut store);
        } else {
            process_kitty_apc(&mut state, control, b64.as_bytes(), 6, 2, &mut store);
        }

        assert!(!store.images.is_empty(), "image should be stored");
        assert!(!store.placements.is_empty(), "placement should exist");

        let placement = &store.placements[0];
        assert_eq!(placement.row, 6);
        assert_eq!(placement.col, 2);
        assert_eq!(placement.width_cols, 16);
        assert_eq!(placement.height_rows, 12);

        let img_id = placement.image_id;
        let stored = store.get_image(img_id);
        assert!(stored.is_some(), "stored image should be retrievable");

        let rgba = stored.unwrap();
        assert_eq!(rgba.rgba.width(), logo_w);
        assert_eq!(rgba.rgba.height(), logo_h);

        let px = rgba.rgba.get_pixel(10, 3);
        assert_eq!(
            *px,
            Rgba([0xdc, 0x32, 0x2f, 255]),
            "accent bar pixel should be red"
        );

        let stem_px = rgba.rgba.get_pixel(24, 10);
        assert_eq!(
            *stem_px,
            Rgba([0xeb, 0xdb, 0xe2, 255]),
            "stem pixel should be light"
        );
    }
}
