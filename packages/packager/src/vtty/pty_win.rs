//! Cross-platform PTY backend using `portable-ty` crate.
//!
//! On Windows: ConPTY via native_pty_system()
//! On Unix: forkpty via native_pty_system()

use std::io::{self, Read, Write};
use std::sync::{Mutex, MutexGuard};

use portable_pty::{
    CommandBuilder, native_pty_system, PtySize,
    MasterPty, Child, ChildKiller,
};

fn to_io(e: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e.to_string())
}

/// A cross-platform PTY session.
pub struct ConPty {
    master: Box<dyn MasterPty + Send>,
    child: Mutex<Box<dyn Child + Send + Sync>>,
    killer: Box<dyn ChildKiller + Send + Sync>,
    writer: Mutex<Option<Box<dyn Write + Send>>>,
}

impl ConPty {
    /// Spawn a command in a new PTY session.
    pub fn spawn(command: &str, cols: u16, rows: u16, cwd: Option<&str>) -> io::Result<(Self, u32)> {
        let pty_system = native_pty_system();
        let size = PtySize {
            rows, cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        let pair = pty_system.openpty(size).map_err(to_io)?;

        let mut cmd = CommandBuilder::new(command);
        if let Some(dir) = cwd { cmd.cwd(dir); }
        let child = pair.slave.spawn_command(cmd).map_err(to_io)?;

        let pid = child.process_id().unwrap_or(0);
        let killer = child.clone_killer();

        Ok((
            Self {
                master: pair.master,
                child: Mutex::new(child),
                killer,
                writer: Mutex::new(None),
            },
            pid,
        ))
    }

    /// Write data to the PTY input.
    pub fn write(&self, data: &[u8]) -> io::Result<usize> {
        let mut guard = self.writer.lock().map_err(|_| to_io("writer lock poisoned"))?;
        if guard.is_none() {
            *guard = Some(self.master.take_writer().map_err(to_io)?);
        }
        if let Some(ref mut w) = *guard {
            w.write(data)
        } else {
            Err(to_io("no writer"))
        }
    }

    /// Read available data from PTY output (non-blocking clone reader).
    pub fn read_nonblocking(&self, buf: &mut [u8]) -> io::Result<usize> {
        match self.master.try_clone_reader() {
            Ok(mut reader) => reader.read(buf),
            Err(e) => Err(to_io(e)),
        }
    }

    /// Resize the PTY.
    pub fn resize(&self, cols: u16, rows: u16) -> io::Result<()> {
        self.master.resize(PtySize {
            rows, cols, pixel_width: 0, pixel_height: 0,
        }).map_err(to_io)
    }

    /// Check if child process is still alive.
    pub fn is_alive(&self) -> bool {
        let mut guard = match self.child.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };
        match guard.try_wait() {
            Ok(None) => true,
            Ok(Some(_)) => false,
            Err(_) => false,
        }
    }

    /// Kill the child process and close PTY.
    pub fn kill(&mut self) -> io::Result<()> {
        self.killer.kill()
    }

    /// Returns the PID of the child process.
    pub fn pid(&self) -> u32 {
        self.child.lock().ok().and_then(|c| c.process_id()).unwrap_or(0)
    }
}
