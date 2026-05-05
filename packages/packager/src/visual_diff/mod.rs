use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use image::{GenericImageView, ImageBuffer, Rgba};
use serde::Serialize;

pub struct DiffConfig {
    pub tolerance: f32,
    pub output_dir: PathBuf,
    pub baseline_dir: PathBuf,
    pub generate_html: bool,
    pub fail_on_diff: bool,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            tolerance: 0.01,
            output_dir: PathBuf::from("target/visual-diff"),
            baseline_dir: PathBuf::from("tests/visual/baseline"),
            generate_html: true,
            fail_on_diff: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffResult {
    pub name: String,
    pub passed: bool,
    pub pixel_diff_ratio: f32,
    pub total_pixels: u64,
    pub diff_pixels: u64,
    pub baseline_path: Option<String>,
    pub actual_path: String,
    pub diff_image_path: Option<String>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffReport {
    pub timestamp: String,
    pub tolerance: f32,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<DiffResult>,
}

#[allow(dead_code)]
fn rgba_distance(a: &Rgba<u8>, b: &Rgba<u8>) -> f32 {
    let dr = a[0].abs_diff(b[0]) as f32;
    let dg = a[1].abs_diff(b[1]) as f32;
    let db = a[2].abs_diff(b[2]) as f32;
    let da = a[3].abs_diff(b[3]) as f32;
    (dr * dr + dg * dg + db * db + da * da).sqrt() / 510.0
}

pub fn compare_images(baseline: &Path, actual: &Path, config: &DiffConfig) -> Result<DiffResult> {
    let name = actual
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let baseline_img = image::open(baseline)
        .with_context(|| format!("Failed to open baseline: {}", baseline.display()))?;
    let actual_img = image::open(actual)
        .with_context(|| format!("Failed to open actual: {}", actual.display()))?;

    let (w, h) = baseline_img.dimensions();
    let (aw, ah) = actual_img.dimensions();

    if w != aw || h != ah {
        return Ok(DiffResult {
            name,
            passed: false,
            pixel_diff_ratio: 1.0,
            total_pixels: (w * h) as u64,
            diff_pixels: (w * h) as u64,
            baseline_path: Some(baseline.display().to_string()),
            actual_path: actual.display().to_string(),
            diff_image_path: None,
            width: w,
            height: h,
        });
    }

    let baseline_rgba = baseline_img.to_rgba8();
    let actual_rgba = actual_img.to_rgba8();

    let total_pixels = (w * h) as u64;
    let mut diff_count: u64 = 0;

    let mut diff_img = ImageBuffer::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let bp = baseline_rgba.get_pixel(x, y);
            let ap = actual_rgba.get_pixel(x, y);

            if bp != ap {
                diff_count += 1;
                let alpha_f = 128u16;
                let inv_alpha = 256u16 - alpha_f;
                let blended = Rgba([
                    ((ap[0] as u16 * inv_alpha + 255u16 * alpha_f) >> 8) as u8,
                    ((ap[1] as u16 * inv_alpha) >> 8) as u8,
                    ((ap[2] as u16 * inv_alpha) >> 8) as u8,
                    255u8,
                ]);
                diff_img.put_pixel(x, y, blended);
            } else {
                diff_img.put_pixel(x, y, *ap);
            }
        }
    }

    let ratio = if total_pixels > 0 {
        diff_count as f32 / total_pixels as f32
    } else {
        0.0
    };

    let passed = ratio <= config.tolerance;

    let diff_image_path = if !passed || config.generate_html {
        let diff_filename = format!("{}_diff.png", name);
        let diff_path = config.output_dir.join(&diff_filename);
        fs::create_dir_all(&config.output_dir)?;
        diff_img
            .save(&diff_path)
            .with_context(|| format!("Failed to save diff image: {}", diff_path.display()))?;
        Some(diff_path.display().to_string())
    } else {
        None
    };

    Ok(DiffResult {
        name,
        passed,
        pixel_diff_ratio: ratio,
        total_pixels,
        diff_pixels: diff_count,
        baseline_path: Some(baseline.display().to_string()),
        actual_path: actual.display().to_string(),
        diff_image_path,
        width: w,
        height: h,
    })
}

pub fn decode_screenshot(base64_data: &str, output: &Path) -> Result<()> {
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64_data)
        .context("Failed to decode base64 screenshot data")?;
    fs::create_dir_all(
        output
            .parent()
            .context("Output path has no parent directory")?,
    )?;
    fs::write(output, bytes)?;
    Ok(())
}

pub fn run_visual_diff(actual_images: &[PathBuf], config: &DiffConfig) -> Result<DiffReport> {
    fs::create_dir_all(&config.output_dir)?;
    fs::create_dir_all(&config.baseline_dir)?;

    let mut results = Vec::new();

    for actual_path in actual_images {
        let name = actual_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let baseline_path = config
            .baseline_dir
            .join(actual_path.file_name().context("Missing filename")?);

        if !baseline_path.exists() {
            results.push(DiffResult {
                name: name.clone(),
                passed: false,
                pixel_diff_ratio: 1.0,
                total_pixels: 0,
                diff_pixels: 0,
                baseline_path: None,
                actual_path: actual_path.display().to_string(),
                diff_image_path: None,
                width: 0,
                height: 0,
            });
            eprintln!("  MISSING BASELINE: {} (copy to create baseline)", name);
            continue;
        }

        match compare_images(&baseline_path, actual_path, config) {
            Ok(result) => {
                let status = if result.passed { "PASS" } else { "FAIL" };
                eprintln!(
                    "  {}: {} ({:.4}% diff, {}/{})",
                    result.name,
                    status,
                    result.pixel_diff_ratio * 100.0,
                    result.diff_pixels,
                    result.total_pixels
                );
                results.push(result);
            }
            Err(e) => {
                eprintln!("  ERROR: {} - {}", name, e);
                results.push(DiffResult {
                    name,
                    passed: false,
                    pixel_diff_ratio: 1.0,
                    total_pixels: 0,
                    diff_pixels: 0,
                    baseline_path: Some(baseline_path.display().to_string()),
                    actual_path: actual_path.display().to_string(),
                    diff_image_path: None,
                    width: 0,
                    height: 0,
                });
            }
        }
    }

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;

    let report = DiffReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        tolerance: config.tolerance,
        total: results.len(),
        passed,
        failed,
        results: results.clone(),
    };

    if config.generate_html {
        generate_html_report(&report, config)?;
    }

    let json_path = config.output_dir.join("report.json");
    fs::write(&json_path, serde_json::to_string_pretty(&report)?)
        .with_context(|| format!("Failed to write report: {}", json_path.display()))?;

    Ok(report)
}

fn generate_html_report(report: &DiffReport, config: &DiffConfig) -> Result<()> {
    let html = build_html_report(report, config);
    let path = config.output_dir.join("index.html");
    fs::write(&path, html)?;
    Ok(())
}

fn build_html_report(report: &DiffReport, _config: &DiffConfig) -> String {
    let _status_class = if report.failed == 0 { "pass" } else { "fail" };
    let rows: Vec<String> = report.results.iter().map(|r| {
        let cls = if r.passed { "pass" } else { "fail" };
        let diff_img = match &r.diff_image_path {
            Some(p) => {
                let p_rel = Path::new(p).file_name()
                    .and_then(|n| n.to_str()).unwrap_or("");
                format!(
                    r#"<div class="slider-wrap"><div class="slider"><img src="{}" class="diff-img"/></div></div>"#,
                    p_rel
                )
            }
            None => "<span class=\"no-diff\">No diff image</span>".into(),
        };
        format!(
            r#"<tr class="{}">
      <td>{}</td>
      <td>{}</td>
      <td>{:.4}%</td>
      <td>{}</td>
      <td>{}</td>
      <td class="diff-cell">{}</td>
    </tr>"#,
            cls,
            html_escape(&r.name),
            if r.passed { "&#10004; PASS" } else { "&#10008; FAIL" },
            r.pixel_diff_ratio * 100.0,
            r.diff_pixels,
            r.total_pixels,
            diff_img
        )
    }).collect();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"/>
<title>Visual Regression Report</title>
<style>
  body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 20px; background: #f8f9fa; color: #333; }}
  h1 {{ margin-bottom: 4px; }}
  .meta {{ color: #666; margin-bottom: 16px; }}
  table {{ border-collapse: collapse; width: 100%; background: white; box-shadow: 0 1px 3px rgba(0,0,0,.12); }}
  th {{ background: #555; color: white; padding: 10px 14px; text-align: left; }}
  td {{ padding: 8px 14px; border-bottom: 1px solid #eee; vertical-align: middle; }}
  tr:hover {{ background: #fafafa; }}
  tr.pass td:first-child {{ border-left: 3px solid #28a745; }}
  tr.fail td:first-child {{ border-left: 3px solid #dc3545; }}
  .pass {{ color: #28a745; }} .fail {{ color: #dc3545; }}
  .summary {{ display: inline-flex; gap: 24px; margin: 16px 0; font-size: 15px; }}
  .summary strong {{ font-size: 18px; }}
  .slider-wrap {{ max-width: 400px; overflow-x: auto; }}
  .slider img {{ max-width: 360px; height: auto; border: 1px solid #ddd; }}
  .diff-cell {{ min-width: 380px; }}
  .no-diff {{ color: #999; font-style: italic; }}
  .badge {{ display: inline-block; padding: 2px 10px; border-radius: 12px; font-size: 13px; font-weight: 600; }}
  .badge-pass {{ background: #d4edda; color: #155724; }}
  .badge-fail {{ background: #f8d7da; color: #721c24; }}
</style>
</head>
<body>
<h1>Visual Regression Report</h1>
<p class="meta">{}</p>
<div class="summary">
  <span>Total: <strong>{}</strong></span>
  <span class="pass">Passed: <strong>{}</strong></span>
  <span class="fail">Failed: <strong>{}</strong></span>
  <span>Tolerance: <strong>{:.1}%</strong></span>
</div>
<table>
  <thead><tr><th>Name</th><th>Status</th><th>Diff Ratio</th><th>Different Pixels</th><th>Total Pixels</th><th>Diff Image</th></tr></thead>
  <tbody>{}</tbody>
</table>
</body>
</html>"#,
        report.timestamp,
        report.total,
        report.passed,
        report.failed,
        report.tolerance * 100.0,
        rows.join("\n")
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn update_baseline(actual_images: &[PathBuf], baseline_dir: &Path) -> Result<usize> {
    fs::create_dir_all(baseline_dir)?;
    let mut count = 0;
    for path in actual_images {
        let filename = path.file_name().context("Missing filename")?;
        let dest = baseline_dir.join(filename);
        fs::copy(path, &dest)?;
        count += 1;
    }
    Ok(count)
}
