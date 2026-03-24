//! Hikari SSR Test
//!
//! This test program renders the hikari website WASM component using tairitsu-ssr
//! and verifies the output HTML structure.

use anyhow::Result;
use std::fs;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== Hikari SSR Test ===\n");

    // Read the hikari website WASM file
    let wasm_path = "/mnt/sdb1/hikari/public/website.wasm";
    println!("Reading WASM file from: {}", wasm_path);

    let wasm_bytes = fs::read(wasm_path)?;
    println!(
        "WASM file size: {} bytes ({:.2} MB)\n",
        wasm_bytes.len(),
        wasm_bytes.len() as f64 / 1024.0 / 1024.0
    );

    // Create SSR config
    let config = tairitsu_ssr::SsrConfig::new(1920, 1080);
    println!(
        "SSR Config: viewport={}x{}\n",
        config.viewport_width, config.viewport_height
    );

    // Render to HTML
    println!("Rendering WASM component to HTML...");
    let html = tairitsu_ssr::render_to_html(&wasm_bytes, config)?;

    println!(
        "Rendered HTML size: {} bytes ({:.2} KB)\n",
        html.len(),
        html.len() as f64 / 1024.0
    );

    // Analyze the HTML structure
    println!("=== HTML Structure Analysis ===\n");

    // Check for #hikari-app
    let has_hikari_app = html.contains("id=\"hikari-app\"");
    println!(
        "[{}] #hikari-app element",
        if has_hikari_app { "OK" } else { "MISSING" }
    );

    // Check for .hi-layout class
    let has_hi_layout = html.contains("hi-layout") || html.contains("class=\"hi-layout\"");
    println!(
        "[{}] .hi-layout class",
        if has_hi_layout { "OK" } else { "MISSING" }
    );

    // Check for .hikari-page elements
    let page_count = html.matches("hikari-page").count();
    println!(
        "[{}] .hikari-page elements (count: {})",
        if page_count > 0 { "OK" } else { "MISSING" },
        page_count
    );

    // Count other interesting elements
    let div_count = html.matches("<div").count();
    let button_count = html.matches("<button").count() + html.matches("<Button").count();
    let anchor_count = html.matches("<a").count();

    println!("\n=== Element Counts ===");
    println!("Div elements: {}", div_count);
    println!("Button elements: {}", button_count);
    println!("Anchor elements: {}", anchor_count);

    // Print a preview of the HTML (first 500 chars)
    println!("\n=== HTML Preview (first 500 chars) ===");
    let preview = if html.len() > 500 {
        format!("{}...", &html[..500])
    } else {
        html.clone()
    };
    println!("{}", preview);

    // Print last 500 chars
    if html.len() > 1000 {
        println!("\n=== HTML Preview (last 500 chars) ===");
        let preview = format!("...{}", &html[html.len() - 500..]);
        println!("{}", preview);
    }

    // Save full HTML to file
    let output_path = "/tmp/hikari-ssr-output.html";
    fs::write(output_path, &html)?;
    println!("\n=== Output Saved ===");
    println!("Full HTML saved to: {}", output_path);

    // Summary
    println!("\n=== Test Summary ===");
    let all_checks_passed = has_hikari_app && has_hi_layout && page_count > 0;
    if all_checks_passed {
        println!("✓ All critical checks passed!");
        println!("✓ SSR is working correctly for hikari website");
    } else {
        println!("✗ Some checks failed:");
        if !has_hikari_app {
            println!("  - #hikari-app element not found");
        }
        if !has_hi_layout {
            println!("  - .hi-layout class not found");
        }
        if page_count == 0 {
            println!("  - .hikari-page elements not found");
        }
    }

    Ok(())
}
