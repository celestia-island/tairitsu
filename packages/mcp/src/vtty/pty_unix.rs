use std::{
    io::{self, Write},
    os::unix::io::RawFd,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use portable_pty::{native_pty_system, Child, ChildKiller, CommandBuilder, MasterPty, PtySize};

fn to_io(e: impl std::fmt::Display) -> io::Error {
    io::Error::other(e.to_string())
}

pub struct UnixPty {
    master: Box<dyn MasterPty + Send>,
    read_fd: RawFd,
    child: Mutex<Box<dyn Child + Send + Sync>>,
    killer: Box<dyn ChildKiller + Send + Sync>,
    writer: Mutex<Option<Box<dyn Write + Send>>>,
}

impl Drop for UnixPty {
    fn drop(&mut self) {
        if self.read_fd >= 0 {
            unsafe { libc::close(self.read_fd) };
        }
    }
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

        let master_fd = pair
            .master
            .as_raw_fd()
            .ok_or_else(|| to_io("master has no raw fd"))?;

        let read_fd = unsafe { libc::dup(master_fd) };
        if read_fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let mut cmd = CommandBuilder::new("/bin/bash");
        cmd.arg("-c");
        cmd.arg(command);
        cmd.env("TERM", "xterm-256color");
        if let Some(dir) = cwd {
            cmd.cwd(dir);
        }
        let child = pair.slave.spawn_command(cmd).map_err(to_io)?;

        let killer = child.clone_killer();

        Ok(Self {
            master: pair.master,
            read_fd,
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

    pub fn reader_loop(
        read_fd: RawFd,
        screen: Arc<std::sync::Mutex<super::screen::Vt100Screen>>,
        running: Arc<AtomicBool>,
    ) {
        eprintln!("[vtty-reader] started, fd={}", read_fd);
        let mut buf = vec![0u8; 65536];
        let mut pfd = libc::pollfd {
            fd: read_fd,
            events: libc::POLLIN,
            revents: 0,
        };
        while running.load(Ordering::Relaxed) {
            pfd.revents = 0;
            let ready = unsafe { libc::poll(&mut pfd, 1, 100) };
            if ready < 0 {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                eprintln!("[vtty-reader] poll error on fd={}: {}", read_fd, err);
                break;
            }
            if ready > 0 && (pfd.revents & libc::POLLIN) != 0 {
                let n = unsafe {
                    libc::read(read_fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len())
                };
                if n > 0 {
                    if let Ok(mut s) = screen.lock() {
                        s.process(&buf[..n as usize]);
                    }
                } else if n == 0 {
                    eprintln!("[vtty-reader] EOF on fd={}", read_fd);
                    break;
                } else {
                    let err = io::Error::last_os_error();
                    if err.kind() == io::ErrorKind::Interrupted {
                        continue;
                    }
                    if err.kind() != io::ErrorKind::WouldBlock {
                        eprintln!("[vtty-reader] read error on fd={}: {}", read_fd, err);
                        break;
                    }
                }
            }
            if pfd.revents & (libc::POLLHUP | libc::POLLERR | libc::POLLNVAL) != 0 {
                loop {
                    let n = unsafe {
                        libc::read(read_fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len())
                    };
                    if n > 0 {
                        if let Ok(mut s) = screen.lock() {
                            s.process(&buf[..n as usize]);
                        }
                    } else {
                        break;
                    }
                }
                eprintln!(
                    "[vtty-reader] poll hangup/error on fd={}, revents={}",
                    read_fd, pfd.revents
                );
                break;
            }
        }
        eprintln!("[vtty-reader] exiting, fd={}", read_fd);
    }

    pub fn read_fd(&self) -> RawFd {
        self.read_fd
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
