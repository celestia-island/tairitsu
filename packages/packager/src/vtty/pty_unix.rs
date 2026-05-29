use std::{
    io::{self, Read, Write},
    sync::Mutex,
    time::Duration,
};

use portable_pty::{native_pty_system, Child, ChildKiller, CommandBuilder, MasterPty, PtySize};

fn to_io(e: impl std::fmt::Display) -> io::Error {
    io::Error::other(e.to_string())
}

pub struct UnixPty {
    master: Box<dyn MasterPty + Send>,
    child: Mutex<Box<dyn Child + Send + Sync>>,
    killer: Box<dyn ChildKiller + Send + Sync>,
    writer: Mutex<Option<Box<dyn Write + Send>>>,
}

impl UnixPty {
    pub fn spawn(command: &str, cols: u16, rows: u16, cwd: Option<&str>) -> io::Result<Self> {
        let pty_system = native_pty_system();
        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        let pair = pty_system.openpty(size).map_err(to_io)?;

        let mut cmd = CommandBuilder::new("/bin/bash");
        cmd.arg("-c");
        cmd.arg(command);
        cmd.env("TERM", "xterm-256color");
        if let Some(dir) = cwd {
            cmd.cwd(dir);
        }
        let child = pair.slave.spawn_command(cmd).map_err(to_io)?;

        {
            let master_fd = pair.master.as_raw_fd().unwrap_or(-1);
            if master_fd >= 0 {
                let mut termios: libc::termios = unsafe { std::mem::zeroed() };
                if unsafe { libc::tcgetattr(master_fd, &mut termios) } == 0 {
                    termios.c_lflag &= !(libc::ECHO | libc::ECHONL);
                    unsafe { libc::tcsetattr(master_fd, libc::TCSANOW, &termios) };
                }
            }
        }

        let killer = child.clone_killer();

        Ok(Self {
            master: pair.master,
            child: Mutex::new(child),
            killer,
            writer: Mutex::new(None),
        })
    }

    pub fn write(&self, data: &[u8]) -> io::Result<usize> {
        let mut guard = self
            .writer
            .lock()
            .map_err(|_| to_io("writer lock poisoned"))?;
        if guard.is_none() {
            *guard = Some(self.master.take_writer().map_err(to_io)?);
        }
        if let Some(ref mut w) = *guard {
            w.write_all(data)?;
            w.flush()?;
            Ok(data.len())
        } else {
            Err(to_io("no writer"))
        }
    }

    pub fn read_nonblocking(&self, buf: &mut [u8]) -> io::Result<usize> {
        let reader = self.master.try_clone_reader().map_err(to_io)?;
        let (tx, rx) = std::sync::mpsc::channel();
        let mut owned_buf = vec![0u8; buf.len()];
        std::thread::spawn(move || {
            let n = {
                let mut r = reader;
                r.read(&mut owned_buf)
            };
            let _ = tx.send((n, owned_buf));
        });
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok((n, data)) => {
                let len = n.unwrap_or(0).min(buf.len().min(data.len()));
                buf[..len].copy_from_slice(&data[..len]);
                Ok(len)
            }
            Err(_) => Ok(0),
        }
    }

    pub fn resize(&self, cols: u16, rows: u16) -> io::Result<()> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(to_io)
    }

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

    pub fn kill_and_reap(&mut self) -> io::Result<()> {
        self.killer.kill()?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let mut guard = match self.child.lock() {
                Ok(g) => g,
                Err(_) => break,
            };
            match guard.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if std::time::Instant::now() > deadline {
                        let _ = self.killer.kill();
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        let _ = guard.try_wait();
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(_) => break,
            }
        }
        Ok(())
    }

    pub fn pid(&self) -> u32 {
        self.child
            .lock()
            .ok()
            .and_then(|c| c.process_id())
            .unwrap_or(0)
    }
}
