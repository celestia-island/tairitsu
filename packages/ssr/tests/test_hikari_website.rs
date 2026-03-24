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
            
            // Basic assertions
            assert!(html.contains("<div"), "HTML should contain a div element");
            assert!(html.len() > 100, "HTML should be non-empty");
        }
        Err(e) => {
            println!("SSR failed: {:?}", e);
            panic!("SSR failed: {}", e);
        }
    }
}
