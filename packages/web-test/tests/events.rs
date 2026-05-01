use serde::Deserialize;

const BASE_URL: &str = "http://localhost:3001";

#[derive(Deserialize)]
struct ApiResponse {
    ok: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

fn api_post(path: &str, body: &serde_json::Value) -> String {
    reqwest::blocking::Client::new()
        .post(format!("{}{}", BASE_URL, path))
        .json(body)
        .send()
        .expect("debug API request failed")
        .text()
        .expect("response read failed")
}

fn navigate(url: &str) {
    let text = api_post("/navigate", &serde_json::json!({ "url": url }));
    let msg: ApiResponse = serde_json::from_str(&text).unwrap();
    assert!(msg.ok, "navigate failed: {:?}", msg.error);
    std::thread::sleep(std::time::Duration::from_millis(800));
}

fn evaluate(js: &str) -> serde_json::Value {
    let text = api_post("/evaluate", &serde_json::json!({ "expression": js, "await_promise": false }));
    let msg: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(msg["ok"].as_bool().unwrap(), "evaluate failed: {}", msg);
    msg["data"]["result"].clone()
}

fn click(selector: &str) {
    let text = api_post("/click", &serde_json::json!({ "selector": selector }));
    let msg: ApiResponse = serde_json::from_str(&text).unwrap();
    assert!(msg.ok, "click failed: {:?}", msg.error);
    std::thread::sleep(std::time::Duration::from_millis(300));
}

#[test]
fn wasm_runtime_initialized() {
    navigate("/event-test");
    let result = evaluate("!!globalThis.__wasmExports && !!globalThis.__listenerHandles && globalThis.__listenerHandles.size > 0");
    assert_eq!(result, serde_json::Value::Bool(true), "WASM runtime not initialized");
}

#[test]
fn click_listener_registered() {
    navigate("/event-test");
    let result = evaluate(r#"
        (function() {
            var btn = document.getElementById('event-test-btn');
            if (!btn) return false;
            if (!globalThis.__listenerHandles) return false;
            for (var [id, info] of globalThis.__listenerHandles) {
                if (info.element === btn && info.type === 'click') return true;
            }
            return false;
        })()
    "#);
    assert_eq!(result, serde_json::Value::Bool(true), "Click listener not found on button");
}

#[test]
fn click_count_updates() {
    navigate("/event-test");
    let before = evaluate("document.getElementById('event-test-count')?.textContent?.trim() || ''");
    assert!(before.as_str().unwrap_or("").contains("clicks: 0"), "Expected initial count 0, got: {:?}", before);

    click("#event-test-btn");

    let after = evaluate("document.getElementById('event-test-count')?.textContent?.trim() || ''");
    assert!(after.as_str().unwrap_or("").contains("clicks: 1"), "Expected count 1 after click, got: {:?}", after);
}

#[test]
fn dom_element_reachable() {
    navigate("/event-test");
    let result = evaluate(r#"
        (function() {
            var btn = document.getElementById('event-test-btn');
            if (!btn || !globalThis.__elementHandles) return false;
            for (var [handle, el] of globalThis.__elementHandles) {
                if (el === btn) return true;
            }
            return false;
        })()
    "#);
    assert_eq!(result, serde_json::Value::Bool(true), "Button not reachable via __elementHandles");
}

#[test]
fn all_pages_no_console_errors() {
    for url in &["/", "/event-test"] {
        navigate(url);
        let result = evaluate("document.getElementById('hikari-app') !== null");
        assert_eq!(result, serde_json::Value::Bool(true), "#hikari-app not found on {}", url);
    }
}
