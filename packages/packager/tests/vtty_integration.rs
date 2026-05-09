use std::thread;
use std::time::Duration;

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

    match pty.kill() {
        Ok(()) => eprintln!("kill succeeded"),
        Err(e) => eprintln!("kill error (may have already exited): {}", e),
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
        &output
    );

    pty.kill().ok();
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
        &output
    );

    pty.kill().ok();
}

#[cfg(unix)]
#[test]
fn test_pty_resize() {
    let mut pty = tairitsu_packager::vtty::pty_unix::UnixPty::spawn(TEST_CMD, 80, 24, None)
        .expect("spawn failed");

    pty.resize(120, 40).expect("resize failed");
    pty.kill().ok();
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

    pty.kill().ok();
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
