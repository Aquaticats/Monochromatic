//! One place that names the `io::ErrorKind` a failure carries.
//!
//! A caller that inspects `io::Error::kind` is reading a policy.
//! The mapping from failure to kind lives here instead of at each construction
//! site.

/// Standard I/O error type the constructors below build.
use std::io;

/// Wraps a caller-supplied value the program rejects as [`io::ErrorKind::InvalidInput`].
pub(crate) fn invalid_input(message: impl Into<String>) -> io::Error {
    return io::Error::new(io::ErrorKind::InvalidInput, message.into());
}

/// Wraps text the program could not read as its own format as [`io::ErrorKind::InvalidData`].
pub(crate) fn invalid_data(message: impl Into<String>) -> io::Error {
    return io::Error::new(io::ErrorKind::InvalidData, message.into());
}

/// Wraps a resource the program expected to exist as [`io::ErrorKind::NotFound`].
pub(crate) fn not_found(message: impl Into<String>) -> io::Error {
    return io::Error::new(io::ErrorKind::NotFound, message.into());
}
