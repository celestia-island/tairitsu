use anyhow::{Context, Result};
use std::path::Path;

pub struct ScssCompiler {
    options: CompilerOptions,
}

#[derive(Clone, Debug)]
pub struct CompilerOptions {
    pub minify: bool,
    pub source_map: bool,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            minify: true,
            source_map: false,
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

    pub fn compile(&self, scss: &str) -> Result<String> {
        let css = self.parse_scss(scss)?;

        let css = if self.options.minify {
            self.minify_css(&css)
        } else {
            css
        };

        Ok(css)
    }

    pub fn compile_file(&self, path: &Path) -> Result<String> {
        let scss = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read SCSS file: {:?}", path))?;

        self.compile(&scss)
    }

    fn parse_scss(&self, scss: &str) -> Result<String> {
        let mut css = String::new();
        let mut selectors: Vec<String> = Vec::new();
        let mut in_comment = false;

        for line in scss.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("/*") {
                in_comment = true;
                continue;
            }

            if in_comment {
                if trimmed.ends_with("*/") {
                    in_comment = false;
                }
                continue;
            }

            if trimmed.starts_with("//") || trimmed.is_empty() {
                continue;
            }

            if trimmed.ends_with('{') {
                let selector = trimmed.trim_end_matches('{').trim();
                let full_selector = self.build_selector(&selectors, selector);
                selectors.push(selector.to_string());
                css.push_str(&format!("{} {{\n", full_selector));
            } else if trimmed == "}" {
                selectors.pop();
                css.push_str("}\n");
            } else if trimmed.contains(':') {
                css.push_str(&format!("  {};\n", trimmed));
            }
        }

        Ok(css)
    }

    fn build_selector(&self, parent_selectors: &[String], selector: &str) -> String {
        if parent_selectors.is_empty() {
            selector.to_string()
        } else if selector.starts_with('&') {
            let parent = parent_selectors.join(" ");
            selector.replacen('&', &parent, 1)
        } else {
            format!("{} {}", parent_selectors.join(" "), selector)
        }
    }

    fn minify_css(&self, css: &str) -> String {
        css.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .replace("  ", " ")
            .replace("{ ", "{")
            .replace(" }", "}")
            .replace("; ", ";")
            .replace(": ", ":")
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
        let scss = r#"
            .container {
                width: 100%;
                padding: 16px;
            }
        "#;

        let css = compiler.compile(scss).unwrap();
        assert!(css.contains(".container"));
        assert!(css.contains("width:100%"));
    }

    #[test]
    fn test_compile_nested_scss() {
        let compiler = ScssCompiler::new();
        let scss = r#"
            .button {
                padding: 8px;
                
                &:hover {
                    opacity: 0.8;
                }
            }
        "#;

        let css = compiler.compile(scss).unwrap();
        assert!(css.contains(".button"));
        assert!(css.contains(".button:hover"));
    }

    #[test]
    fn test_minification() {
        let compiler = ScssCompiler::new();
        let scss = r#"
            .test {
                color: red;
                margin: 0;
            }
        "#;

        let css = compiler.compile(scss).unwrap();
        assert!(!css.contains('\n'));
        assert!(css.contains(".test{color:red;margin:0}"));
    }
}
