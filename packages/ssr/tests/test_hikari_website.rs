use tairitsu_ssr::{render_to_html, SsrConfig};

#[test]
fn test_hikari_website() {
    let wasm_path = "/mnt/sdb1/hikari/public/website.wasm";
    let wasm_bytes = std::fs::read(wasm_path).expect("Failed to read WASM file");
    println!("Loaded {} bytes from {}", wasm_bytes.len(), wasm_path);

    let config = SsrConfig::default();
    match render_to_html(&wasm_bytes, config) {
        Ok(html) => {
            println!("SSR successful!");
            println!("HTML length: {} bytes", html.len());
            println!("HTML content: {}", html);

            // Basic assertions - the main goal is to not crash with type marshaling errors
            assert!(html.contains("<body>"), "HTML should contain a body element");
        }
        Err(e) => {
            println!("SSR failed: {:?}", e);
            panic!("SSR failed: {}", e);
        }
    }
}
