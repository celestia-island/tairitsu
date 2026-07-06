use std::{io::Write, path::Path, process::Command};

use super::sources::IconSourceDef;

pub fn generate_woff_subset(
    source: &IconSourceDef,
    icon_names: &[String],
    extracted_dir: &Path,
    output_dir: &Path,
) -> crate::Result<()> {
    let font_file = match source.font_file {
        Some(f) => f,
        None => {
            return Err(crate::TairitsuPackagerError::IconFetchError(format!(
                "Icon set '{}' has no font file configured",
                source.name
            )))
        }
    };

    let font_family = source.font_family.unwrap_or(source.name);

    let source_font = extracted_dir.join(font_file);
    if !source_font.exists() {
        return Err(crate::TairitsuPackagerError::IconFetchError(format!(
            "Font file not found: {}",
            source_font.display()
        )));
    }

    std::fs::create_dir_all(output_dir)?;

    let woff2_output = output_dir.join(format!("{}.woff2", source.name));

    let mut codepoints = Vec::new();
    for name in icon_names {
        if let Some(cp) = lookup_codepoint(source, extracted_dir, name) {
            codepoints.push(cp);
        }
    }

    if codepoints.is_empty() {
        crate::log_warn!(
            "No codepoints found for '{}' — skipping woff generation",
            source.name
        );
        return Ok(());
    }

    crate::log_info!(
        "Generating woff2 subset for '{}' with {} glyphs...",
        source.name,
        codepoints.len()
    );

    let unicodes_arg = codepoints.join(",");

    let status = Command::new("hb-subset")
        .args([
            "--unicodes",
            &unicodes_arg,
            "--output-file",
            &woff2_output.to_string_lossy(),
            "--output-format=woff2",
            &source_font.to_string_lossy(),
        ])
        .status()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                crate::TairitsuPackagerError::IconFetchError(
                    "hb-subset not found. Install: apt install harfbuzz / brew install harfbuzz"
                        .to_string(),
                )
            } else {
                crate::TairitsuPackagerError::IconFetchError(format!(
                    "Failed to run hb-subset: {}",
                    e
                ))
            }
        })?;

    if !status.success() {
        return Err(crate::TairitsuPackagerError::IconFetchError(
            "hb-subset failed".to_string(),
        ));
    }

    let css_path = output_dir.join(format!("{}.css", source.name));
    let mut css = std::fs::File::create(&css_path)?;
    write_font_css(&mut css, source.name, font_family, &woff2_output)?;

    crate::log_info!(
        "Generated {} ({} glyphs) + {}",
        woff2_output.display(),
        codepoints.len(),
        css_path.display()
    );

    Ok(())
}

fn lookup_codepoint(
    source: &IconSourceDef,
    extracted_dir: &Path,
    icon_name: &str,
) -> Option<String> {
    if source.name == "mdi" {
        let meta_path = extracted_dir.join("meta.json");
        if meta_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&meta_path) {
                if let Ok(entries) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
                    for entry in &entries {
                        if entry.get("name").and_then(|n| n.as_str()) == Some(icon_name) {
                            if let Some(cp) = entry.get("codepoint").and_then(|c| c.as_str()) {
                                return Some(format!("U+{}", cp));
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn write_font_css(
    f: &mut std::fs::File,
    set_name: &str,
    font_family: &str,
    woff2_path: &Path,
) -> std::io::Result<()> {
    let filename = woff2_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| format!("{}.woff2", set_name));

    writeln!(f, "@font-face {{")?;
    writeln!(f, "  font-family: '{}';", font_family)?;
    writeln!(f, "  src: url('{}') format('woff2');", filename)?;
    writeln!(f, "  font-weight: normal;")?;
    writeln!(f, "  font-style: normal;")?;
    writeln!(f, "  font-display: block;")?;
    writeln!(f, "}}")?;
    writeln!(f)?;
    writeln!(f, ".hikari-icon-font[data-set=\"{}\"] {{", set_name)?;
    writeln!(f, "  font-family: '{}', sans-serif;", font_family)?;
    writeln!(f, "}}")
}

pub fn is_hb_subset_available() -> bool {
    Command::new("hb-subset")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
