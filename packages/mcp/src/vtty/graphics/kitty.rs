use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::collections::HashMap;

use super::{decode_png, decode_rgb, decode_rgba, InlineImageStore};

pub struct KittyGraphicsState {
    pub(crate) control: String,
    pub(crate) payload: Vec<u8>,
    pub(crate) image_id: u32,
    pub(crate) chunked: bool,
    pub(crate) first_chunk: bool,
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

pub(crate) fn parse_kitty_control(control: &str) -> HashMap<String, String> {
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
                    store.add_placement(super::ImagePlacement {
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
        let inline = super::InlineImage { rgba };

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
            store.add_placement(super::ImagePlacement {
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
) -> Option<super::ImgBuf> {
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
