//! Unix PTY backend using `portable-pty` crate.
//!
//! Falls back to forkpty via portable-pty's Unix support.

use std::io::{self, Read, Write};
use std::io::ErrorKind;

use portable_pty::{CommandBuilder, PtySize, MasterPty as PortMasterPty};

/// A Unix PTY session (re-exports ConPty for simplicity).
pub type UnixPty = super::pty_win::ConPty;
