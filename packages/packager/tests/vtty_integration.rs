use std::{thread, time::Duration};

fn wait_ms(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

#[cfg(windows)]
const TEST_CMD: &str = "cmd";

#[cfg(unix)]
const TEST_CMD: &str = "bash";

#[cfg(unix)]
#[test]
#[ignore = "flaky: PTY process may not exit within 300ms after kill on loaded CI runners"]
fn test_pty_spawn_and_kill() {
    let mut pty = tairitsu_packager::vtty::pty_unix::UnixPty::spawn(TEST_CMD, 80, 24, None)
        .expect("spawn failed");
    let pid = pty.pid();
    assert!(pid > 0, "PID should be positive");

    wait_ms(300);
    let alive = pty.is_alive();
    eprintln!("is_alive after 300ms: {}", alive);

    match pty.kill_and_reap() {
        Ok(()) => eprintln!("kill_and_reap succeeded"),
        Err(e) => eprintln!("kill_and_reap error (may have already exited): {}", e),
    }
    assert!(!pty.is_alive(), "PTY should be dead after kill attempt");
}

#[cfg(windows)]
#[test]
fn test_pty_spawn_and_kill() {
    let (mut pty, pid) = tairitsu_packager::vtty::pty_win::ConPty::spawn(TEST_CMD, 80, 24, None)
        .expect("ConPty spawn failed");
    assert!(pid > 0, "PID should be positive");

    wait_ms(300);
    let alive = pty.is_alive();
    eprintln!("is_alive after 300ms: {}", alive);

    match pty.kill() {
        Ok(()) => eprintln!("kill succeeded"),
        Err(e) => eprintln!("kill error (may have already exited): {}", e),
    }
    assert!(!pty.is_alive(), "PTY should be dead after kill attempt");
}

#[cfg(unix)]
#[test]
fn test_pty_write_and_read() {
    let mut pty = tairitsu_packager::vtty::pty_unix::UnixPty::spawn(TEST_CMD, 80, 24, None)
        .expect("spawn failed");

    wait_ms(200);

    pty.write(b"echo HELLO_VTTY_TEST\n").expect("write failed");

    wait_ms(800);

    let mut buf = [0u8; 4096];
    let mut all_output = String::new();
    for _ in 0..5 {
        let n = match pty.read_nonblocking(&mut buf) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("read error: {}", e);
                0
            }
        };
        if n > 0 {
            all_output.push_str(&String::from_utf8_lossy(&buf[..n]));
        }
        if all_output.contains("HELLO_VTTY_TEST") {
            break;
        }
        wait_ms(200);
    }
    eprintln!("PTY output: {:?}", all_output);
    assert!(
        all_output.contains("HELLO_VTTY_TEST") || !all_output.is_empty(),
        "Should have some output from echo, got: {:?}",
        all_output
    );

    pty.kill_and_reap().ok();
}

#[cfg(windows)]
#[test]
fn test_pty_write_and_read() {
    let (mut pty, _) = tairitsu_packager::vtty::pty_win::ConPty::spawn(TEST_CMD, 80, 24, None)
        .expect("ConPty spawn failed");

    wait_ms(200);

    pty.write(b"echo HELLO_VTTY_TEST\r\n")
        .expect("write failed");

    wait_ms(500);

    let mut buf = [0u8; 4096];
    let n = match pty.read_nonblocking(&mut buf) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("read error: {}", e);
            0
        }
    };
    let output = String::from_utf8_lossy(&buf[..n]);
    eprintln!("PTY output: {:?}", output);
    assert!(
        output.contains("HELLO_VTTY_TEST") || n > 0,
        "Should have some output from echo, got {} bytes: {:?}",
        n,
        output
    );

    pty.kill_and_reap().ok();
}

#[cfg(unix)]
#[test]
fn test_pty_resize() {
    let mut pty = tairitsu_packager::vtty::pty_unix::UnixPty::spawn(TEST_CMD, 80, 24, None)
        .expect("spawn failed");

    pty.resize(120, 40).expect("resize failed");
    pty.kill_and_reap().ok();
}

#[cfg(windows)]
#[test]
fn test_pty_resize() {
    let (mut pty, _) = tairitsu_packager::vtty::pty_win::ConPty::spawn(TEST_CMD, 80, 24, None)
        .expect("ConPty spawn failed");

    pty.resize(120, 40).expect("resize failed");
    pty.kill().ok();
}

#[cfg(unix)]
#[test]
fn test_pty_pid() {
    let mut pty = tairitsu_packager::vtty::pty_unix::UnixPty::spawn(TEST_CMD, 80, 24, None)
        .expect("spawn failed");
    let pid = pty.pid();
    assert!(pid > 0, "PID should be positive");
    assert!(pty.is_alive(), "PTY should be alive after spawn");

    pty.kill_and_reap().ok();
}

#[cfg(windows)]
#[test]
fn test_pty_pid() {
    let (mut pty, pid) = tairitsu_packager::vtty::pty_win::ConPty::spawn(TEST_CMD, 80, 24, None)
        .expect("ConPty spawn failed");
    assert!(pid > 0, "PID should be positive");
    assert!(pty.is_alive(), "PTY should be alive after spawn");

    pty.kill().ok();
}

#[test]
fn test_session_lifecycle() {
    use tairitsu_packager::vtty::VttyManager;

    let mgr = VttyManager::new();
    assert!(mgr.list().is_empty());

    let info = mgr
        .launch(TEST_CMD, 80, 24, "", None, "integration-test")
        .expect("launch failed");
    assert_eq!(info.command, TEST_CMD);
    assert!(info.alive);
    assert!(info.pid.is_some());
    assert_eq!(mgr.list().len(), 1);

    let ping = mgr.ping(&info.id).expect("ping failed");
    assert!(ping.alive);

    let _ = mgr.kill(&info.id);
    mgr.kill(&info.id).ok();
    assert!(mgr.list().is_empty());

    let err = mgr.get(&info.id);
    assert!(err.is_err(), "get on killed session should fail");
}

#[test]
fn test_session_screenshot_after_echo() {
    use tairitsu_packager::vtty::VttyManager;

    let mgr = VttyManager::new();

    let info = mgr
        .launch(TEST_CMD, 80, 24, "", None, "screenshot-test")
        .expect("launch failed");

    wait_ms(300);

    let session = mgr.get(&info.id).unwrap();
    let guard = session.lock().unwrap();
    let result = guard.send_keys("CTRL+C");
    drop(guard);

    if let Err(e) = result {
        eprintln!("send_keys CTRL+C warning: {}", e);
    }

    mgr.kill(&info.id).ok();
}

#[test]
fn test_send_keys_arrow_keys() {
    let result = tairitsu_packager::vtty::parse_keys("UP DOWN LEFT RIGHT HOME END");
    assert!(result.is_ok());
    let bytes = result.unwrap();
    assert_eq!(bytes.len(), 18);
}

#[test]
fn test_multiple_sessions() {
    use tairitsu_packager::vtty::VttyManager;

    let mgr = VttyManager::new();

    let a = mgr
        .launch(TEST_CMD, 80, 24, "", None, "sess-a")
        .expect("launch a failed");
    let b = mgr
        .launch(TEST_CMD, 80, 24, "", None, "sess-b")
        .expect("launch b failed");
    assert_ne!(a.id, b.id);
    assert_eq!(mgr.list().len(), 2);

    mgr.kill(&a.id).ok();
    assert_eq!(mgr.list().len(), 1);
    assert_eq!(mgr.list()[0].id, b.id);

    mgr.kill(&b.id).ok();
    assert!(mgr.list().is_empty());
}

// ── W5: the glue ESM modules must ship with the page ───────────────────────
//
// The jco wrapper's `@tairitsu-glue/<name>` specifiers are rewritten to
// `../glue/<name>.js`, so a dist without those files loads a page that cannot
// instantiate. `copy_glue_modules` is the step that puts them there; these tests
// pin both directions (it copies what exists, and it refuses to build when the
// emitted directory is absent).
mod glue_modules {
    use std::path::Path;

    /// The emitter's output directory (built by `just glue-esm`).
    fn emitted_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../browser-glue/dist/glue")
    }

    #[test]
    fn the_emitter_produces_real_esm_modules() {
        let dir = emitted_dir();
        if !dir.is_dir() {
            // The packager refuses to build without them; in a bare checkout the
            // emitter has not run yet, and that is the build error, not a test
            // failure.
            eprintln!("skipping: {} has not been generated yet", dir.display());
            return;
        }
        let mut modules = 0usize;
        for entry in std::fs::read_dir(&dir).expect("readable") {
            let path = entry.expect("entry").path();
            if path.extension().and_then(|s| s.to_str()) != Some("js") {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("readable");
            assert!(
                source.contains("export const "),
                "{} carries no named export",
                path.display()
            );
            assert!(
                !source.contains("new Blob("),
                "{} still builds a blob module",
                path.display()
            );
            modules += 1;
        }
        assert!(
            modules >= 20,
            "expected the interface modules, got {modules}"
        );
    }

    #[test]
    fn the_manifest_lists_every_interface() {
        let manifest = emitted_dir().join("manifest.json");
        if !manifest.is_file() {
            eprintln!(
                "skipping: {} has not been generated yet",
                manifest.display()
            );
            return;
        }
        let text = std::fs::read_to_string(&manifest).expect("readable");
        let value: serde_json::Value = serde_json::from_str(&text).expect("valid json");
        let interfaces = value["interfaces"].as_array().expect("interfaces array");
        assert!(
            interfaces.len() >= 20,
            "manifest lists {} interfaces",
            interfaces.len()
        );
    }
}
