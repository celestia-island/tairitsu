use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use image::{ImageBuffer, Rgba};
use std::io::Write;

fn generate_logo() -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let w = 48u32;
    let h = 24u32;
    let mut img = ImageBuffer::from_pixel(w, h, Rgba([0x00, 0x2b, 0x36, 255]));

    let accent = Rgba([0xdc, 0x32, 0x2f, 255]);
    let fg = Rgba([0xeb, 0xdb, 0xe2, 255]);
    let cyan = Rgba([0x2a, 0xa1, 0x98, 255]);

    let bar_y = 3;
    for x in 4..44 {
        img.put_pixel(x, bar_y, accent);
        img.put_pixel(x, bar_y + 1, accent);
    }

    let stem_x = 23;
    for y in bar_y + 2..20 {
        img.put_pixel(stem_x, y, fg);
        img.put_pixel(stem_x + 1, y, fg);
        img.put_pixel(stem_x + 2, y, fg);
    }

    for y in 6..10 {
        for x in 34..43 {
            if (x + y) % 2 == 0 {
                img.put_pixel(x, y, cyan);
            }
        }
    }

    img
}

fn emit_kitty_png(img: &ImageBuffer<Rgba<u8>, Vec<u8>>, stdout: &mut std::io::Stdout) {
    let mut png_data = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)
        .unwrap();
    let b64 = BASE64.encode(&png_data);

    let control = "f=100,a=T,c=16,r=12";
    let chunk_size = 4096;

    let chunks: Vec<&str> = b64.as_str().as_bytes().chunks(chunk_size)
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect();

    if chunks.len() <= 1 {
        write!(stdout, "\x1b_G{};{}\x1b\\", control, b64).unwrap();
    } else {
        for (i, chunk) in chunks.iter().enumerate() {
            let m = if i < chunks.len() - 1 { "m=1" } else { "m=0" };
            let ctrl = if i == 0 { control } else { m };
            write!(stdout, "\x1b_G{};{}\x1b\\", ctrl, chunk).unwrap();
        }
    }

    stdout.flush().unwrap();
}

fn main() {
    let logo = generate_logo();

    print!("\x1b[2J\x1b[H");

    print!("\x1b[38;5;166m╭──────────────────────────────────────╮\x1b[0m\r\n");
    print!("\x1b[38;5;166m│                                      │\x1b[0m\r\n");
    print!("\x1b[38;5;166m│   \x1b[38;5;231mTairitsu\x1b[0m\x1b[38;5;166m VTty Graphics Test     │\x1b[0m\r\n");
    print!("\x1b[38;5;166m│                                      │\x1b[0m\r\n");
    print!("\x1b[38;5;166m├──────────────────────────────────────┤\x1b[0m\r\n");
    print!("\x1b[38;5;166m│                                      │\x1b[0m\r\n");

    let mut stdout = std::io::stdout();
    emit_kitty_png(&logo, &mut stdout);

    print!("\r\n");
    print!("\x1b[38;5;166m│                                      │\x1b[0m\r\n");
    print!("\x1b[38;5;166m│   \x1b[38;5;37m✓ Kitty Graphics Protocol (PNG)  \x1b[0m\x1b[38;5;166m│\x1b[0m\r\n");
    print!("\x1b[38;5;166m│   \x1b[38;5;37m✓ Sixel DCS decoder            \x1b[0m\x1b[38;5;166m│\x1b[0m\r\n");
    print!("\x1b[38;5;166m│   \x1b[38;5;37m✓ OSC 1337 (iTerm2) support      \x1b[0m\x1b[38;5;166m│\x1b[0m\r\n");
    print!("\x1b[38;5;166m│                                      │\x1b[0m\r\n");
    print!("\x1b[38;5;166m╰──────────────────────────────────────╯\x1b[0m\r\n");

    stdout.flush().unwrap();

    std::thread::sleep(std::time::Duration::from_secs(2));

    print!("\x1b_Ga=d,i=1\x1b\\\r\n");
    stdout.flush().unwrap();
}
