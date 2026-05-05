use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use base64::Engine;
use serde::Deserialize;

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_NAVIGATE_WAIT_MS: u64 = 800;
const DEFAULT_CLICK_WAIT_MS: u64 = 300;

#[derive(Debug, Clone)]
pub struct TestConfig {
    pub base_url: String,
    pub baseline_dir: PathBuf,
    pub actual_dir: PathBuf,
    pub tolerance: f32,
    pub update_baselines: bool,
    pub pages: Vec<PageSpec>,
}

#[derive(Debug, Clone)]
pub struct PageSpec {
    pub url: &'static str,
    pub name: &'static str,
    pub interactions: &'static [(&'static str, &'static str)],
}

#[derive(Debug)]
pub struct TestReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<PageResult>,
}

#[derive(Debug)]
pub struct PageResult {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct ApiResp {
    ok: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .build()
        .expect("reqwest client")
}

fn api_post(client: &reqwest::blocking::Client, base_url: &str, path: &str, body: &serde_json::Value) -> Result<String> {
    let resp = client
        .post(format!("{}{}", base_url, path))
        .json(body)
        .send()
        .context("debug API request failed")?;
    Ok(resp.text().context("reading response body")?)
}

fn api_get(client: &reqwest::blocking::Client, base_url: &str, path: &str) -> Result<String> {
    let resp = client
        .get(format!("{}{}", base_url, path))
        .send()
        .context("debug API request failed")?;
    Ok(resp.text().context("reading response body")?)
}

fn check_health(client: &reqwest::blocking::Client, base_url: &str) -> Result<()> {
    let text = api_get(client, base_url, "/health")?;
    let v: serde_json::Value = serde_json::from_str(&text)?;
    if v["ok"].as_bool() != Some(true) {
        anyhow::bail!("Debug API health check failed: {}", text);
    }
    Ok(())
}

fn navigate(client: &reqwest::blocking::Client, base_url: &str, url: &str) -> Result<()> {
    let text = api_post(client, base_url, "/navigate", &serde_json::json!({ "url": url }))?;
    let v: ApiResp = serde_json::from_str(&text)?;
    if !v.ok {
        anyhow::bail!("navigate {} failed: {:?}", url, v.error);
    }
    std::thread::sleep(Duration::from_millis(DEFAULT_NAVIGATE_WAIT_MS));
    Ok(())
}

fn evaluate(client: &reqwest::blocking::Client, base_url: &str, js: &str) -> Result<serde_json::Value> {
    let text = api_post(client, base_url, "/evaluate", &serde_json::json!({ "expression": js, "await_promise": false }))?;
    let v: serde_json::Value = serde_json::from_str(&text)?;
    if v["ok"].as_bool() != Some(true) {
        anyhow::bail!("evaluate failed: {}", v);
    }
    Ok(v["data"]["result"].clone())
}

fn click(client: &reqwest::blocking::Client, base_url: &str, selector: &str) -> Result<()> {
    let text = api_post(client, base_url, "/click", &serde_json::json!({ "selector": selector }))?;
    let v: ApiResp = serde_json::from_str(&text)?;
    if !v.ok {
        anyhow::bail!("click {} failed: {:?}", selector, v.error);
    }
    std::thread::sleep(Duration::from_millis(DEFAULT_CLICK_WAIT_MS));
    Ok(())
}

fn screenshot(client: &reqwest::blocking::Client, base_url: &str) -> Result<image::DynamicImage> {
    let text = api_post(client, base_url, "/screenshot", &serde_json::json!({ "full_page": false }))?;
    let v: serde_json::Value = serde_json::from_str(&text)?;
    if v["ok"].as_bool() != Some(true) {
        anyhow::bail!("screenshot failed: {}", v);
    }
    let b64 = v["data"]["data"].as_str().context("missing base64 data")?;
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64).context("base64 decode")?;
    image::load_from_memory(&bytes).context("png decode")
}

fn compare_images(baseline: &image::DynamicImage, actual: &image::DynamicImage, tolerance: f32) -> (bool, f32, u64, u64) {
    use image::GenericImageView;
    let (bw, bh) = baseline.dimensions();
    let (aw, ah) = actual.dimensions();
    if bw != aw || bh != ah {
        return (false, 1.0, 0, 0);
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

pub fn run_tests(config: &TestConfig) -> Result<TestReport> {
    let client = client();
    check_health(&client, &config.base_url)?;

    std::fs::create_dir_all(&config.baseline_dir)?;
    std::fs::create_dir_all(&config.actual_dir)?;

    let mut report = TestReport { total: 0, passed: 0, failed: 0, results: Vec::new() };

    for page in &config.pages {
        report.total += 1;
        match run_page(&client, config, page) {
            Ok(detail) => {
                report.passed += 1;
                report.results.push(PageResult { name: page.name.to_string(), passed: true, detail });
            }
            Err(e) => {
                report.failed += 1;
                report.results.push(PageResult { name: page.name.to_string(), passed: false, detail: format!("{}", e) });
            }
        }
    }

    Ok(report)
}

fn run_page(
    client: &reqwest::blocking::Client,
    config: &TestConfig,
    page: &PageSpec,
) -> Result<String> {
    navigate(client, &config.base_url, page.url)?;
    let img = screenshot(client, &config.base_url)?;

    if config.update_baselines {
        let path = config.baseline_dir.join(format!("{}.png", page.name));
        img.save(&path)?;
        return Ok("baseline updated".into());
    }

    let actual_path = config.actual_dir.join(format!("{}.png", page.name));
    img.save(&actual_path)?;

    let baseline_path = config.baseline_dir.join(format!("{}.png", page.name));
    if !baseline_path.exists() {
        anyhow::bail!("no baseline (run with --update-baselines to create)");
    }

    let baseline_img = image::open(&baseline_path)?;
    let (passed, ratio, diff, total) = compare_images(&baseline_img, &img, config.tolerance);

    if !passed {
        anyhow::bail!("{:.2}% diff ({}/{})", ratio * 100.0, diff, total);
    }

    for &(action, selector) in page.interactions {
        if action == "click" {
            click(client, &config.base_url, selector)?;
        }
        let _ = screenshot(client, &config.base_url);
    }

    Ok(format!("PASS ({:.4}% diff, {}/{})", ratio * 100.0, diff, total))
}

pub fn run_events(client: &reqwest::blocking::Client, base_url: &str, pages: &[PageSpec]) -> Vec<PageResult> {
    let mut results = Vec::new();
    for page in pages {
        if let Err(e) = navigate(client, base_url, page.url) {
            results.push(PageResult { name: format!("{}: navigate", page.name), passed: false, detail: format!("{}", e) });
            continue;
        }

        let check = evaluate(client, base_url, "document.getElementById('ts-app') !== null");
        match check {
            Ok(v) if v == serde_json::Value::Bool(true) => {
                results.push(PageResult { name: format!("{}: app-mount", page.name), passed: true, detail: "ok".into() });
            }
            Ok(v) => {
                results.push(PageResult { name: format!("{}: app-mount", page.name), passed: false, detail: format!("ts-app not mounted: {:?}", v) });
            }
            Err(e) => {
                results.push(PageResult { name: format!("{}: app-mount", page.name), passed: false, detail: format!("{}", e) });
            }
        }
    }
    results
}
