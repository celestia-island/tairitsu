use std::{
    io::{self, Read, Write},
    os::unix::io::RawFd,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use portable_pty::{Child, ChildKiller, CommandBuilder, MasterPty, PtySize, native_pty_system};

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
        unsafe { libc::close(self.read_fd) };
    }
}

fn make_nonblocking(fd: RawFd) -> io::Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL, 0) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn extract_master_fd(master: &dyn MasterPty) -> RawFd {
    use std::os::unix::io::IntoRawFd;
    let reader: Box<dyn Read + Send> = master.try_clone_reader().expect("clone PTY reader");
    let raw: *mut dyn Read = Box::into_raw(reader);
    let file: Box<std::fs::File> = unsafe { Box::from_raw(raw as *mut std::fs::File) };
    file.into_raw_fd()
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

        let read_fd = extract_master_fd(&*pair.master);
        make_nonblocking(read_fd)?;

        let mut cmd = CommandBuilder::new("/bin/bash");
        cmd.arg("-c");
        cmd.arg(command);
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
            w.write(data)
        } else {
            Err(to_io("no writer"))
        }
    }

    pub fn read_nonblocking(&self, buf: &mut [u8]) -> io::Result<usize> {
        let n = unsafe {
            libc::read(
                self.read_fd,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
            )
        };
        if n >= 0 {
            Ok(n as usize)
        } else {
            let err = io::Error::last_os_error();
            if err.kind() == io::ErrorKind::WouldBlock {
                Ok(0)
            } else {
                Err(err)
            }
        }
    }

    pub fn reader_loop(
        read_fd: RawFd,
        screen: Arc<std::sync::Mutex<super::screen::Vt100Screen>>,
        running: Arc<AtomicBool>,
    ) {
        let mut buf = vec![0u8; 65536];
        while running.load(Ordering::Relaxed) {
            let n =
                unsafe { libc::read(read_fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
            if n > 0 {
                if let Ok(mut s) = screen.lock() {
                    s.process(&buf[..n as usize]);
                }
            } else if n == 0 {
                break;
            } else {
                let err = io::Error::last_os_error();
                if err.kind() != io::ErrorKind::WouldBlock {
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
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

    pub fn kill(&mut self) -> io::Result<()> {
        self.killer.kill()
    }

    pub fn pid(&self) -> u32 {
        self.child
            .lock()
            .ok()
            .and_then(|c| c.process_id())
            .unwrap_or(0)
    }
}
