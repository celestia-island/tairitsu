use std::{cell::RefCell, fmt, io::Write};

use tracing_subscriber::layer::SubscriberExt;

thread_local! {
    static ACTIVE_PB: RefCell<Option<indicatif::ProgressBar>> = const { RefCell::new(None) };
}

pub fn set_active_pb(pb: &indicatif::ProgressBar) {
    ACTIVE_PB.with(|cell| {
        *cell.borrow_mut() = Some(pb.clone());
    });
}

pub fn clear_active_pb() {
    ACTIVE_PB.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

fn stdout_is_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

fn stderr_is_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stderr().is_terminal()
}

enum Level {
    Ok,
    Warn,
    Fail,
    Info,
    Progress,
    Debug,
}

impl Level {
    fn tag(&self) -> &'static str {
        match self {
            Level::Ok => "  OK  ",
            Level::Warn => " WARN ",
            Level::Fail => " FAIL ",
            Level::Info => " INFO ",
            Level::Progress => "  ..  ",
            Level::Debug => " DEBUG ",
        }
    }

    fn color_code(&self) -> &'static str {
        match self {
            Level::Ok => "\x1b[1;32m",
            Level::Warn => "\x1b[1;33m",
            Level::Fail => "\x1b[1;31m",
            Level::Info => "\x1b[1;36m",
            Level::Progress => "\x1b[2m",
            Level::Debug => "\x1b[2m",
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
    if std::env::var("TAIRITSU_LOG_TS").is_ok() {
        return true;
    }
    is_daemon() && !stdout_is_tty()
}

fn should_color() -> bool {
    if std::env::var("TAIRITSU_COLOR").is_ok() {
        return true;
    }
    stdout_is_tty() || stderr_is_tty()
}

fn emit(level: Level, stream: StdStream, args: fmt::Arguments<'_>) {
    let use_color = should_color();
    let tag = format_tag(&level, use_color);
    let msg = args.to_string();
    let ts = if should_timestamp() {
        format!("{} ", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3f"))
    } else {
        String::new()
    };
    let line = format!("{}{} {}", ts, tag, msg);

    match stream {
        StdStream::Stdout => {
            let handled = ACTIVE_PB.with(|cell| {
                if let Some(ref pb) = *cell.borrow() {
                    pb.println(line.clone());
                    true
                } else {
                    false
                }
            });
            if !handled {
                let _ = writeln!(std::io::stdout(), "{}", line);
                let _ = std::io::stdout().flush();
            }
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
logfn!(debug_log, Level::Debug, StdStream::Stderr);

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

pub struct LogfmtLayer {
    max_level: tracing::Level,
}

impl LogfmtLayer {
    pub fn new(max_level: tracing::Level) -> Self {
        Self { max_level }
    }
}

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for LogfmtLayer {
    fn enabled(
        &self,
        metadata: &tracing::Metadata<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        metadata.level() <= &self.max_level
    }

    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = EventVisitor(String::new());
        event.record(&mut visitor);

        let (level, stream) = match *event.metadata().level() {
            tracing::Level::ERROR => (Level::Fail, StdStream::Stderr),
            tracing::Level::WARN => (Level::Warn, StdStream::Stderr),
            tracing::Level::INFO => (Level::Info, StdStream::Stdout),
            tracing::Level::DEBUG => (Level::Debug, StdStream::Stderr),
            tracing::Level::TRACE => (Level::Debug, StdStream::Stderr),
        };

        let args = format_args!("{}", visitor.0);
        emit(level, stream, args);
    }
}

struct EventVisitor(String);

impl tracing::field::Visit for EventVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0 = value.to_string();
        } else if !self.0.is_empty() {
            self.0.push_str(&format!(" {}={}", field.name(), value));
        } else {
            self.0 = format!("{}={}", field.name(), value);
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{:?}", value);
        } else if !self.0.is_empty() {
            self.0.push_str(&format!(" {}={:?}", field.name(), value));
        } else {
            self.0 = format!("{}={:?}", field.name(), value);
        }
    }
}

pub fn init_tracing(max_level: tracing::Level) {
    enable_ansi();
    let layer = LogfmtLayer::new(max_level);
    let subscriber = tracing_subscriber::registry().with(layer);
    let _ = tracing::subscriber::set_global_default(subscriber);
}

pub fn init() {
    enable_ansi();
}

fn enable_ansi() {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
        use windows_sys::Win32::System::Console::{
            GetConsoleMode, GetStdHandle, SetConsoleMode, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
            STD_ERROR_HANDLE, STD_OUTPUT_HANDLE,
        };
        unsafe {
            for handle_id in [STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
                let handle = GetStdHandle(handle_id);
                if handle != INVALID_HANDLE_VALUE && !handle.is_null() {
                    let mut mode: u32 = 0;
                    if GetConsoleMode(handle, &mut mode) != 0 {
                        SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
                    }
                }
            }
        }
    }
}
