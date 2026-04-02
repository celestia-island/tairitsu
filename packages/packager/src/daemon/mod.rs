//! Daemon mode support for tairitsu-packager
//!
//! Provides background service for compilation with hot-reload capabilities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    path::PathBuf,
    process::Command,
    sync::OnceLock,
};

static PROJECT_ROOT: OnceLock<PathBuf> = OnceLock::new();

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
    env::var("TAIRITSU_DAEMON").is_ok()
}

/// Daemonize the current process using the `daemonize` crate.
///
/// Performs double-fork + setsid so the process detaches from the
/// controlling terminal and survives the parent exiting.
#[cfg(unix)]
pub fn daemonize_self() -> std::io::Result<()> {
    use daemonize::Daemonize;
    use std::fs::File;

    let stdout = File::create(daemon_log_path().with_extension("stdout"))?;
    let stderr = File::create(daemon_log_path().with_extension("stderr"))?;

    let cwd = env::current_dir()?;

    let daemonize = Daemonize::new()
        .working_directory(&cwd)
        .stdout(stdout)
        .stderr(stderr);

    daemonize.start().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("daemonize failed: {}", e),
        )
    })
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
    if let Ok(pid) = read_pid() {
        if pid > 0 {
            // Check if process exists
            return check_process_exists(pid);
        }
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

/// Kill the daemon process
pub fn kill_daemon() -> std::io::Result<bool> {
    if let Ok(pid) = read_pid() {
        if pid > 0 && check_process_exists(pid) {
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

            // Remove PID file
            let _ = fs::remove_file(pid_file_path());
            return Ok(true);
        }
    }
    Ok(false)
}

/// Fork the process to run as a daemon
pub fn fork_daemon() -> std::io::Result<()> {
    fork_daemon_with_args(std::env::args())
}

/// Fork the process to run as a daemon with specific arguments
pub fn fork_daemon_with_args<I, S>(args: I) -> std::io::Result<()>
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

        Command::new(&exe)
            .env("TAIRITSU_DAEMON", "1")
            .args(&args)
            .stdout(File::create(log_path.with_extension("stdout"))?)
            .stderr(File::create(log_path.with_extension("stderr"))?)
            .spawn()?;

        Ok(())
    }

    #[cfg(windows)]
    {
        let exe = env::current_exe()?;
        let args: Vec<String> = args
            .into_iter()
            .skip(1) // Skip argv[0] which is the program name
            .map(|s| s.as_ref().to_string_lossy().into_owned())
            .collect();

        // Ensure parent directory exists for log files
        let log_path = daemon_log_path();
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // On Windows, use START to create a detached process
        let _ = Command::new("cmd")
            .args(&["/C", "start", "/B", &exe.to_string_lossy()])
            .args(&args)
            .env("TAIRITSU_DAEMON", "1")
            .spawn()?;

        Ok(())
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
