use anyhow::Result;

pub struct StyleInjector {
    inject_styles: bool,
}

impl StyleInjector {
    pub fn new() -> Self {
        Self {
            inject_styles: true,
        }
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
        assert!(code.contains(css));
    }

    #[test]
    fn test_inject_into_html() {
        let injector = StyleInjector::new();
        let html = r#"<!DOCTYPE html><html><head></head><body></body></html>"#;
        let css = ".test { color: red; }";

        let result = injector.inject_into_html(html, css).unwrap();
        assert!(result.contains("<script>"));
        assert!(result.contains("</head>"));
    }
}
