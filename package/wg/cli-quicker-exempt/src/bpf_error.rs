//! Typed BPF syscall errors used for behavior selection without diagnostic string matching.

/// Diagnostic formatting implementation.
use std::fmt;
/// Standard operating-system error wrapper.
use std::io;

/// `BPF_OBJ_PIN EINVAL` with destination and original error.
#[derive(Debug)]
struct PinObjectInvalidError {
    /// Destination path attempted by failed pin syscall.
    path: String,
    /// Original operating-system error.
    source: io::Error,
}

/// Renders command,
///  path,
///  kernel-regression hint,
///  and original diagnostic.
impl fmt::Display for PinObjectInvalidError {
    /// Writes stable human-readable diagnostic without controlling behavior selection.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(
            formatter,
            "BPF_OBJ_PIN {}: {}; an affected SELinux kernel may need the fix for regression 9722955b5430",
            self.path, self.source
        );
    }
}

/// Exposes original operating-system error to standard diagnostic chain.
impl std::error::Error for PinObjectInvalidError {
    /// Returns operating-system cause for structured error reporting.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        return Some(&self.source);
    }
}

/// Wraps invalid pin syscall as typed error payload.
pub fn pin_object_invalid(path: String, source: io::Error) -> io::Error {
    return io::Error::new(
        source.kind(),
        PinObjectInvalidError {
            path,
            source,
        },
    );
}

/// Reports typed invalid pin boundary without locale-sensitive message inspection.
pub fn is_pin_object_invalid(error: &io::Error) -> bool {
    return error
        .get_ref()
        .is_some_and(|source| return source.downcast_ref::<PinObjectInvalidError>().is_some());
}
