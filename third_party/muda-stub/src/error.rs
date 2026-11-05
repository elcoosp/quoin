//! Error and result types (mirrors `muda::Error` / `muda::Result`).

use std::fmt;

/// Errors returned by menu operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A platform error (message retained; the stub performs no platform
    /// calls).
    Os(String),
    /// The operation requires a native menu backend the stub does not
    /// provide (always the case for `init_for_*` / `show_for_*`).
    NotSupported,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Os(message) => write!(f, "{message}"),
            Error::NotSupported => write!(
                f,
                "native menus are unsupported by the muda stub (see quoin third_party/muda-stub)"
            ),
        }
    }
}

impl std::error::Error for Error {}

/// Convenience alias mirroring `muda::Result<T>`.
pub type Result<T> = std::result::Result<T, Error>;
