use base64::Engine;
use image::GenericImageView;
use serde::Deserialize;

const BASE_URL: &str = "http://localhost:3001";

struct PageSpec {
    url: &'static str,
    name: &'static str,
    interactions: &'static [(&'static str, &'static str)],
}

static PAGES: &[PageSpec] = &[
    PageSpec {
        url: "/",
        name: "home",
        interactions: &[],
    },
    PageSpec {
        url: "/event-test",
        name: "event_test",
        interactions: &[("click", "#event-test-btn")],
    },
];

#[derive(Deserialize)]
struct ApiResponse {
    ok: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct ScreenshotRaw {
    data: serde_json::Value,
    mime_type: String,
    width: u32,
    height: u32,
}

fn api_get(path: &str) -> reqwest::blocking::Response {
    reqwest::blocking::get(format!("{}{}", BASE_URL, path)).expect("debug API not reachable")
}

fn api_post(path: &str, body: &serde_json::Value) -> reqwest::blocking::Response {
    reqwest::blocking::Client::new()
        .post(format!("{}{}", BASE_URL, path))
        .json(body)
        .send()
        .expect("debug API request failed")
}

fn api_post_raw(path: &str, body: &serde_json::Value) -> String {
    api_post(path, body).text().expect("response read failed")
}

fn navigate(url: &str) {
    let text = api_post_raw("/navigate", &serde_json::json!({ "url": url }));
    let msg: ApiResponse = serde_json::from_str(&text).unwrap();
    assert!(msg.ok, "navigate failed: {:?}", msg.error);
    std::thread::sleep(std::time::Duration::from_millis(800));
}

fn screenshot() -> image::DynamicImage {
    let text = api_post_raw("/screenshot", &serde_json::json!({ "full_page": false }));
    let msg: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(msg["ok"].as_bool().unwrap(), "screenshot failed: {}", msg);
    let b64 = msg["data"]["data"].as_str().unwrap();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .expect("base64 decode");
    image::load_from_memory(&bytes).expect("png decode")
}

fn click(selector: &str) {
    let text = api_post_raw("/click", &serde_json::json!({ "selector": selector }));
    let msg: ApiResponse = serde_json::from_str(&text).unwrap();
    assert!(msg.ok, "click failed: {:?}", msg.error);
    std::thread::sleep(std::time::Duration::from_millis(300));
}

fn baseline_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("baselines")
}

fn actual_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join("target/web-test/actual")
}

fn compare_images(baseline: &image::DynamicImage, actual: &image::DynamicImage, tolerance: f32) -> (bool, f32, u64, u64) {
    let (bw, bh) = baseline.dimensions();
    let (aw, ah) = actual.dimensions();
    if bw != aw || bh != ah {
        return (false, 1.0, (bw * bh) as u64, (bw * bh) as u64);
    }
    let ba = baseline.to_rgba8();
    let aa = actual.to_rgba8();
    let total = (bw * bh) as u64;
    let mut diff: u64 = 0;
    for y in 0..bh {
        for x in 0..bw {
            if ba.get_pixel(x, y) != aa.get_pixel(x, y) {
                diff += 1;
            }
        }
    }
    let ratio = if total > 0 { diff as f32 / total as f32 } else { 0.0 };
    (ratio <= tolerance, ratio, diff, total)
}

#[test]
fn debug_api_is_healthy() {
    let resp = api_get("/health");
    assert!(resp.status().is_success());
}

#[test]
fn all_pages_render() {
    let baseline = baseline_dir();
    let actual = actual_dir();
    std::fs::create_dir_all(&baseline).unwrap();
    std::fs::create_dir_all(&actual).unwrap();

    let mut failures = Vec::new();

    for page in PAGES {
        navigate(page.url);
        let img = screenshot();
        let actual_path = actual.join(format!("{}.png", page.name));
        img.save(&actual_path).unwrap();

        let baseline_path = baseline.join(format!("{}.png", page.name));
        if !baseline_path.exists() {
            eprintln!("  MISSING BASELINE: {} (copy {} to create)", page.name, actual_path.display());
            failures.push(format!("{}: no baseline", page.name));
            continue;
        }

        let baseline_img = image::open(&baseline_path).unwrap();
        let (passed, ratio, diff, total) = compare_images(&baseline_img, &img, 0.05);
        eprintln!("  {}: {} ({:.4}% diff, {}/{})", page.name, if passed { "PASS" } else { "FAIL" }, ratio * 100.0, diff, total);
        if !passed {
            failures.push(format!("{}: {:.2}% diff", page.name, ratio * 100.0));
        }

        for &(action, selector) in page.interactions {
            if action == "click" {
                click(selector);
            }
            let img2 = screenshot();
            let inter_path = actual.join(format!("{}_interact.png", page.name));
            img2.save(&inter_path).unwrap();
        }
    }

    assert!(failures.is_empty(), "Visual regression failures:\n  {}", failures.join("\n  "));
}

#[test]
fn update_baselines() {
    if std::env::var("UPDATE_BASELINES").is_err() {
        eprintln!("Skipping baseline update (set UPDATE_BASELINES=1 to enable)");
        return;
    }

    let baseline = baseline_dir();
    std::fs::create_dir_all(&baseline).unwrap();

    for page in PAGES {
        navigate(page.url);
        let img = screenshot();
        let path = baseline.join(format!("{}.png", page.name));
        img.save(&path).unwrap();
        eprintln!("  Updated baseline: {}", path.display());
    }
}
