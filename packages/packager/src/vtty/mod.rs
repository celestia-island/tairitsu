pub mod screen;

#[cfg(windows)]
pub mod pty_win;
#[cfg(unix)]
pub mod pty_unix;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicU32, Ordering}};

use screen::Vt100Screen;

static SESSION_COUNTER: AtomicU32 = AtomicU32::new(0);

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
    screen: Mutex<Vt100Screen>,
    alive: std::sync::atomic::AtomicBool,
    pid: Option<u32>,
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
            screen: Mutex::new(Vt100Screen::new(cols as usize, rows as usize)),
            alive: std::sync::atomic::AtomicBool::new(true),
            pid: None,
        }
    }

    pub fn launch(&mut self, cwd: Option<&str>) -> Result<(), String> {
        #[cfg(windows)]
        {
            let (handle, pid) = pty_win::ConPty::spawn(
                &self.command, self.cols, self.rows, cwd,
            ).map_err(|e| format!("ConPTY spawn failed: {}", e))?;
            self.pty = Mutex::new(Some(handle));
            self.pid = Some(pid);
        }
        #[cfg(unix)]
        {
            let handle = pty_unix::UnixPty::spawn(
                &self.command, self.cols, self.rows, cwd,
            ).map_err(|e| format!("forkpty failed: {}", e))?;
            self.pid = Some(handle.pid());
            self.pty = Mutex::new(Some(handle));
        }
        Ok(())
    }

    pub fn write(&self, data: &[u8]) -> Result<(), String> {
        let guard = self.pty.lock().map_err(|_| "PTY lock poisoned".to_string())?;
        if let Some(ref pty) = *guard {
            pty.write(data).map_err(|e| format!("PTY write failed: {}", e))?;
            Ok(())
        } else {
            Err("No PTY handle".into())
        }
    }

    // ... (keep rest of VttySession) ...

    pub fn send_keys(&self, keys: &str) -> Result<(), String> {
        let bytes = parse_keys(keys)?;
        self.write(&bytes)
    }

    pub fn send_text(&self, text: &str) -> Result<(), String> {
        let encoded: Vec<u8> = text.replace('\n', "\r").as_bytes().to_vec();
        self.write(&encoded)
    }

    pub fn read_and_update(&self) -> Result<usize, String> {
        let mut buf = [0u8; 65536];
        let mut total = 0;
        loop {
            let n = {
                let guard = self.pty.lock().map_err(|_| "PTY lock poisoned".to_string())?;
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
                self.screen.lock().map_err(|_| "screen lock poisoned")?.process(&buf[..n]);
                total += n;
            }
        }
        Ok(total)
    }

    pub fn screenshot(&self) -> String {
        self.screen.lock().map(|s| s.get_text()).unwrap_or_default()
    }

    pub fn get_line(&self, row: usize) -> String {
        self.screen.lock().map(|s| s.get_line(row)).unwrap_or_default()
    }

    pub fn find_text(&self, pattern: &str) -> Vec<(usize, usize)> {
        self.screen.lock().map(|s| s.find_text(pattern)).unwrap_or_default()
    }

    pub fn resize(&self, new_cols: u16, new_rows: u16) -> Result<(), String> {
        {
            let guard = self.pty.lock().map_err(|_| "PTY lock poisoned".to_string())?;
            if let Some(ref pty) = *guard {
                pty.resize(new_cols, new_rows).map_err(|e| format!("PTY resize failed: {}", e))?;
            }
        }
        self.screen.lock().map_err(|_| "screen lock poisoned")?.resize(new_cols as usize, new_rows as usize);
        Ok(())
    }

    pub fn is_alive(&self) -> bool {
        if !self.alive.load(Ordering::Relaxed) { return false; }
        let guard = self.pty.lock();
        let guard = match guard { Ok(g) => g, Err(_) => return false };
        if let Some(ref pty) = *guard {
            let alive = pty.is_alive();
            if !alive { self.alive.store(false, Ordering::Relaxed); }
            alive
        } else {
            false
        }
    }

    pub fn kill(&mut self) -> Result<(), String> {
        self.alive.store(false, Ordering::Relaxed);
        let mut guard = self.pty.lock().map_err(|_| "PTY lock poisoned".to_string())?;
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
        Self { sessions: Mutex::new(HashMap::new()) }
    }

    pub fn launch(&self, command: &str, cols: u16, rows: u16, _env: &str, cwd: Option<&str>, name: &str) -> Result<SessionInfo, String> {
        let id = format!("vtty-{}", SESSION_COUNTER.fetch_add(1, Ordering::Relaxed));
        let mut session = VttySession::new(id.clone(), name.to_string(), command.to_string(), cols, rows);
        session.launch(cwd)?;
        let info = session.info();
        let arc = Arc::new(Mutex::new(session));
        self.sessions.lock().map_err(|_| "sessions lock poisoned".to_string())?.insert(id.clone(), arc);
        Ok(info)
    }

    pub fn get(&self, sid: &str) -> Result<Arc<Mutex<VttySession>>, String> {
        self.sessions.lock()
            .map_err(|_| "lock poisoned".to_string())?
            .get(sid)
            .cloned()
            .ok_or_else(|| format!("Session '{}' not found", sid))
    }

    pub fn kill(&self, sid: &str) -> Result<SessionInfo, String> {
        let session = self.get(sid)?;
        let mut guard = session.lock().map_err(|_| "session lock poisoned".to_string())?;
        let info = guard.info();
        guard.kill()?;
        self.sessions.lock().map_err(|_| "lock poisoned".to_string())?.remove(sid);
        Ok(info)
    }

    pub fn list(&self) -> Vec<SessionInfo> {
        self.sessions.lock()
            .map(|g| g.values().filter_map(|s| s.lock().ok()).map(|s| s.info()).collect())
            .unwrap_or_default()
    }

    pub fn ping(&self, sid: &str) -> Result<SessionInfo, String> {
        let session = self.get(sid)?;
        let guard = session.lock().map_err(|_| "session lock poisoned".to_string())?;
        // Trigger a read to refresh screen state
        let _ = guard.read_and_update();
        Ok(guard.info())
    }
}

impl Default for VttyManager {
    fn default() -> Self { Self::new() }
}

// ─────────────────────────────────────────────────────
// Key parsing — maps key names to terminal escape sequences
// ─────────────────────────────────────────────────────

fn parse_keys(keys_str: &str) -> Result<Vec<u8>, String> {
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
                let ch = part[5..].chars().next().ok_or_else(|| format!("No letter after ctrl+ in '{}'", part))?;
                if ('a'..='z').contains(&ch.to_ascii_lowercase()) || ('A'..='Z').contains(&ch) {
                    buf.push(ch.to_ascii_uppercase() as u8 - b'A' + 1);
                } else {
                    return Err(format!("Invalid ctrl+key: {}", part));
                }
            }
            s if s.starts_with("ALT+") => {
                buf.push(0x1b);
                for ch in part[4..].chars() { buf.push(ch as u8); }
            }
            s if s.starts_with("SHIFT+") => {
                for ch in part[6..].chars() { buf.push(ch as u8); }
            }
            s => {
                for ch in s.chars() { buf.push(ch as u8); }
            }
        }
    }
    Ok(buf)
}
