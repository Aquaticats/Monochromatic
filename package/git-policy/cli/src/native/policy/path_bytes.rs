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

use std::ffi::{OsStr, OsString};
use std::path::Path;

/// What:
///  Raw path bytes as an operating-system string for a command line or a file name.
///  `&[u8]` borrows the bytes;
///  the result owns a copy.
/// Why:
///   A Git path is bytes,
///  and only the platform decides how to pass them on:
///  Unix takes them as they are,
///  other platforms have text names,
/// so the text with replacement characters is the closest name.
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
///  The bytes of an operating-system path,
/// so a check or a writer can look at the name Git listed instead of a spelling of it.
///  `&[u8]` borrows from `path`.
/// Why:
///   A file name is bytes,
/// and only the properties of those bytes say whether a name is plain or where it leads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.from(path)
/// ```
pub fn path_bytes_of(path: &Path) -> &[u8] {
    use std::os::unix::ffi::OsStrExt;
    return path.as_os_str().as_bytes();
}

/// What:
///  The bytes of one command-line argument,
/// so a token that is a value (a message,
/// a message file's name) reaches Git as it was typed.
///  `&[u8]` borrows from `argument`.
/// Why:
///   An argument is bytes and not text:
/// a commit message or a file name the person typed may hold a byte that is not UTF-8,
///  and a spelling with replacement characters names another message and another file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.from(argument)
/// ```
pub fn argument_bytes(argument: &OsStr) -> &[u8] {
    // The encoded form is the operating system's own bytes on Unix and the one reversible form elsewhere.
    return argument.as_encoded_bytes();
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
