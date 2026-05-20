use image::{ImageBuffer, Rgba};

use super::{InlineImage, InlineImageStore};

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
        store.add_placement(super::ImagePlacement {
            image_id: id,
            row: cursor_row,
            col: cursor_col,
            width_cols: 0,
            height_rows: 0,
            z_index: 0,
        });
    }
}

pub(crate) fn decode_sixel(data: &[u8]) -> Option<super::ImgBuf> {
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
