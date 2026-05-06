use anyhow::Result;

pub struct StyleInjector;

impl StyleInjector {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_injection_code(&self, css: &str) -> Result<String> {
        let code = format!(
            r#"
(function() {{
    const style = document.createElement('style');
    style.type = 'text/css';
    style.textContent = `{}`;
    document.head.appendChild(style);
}})();
"#,
            css.replace('`', "\\`").replace('$', "\\$")
        );

        Ok(code)
    }

    pub fn generate_cssom_injection(&self, css: &str) -> Result<String> {
        let rules = self.parse_css_rules(css);

        let mut code = String::from("(function() {\n");
        code.push_str("  const style = document.createElement('style');\n");
        code.push_str("  const sheet = style.sheet;\n");
        code.push_str("  \n");

        for rule in rules {
            code.push_str(&format!(
                "  try {{ sheet.insertRule('{}', sheet.cssRules.length); }} catch(e) {{}}\n",
                rule.replace('\'', "\\'")
            ));
        }

        code.push_str("  document.head.appendChild(style);\n");
        code.push_str("})();\n");

        Ok(code)
    }

    fn parse_css_rules(&self, css: &str) -> Vec<String> {
        css.split('}')
            .filter(|s| !s.trim().is_empty())
            .map(|s| format!("{} }}", s.trim()))
            .collect()
    }

    pub fn inject_into_html(&self, html: &str, css: &str) -> Result<String> {
        let injection = self.generate_injection_code(css)?;

        if let Some(head_pos) = html.find("</head>") {
            let mut result = String::with_capacity(html.len() + injection.len());
            result.push_str(&html[..head_pos]);
            result.push_str("<script>");
            result.push_str(&injection);
            result.push_str("</script>");
            result.push_str(&html[head_pos..]);
            Ok(result)
        } else {
            Ok(html.to_string())
        }
    }

    pub fn generate_style_block(&self, css: &str) -> String {
        format!("<style>\n{}\n</style>", css)
    }

    pub fn inject_style_into_html(&self, html: &str, css: &str) -> Result<String> {
        let style_block = self.generate_style_block(css);

        if let Some(head_pos) = html.find("</head>") {
            let mut result = String::with_capacity(html.len() + style_block.len());
            result.push_str(&html[..head_pos]);
            result.push_str(&style_block);
            result.push_str(&html[head_pos..]);
            Ok(result)
        } else {
            Ok(html.to_string())
        }
    }
}

impl Default for StyleInjector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_injection_code() {
        let injector = StyleInjector::new();
        let css = ".test { color: red; }";

        let code = injector.generate_injection_code(css).unwrap();
        assert!(code.contains("createElement('style')"));
        assert!(code.contains("document.head.appendChild"));
        assert!(code.contains(".test { color: red; }"));
    }

    #[test]
    fn test_inject_into_html() {
        let injector = StyleInjector::new();
        let html = r#"<!DOCTYPE html><html><head></head><body></body></html>"#;
        let css = ".test { color: red; }";

        let result = injector.inject_into_html(html, css).unwrap();
        assert!(result.contains("<script>"));
        assert!(result.contains("</script>"));
        assert!(result.contains("</head>"));
        assert!(result.contains("document.head.appendChild(style)"));
    }

    #[test]
    fn test_inject_into_html_without_head() {
        let injector = StyleInjector::new();
        let html = r#"<!DOCTYPE html><html><body>No head tag</body></html>"#;
        let css = ".test { color: red; }";

        let result = injector.inject_into_html(html, css).unwrap();
        // Should return original HTML unchanged
        assert_eq!(result, html);
    }

    #[test]
    fn test_inject_into_html_with_existing_head_content() {
        let injector = StyleInjector::new();
        let html =
            r#"<!DOCTYPE html><html><head><meta charset="UTF-8"></head><body></body></html>"#;
        let css = "body { margin: 0; }";

        let result = injector.inject_into_html(html, css).unwrap();
        assert!(result.contains("<meta charset=\"UTF-8\">"));
        assert!(result.contains("<script>"));
        assert!(result.contains("</head>"));
        // Verify script is inserted before </head>
        let head_close_pos = result.find("</head>").unwrap();
        let script_open_pos = result.find("<script>").unwrap();
        assert!(script_open_pos < head_close_pos);
    }

    #[test]
    fn test_generate_cssom_injection() {
        let injector = StyleInjector::new();
        let css = ".test { color: red; }\n.body { margin: 0; }";

        let code = injector.generate_cssom_injection(css).unwrap();
        assert!(code.contains("createElement('style')"));
        assert!(code.contains("sheet.insertRule"));
        assert!(code.contains("document.head.appendChild"));
    }

    #[test]
    fn test_parse_css_rules() {
        let injector = StyleInjector::new();
        let css = ".test { color: red; } .body { margin: 0; }";

        let rules = injector.parse_css_rules(css);
        assert_eq!(rules.len(), 2);
        assert!(rules[0].contains(".test"));
        assert!(rules[1].contains(".body"));
    }

    #[test]
    fn test_parse_css_rules_empty() {
        let injector = StyleInjector::new();
        let css = "";

        let rules = injector.parse_css_rules(css);
        assert!(rules.is_empty());
    }

    #[test]
    fn test_parse_css_rules_with_whitespace() {
        let injector = StyleInjector::new();
        let css = "   .test { color: red; }    \n\n   .body { margin: 0; }   ";

        let rules = injector.parse_css_rules(css);
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_generate_injection_code_with_backticks() {
        let injector = StyleInjector::new();
        let css = ".test { content: '`backtick`'; }";

        let code = injector.generate_injection_code(css).unwrap();
        // Backticks should be escaped
        assert!(code.contains("\\`"));
    }

    #[test]
    fn test_generate_injection_code_with_dollar_signs() {
        let injector = StyleInjector::new();
        let css = ".test { content: '$100'; }";

        let code = injector.generate_injection_code(css).unwrap();
        // Dollar signs should be escaped
        assert!(code.contains("\\$"));
    }

    #[test]
    fn test_generate_cssom_injection_with_single_quotes() {
        let injector = StyleInjector::new();
        let css = ".test { content: 'quoted'; }";

        let code = injector.generate_cssom_injection(css).unwrap();
        // Single quotes in CSS should be escaped in the insertRule call
        assert!(code.contains("\\'"));
    }

    #[test]
    fn test_style_injector_default() {
        let injector = StyleInjector;
        let css = ".test { color: blue; }";

        let code = injector.generate_injection_code(css).unwrap();
        assert!(code.contains("createElement('style')"));
    }

    #[test]
    fn test_inject_complex_css() {
        let injector = StyleInjector::new();
        let css = r#"
            .container {
                display: flex;
                justify-content: center;
            }
            @media (max-width: 768px) {
                .container { flex-direction: column; }
            }
        "#;

        let code = injector.generate_injection_code(css).unwrap();
        assert!(code.contains("display: flex"));
        assert!(code.contains("@media"));
    }

    #[test]
    fn test_inject_into_html_multiple_times() {
        let injector = StyleInjector::new();
        let html = "<html><head></head><body></body></html>";
        let css1 = ".a { color: red; }";
        let css2 = ".b { color: blue; }";

        let result1 = injector.inject_into_html(html, css1).unwrap();
        let result2 = injector.inject_into_html(&result1, css2).unwrap();

        // Both styles should be injected
        assert!(result2.contains(".a { color: red; }"));
        assert!(result2.contains(".b { color: blue; }"));
    }

    #[test]
    fn test_generate_style_block() {
        let injector = StyleInjector::new();
        let css = ".test { color: red; }";

        let block = injector.generate_style_block(css);
        assert!(block.starts_with("<style>"));
        assert!(block.ends_with("</style>"));
        assert!(block.contains(".test { color: red; }"));
    }

    #[test]
    fn test_generate_style_block_empty() {
        let injector = StyleInjector::new();
        let block = injector.generate_style_block("");
        assert_eq!(block, "<style>\n\n</style>");
    }

    #[test]
    fn test_inject_style_into_html() {
        let injector = StyleInjector::new();
        let html =
            r#"<!DOCTYPE html><html><head><meta charset="UTF-8"></head><body></body></html>"#;
        let css = "body { margin: 0; }";

        let result = injector.inject_style_into_html(html, css).unwrap();
        assert!(result.contains("<style>"));
        assert!(result.contains("</style>"));
        assert!(result.contains("</head>"));
        assert!(result.contains("body { margin: 0; }"));

        let head_close_pos = result.find("</head>").unwrap();
        let style_open_pos = result.find("<style>").unwrap();
        assert!(style_open_pos < head_close_pos);
    }

    #[test]
    fn test_inject_style_into_html_without_head() {
        let injector = StyleInjector::new();
        let html = r#"<!DOCTYPE html><html><body>No head tag</body></html>"#;
        let css = ".test { color: red; }";

        let result = injector.inject_style_into_html(html, css).unwrap();
        assert_eq!(result, html);
    }

    #[test]
    fn test_inject_style_into_html_with_complex_css() {
        let injector = StyleInjector::new();
        let html = "<html><head></head><body></body></html>";
        let css = r#"
            .container { display: flex; }
            @media (max-width: 768px) { .container { flex-direction: column; } }
        "#;

        let result = injector.inject_style_into_html(html, css).unwrap();
        assert!(result.contains("display: flex"));
        assert!(result.contains("@media"));
    }

    #[test]
    fn test_inject_style_into_html_multiple_css() {
        let injector = StyleInjector::new();
        let html = "<html><head></head><body></body></html>";
        let css1 = ".a { color: red; }";
        let css2 = ".b { color: blue; }";

        let result1 = injector.inject_style_into_html(html, css1).unwrap();
        let result2 = injector.inject_style_into_html(&result1, css2).unwrap();

        assert!(result2.contains("<style>"));
        assert!(result2.contains(".a { color: red; }"));
        assert!(result2.contains(".b { color: blue; }"));
    }

    #[test]
    fn test_inject_style_vs_script_injection() {
        let injector = StyleInjector::new();
        let html = "<html><head></head><body></body></html>";
        let css = ".test { color: red; }";

        let style_result = injector.inject_style_into_html(html, css).unwrap();
        let script_result = injector.inject_into_html(html, css).unwrap();

        assert!(style_result.contains("<style>"));
        assert!(!style_result.contains("<script>"));
        assert!(script_result.contains("<script>"));
        assert!(!script_result.contains("<style>"));
    }
}
