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
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
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
                .stack_size(32 * 1024)
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

    pub fn read_and_update(&self) -> Result<usize, String> {
        #[cfg(unix)]
        {
            let guard = self
                .pty
                .lock()
                .map_err(|_| "PTY lock poisoned".to_string())?;
            if let Some(ref pty) = *guard {
                let mut buf = [0u8; 4096];
                let mut total = 0;
                loop {
                    match pty.read_nonblocking(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            self.screen
                                .lock()
                                .map_err(|_| "screen lock poisoned")?
                                .process(&buf[..n]);
                            total += n;
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                        Err(e) => return Err(format!("PTY read error: {}", e)),
                    }
                }
                Ok(total)
            } else {
                Ok(0)
            }
        }
        #[cfg(windows)]
        {
            let mut buf = vec![0u8; 65536];
            let mut total = 0;
            loop {
                let n = {
                    let guard = self
                        .pty
                        .lock()
                        .map_err(|_| "PTY lock poisoned".to_string())?;
                    if let Some(ref pty) = *guard {
                        match pty.read_nonblocking(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => n,
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                            Err(e) => return Err(format!("PTY read error: {}", e)),
                        }
                    } else {
                        break;
                    }
                };
                if n > 0 {
                    self.screen
                        .lock()
                        .map_err(|_| "screen lock poisoned")?
                        .process(&buf[..n]);
                    total += n;
                }
            }
            Ok(total)
        }
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

    pub fn resize(&self, new_cols: u16, new_rows: u16) -> Result<(), String> {
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
        if let Ok(mut guard) = self.reader_handle.lock()
            && let Some(handle) = guard.take()
        {
            let _ = handle.join();
        }
        let mut guard = self
            .pty
            .lock()
            .map_err(|_| "PTY lock poisoned".to_string())?;
        if let Some(mut pty) = guard.take() {
            pty.kill().map_err(|e| format!("PTY kill failed: {}", e))?;
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
        let _ = guard.kill();
        drop(guard);
        self.sessions
            .lock()
            .map_err(|_| "lock poisoned".to_string())?
            .remove(sid);
        Ok(info)
    }

    pub fn list(&self) -> Vec<SessionInfo> {
        self.sessions
            .lock()
            .map(|g| {
                g.values()
                    .filter_map(|s| s.lock().ok())
                    .map(|s| s.info())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn ping(&self, sid: &str) -> Result<SessionInfo, String> {
        let session = self.get(sid)?;
        let guard = session
            .lock()
            .map_err(|_| "session lock poisoned".to_string())?;
        // Trigger a read to refresh screen state
        let _ = guard.read_and_update();
        Ok(guard.info())
    }
}

impl Default for VttyManager {
    fn default() -> Self {
        Self::new()
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
