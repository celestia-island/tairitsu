use anyhow::Result;
use std::collections::HashSet;

pub struct CssExtractor {
    options: ExtractorOptions,
}

#[derive(Clone, Debug)]
pub struct ExtractorOptions {
    pub remove_duplicates: bool,
    pub sort_properties: bool,
    pub remove_unused: bool,
}

impl Default for ExtractorOptions {
    fn default() -> Self {
        Self {
            remove_duplicates: true,
            sort_properties: false,
            remove_unused: false,
        }
    }
}

impl CssExtractor {
    pub fn new() -> Self {
        Self {
            options: ExtractorOptions::default(),
        }
    }

    pub fn with_options(options: ExtractorOptions) -> Self {
        Self { options }
    }

    pub fn optimize(&self, css: &str) -> Result<String> {
        let mut optimized = css.to_string();

        if self.options.remove_duplicates {
            optimized = self.remove_duplicate_rules(&optimized);
        }

        if self.options.sort_properties {
            optimized = self.sort_css_properties(&optimized);
        }

        optimized = self.clean_whitespace(&optimized);

        Ok(optimized)
    }

    fn remove_duplicate_rules(&self, css: &str) -> String {
        let mut seen_rules: HashSet<String> = HashSet::new();
        let mut result = String::new();

        for rule in css.split('}') {
            let trimmed = rule.trim();
            if trimmed.is_empty() {
                continue;
            }

            let normalized = self.normalize_rule(trimmed);

            if seen_rules.insert(normalized) {
                result.push_str(trimmed);
                result.push('}');
            }
        }

        result
    }

    fn normalize_rule(&self, rule: &str) -> String {
        let mut normalized: Vec<String> = rule
            .split(';')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        normalized.sort();
        normalized.join(";")
    }

    fn sort_css_properties(&self, css: &str) -> String {
        let mut result = String::new();

        for rule in css.split('}') {
            let trimmed = rule.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(brace_pos) = trimmed.find('{') {
                let selector = &trimmed[..brace_pos];
                let properties = &trimmed[brace_pos + 1..];

                let mut props: Vec<&str> = properties
                    .split(';')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();

                props.sort();

                result.push_str(selector);
                result.push('{');
                result.push_str(&props.join(";"));
                result.push_str("};");
            }
        }

        result
    }

    fn clean_whitespace(&self, css: &str) -> String {
        // Normalize whitespace: collapse multiple whitespace to single space
        // Keep spaces around CSS punctuation for readability
        let mut result = String::new();
        let mut prev_was_space = false;

        for ch in css.chars() {
            if ch.is_whitespace() {
                prev_was_space = true;
            } else {
                if prev_was_space {
                    // Only add space if needed for readability (not before/after punctuation)
                    if !result.is_empty() {
                        let last = result.chars().last().unwrap_or(' ');
                        if !Self::is_css_punctuation(last) && !Self::is_css_punctuation(ch) {
                            result.push(' ');
                        }
                    }
                    prev_was_space = false;
                }
                result.push(ch);
            }
        }

        result
    }

    fn is_css_punctuation(ch: char) -> bool {
        matches!(ch, '{' | '}' | ':' | ';' | ',')
    }

    pub fn extract_from_rust_source(&self, source: &str) -> Result<Vec<String>> {
        let mut styles = Vec::new();

        let scss_macro_pattern = regex::Regex::new(r#"scss!\s*\(\s*"([^"]+)"\s*\)"#)?;

        for cap in scss_macro_pattern.captures_iter(source) {
            if let Some(scss_content) = cap.get(1) {
                styles.push(scss_content.as_str().to_string());
            }
        }

        Ok(styles)
    }
}

impl Default for CssExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_duplicates() {
        let extractor = CssExtractor::new();
        let css = r#"
            .button { color: red; }
            .button { color: red; }
            .container { width: 100%; }
        "#;

        let optimized = extractor.optimize(css).unwrap();
        assert!(optimized.matches(".button").count() == 1);
    }

    #[test]
    fn test_clean_whitespace() {
        let extractor = CssExtractor::new();
        let css = ".test  {  color  :  red  ;  }";

        let optimized = extractor.optimize(css).unwrap();
        // Verify whitespace is cleaned and structure is preserved
        assert!(
            !optimized.contains("  "),
            "Should not have double spaces: {}",
            optimized
        );
        assert!(
            optimized.contains(".test"),
            "Should contain selector: {}",
            optimized
        );
        assert!(
            optimized.contains("color"),
            "Should contain property: {}",
            optimized
        );
        assert!(
            optimized.contains("red"),
            "Should contain value: {}",
            optimized
        );
    }
}
