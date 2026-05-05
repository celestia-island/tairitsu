//! Route discovery from html.head JavaScript
//!
//! Parses client-side ROUTES definitions embedded in Cargo.toml's
//! `[package.metadata.tairitsu.html.head]` field into structured data
//! that the build pipeline, dev server, and SSR can use.

/// A discovered route mapping: (URL path, page ID)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredRoute {
    pub path: String,
    pub page_id: String,
}

impl DiscoveredRoute {
    pub fn new(path: String, page_id: String) -> Self {
        Self { path, page_id }
    }

    /// Returns true if this is a root/empty route
    pub fn is_root(&self) -> bool {
        self.path.is_empty() || self.path == "/"
    }

    /// Returns the filesystem-safe path component (without leading slash)
    pub fn fs_path(&self) -> &str {
        self.path.strip_prefix('/').unwrap_or(&self.path)
    }
}

/// Parse a JavaScript ROUTES object literal from html.head content.
///
/// Supports formats like:
/// ```js
/// const ROUTES = {
///   '/path': 'page-id',
///   '': 'home',
/// };
/// ```
///
/// Also handles single-quoted keys, trailing commas, and various whitespace.
pub fn discover_routes(head_js: &str) -> Vec<DiscoveredRoute> {
    let mut routes = Vec::new();

    if let Some(routes_obj) = extract_routes_object(head_js) {
        parse_route_entries(&routes_obj, &mut routes);
    }

    routes.sort_by(|a, b| a.path.cmp(&b.path));
    routes.dedup_by(|a, b| a.path == b.path);
    routes
}

fn extract_routes_object(head_js: &str) -> Option<String> {
    let start_markers = ["const ROUTES", "let ROUTES", "var ROUTES", "ROUTES"];
    let mut start = None;
    let mut brace_depth = 0;
    let mut in_object = false;
    let mut obj_start = 0;

    for marker in &start_markers {
        if let Some(pos) = head_js.find(marker) {
            let after_marker = &head_js[pos + marker.len()..];
            if let Some(eq_pos) = after_marker.find('=') {
                let after_eq = &after_marker[eq_pos + 1..];
                if let Some(brace_pos) = after_eq.find('{') {
                    start = Some(pos + marker.len() + eq_pos + 1 + brace_pos);
                    break;
                }
            }
        }
    }

    let start_pos = start?;
    let chars: Vec<char> = head_js[start_pos..].chars().collect();
    let mut end = 0;

    for (i, &ch) in chars.iter().enumerate() {
        match ch {
            '{' => {
                if !in_object {
                    in_object = true;
                    obj_start = i;
                }
                brace_depth += 1;
            }
            '}' => {
                brace_depth -= 1;
                if in_object && brace_depth == 0 {
                    end = i;
                    break;
                }
            }
            '"' | '\'' if in_object => {
                let quote = ch;
                let mut j = i + 1;
                while j < chars.len() {
                    if chars[j] == '\\' {
                        j += 2;
                        continue;
                    }
                    if chars[j] == quote {
                        break;
                    }
                    j += 1;
                }
            }
            _ => {}
        }
    }

    if end > obj_start {
        Some(chars[obj_start..=end].iter().collect())
    } else {
        None
    }
}

fn parse_route_entries(obj_text: &str, routes: &mut Vec<DiscoveredRoute>) {
    let chars: Vec<char> = obj_text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        skip_whitespace(&chars, &mut i);

        if i >= chars.len() || chars[i] == '}' {
            break;
        }

        let key = match parse_string_literal(&chars, &mut i) {
            Some(k) => k,
            None => {
                i += 1;
                continue;
            }
        };

        skip_whitespace(&chars, &mut i);

        if i < chars.len() && chars[i] == ':' {
            i += 1;
        } else {
            continue;
        }

        skip_whitespace(&chars, &mut i);

        let value = match parse_string_literal(&chars, &mut i) {
            Some(v) => v,
            None => {
                skip_to_next_entry(&chars, &mut i);
                continue;
            }
        };

        routes.push(DiscoveredRoute::new(key, value));

        skip_whitespace(&chars, &mut i);

        if i < chars.len() && chars[i] == ',' {
            i += 1;
        }
    }
}

fn parse_string_literal(chars: &[char], pos: &mut usize) -> Option<String> {
    if *pos >= chars.len() {
        return None;
    }

    let quote = match chars[*pos] {
        '"' | '\'' => chars[*pos],
        _ => return None,
    };

    *pos += 1;
    let mut result = String::new();

    while *pos < chars.len() {
        match chars[*pos] {
            '\\' => {
                *pos += 1;
                if *pos < chars.len() {
                    match chars[*pos] {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        c => result.push(c),
                    }
                    *pos += 1;
                }
            }
            c if c == quote => {
                *pos += 1;
                return Some(result);
            }
            c => {
                result.push(c);
                *pos += 1;
            }
        }
    }

    None
}

fn skip_whitespace(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && chars[*pos].is_whitespace() {
        *pos += 1;
    }
}

fn skip_to_next_entry(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() {
        match chars[*pos] {
            ',' => {
                *pos += 1;
                return;
            }
            '}' => return,
            _ => *pos += 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_routes() {
        let head = r#"
            const ROUTES = {
                '/': 'home',
                '/about': 'page-about',
            };
        "#;
        let routes = discover_routes(head);
        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0].path, "/");
        assert_eq!(routes[0].page_id, "home");
        assert_eq!(routes[1].path, "/about");
        assert_eq!(routes[1].page_id, "page-about");
    }

    #[test]
    fn test_parse_single_quoted_routes() {
        let head =
            "const ROUTES = {'/components/button': 'component-button', '/form': 'form-page'};";
        let routes = discover_routes(head);
        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0].path, "/components/button");
        assert_eq!(routes[1].path, "/form");
    }

    #[test]
    fn test_parse_empty_routes() {
        let head = "const ROUTES = {};";
        let routes = discover_routes(head);
        assert!(routes.is_empty());
    }

    #[test]
    fn test_no_routes_in_head() {
        let head = "<meta name='viewport' content='width=device-width'>";
        let routes = discover_routes(head);
        assert!(routes.is_empty());
    }

    #[test]
    fn test_parse_with_trailing_commas() {
        let head = r#"const ROUTES = { '/a': 'pa', '/b': 'pb', };"#;
        let routes = discover_routes(head);
        assert_eq!(routes.len(), 2);
    }

    #[test]
    fn test_discovered_route_helpers() {
        let r = DiscoveredRoute::new("/".to_string(), "home".to_string());
        assert!(r.is_root());
        assert_eq!(r.fs_path(), "");

        let r2 = DiscoveredRoute::new("/components/layer1/button".to_string(), "btn".to_string());
        assert!(!r2.is_root());
        assert_eq!(r2.fs_path(), "components/layer1/button");

        let r3 = DiscoveredRoute::new("".to_string(), "home".to_string());
        assert!(r3.is_root());
        assert_eq!(r3.fs_path(), "");
    }

    #[test]
    fn test_complex_realistic_routes() {
        let head = r#"
<script>
const ROUTES = {
    '': 'home',
    '/': 'home',
    '/components': 'components',
    '/components/layer1/button': 'component-button',
    '/components/layer1/form': 'component-form',
    '/demos/dashboard': 'demos-dashboard',
};
function navigate() { /* ... */ }
</script>
        "#;
        let routes = discover_routes(head);
        assert_eq!(routes.len(), 6);
        assert_eq!(routes[0].path, "");
        assert_eq!(routes[1].path, "/");
    }
}
