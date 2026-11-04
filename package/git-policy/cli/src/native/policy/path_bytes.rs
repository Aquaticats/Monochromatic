//! What:
//!  Exact repository path bytes as the operating system takes them,
//!  and as text for messages.
//! Why:
//!  Git allows any bytes in a file name.
//!  The commit path keeps a path as `Vec<u8>` from Git's listing to Git's next argument,
//!  so a name that is not UTF-8 reaches Git unchanged;
//!  only a message for a person shows the text with replacement characters.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const argument = Buffer.from(bytes); const shown = bytes.toString('utf8');
//! ```

use std::ffi::OsString;

/// What:
///  Raw path bytes as an operating-system string for a command line or a file name.
///  `&[u8]` borrows the bytes;
///  the result owns a copy.
/// Why:
///   A Git path is bytes,
///  and only the platform decides how to pass them on:
///  Unix takes them as they are,
///  other platforms have text names, so the text with replacement characters is the closest name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.from(bytes)
/// ```
pub fn os_from_path_bytes(bytes: &[u8]) -> OsString {
    #[cfg(unix)]
    {
        /// The Unix conversion from raw bytes.
        use std::os::unix::ffi::OsStringExt;
        return OsString::from_vec(bytes.to_vec());
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return OsString::from(String::from_utf8_lossy(bytes).into_owned());
    }
}

/// What:
///  The readable text of path bytes for a message,
///  with each invalid sequence replaced by U+FFFD.
/// Why:
///   Messages are text,
///  and the exact bytes travel in the field next to them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// bytes.toString('utf8')
/// ```
pub fn path_message_text(bytes: &[u8]) -> String {
    // `.into_owned()` copies the borrowed text only when no replacement was needed.
    return String::from_utf8_lossy(bytes).into_owned();
}

/// Conversion controls stay out of the release executable.
#[cfg(test)]
#[path = "path_bytes_tests.rs"]
mod tests;
