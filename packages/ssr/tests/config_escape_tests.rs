use tairitsu_ssr::{FullDocumentConfig, SsrDom};

#[test]
fn test_full_document_config_escapes_lang() {
    let config = FullDocumentConfig {
        lang: "en\"><script>alert(1)</script>".to_string(),
        ..Default::default()
    };
    let dom = SsrDom::new();
    let html = dom.render_full_document_html(&config);
    assert!(
        !html.contains("<script>alert(1)</script>"),
        "Lang attribute should be escaped against injection"
    );
}

#[test]
fn test_full_document_config_escapes_charset() {
    let config = FullDocumentConfig {
        charset: "utf-8\"><meta http-equiv=\"refresh".to_string(),
        ..Default::default()
    };
    let dom = SsrDom::new();
    let html = dom.render_full_document_html(&config);
    let charset_section = html.split("charset=\"").nth(1).unwrap_or("");
    assert!(
        !charset_section.starts_with("utf-8\"><"),
        "Charset attribute value should be escaped, got: {}",
        &charset_section[..charset_section.len().min(60)]
    );
}

#[test]
fn test_full_document_config_normal_values() {
    let config = FullDocumentConfig {
        lang: "en".to_string(),
        charset: "utf-8".to_string(),
        title: "Test Page".to_string(),
        ..Default::default()
    };
    let dom = SsrDom::new();
    let html = dom.render_full_document_html(&config);
    assert!(html.contains(r#"lang="en""#));
    assert!(html.contains(r#"charset="utf-8""#));
    assert!(html.contains("<title>Test Page</title>"));
}

#[test]
fn test_full_document_config_title_escaping() {
    let config = FullDocumentConfig {
        title: "<script>alert('xss')</script>".to_string(),
        ..Default::default()
    };
    let dom = SsrDom::new();
    let html = dom.render_full_document_html(&config);
    assert!(
        !html.contains("<script>alert"),
        "Title should be HTML-escaped"
    );
    assert!(
        html.contains("&lt;script&gt;"),
        "Title should contain escaped tags"
    );
}
