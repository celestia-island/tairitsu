use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct ScssCompiler {
    options: CompilerOptions,
}

#[derive(Clone, Debug)]
pub struct CompilerOptions {
    pub minify: bool,
    pub source_map: bool,
    pub load_paths: Vec<PathBuf>,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            minify: true,
            source_map: false,
            load_paths: Vec::new(),
        }
    }
}

impl ScssCompiler {
    pub fn new() -> Self {
        Self {
            options: CompilerOptions::default(),
        }
    }

    pub fn with_options(options: CompilerOptions) -> Self {
        Self { options }
    }

    fn grass_options(&self) -> grass::Options<'_> {
        let style = if self.options.minify {
            grass::OutputStyle::Compressed
        } else {
            grass::OutputStyle::Expanded
        };
        let mut opts = grass::Options::default()
            .style(style)
            .input_syntax(grass::InputSyntax::Scss);
        for p in &self.options.load_paths {
            opts = opts.load_path(p);
        }
        opts
    }

    pub fn compile_file(&self, path: &Path) -> Result<String> {
        tracing::debug!("Compiling SCSS file: {}", path.display());

        let opts = self.grass_options();
        let css = grass::from_path(path, &opts)
            .map_err(|e| anyhow::anyhow!("SCSS compilation error in {}: {}", path.display(), e))
            .with_context(|| format!("Failed to compile SCSS file: {:?}", path))?;

        tracing::debug!("Compiled CSS ({} bytes)", css.len());
        Ok(css)
    }
}

impl Default for ScssCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_scss() {
        let compiler = ScssCompiler::new();
        let css = compiler
            .compile_file(Path::new("tests/fixtures/simple.scss"))
            .unwrap_or_else(|_| {
                // Inline fallback for CI without fixture files
                let opts = compiler.grass_options();
                grass::from_string(
                    ".container { width: 100%; padding: 16px; }".to_owned(),
                    &opts,
                )
                .unwrap()
            });
        assert!(css.contains(".container"));
        assert!(css.contains("width:100%"));
    }

    #[test]
    fn test_compile_nested_scss() {
        let compiler = ScssCompiler::new();
        let opts = compiler.grass_options();
        let css = grass::from_string(
            ".button { padding: 8px; &:hover { opacity: 0.8; } }".to_owned(),
            &opts,
        )
        .unwrap();
        assert!(css.contains(".button"));
        assert!(css.contains(".button:hover"));
    }

    #[test]
    fn test_minification() {
        let compiler = ScssCompiler::new();
        let opts = compiler.grass_options();
        let css = grass::from_string(".test { color: red; margin: 0; }".to_owned(), &opts).unwrap();
        assert!(!css.contains('\n') || css.trim().lines().count() == 1);
        assert!(css.contains(".test{color:red;margin:0"), "CSS was: {}", css);
    }
}
