use std::fmt;
use std::io::Write;

fn stdout_is_tty() -> bool {
    atty::is(atty::Stream::Stdout)
}

fn stderr_is_tty() -> bool {
    atty::is(atty::Stream::Stderr)
}

enum Level {
    Ok,
    Warn,
    Fail,
    Info,
    Progress,
}

impl Level {
    fn tag(&self) -> &'static str {
        match self {
            Level::Ok => "  OK  ",
            Level::Warn => " WARN ",
            Level::Fail => " FAIL ",
            Level::Info => " INFO ",
            Level::Progress => "  ..  ",
        }
    }

    fn color_code(&self) -> &'static str {
        match self {
            Level::Ok => "\x1b[1;32m",
            Level::Warn => "\x1b[1;33m",
            Level::Fail => "\x1b[1;31m",
            Level::Info => "\x1b[1;36m",
            Level::Progress => "\x1b[2m",
        }
    }
}

const RESET: &str = "\x1b[0m";

fn format_tag(level: &Level, use_color: bool) -> String {
    let tag = format!("[{}]", level.tag());
    if use_color {
        format!("{}{}{}", level.color_code(), tag, RESET)
    } else {
        tag
    }
}

fn is_daemon() -> bool {
    #[cfg(feature = "tokio")]
    {
        if crate::daemon::is_daemon() {
            return true;
        }
    }
    if std::env::var("TAIRITSU_DAEMON").is_ok() {
        return true;
    }
    #[cfg(windows)]
    {
        static CACHED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| {
            let v = std::env::args_os()
                .skip(1)
                .any(|arg| arg == std::ffi::OsStr::new("--daemon-child-process"));
            CACHED.store(v, std::sync::atomic::Ordering::Relaxed);
        });
        CACHED.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn should_timestamp() -> bool {
    is_daemon() && !stdout_is_tty()
}

fn emit(level: Level, stream: StdStream, args: fmt::Arguments<'_>) {
    let use_color = match stream {
        StdStream::Stdout => stdout_is_tty(),
        StdStream::Stderr => stderr_is_tty(),
    };
    let tag = format_tag(&level, use_color);
    let msg = args.to_string();
    let ts = if should_timestamp() {
        format!(
            "{} ",
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3f")
        )
    } else {
        String::new()
    };
    let line = format!("{}{} {}", ts, tag, msg);

    match stream {
        StdStream::Stdout => {
            let _ = writeln!(std::io::stdout(), "{}", line);
            let _ = std::io::stdout().flush();
        }
        StdStream::Stderr => {
            let _ = writeln!(std::io::stderr(), "{}", line);
            let _ = std::io::stderr().flush();
        }
    }
}

#[derive(Clone, Copy)]
enum StdStream {
    Stdout,
    Stderr,
}

macro_rules! logfn {
    ($name:ident, $level:expr, $stream:expr) => {
        #[allow(unused_macros)]
        pub fn $name(args: fmt::Arguments<'_>) {
            emit($level, $stream, args)
        }
    };
}

logfn!(ok, Level::Ok, StdStream::Stdout);
logfn!(warn, Level::Warn, StdStream::Stderr);
logfn!(fail, Level::Fail, StdStream::Stderr);
logfn!(info, Level::Info, StdStream::Stdout);
logfn!(progress, Level::Progress, StdStream::Stdout);

#[macro_export]
macro_rules! log_ok {
    ($($arg:tt)*) => { $crate::logfmt::ok(format_args!($($arg)*)) }
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => { $crate::logfmt::warn(format_args!($($arg)*)) }
}

#[macro_export]
macro_rules! log_fail {
    ($($arg:tt)*) => { $crate::logfmt::fail(format_args!($($arg)*)) }
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => { $crate::logfmt::info(format_args!($($arg)*)) }
}

#[macro_export]
macro_rules! log_progress {
    ($($arg:tt)*) => { $crate::logfmt::progress(format_args!($($arg)*)) }
}
