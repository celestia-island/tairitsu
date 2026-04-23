//! Daemon mode support for tairitsu-packager
//!
//! Provides background service for compilation with hot-reload capabilities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf, process::Command, sync::OnceLock};

static PROJECT_ROOT: OnceLock<PathBuf> = OnceLock::new();

#[cfg(windows)]
const DAEMON_CHILD_ARG: &str = "--daemon-child-process";

pub fn set_project_root(path: PathBuf) {
    let _ = PROJECT_ROOT.set(path);
}

fn project_root() -> PathBuf {
    PROJECT_ROOT
        .get()
        .cloned()
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn daemon_dir() -> PathBuf {
    project_root().join("target")
}

/// Path to the daemon log file in target directory
fn daemon_log_path() -> PathBuf {
    daemon_dir().join("tairitsu-packager.log")
}

/// Path to the PID file
fn pid_file_path() -> PathBuf {
    daemon_dir().join("tairitsu-packager.pid")
}

/// Path to the daemon readiness signal file
fn ready_file_path() -> PathBuf {
    daemon_dir().join("tairitsu-packager.ready")
}

/// Write the daemon readiness signal (called by child after successful initial build)
pub fn signal_ready() -> std::io::Result<()> {
    if let Some(parent) = ready_file_path().parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(ready_file_path(), "ready")
}

/// Write the daemon failure signal (called by child when initial build fails)
pub fn signal_failed(error: &str) -> std::io::Result<()> {
    if let Some(parent) = ready_file_path().parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(ready_file_path(), error)
}

/// Wait for the daemon child to signal readiness or failure.
/// If `child_pid` is provided, detects early child-exit so the parent
/// doesn't sit idle for the full timeout after a crash.
/// Returns `Ok(true)` if ready, `Ok(false)` if failed (error printed), `Err` on timeout/io.
pub fn wait_for_child_signal(timeout_secs: u64, child_pid: Option<u32>) -> std::io::Result<bool> {
    let ready_path = ready_file_path();
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);

    while start.elapsed() < timeout {
        if let Ok(content) = fs::read_to_string(&ready_path) {
            let _ = fs::remove_file(&ready_path);
            if content.trim() == "ready" {
                return Ok(true);
            } else {
                eprintln!("  ✗  Daemon initial build failed:");
                eprintln!("     {}", content.trim());
                return Ok(false);
            }
        }

        if let Some(pid) = child_pid {
            if !check_process_exists(pid) {
                std::thread::sleep(std::time::Duration::from_millis(500));
                if let Ok(content) = fs::read_to_string(&ready_path) {
                    let _ = fs::remove_file(&ready_path);
                    if content.trim() == "ready" {
                        return Ok(true);
                    }
                    eprintln!("  ✗  Daemon initial build failed:");
                    eprintln!("     {}", content.trim());
                    return Ok(false);
                }
                let _ = fs::remove_file(&ready_path);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    format!("Daemon child process (PID {}) exited unexpectedly. Check logs at {}", pid, daemon_log_path().with_extension("stdout").display()),
                ));
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    let _ = fs::remove_file(&ready_path);
    Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        format!(
            "Daemon did not signal within {}s. Check logs at {}",
            timeout_secs,
            daemon_log_path().with_extension("stdout").display()
        ),
    ))
}

/// Clean up the readiness signal file
pub fn cleanup_ready_file() {
    let _ = fs::remove_file(ready_file_path());
}

/// Daemon status and log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    /// PID of the daemon process
    pub pid: u32,
    /// Start time of the daemon
    pub start_time: DateTime<Utc>,
    /// Recent compilation logs
    pub build_logs: Vec<BuildLogEntry>,
}

/// Single build log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildLogEntry {
    /// Timestamp of the build
    pub timestamp: DateTime<Utc>,
    /// Module being built
    pub module: String,
    /// Whether the build succeeded
    pub success: bool,
    /// Error message if build failed
    pub error: Option<String>,
}

/// Check if running in a TTY
pub fn is_tty() -> bool {
    atty::is(atty::Stream::Stdout)
}

/// Check if the current process is a daemon (already backgrounded)
pub fn is_daemon() -> bool {
    #[cfg(windows)]
    {
        static HAS_DAEMON_CHILD_ARG: OnceLock<bool> = OnceLock::new();

        env::var("TAIRITSU_DAEMON").is_ok()
            || *HAS_DAEMON_CHILD_ARG.get_or_init(|| {
                env::args_os()
                    .skip(1)
                    .any(|arg| arg == std::ffi::OsStr::new(DAEMON_CHILD_ARG))
            })
    }

    #[cfg(not(windows))]
    {
        env::var("TAIRITSU_DAEMON").is_ok()
    }
}

/// Daemonize the current process.
///
/// Performs `setsid()` to detach from the controlling terminal and
/// redirects stdin/stdout/stderr so the process can survive the
/// parent exiting. Does NOT fork — the caller is already a child
/// process spawned by `fork_daemon()`.
#[cfg(unix)]
pub fn daemonize_self() -> std::io::Result<()> {
    use std::fs::File;
    use std::os::unix::io::AsRawFd;

    let stdout = File::create(daemon_log_path().with_extension("stdout"))?;
    let stderr = File::create(daemon_log_path().with_extension("stderr"))?;

    unsafe {
        // Detach from controlling terminal
        if libc::setsid() == -1 {
            return Err(std::io::Error::last_os_error());
        }

        // Redirect stdin to /dev/null
        let devnull = std::ffi::CString::new("/dev/null").unwrap();
        let null_fd = libc::open(devnull.as_ptr(), libc::O_RDWR);
        if null_fd == -1 {
            return Err(std::io::Error::last_os_error());
        }
        if libc::dup2(null_fd, libc::STDIN_FILENO) == -1 {
            libc::close(null_fd);
            return Err(std::io::Error::last_os_error());
        }

        // Redirect stdout
        if libc::dup2(stdout.as_raw_fd(), libc::STDOUT_FILENO) == -1 {
            libc::close(null_fd);
            return Err(std::io::Error::last_os_error());
        }

        // Redirect stderr
        if libc::dup2(stderr.as_raw_fd(), libc::STDERR_FILENO) == -1 {
            libc::close(null_fd);
            return Err(std::io::Error::last_os_error());
        }

        libc::close(null_fd);
    }

    Ok(())
}

#[cfg(windows)]
pub fn daemonize_self() -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use std::os::windows::io::IntoRawHandle;

    let stdout_path = daemon_log_path().with_extension("stdout");
    let stderr_path = daemon_log_path().with_extension("stderr");

    let stdout_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&stdout_path)?;
    let stderr_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&stderr_path)?;

    unsafe {
        use windows_sys::Win32::System::Console::{
            STD_ERROR_HANDLE, STD_OUTPUT_HANDLE, SetStdHandle,
        };

        let stdout_handle = stdout_file.into_raw_handle();
        let stderr_handle = stderr_file.into_raw_handle();

        if SetStdHandle(STD_OUTPUT_HANDLE, stdout_handle) == 0 {
            return Err(std::io::Error::last_os_error());
        }
        if SetStdHandle(STD_ERROR_HANDLE, stderr_handle) == 0 {
            return Err(std::io::Error::last_os_error());
        }
    }

    Ok(())
}

/// Write daemon status to log file
pub fn write_daemon_status(status: &DaemonStatus) -> std::io::Result<()> {
    let log_path = daemon_log_path();

    // Ensure parent directory exists
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(status)?;
    fs::write(&log_path, json)?;

    Ok(())
}

/// Read daemon status from log file
pub fn read_daemon_status() -> std::io::Result<DaemonStatus> {
    let log_path = daemon_log_path();
    let content = fs::read_to_string(&log_path)?;
    let status: DaemonStatus = serde_json::from_str(&content)?;
    Ok(status)
}

/// Write PID file
pub fn write_pid_file(pid: u32) -> std::io::Result<()> {
    let pid_path = pid_file_path();

    if let Some(parent) = pid_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&pid_path, pid.to_string())?;
    Ok(())
}

/// Read PID from PID file
pub fn read_pid() -> std::io::Result<u32> {
    let pid_path = pid_file_path();
    let content = fs::read_to_string(&pid_path)?;
    Ok(content.trim().parse().unwrap_or(0))
}

/// Check if a daemon is running
pub fn is_daemon_running() -> bool {
    if let Ok(pid) = read_pid()
        && pid > 0
    {
        return check_process_exists(pid);
    }
    false
}

/// Check if a process with the given PID exists
pub fn check_process_exists(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use std::process::Command;
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[cfg(windows)]
    {
        // On Windows, use tasklist to check if process exists
        Command::new("tasklist")
            .args(&["/FI", &format!("PID eq {}", pid)])
            .output()
            .map(|output| {
                let output = String::from_utf8_lossy(&output.stdout);
                output.contains(&pid.to_string())
            })
            .unwrap_or(false)
    }
}

/// Information about the process owning a port.
pub struct PortOwnerInfo {
    pub pid: u32,
    pub exe_path: Option<PathBuf>,
}

/// Find the PID of the process listening on `port` (TCP, IPv4 localhost).
fn find_pid_on_port(port: u16) -> Option<u32> {
    #[cfg(windows)]
    {
        let output = Command::new("netstat")
            .args(&["-ano", "-p", "TCP"])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        let needle = format!("  127.0.0.1:{} ", port);
        for line in text.lines() {
            if line.contains(&needle) && line.contains("LISTENING") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(pid_str) = parts.last() {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        return Some(pid);
                    }
                }
            }
        }
        None
    }

    #[cfg(unix)]
    {
        let output = Command::new("lsof")
            .args(&["-ti", &format!(":{}", port), "-sTCP:LISTEN"])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        text.lines()
            .next()
            .and_then(|line| line.trim().parse::<u32>().ok())
    }
}

/// Get the executable path of a process by PID.
fn get_process_exe(pid: u32) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let output = Command::new("wmic")
            .args(&[
                "process",
                &format!("where ProcessId={}", pid),
                "get",
                "ExecutablePath",
                "/format:value",
            ])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if let Some(path) = line.strip_prefix("ExecutablePath=") {
                let trimmed = path.trim();
                if !trimmed.is_empty() {
                    return Some(PathBuf::from(trimmed));
                }
            }
        }
        None
    }

    #[cfg(unix)]
    {
        std::fs::read_link(format!("/proc/{}/exe", pid)).ok()
    }
}

/// Return information about the process currently listening on `port`.
///
/// Uses platform-native tools (`netstat`/`lsof`) to find the owning PID,
/// then resolves its executable path.
pub fn port_owner_info(port: u16) -> Option<PortOwnerInfo> {
    let pid = find_pid_on_port(port)?;
    let exe_path = get_process_exe(pid);
    Some(PortOwnerInfo { pid, exe_path })
}

/// Kill the daemon process and wait for it to exit
pub fn kill_daemon() -> std::io::Result<bool> {
    if let Ok(pid) = read_pid()
        && pid > 0
        && check_process_exists(pid)
    {
        #[cfg(unix)]
        {
            let _ = Command::new("kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .output();
        }

        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(&["/PID", &pid.to_string(), "/F"])
                .output();
        }

        // Wait for the process to actually exit (up to 5s)
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(5) {
            if !check_process_exists(pid) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        if check_process_exists(pid) {
            eprintln!("Warning: daemon process {} did not exit within 5s", pid);
        }

        let _ = fs::remove_file(pid_file_path());
        return Ok(true);
    }
    Ok(false)
}

#[cfg(windows)]
fn to_wide_null(value: &std::ffi::OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    value
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>()
}

#[cfg(windows)]
fn append_windows_quoted_arg(command_line: &mut Vec<u16>, value: &std::ffi::OsStr) {
    use std::os::windows::ffi::OsStrExt;

    const SPACE: u16 = b' ' as u16;
    const TAB: u16 = b'\t' as u16;
    const QUOTE: u16 = b'"' as u16;
    const BACKSLASH: u16 = b'\\' as u16;

    let units = value.encode_wide().collect::<Vec<u16>>();
    let needs_quotes = units.is_empty()
        || units
            .iter()
            .any(|unit| matches!(*unit, SPACE | TAB | QUOTE));

    if !needs_quotes {
        command_line.extend_from_slice(&units);
        return;
    }

    command_line.push(QUOTE);

    let mut pending_backslashes = 0usize;
    for unit in units {
        match unit {
            BACKSLASH => {
                pending_backslashes += 1;
            }
            QUOTE => {
                command_line.extend(std::iter::repeat_n(BACKSLASH, pending_backslashes * 2 + 1));
                command_line.push(unit);
                pending_backslashes = 0;
            }
            _ => {
                command_line.extend(std::iter::repeat_n(BACKSLASH, pending_backslashes));
                command_line.push(unit);
                pending_backslashes = 0;
            }
        }
    }

    command_line.extend(std::iter::repeat_n(BACKSLASH, pending_backslashes * 2));
    command_line.push(QUOTE);
}

#[cfg(windows)]
fn build_windows_command_line(exe: &std::path::Path, args: &[std::ffi::OsString]) -> Vec<u16> {
    let mut command_line = Vec::new();

    append_windows_quoted_arg(&mut command_line, exe.as_os_str());
    for arg in args {
        command_line.push(b' ' as u16);
        append_windows_quoted_arg(&mut command_line, arg.as_os_str());
    }
    command_line.push(b' ' as u16);
    append_windows_quoted_arg(&mut command_line, std::ffi::OsStr::new(DAEMON_CHILD_ARG));
    command_line.push(0);

    command_line
}

#[cfg(windows)]
fn spawn_daemon_process_windows(
    exe: &std::path::Path,
    args: &[std::ffi::OsString],
) -> std::io::Result<u32> {
    use std::{io, mem, ptr};
    use windows_sys::Win32::Foundation::{
        CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };
    use windows_sys::Win32::System::Threading::{
        CreateProcessW, DeleteProcThreadAttributeList, EXTENDED_STARTUPINFO_PRESENT,
        InitializeProcThreadAttributeList, PROC_THREAD_ATTRIBUTE_HANDLE_LIST, PROCESS_INFORMATION,
        STARTF_USESTDHANDLES, STARTUPINFOEXW, UpdateProcThreadAttribute,
    };

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    struct OwnedHandle(HANDLE);

    impl OwnedHandle {
        fn nul(desired_access: u32) -> io::Result<Self> {
            let nul = to_wide_null(std::ffi::OsStr::new("NUL"));
            let mut security_attributes = SECURITY_ATTRIBUTES {
                nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: ptr::null_mut(),
                bInheritHandle: 1,
            };

            let handle = unsafe {
                CreateFileW(
                    nul.as_ptr(),
                    desired_access,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    &mut security_attributes,
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    ptr::null_mut(),
                )
            };

            if handle.is_null() || handle == INVALID_HANDLE_VALUE {
                return Err(io::Error::last_os_error());
            }

            Ok(Self(handle))
        }

        fn raw(&self) -> HANDLE {
            self.0
        }
    }

    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
                unsafe {
                    CloseHandle(self.0);
                }
            }
        }
    }

    struct OwnedAttributeList {
        _buffer: Vec<u8>,
        ptr: *mut std::ffi::c_void,
    }

    impl OwnedAttributeList {
        fn new(attribute_count: u32) -> io::Result<Self> {
            let mut bytes_required = 0usize;

            unsafe {
                let _ = InitializeProcThreadAttributeList(
                    ptr::null_mut(),
                    attribute_count,
                    0,
                    &mut bytes_required,
                );
            }

            let mut buffer = vec![0u8; bytes_required];
            let ptr = buffer.as_mut_ptr() as *mut std::ffi::c_void;

            let initialized = unsafe {
                InitializeProcThreadAttributeList(ptr, attribute_count, 0, &mut bytes_required)
            };
            if initialized == 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(Self {
                _buffer: buffer,
                ptr,
            })
        }

        fn set_handle_list(&mut self, handles: &mut [HANDLE]) -> io::Result<()> {
            let updated = unsafe {
                UpdateProcThreadAttribute(
                    self.ptr,
                    0,
                    PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                    handles.as_mut_ptr() as *mut _,
                    mem::size_of_val(handles),
                    ptr::null_mut(),
                    ptr::null_mut(),
                )
            };
            if updated == 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(())
        }
    }

    impl Drop for OwnedAttributeList {
        fn drop(&mut self) {
            unsafe {
                DeleteProcThreadAttributeList(self.ptr);
            }
        }
    }

    let stdin_handle = OwnedHandle::nul(GENERIC_READ)?;
    let stdout_handle = OwnedHandle::nul(GENERIC_WRITE)?;
    let stderr_handle = OwnedHandle::nul(GENERIC_WRITE)?;

    let mut inherited_handles = [stdin_handle.raw(), stdout_handle.raw(), stderr_handle.raw()];
    let mut attribute_list = OwnedAttributeList::new(1)?;
    attribute_list.set_handle_list(&mut inherited_handles)?;

    let application_name = to_wide_null(exe.as_os_str());
    let mut command_line = build_windows_command_line(exe, args);

    let mut startup_info = STARTUPINFOEXW {
        StartupInfo: unsafe { mem::zeroed() },
        lpAttributeList: attribute_list.ptr,
    };
    startup_info.StartupInfo.cb = mem::size_of::<STARTUPINFOEXW>() as u32;
    startup_info.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup_info.StartupInfo.hStdInput = stdin_handle.raw();
    startup_info.StartupInfo.hStdOutput = stdout_handle.raw();
    startup_info.StartupInfo.hStdError = stderr_handle.raw();

    let mut process_info: PROCESS_INFORMATION = unsafe { mem::zeroed() };
    let created = unsafe {
        CreateProcessW(
            application_name.as_ptr(),
            command_line.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            1,
            CREATE_NO_WINDOW | EXTENDED_STARTUPINFO_PRESENT,
            ptr::null(),
            ptr::null(),
            &mut startup_info as *mut STARTUPINFOEXW as *mut _,
            &mut process_info,
        )
    };
    if created == 0 {
        return Err(io::Error::last_os_error());
    }

    let _process_handle = OwnedHandle(process_info.hProcess);
    let _thread_handle = OwnedHandle(process_info.hThread);

    Ok(process_info.dwProcessId)
}

/// Fork the process to run as a daemon.
/// Returns the child PID on success.
pub fn fork_daemon() -> std::io::Result<u32> {
    fork_daemon_with_args(std::env::args())
}

/// Fork the process to run as a daemon with specific arguments.
/// Returns the child PID on success.
pub fn fork_daemon_with_args<I, S>(args: I) -> std::io::Result<u32>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    #[cfg(unix)]
    {
        let exe = env::current_exe()?;
        let args: Vec<String> = args
            .into_iter()
            .skip(1)
            .map(|s| s.as_ref().to_string_lossy().into_owned())
            .collect();

        let log_path = daemon_log_path();
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let child = Command::new(&exe)
            .env("TAIRITSU_DAEMON", "1")
            .args(&args)
            .spawn()?;

        let pid = child.id();
        // Child::drop() would wait for the daemon to exit; detach instead.
        std::mem::forget(child);

        Ok(pid)
    }

    #[cfg(windows)]
    {
        let exe = env::current_exe()?;
        let args: Vec<std::ffi::OsString> = args
            .into_iter()
            .skip(1)
            .map(|s| s.as_ref().to_os_string())
            .collect();

        let log_path = daemon_log_path();
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let pid = spawn_daemon_process_windows(&exe, &args)?;
        Ok(pid)
    }
}

/// Print daemon status and recent logs
pub fn print_daemon_status() -> std::io::Result<()> {
    if !is_daemon_running() {
        println!("No daemon is currently running.");
        println!();
        println!("Start a daemon with: tairitsu dev --daemon");
        return Ok(());
    }

    let status = read_daemon_status()?;
    let uptime = Utc::now() - status.start_time;

    println!("Daemon Status:");
    println!("  PID: {}", status.pid);
    println!(
        "  Running since: {}",
        status.start_time.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("  Uptime: {}", format_duration(uptime));
    println!();

    if status.build_logs.is_empty() {
        println!("No builds recorded yet.");
    } else {
        println!("Recent builds ({}):", status.build_logs.len());
        for log in status.build_logs.iter().rev().take(10) {
            let status_icon = if log.success { "✓" } else { "✗" };
            println!(
                "  [{}] {} {} - {}",
                log.timestamp.format("%Y-%m-%d %H:%M:%S"),
                status_icon,
                log.module,
                if log.success { "OK" } else { "FAILED" }
            );
            if let Some(error) = &log.error {
                println!("    Error: {}", error);
            }
        }
    }

    Ok(())
}

/// Format a duration for display
fn format_duration(duration: chrono::Duration) -> String {
    let secs = duration.num_seconds();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Print non-TTY message with daemon hint
pub fn print_non_tty_hint() {
    eprintln!("Not running in a TTY - running in non-interactive mode.");
    eprintln!();
    eprintln!("For development with hot-reload, use the daemon mode:");
    eprintln!("  tairitsu dev --daemon");
    eprintln!();
    eprintln!("To control the daemon:");
    eprintln!("  tairitsu dev --daemon       # Start daemon (or show status if running)");
    eprintln!("  tairitsu dev --daemon --force   # Restart daemon");
    eprintln!("  tairitsu dev --daemon --shutdown # Stop daemon");
}

/// Append a build log entry to the daemon status
pub fn append_build_log(module: &str, success: bool, error: Option<&str>) -> std::io::Result<()> {
    let mut status = if daemon_log_path().exists() {
        read_daemon_status().unwrap_or_else(|_| DaemonStatus {
            pid: std::process::id(),
            start_time: Utc::now(),
            build_logs: Vec::new(),
        })
    } else {
        DaemonStatus {
            pid: std::process::id(),
            start_time: Utc::now(),
            build_logs: Vec::new(),
        }
    };

    status.pid = std::process::id();

    let entry = BuildLogEntry {
        timestamp: Utc::now(),
        module: module.to_string(),
        success,
        error: error.map(|s| s.to_string()),
    };

    status.build_logs.push(entry);

    // Keep only last 100 entries
    if status.build_logs.len() > 100 {
        status.build_logs = status.build_logs.into_iter().rev().take(100).collect();
        status.build_logs.reverse();
    }

    write_daemon_status(&status)?;
    Ok(())
}
