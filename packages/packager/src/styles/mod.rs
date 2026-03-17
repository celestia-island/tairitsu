mod compiler;
mod extractor;
mod injector;

use anyhow::Result;

pub use compiler::ScssCompiler;
pub use extractor::CssExtractor;
pub use injector::StyleInjector;

pub struct ScssBuildResult {
    pub css: String,
    pub source_map: Option<String>,
}

pub fn compile_scss_files(
    input_dir: &std::path::Path,
    output_dir: &std::path::Path,
) -> Result<ScssBuildResult> {
    let compiler = ScssCompiler::new();
    let extractor = CssExtractor::new();

    let scss_files = find_scss_files(input_dir)?;
    let mut all_css = String::new();

    for scss_file in scss_files {
        let css = compiler.compile_file(&scss_file)?;
        all_css.push_str(&css);
        all_css.push('\n');
    }

    let optimized_css = extractor.optimize(&all_css)?;

    std::fs::create_dir_all(output_dir)?;
    let output_path = output_dir.join("styles.css");
    std::fs::write(&output_path, &optimized_css)?;

    Ok(ScssBuildResult {
        css: optimized_css,
        source_map: None,
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
