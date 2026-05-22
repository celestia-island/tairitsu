use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::collections::HashMap;

use super::{decode_png, InlineImage, InlineImageStore};

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
