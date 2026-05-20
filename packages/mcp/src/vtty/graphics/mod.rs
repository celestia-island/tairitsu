mod kitty;
mod osc1337;
mod sixel;

pub use kitty::{process_kitty_apc, KittyGraphicsState};
pub use osc1337::process_osc_1337;
pub use sixel::process_sixel;

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
    pub(crate) images: HashMap<u32, InlineImage>,
    placements: Vec<ImagePlacement>,
    pub(crate) next_auto_id: u32,
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

pub(crate) fn decode_png(data: &[u8]) -> Option<ImgBuf> {
    let img = image::load_from_memory(data).ok()?;
    Some(img.to_rgba8())
}

pub(crate) fn decode_rgba(data: &[u8], w: u32, h: u32) -> Option<ImgBuf> {
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

pub(crate) fn decode_rgb(data: &[u8], w: u32, h: u32) -> Option<ImgBuf> {
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

#[cfg(test)]
mod tests {
    use base64::Engine;
    use super::*;

    #[test]
    fn test_sixel_simple_red_pixel() {
        let sixel = b"#1;2;100;0;0!6~";
        let img = sixel::decode_sixel(sixel);
        assert!(img.is_some());
        let img = img.unwrap();
        assert_eq!(img.width(), 6);
        assert_eq!(img.height(), 6);
        assert_eq!(img.get_pixel(0, 0), &Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_sixel_two_bands() {
        let sixel = b"#0;2;0;100;0~$-~";
        let img = sixel::decode_sixel(sixel);
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
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

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
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

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
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

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
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

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
