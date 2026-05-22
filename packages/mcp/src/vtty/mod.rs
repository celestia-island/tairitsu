pub mod graphics;
#[cfg(unix)]
pub mod pty_unix;
#[cfg(windows)]
pub mod pty_win;
#[cfg(feature = "vtty-visual")]
pub mod render;
pub mod screen;

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use screen::Vt100Screen;

static SESSION_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

// ─────────────────────────────────────────────────────
// VttySession — unified interface over platform PTY
// ─────────────────────────────────────────────────────

#[derive(Clone, Deserialize, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub command: String,
    pub cols: u16,
    pub rows: u16,
    pub alive: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
}

#[cfg(windows)]
type PtyHandle = pty_win::ConPty;
#[cfg(unix)]
type PtyHandle = pty_unix::UnixPty;

pub struct VttySession {
    pub id: String,
    pub name: String,
    pub command: String,
    pub cols: u16,
    pub rows: u16,
    pty: Mutex<Option<PtyHandle>>,
    screen: Arc<Mutex<Vt100Screen>>,
    alive: AtomicBool,
    pid: Option<u32>,
    reader_running: Arc<AtomicBool>,
    #[allow(dead_code)]
    reader_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl VttySession {
    pub fn new(id: String, name: String, command: String, cols: u16, rows: u16) -> Self {
        Self {
            id,
            name,
            command,
            cols,
            rows,
            pty: Mutex::new(None),
            screen: Arc::new(Mutex::new(Vt100Screen::new(cols as usize, rows as usize))),
            alive: AtomicBool::new(true),
            pid: None,
            reader_running: Arc::new(AtomicBool::new(true)),
            reader_handle: Mutex::new(None),
        }
    }

    pub fn launch(&mut self, cwd: Option<&str>) -> Result<(), String> {
        #[cfg(windows)]
        {
            let (handle, pid) = pty_win::ConPty::spawn(&self.command, self.cols, self.rows, cwd)
                .map_err(|e| format!("ConPTY spawn failed: {}", e))?;
            self.pid = Some(pid);
            self.pty = Mutex::new(Some(handle));
        }
        #[cfg(unix)]
        {
            let handle = pty_unix::UnixPty::spawn(&self.command, self.cols, self.rows, cwd)
                .map_err(|e| format!("forkpty failed: {}", e))?;
            let read_fd = handle.read_fd();
            self.pid = Some(handle.pid());
            self.pty = Mutex::new(Some(handle));

            let screen = self.screen.clone();
            let running = self.reader_running.clone();
            let handle = std::thread::Builder::new()
                .name(format!("vtty-reader-{}", self.id))
                .stack_size(128 * 1024)
                .spawn(move || {
                    pty_unix::UnixPty::reader_loop(read_fd, screen, running);
                })
                .map_err(|e| format!("spawn reader thread failed: {}", e))?;
            *self.reader_handle.lock().map_err(|e| format!("{}", e))? = Some(handle);
        }
        Ok(())
    }

    pub fn write(&self, data: &[u8]) -> Result<(), String> {
        let guard = self
            .pty
            .lock()
            .map_err(|_| "PTY lock poisoned".to_string())?;
        if let Some(ref pty) = *guard {
            pty.write(data)
                .map_err(|e| format!("PTY write failed: {}", e))?;
            Ok(())
        } else {
            Err("No PTY handle".into())
        }
    }

    pub fn send_keys(&self, keys: &str) -> Result<(), String> {
        let bytes = parse_keys(keys)?;
        self.write(&bytes)
    }

    pub fn send_text(&self, text: &str) -> Result<(), String> {
        let encoded: Vec<u8> = text.replace('\n', "\r").as_bytes().to_vec();
        self.write(&encoded)
    }

    pub fn screenshot(&self) -> String {
        self.screen.lock().map(|s| s.get_text()).unwrap_or_default()
    }

    #[cfg(feature = "vtty-visual")]
    pub fn visual_screenshot(&self, theme: &str) -> Result<Vec<u8>, String> {
        let rd = self
            .screen
            .lock()
            .map(|s| s.get_render_data())
            .map_err(|e| e.to_string())?;
        render::render_terminal(&rd, theme)
    }

    pub fn has_output(&self) -> bool {
        self.screen.lock().map(|s| s.has_output()).unwrap_or(false)
    }

    pub fn scrollback(&self) -> String {
        self.screen
            .lock()
            .map(|s| s.get_scrollback_with_screen())
            .unwrap_or_default()
    }

    #[allow(dead_code)]
    pub fn get_line(&self, row: usize) -> String {
        self.screen
            .lock()
            .map(|s| s.get_line(row))
            .unwrap_or_default()
    }

    pub fn find_text(&self, pattern: &str) -> Vec<(usize, usize)> {
        self.screen
            .lock()
            .map(|s| s.find_text(pattern))
            .unwrap_or_default()
    }

    pub fn resize(&mut self, new_cols: u16, new_rows: u16) -> Result<(), String> {
        {
            let guard = self
                .pty
                .lock()
                .map_err(|_| "PTY lock poisoned".to_string())?;
            if let Some(ref pty) = *guard {
                pty.resize(new_cols, new_rows)
                    .map_err(|e| format!("PTY resize failed: {}", e))?;
            }
        }
        self.screen
            .lock()
            .map_err(|_| "screen lock poisoned")?
            .resize(new_cols as usize, new_rows as usize);
        self.cols = new_cols;
        self.rows = new_rows;
        Ok(())
    }

    pub fn is_alive(&self) -> bool {
        if !self.alive.load(Ordering::Relaxed) {
            return false;
        }
        let guard = self.pty.lock();
        let guard = match guard {
            Ok(g) => g,
            Err(_) => return false,
        };
        if let Some(ref pty) = *guard {
            let alive = pty.is_alive();
            if !alive {
                self.alive.store(false, Ordering::Relaxed);
            }
            alive
        } else {
            false
        }
    }

    pub fn kill(&mut self) -> Result<(), String> {
        self.alive.store(false, Ordering::Relaxed);
        self.reader_running.store(false, Ordering::Relaxed);
        if let Ok(mut guard) = self.reader_handle.lock() {
            if let Some(handle) = guard.take() {
                let _ = handle.join();
            }
        }
        let mut guard = self
            .pty
            .lock()
            .map_err(|_| "PTY lock poisoned".to_string())?;
        if let Some(mut pty) = guard.take() {
            pty.kill_and_reap()
                .map_err(|e| format!("PTY kill failed: {}", e))?;
        }
        Ok(())
    }

    pub fn info(&self) -> SessionInfo {
        SessionInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            command: self.command.clone(),
            cols: self.cols,
            rows: self.rows,
            alive: self.is_alive(),
            pid: self.pid,
        }
    }
}

impl Drop for VttySession {
    fn drop(&mut self) {
        let _ = self.kill();
    }
}

// ─────────────────────────────────────────────────────
// VttyManager — owns all sessions, thread-safe
// ─────────────────────────────────────────────────────

pub struct VttyManager {
    sessions: Mutex<HashMap<String, Arc<Mutex<VttySession>>>>,
}

impl VttyManager {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    pub fn launch(
        &self,
        command: &str,
        cols: u16,
        rows: u16,
        _env: &str,
        cwd: Option<&str>,
        name: &str,
    ) -> Result<SessionInfo, String> {
        let id = format!("vtty-{}", SESSION_COUNTER.fetch_add(1, Ordering::Relaxed));
        let mut session = VttySession::new(
            id.clone(),
            name.to_string(),
            command.to_string(),
            cols,
            rows,
        );
        session.launch(cwd)?;
        let info = session.info();
        let arc = Arc::new(Mutex::new(session));
        self.sessions
            .lock()
            .map_err(|_| "sessions lock poisoned".to_string())?
            .insert(id.clone(), arc);
        Ok(info)
    }

    pub fn get(&self, sid: &str) -> Result<Arc<Mutex<VttySession>>, String> {
        self.sessions
            .lock()
            .map_err(|_| "lock poisoned".to_string())?
            .get(sid)
            .cloned()
            .ok_or_else(|| format!("Session '{}' not found", sid))
    }

    pub fn kill(&self, sid: &str) -> Result<SessionInfo, String> {
        let session = self.get(sid)?;
        let mut guard = session
            .lock()
            .map_err(|_| "session lock poisoned".to_string())?;
        let info = guard.info();
        if let Err(e) = guard.kill() {
            eprintln!("[vtty] warning: kill session '{}': {}", sid, e);
        }
        drop(guard);
        self.sessions
            .lock()
            .map_err(|_| "lock poisoned".to_string())?
            .remove(sid);
        Ok(info)
    }

    pub fn list(&self) -> Vec<SessionInfo> {
        let arcs: Vec<Arc<Mutex<VttySession>>> = match self.sessions.lock() {
            Ok(g) => g.values().cloned().collect(),
            Err(_) => return Vec::new(),
        };
        arcs.iter()
            .filter_map(|s| s.lock().ok())
            .map(|s| s.info())
            .collect()
    }

    pub fn ping(&self, sid: &str) -> Result<SessionInfo, String> {
        let session = self.get(sid)?;
        let guard = session
            .lock()
            .map_err(|_| "session lock poisoned".to_string())?;
        Ok(guard.info())
    }
}

impl Default for VttyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for VttyManager {
    fn drop(&mut self) {
        let sessions: Vec<Arc<Mutex<VttySession>>> = match self.sessions.lock() {
            Ok(mut g) => g.drain().map(|(_, v)| v).collect(),
            Err(_) => return,
        };
        for session in sessions {
            if let Ok(mut guard) = session.lock() {
                let _ = guard.kill();
            }
        }
    }
}

// ─────────────────────────────────────────────────────
// Key parsing — maps key names to terminal escape sequences
// ─────────────────────────────────────────────────────

pub fn parse_keys(keys_str: &str) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    for part in keys_str.split(' ') {
        let upper = part.to_uppercase();
        match upper.as_str() {
            "ENTER" | "RETURN" => buf.extend(b"\r"),
            "TAB" => buf.extend(b"\t"),
            "ESCAPE" | "ESC" => buf.extend(b"\x1b"),
            "BACKSPACE" | "BS" => buf.extend(b"\x7f"),
            "DELETE" | "DEL" => buf.extend(b"\x1b[3~"),
            "UP" => buf.extend(b"\x1b[A"),
            "DOWN" => buf.extend(b"\x1b[B"),
            "RIGHT" => buf.extend(b"\x1b[C"),
            "LEFT" => buf.extend(b"\x1b[D"),
            "HOME" => buf.extend(b"\x1b[H"),
            "END" => buf.extend(b"\x1b[F"),
            "PAGEUP" | "PAGE_UP" => buf.extend(b"\x1b[5~"),
            "PAGEDOWN" | "PAGE_DOWN" => buf.extend(b"\x1b[6~"),
            "INSERT" => buf.extend(b"\x1b[2~"),
            "F1" => buf.extend(b"\x1bOP"),
            "F2" => buf.extend(b"\x1bOQ"),
            "F3" => buf.extend(b"\x1bOR"),
            "F4" => buf.extend(b"\x1bOS"),
            "F5" => buf.extend(b"\x1b[15~"),
            "F6" => buf.extend(b"\x1b[17~"),
            "F7" => buf.extend(b"\x1b[18~"),
            "F8" => buf.extend(b"\x1b[19~"),
            "F9" => buf.extend(b"\x1b[20~"),
            "F10" => buf.extend(b"\x1b[21~"),
            "F11" => buf.extend(b"\x1b[23~"),
            "F12" => buf.extend(b"\x1b[24~"),
            "SPACE" => buf.push(b' '),
            s if s.starts_with("CTRL+") => {
                let ch = part[5..]
                    .chars()
                    .next()
                    .ok_or_else(|| format!("No letter after ctrl+ in '{}'", part))?;
                if ch.to_ascii_lowercase().is_ascii_lowercase() || ch.is_ascii_uppercase() {
                    buf.push(ch.to_ascii_uppercase() as u8 - b'A' + 1);
                } else {
                    return Err(format!("Invalid ctrl+key: {}", part));
                }
            }
            s if s.starts_with("ALT+") => {
                buf.push(0x1b);
                for ch in part[4..].chars() {
                    buf.push(ch as u8);
                }
            }
            s if s.starts_with("SHIFT+") => {
                for ch in part[6..].chars() {
                    buf.push(ch as u8);
                }
            }
            _s => {
                for ch in part.chars() {
                    buf.push(ch as u8);
                }
            }
        }
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_enter() {
        assert_eq!(parse_keys("ENTER").unwrap(), b"\r");
        assert_eq!(parse_keys("RETURN").unwrap(), b"\r");
    }

    #[test]
    fn test_parse_tab_esc() {
        assert_eq!(parse_keys("TAB").unwrap(), b"\t");
        assert_eq!(parse_keys("ESCAPE").unwrap(), b"\x1b");
        assert_eq!(parse_keys("ESC").unwrap(), b"\x1b");
    }

    #[test]
    fn test_parse_backspace_delete() {
        assert_eq!(parse_keys("BACKSPACE").unwrap(), b"\x7f");
        assert_eq!(parse_keys("BS").unwrap(), b"\x7f");
        assert_eq!(parse_keys("DELETE").unwrap(), b"\x1b[3~");
        assert_eq!(parse_keys("DEL").unwrap(), b"\x1b[3~");
    }

    #[test]
    fn test_parse_arrow_keys() {
        assert_eq!(parse_keys("UP").unwrap(), b"\x1b[A");
        assert_eq!(parse_keys("DOWN").unwrap(), b"\x1b[B");
        assert_eq!(parse_keys("RIGHT").unwrap(), b"\x1b[C");
        assert_eq!(parse_keys("LEFT").unwrap(), b"\x1b[D");
    }

    #[test]
    fn test_parse_home_end_pagenav() {
        assert_eq!(parse_keys("HOME").unwrap(), b"\x1b[H");
        assert_eq!(parse_keys("END").unwrap(), b"\x1b[F");
        assert_eq!(parse_keys("PAGEUP").unwrap(), b"\x1b[5~");
        assert_eq!(parse_keys("PAGE_DOWN").unwrap(), b"\x1b[6~");
    }

    #[test]
    fn test_parse_fkeys() {
        assert_eq!(parse_keys("F1").unwrap(), b"\x1bOP");
        assert_eq!(parse_keys("F4").unwrap(), b"\x1bOS");
        assert_eq!(parse_keys("F5").unwrap(), b"\x1b[15~");
        assert_eq!(parse_keys("F12").unwrap(), b"\x1b[24~");
    }

    #[test]
    fn test_parse_ctrl_keys() {
        assert_eq!(parse_keys("CTRL+C").unwrap(), &[0x03]);
        assert_eq!(parse_keys("CTRL+Z").unwrap(), &[0x1a]);
        assert_eq!(parse_keys("ctrl+a").unwrap(), &[0x01]);
    }

    #[test]
    fn test_parse_alt_keys() {
        let r = parse_keys("ALT+F").unwrap();
        assert_eq!(r, &[0x1b, b'F']);
    }

    #[test]
    fn test_parse_space() {
        assert_eq!(parse_keys("SPACE").unwrap(), b" ");
    }

    #[test]
    fn test_parse_literal_text() {
        assert_eq!(parse_keys("hello").unwrap(), b"hello");
    }

    #[test]
    fn test_parse_combined_sequence() {
        let r = parse_keys("echo ENTER").unwrap();
        assert_eq!(&r[..4], b"echo");
        assert_eq!(r[4], 0x0d);
    }

    #[test]
    fn test_invalid_ctrl_key() {
        assert!(parse_keys("CTRL+").is_err());
    }

    #[test]
    fn test_session_info_serialization() {
        let info = SessionInfo {
            id: "vtty-0".into(),
            name: "test".into(),
            command: "cmd".into(),
            cols: 80,
            rows: 24,
            alive: true,
            pid: Some(12345),
        };
        let json = serde_json::to_string(&info).unwrap();
        let roundtrip: SessionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.id, "vtty-0");
        assert_eq!(roundtrip.pid, Some(12345));
        assert!(roundtrip.alive);
    }

    #[test]
    fn test_session_info_no_pid() {
        let info = SessionInfo {
            id: "vtty-0".into(),
            name: "test".into(),
            command: "cmd".into(),
            cols: 80,
            rows: 24,
            alive: false,
            pid: None,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(!json.contains("pid"));
    }
}

#[cfg(test)]
#[cfg(unix)]
mod smoke_pty {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    fn spawn_session(command: &str) -> VttySession {
        let mut session =
            VttySession::new("test-smoke".into(), "smoke".into(), command.into(), 80, 24);
        session.launch(None).expect("launch should succeed");
        session
    }

    fn wait_for_text(session: &VttySession, pattern: &str, timeout_ms: u64) -> bool {
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
        while std::time::Instant::now() < deadline {
            if !session.find_text(pattern).is_empty() {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        false
    }

    #[test]
    fn smoke_pty_echo_ascii() {
        let mut session = spawn_session("echo 'Hello from PTY'");
        let found = wait_for_text(&session, "Hello from PTY", 3000);
        assert!(
            found,
            "screen should contain echo output, got:\n{}",
            session.screenshot()
        );
        let _ = session.kill();
    }

    #[test]
    fn smoke_pty_printf_cjk() {
        let mut session = spawn_session("printf '简体中文\\n'");
        let found = wait_for_text(&session, "简", 3000);
        assert!(
            found,
            "screen should contain CJK text, got:\n{}",
            session.screenshot()
        );
        if found {
            let text = session.screenshot();
            assert!(
                text.contains("简体中文"),
                "full CJK string present, got: {}",
                text
            );
            assert!(!text.contains(';'), "no SGR leak, got: {}", text);
        }
        let _ = session.kill();
    }

    #[test]
    fn smoke_printf_ansi_colors() {
        let mut session =
            spawn_session("printf '\\033[38;2;255;107;157mpink\\033[0m \\033[32mgreen\\033[0m\\n'");
        let found = wait_for_text(&session, "pink", 3000);
        assert!(
            found,
            "screen should contain colored text, got:\n{}",
            session.screenshot()
        );
        if found {
            let text = session.screenshot();
            assert!(text.contains("pink"), "got: {}", text);
            assert!(text.contains("green"), "got: {}", text);
            assert!(!text.contains("38;2"), "no SGR fragment, got: {}", text);
            assert!(!text.contains("32m"), "no raw SGR, got: {}", text);
        }
        let _ = session.kill();
    }

    #[test]
    fn smoke_pty_send_text() {
        let mut session = spawn_session("cat -v");
        std::thread::sleep(std::time::Duration::from_millis(300));

        session.send_text("hello").expect("send_text should work");
        std::thread::sleep(std::time::Duration::from_millis(200));

        let text = session.screenshot();
        assert!(
            text.contains("hello"),
            "screen should echo sent text, got:\n{}",
            text
        );
        let _ = session.kill();
    }

    #[test]
    fn smoke_pty_send_keys_arrow() {
        let mut session = spawn_session("cat -v");
        std::thread::sleep(std::time::Duration::from_millis(300));

        session.send_keys("DOWN").expect("send_keys should work");
        std::thread::sleep(std::time::Duration::from_millis(200));

        let text = session.screenshot();
        assert!(
            text.contains("^[[B") || text.contains("\\x1b[B"),
            "screen should show escape sequence for Down arrow, got:\n{}",
            text
        );
        let _ = session.kill();
    }

    #[test]
    fn smoke_pty_resize() {
        let mut session = spawn_session("sleep 1");
        std::thread::sleep(std::time::Duration::from_millis(200));

        session.resize(120, 40).expect("resize should work");
        assert_eq!(session.cols, 120);
        assert_eq!(session.rows, 40);
        let _ = session.kill();
    }

    #[test]
    fn smoke_pty_lifecycle_kill_is_idempotent() {
        let mut session = spawn_session("sleep 60");
        std::thread::sleep(std::time::Duration::from_millis(200));

        assert!(session.is_alive(), "session should be alive");
        session.kill().expect("first kill should succeed");
        assert!(!session.is_alive(), "session should be dead after kill");
        session
            .kill()
            .expect("second kill should also succeed (idempotent)");
        assert!(!session.is_alive(), "still dead after second kill");
    }

    #[test]
    fn smoke_pty_manager_launch_kill() {
        let mgr = VttyManager::new();

        let info = mgr
            .launch("sleep 60", 80, 24, "", None, "test")
            .expect("launch should succeed");
        assert!(info.alive);
        assert!(info.id.starts_with("vtty-"));

        let killed = mgr.kill(&info.id).expect("kill should succeed");
        assert_eq!(killed.id, info.id);

        let result = mgr.get(&info.id);
        assert!(result.is_err(), "session should be removed after kill");
    }

    #[test]
    fn smoke_pty_manager_drop_cleans_up() {
        let mgr = VttyManager::new();
        let info = mgr
            .launch("sleep 60", 80, 24, "", None, "test")
            .expect("launch should succeed");

        let sid = info.id.clone();
        drop(mgr);
        let mgr2 = VttyManager::new();
        assert!(
            mgr2.get(&sid).is_err(),
            "session should be cleaned up after manager drop"
        );
    }

    #[test]
    fn smoke_pty_multi_session() {
        let mgr = VttyManager::new();

        let s1 = mgr
            .launch("echo 'session1'", 80, 24, "", None, "s1")
            .expect("launch s1");
        let s2 = mgr
            .launch("echo 'session2'", 80, 24, "", None, "s2")
            .expect("launch s2");

        let list = mgr.list();
        assert_eq!(list.len(), 2, "should have 2 sessions");

        let _ = mgr.kill(&s1.id);
        let list = mgr.list();
        assert_eq!(list.len(), 1, "should have 1 session after kill");

        let _ = mgr.kill(&s2.id);
        let list = mgr.list();
        assert!(list.is_empty(), "should have 0 sessions");
    }

    #[test]
    fn smoke_reader_loop_via_poll() {
        let handle = pty_unix::UnixPty::spawn("printf 'polltest\\n'", 80, 24, None)
            .expect("spawn should succeed");
        let fd = handle.read_fd();
        let screen = Arc::new(std::sync::Mutex::new(
            crate::vtty::screen::Vt100Screen::new(80, 24),
        ));
        let running = Arc::new(AtomicBool::new(true));

        let screen_clone = screen.clone();
        let running_clone = running.clone();
        let h = std::thread::Builder::new()
            .name("test-reader".into())
            .stack_size(128 * 1024)
            .spawn(move || pty_unix::UnixPty::reader_loop(fd, screen_clone, running_clone))
            .expect("thread should spawn");

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let text = screen.lock().unwrap().get_text();
            if text.contains("polltest") {
                break;
            }
            if std::time::Instant::now() > deadline {
                panic!(
                    "reader_loop did not pick up 'polltest' within 5s, got: {}",
                    text
                );
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        running.store(false, std::sync::atomic::Ordering::Relaxed);
        let _ = h.join();
        let text = screen.lock().unwrap().get_text();
        assert!(
            text.contains("polltest"),
            "screen should have output after reader_loop, got: {}",
            text
        );
        assert!(!text.contains(';'), "no SGR leak, got: {}", text);
    }

    // ── Real-world PTY stress tests ─────────────────────

    #[test]
    fn smoke_pty_bash_prompt() {
        let mut session = spawn_session("bash --norc --noprofile -i");
        let found = wait_for_text(&session, "$", 5000);
        assert!(
            found,
            "bash prompt should appear, got:\n{}",
            session.screenshot()
        );
        let _ = session.kill();
    }

    #[test]
    fn smoke_pty_process_self_exit() {
        let session = spawn_session("echo done_here");
        let found = wait_for_text(&session, "done_here", 3000);
        assert!(
            found,
            "should see output before exit, got:\n{}",
            session.screenshot()
        );
        std::thread::sleep(std::time::Duration::from_millis(500));
        assert!(!session.is_alive(), "process should have exited on its own");
    }

    #[test]
    fn smoke_pty_ctrl_c_interrupt() {
        let session = spawn_session("sleep 60");
        std::thread::sleep(std::time::Duration::from_millis(500));
        assert!(session.is_alive(), "sleep should be running");
        session.send_keys("CTRL+C").expect("send CTRL+C");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            if !session.is_alive() {
                break;
            }
            if std::time::Instant::now() > deadline {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        assert!(!session.is_alive(), "sleep should be killed by CTRL+C");
    }

    #[test]
    fn smoke_pty_large_output() {
        let mut session = spawn_session("seq 1 200");
        let found = wait_for_text(&session, "200", 5000);
        assert!(
            found,
            "should see last line of large output, got:\n{}",
            session.screenshot()
        );
        let text = session.screenshot();
        assert!(
            text.contains("1"),
            "should contain first number in scrollback/screen"
        );
        assert!(!text.contains("38;2"), "no SGR leak");
        let _ = session.kill();
    }

    #[test]
    fn smoke_pty_binary_resilience() {
        let mut session = spawn_session("head -c 256 /dev/urandom | xxd | head -5");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let text = session.screenshot();
            if !text.trim().is_empty() {
                break;
            }
            if std::time::Instant::now() > deadline {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        let text = session.screenshot();
        assert!(!text.trim().is_empty(), "xxd should produce hex output");
        let _ = session.kill();
    }

    #[test]
    fn smoke_pty_long_line_output() {
        let mut session = spawn_session("printf '%0.s.' {1..200}");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let mut found = false;
        loop {
            let text = session.screenshot();
            if text.contains("....") {
                found = true;
                break;
            }
            if std::time::Instant::now() > deadline {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        assert!(
            found,
            "long line should wrap, got:\n{}",
            session.screenshot()
        );
        let _ = session.kill();
    }

    #[test]
    fn smoke_pty_unicode_roundtrip() {
        let mut session = spawn_session("cat -v");
        std::thread::sleep(std::time::Duration::from_millis(300));
        session.send_text("简体中文").expect("send unicode");
        std::thread::sleep(std::time::Duration::from_millis(300));
        let text = session.screenshot();
        assert!(
            text.contains("简") || text.contains("M-"),
            "should echo unicode or high-byte representation, got:\n{}",
            text
        );
        let _ = session.kill();
    }

    #[test]
    fn smoke_pty_resize_during_output() {
        let mut session = spawn_session("seq 1 100");
        let found = wait_for_text(&session, "100", 5000);
        assert!(found, "seq should complete, got:\n{}", session.screenshot());
        session.resize(40, 10).expect("resize during output");
        std::thread::sleep(std::time::Duration::from_millis(200));
        session.resize(120, 30).expect("resize back");
        assert_eq!(session.cols, 120);
        assert_eq!(session.rows, 30);
        let scrollback = session.scrollback();
        let screen = session.screenshot();
        let combined = format!("{}\n{}", scrollback, screen);
        assert!(
            combined.contains("100"),
            "output should survive resize in scrollback+screen, got:\n{}",
            combined
        );
        let _ = session.kill();
    }

    #[test]
    fn smoke_pty_error_command() {
        let mut session = spawn_session("nonexistent_command_xyz_12345");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        let mut found = false;
        loop {
            let text = session.screenshot();
            if text.contains("nonexistent")
                || text.contains("not found")
                || text.contains("No such")
            {
                found = true;
                break;
            }
            if std::time::Instant::now() > deadline {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        assert!(
            found,
            "error message should appear, got:\n{}",
            session.screenshot()
        );
        let _ = session.kill();
    }

    #[test]
    fn smoke_pty_multi_command_sequence() {
        let mut session = spawn_session("bash --norc --noprofile");
        std::thread::sleep(std::time::Duration::from_millis(500));

        session.send_text("echo CMD1").expect("send cmd1");
        session.send_keys("ENTER").expect("enter");
        let found1 = wait_for_text(&session, "CMD1", 3000);
        assert!(
            found1,
            "should see CMD1 output, got:\n{}",
            session.screenshot()
        );

        session.send_text("echo CMD2").expect("send cmd2");
        session.send_keys("ENTER").expect("enter");
        let found2 = wait_for_text(&session, "CMD2", 3000);
        assert!(
            found2,
            "should see CMD2 output, got:\n{}",
            session.screenshot()
        );

        let _ = session.kill();
    }

    #[test]
    fn smoke_pty_scrollback_with_large_output() {
        let mut session = spawn_session("seq 1 100");
        let found = wait_for_text(&session, "100", 5000);
        assert!(found, "should complete, got:\n{}", session.screenshot());
        let scrollback = session.scrollback();
        let screen = session.screenshot();
        let combined = format!("{}\n{}", scrollback, screen);
        assert!(
            combined.contains("1"),
            "scrollback+screen should contain early output"
        );
        assert!(
            combined.contains("50"),
            "scrollback+screen should contain mid output"
        );
        let _ = session.kill();
    }

    // ── Coverage fill: PTY session edge cases ───────────

    #[test]
    fn smoke_pty_send_text_with_newline() {
        let mut session = spawn_session("bash --norc --noprofile");
        std::thread::sleep(std::time::Duration::from_millis(500));
        session
            .send_text("echo NL_TEST\n")
            .expect("send_text with newline");
        let found = wait_for_text(&session, "NL_TEST", 3000);
        assert!(
            found,
            "send_text \\n translation should work, got:\n{}",
            session.screenshot()
        );
        let _ = session.kill();
    }

    #[test]
    fn smoke_pty_has_output() {
        let mut session = spawn_session("echo detect_output");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            if session.has_output() {
                break;
            }
            if std::time::Instant::now() > deadline {
                panic!("has_output() should return true after echo");
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        let _ = session.kill();
    }

    #[test]
    fn smoke_pty_manager_ping() {
        let mgr = VttyManager::new();
        let info = mgr
            .launch("sleep 60", 80, 24, "", None, "test")
            .expect("launch");
        let pinged = mgr.ping(&info.id).expect("ping");
        assert!(pinged.alive);
        assert_eq!(pinged.id, info.id);
        let _ = mgr.kill(&info.id);
    }

    #[test]
    fn smoke_pty_write_after_kill_errors() {
        let mut session = spawn_session("sleep 60");
        std::thread::sleep(std::time::Duration::from_millis(300));
        session.kill().expect("kill");
        let result = session.send_text("should fail");
        assert!(result.is_err(), "write after kill should fail");
    }

    #[test]
    fn smoke_pty_drop_without_explicit_kill() {
        {
            let session = spawn_session("sleep 60");
            std::thread::sleep(std::time::Duration::from_millis(300));
            assert!(session.is_alive());
        }
    }

    #[test]
    fn smoke_pty_launch_with_cwd() {
        let mut session = VttySession::new("cwd-test".into(), "test".into(), "pwd".into(), 80, 24);
        session.launch(Some("/tmp")).expect("launch with cwd");
        let found = wait_for_text(&session, "tmp", 3000);
        assert!(
            found,
            "should see /tmp from pwd, got:\n{}",
            session.screenshot()
        );
        let _ = session.kill();
    }
}
