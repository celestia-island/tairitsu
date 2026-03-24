mod compiler;
mod extractor;
mod injector;

use anyhow::Result;
use std::path::{Path, PathBuf};

pub use compiler::{CompilerOptions, ScssCompiler};
pub use extractor::CssExtractor;
pub use injector::StyleInjector;

use crate::config::ScssConfig;

pub struct ScssBuildResult {
    pub css: String,
    pub source_map: Option<String>,
    pub output_path: PathBuf,
}

/// Compile SCSS files based on configuration
pub fn compile_scss_with_config(
    config: &ScssConfig,
    project_root: &Path,
    output_dir: &Path,
) -> Result<Vec<ScssBuildResult>> {
    // Resolve load_paths relative to project root
    let resolved_load_paths: Vec<PathBuf> = config
        .load_paths
        .iter()
        .map(|p| {
            let path = project_root.join(p);
            std::fs::canonicalize(&path).unwrap_or(path)
        })
        .collect();

    tracing::info!("SCSS resolved load_paths: {:?}", resolved_load_paths);

    let compiler = ScssCompiler::with_options(compiler::CompilerOptions {
        minify: true,
        source_map: false,
        load_paths: resolved_load_paths,
    });
    std::fs::create_dir_all(output_dir)?;

    let mut results = Vec::new();

    // Collect entries to process
    tracing::info!(
        "SCSS config: {} entries, load_paths: {:?}",
        config.entries.len(),
        config.load_paths
    );

    let entries = if !config.entries.is_empty() {
        // Use explicit multi-entry configuration
        tracing::info!("Using multi-entry configuration");
        config.entries.clone()
    } else if let Some(entry) = &config.entry {
        // Use single entry configuration
        vec![crate::config::ScssEntry {
            entry: entry.clone(),
            output: config.output.clone(),
        }]
    } else {
        // Fallback: discover src/styles/ directory
        let styles_dir = project_root.join("src").join("styles");
        if styles_dir.exists() {
            let result = compile_scss_directory(&styles_dir, output_dir, &compiler)?;
            if let Some(r) = result {
                results.push(r);
            }
        }
        return Ok(results);
    };

    // Process each entry
    for entry in entries {
        tracing::info!("Processing SCSS entry: {} -> {}", entry.entry, entry.output);
        let entry_path = project_root.join(&entry.entry);

        if !entry_path.exists() {
            tracing::warn!("SCSS entry not found: {}", entry_path.display());
            continue;
        }

        let css = if entry_path.is_dir() {
            // Compile directory
            let scss_files = find_scss_files(&entry_path)?;
            let mut all_css = String::new();
            for scss_file in scss_files {
                match compiler.compile_file(&scss_file) {
                    Ok(css) => {
                        all_css.push_str(&css);
                        all_css.push('\n');
                    }
                    Err(e) => {
                        tracing::warn!("Failed to compile {}: {}", scss_file.display(), e);
                    }
                }
            }
            all_css
        } else {
            // Compile single file
            match compiler.compile_file(&entry_path) {
                Ok(css) => css,
                Err(e) => {
                    tracing::warn!("Failed to compile {}: {}", entry_path.display(), e);
                    continue;
                }
            }
        };

        // Grass already produces minified/optimized CSS
        let output_path = output_dir.join(&entry.output);

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&output_path, &css)?;

        results.push(ScssBuildResult {
            css,
            source_map: None,
            output_path: output_path.clone(),
        });

        tracing::info!(
            "Compiled SCSS: {} -> {}",
            entry.entry,
            output_path.display()
        );
    }

    Ok(results)
}

/// Compile all SCSS files in a directory to a single output
fn compile_scss_directory(
    input_dir: &Path,
    output_dir: &Path,
    compiler: &ScssCompiler,
) -> Result<Option<ScssBuildResult>> {
    let scss_files = find_scss_files(input_dir)?;

    if scss_files.is_empty() {
        return Ok(None);
    }

    let mut all_css = String::new();
    for scss_file in scss_files {
        let css = compiler.compile_file(&scss_file)?;
        all_css.push_str(&css);
        all_css.push('\n');
    }

    let output_path = output_dir.join("styles.css");
    std::fs::write(&output_path, &all_css)?;

    Ok(Some(ScssBuildResult {
        css: all_css,
        source_map: None,
        output_path,
    }))
}

/// Legacy function for backward compatibility
pub fn compile_scss_files(
    input_dir: &std::path::Path,
    output_dir: &std::path::Path,
) -> Result<ScssBuildResult> {
    let compiler = ScssCompiler::new();

    let scss_files = find_scss_files(input_dir)?;
    let mut all_css = String::new();

    for scss_file in scss_files {
        let css = compiler.compile_file(&scss_file)?;
        all_css.push_str(&css);
        all_css.push('\n');
    }

    std::fs::create_dir_all(output_dir)?;
    let output_path = output_dir.join("styles.css");
    std::fs::write(&output_path, &all_css)?;

    Ok(ScssBuildResult {
        css: all_css,
        source_map: None,
        output_path,
    })
}

fn find_scss_files(dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();

    if !dir.exists() {
        return Ok(files);
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let mut sub_files = find_scss_files(&path)?;
            files.append(&mut sub_files);
        } else if path.extension().map(|e| e == "scss").unwrap_or(false) {
            files.push(path);
        }
    }

    Ok(files)
}
