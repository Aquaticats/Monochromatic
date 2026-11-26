//! What:
//!  The four encodings the binding and its inventories write:
//!  a path as the lowercase hexadecimal of its bytes,
//!  a folder's identity as inode and birth time without a device,
//!  the add's request as a form word and a hexadecimal argument,
//!  and the pointer as the `.git` file's bytes in hexadecimal.
//! Why:
//!  A path may hold a newline,
//!  which would forge a record line,
//!  and a request may hold `=`,
//!  so both are hexadecimal;
//!  a ZFS name holds neither,
//!  so it is written verbatim (`F9` of the design's facts).
//!  A folder identity drops the device number because the kernel's
//!  device is anonymous and per mount (`F16`),
//!  so a record that lives across a boot cannot hold one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const encoded = { path: hexOf(bytes), identity: `${inode}:${seconds}:${nanos}`, request: `${form} ${hexArg}`, pointer: hexOf(bytes) };
//! ```

use super::binding_words::{NO_REQUEST_ARGUMENT, RequestForm};
use std::path::Path;

/// The base of the hexadecimal digits this module writes.
const HEXADECIMAL_BASE: u32 = 16;

/// The number of bits in one hexadecimal digit.
const HEXADECIMAL_DIGIT_BITS: u32 = 4;

/// A path's bytes as lowercase hexadecimal,
/// which is how the btrfs binding records a destination and a registration.
pub fn path_as_hexadecimal(path: &Path) -> String {
    return bytes_as_hexadecimal(path.as_os_str().as_encoded_bytes());
}

/// Bytes as lowercase hexadecimal, two digits a byte.
pub fn bytes_as_hexadecimal(bytes: &[u8]) -> String {
    let mut digits: String = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        digits.push_str(format!("{byte:02x}").as_str());
    }
    return digits;
}

/// What:
///  The bytes a lowercase hexadecimal string names,
///  or nothing when it is not one.
/// Why:
///  A reader that accepts anything shorter would read a truncated path as a path,
///  and a path decides which repository and slot a record is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = (digits) => /^(?:[0-9a-f]{2})+$/.test(digits) ? Uint8Array.from(digits.match(/../g)!.map(b => parseInt(b, 16))) : undefined;
/// ```
pub fn hexadecimal_as_bytes(digits: &str) -> Option<Vec<u8>> {
    if digits.is_empty() || digits.len() % 2 != 0 || !digits.bytes().all(is_lowercase_hexadecimal) {
        return None;
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(digits.len() / 2);
    for pair in digits.as_bytes().chunks(2) {
        let high: u32 = (pair[0] as char).to_digit(HEXADECIMAL_BASE)?;
        let low: u32 = (pair[1] as char).to_digit(HEXADECIMAL_BASE)?;
        bytes.push(((high << HEXADECIMAL_DIGIT_BITS) | low) as u8);
    }
    return Some(bytes);
}

/// The bytes a hexadecimal path names.
pub fn path_from_hexadecimal(digits: &str) -> Option<std::path::PathBuf> {
    return path_from_bytes(&hexadecimal_as_bytes(digits)?);
}

/// The path one record field\x27s bytes name.
pub fn path_from_bytes(bytes: &[u8]) -> Option<std::path::PathBuf> {
    let text: std::ffi::OsString = std::ffi::OsString::from(std::str::from_utf8(bytes).ok()?);
    return Some(std::path::PathBuf::from(text));
}

/// Whether one byte is a lowercase ASCII hexadecimal digit.
pub fn is_lowercase_hexadecimal(byte: u8) -> bool {
    return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte);
}

/// What:
///  A folder's identity as the record writes it:
///  its inode,
///  its birth seconds and its birth nanoseconds,
///  and never its device number.
/// Why:
///  The device is anonymous and per mount (`F16`),
///  so it cannot be part of a record read after a boot;
///  the inode and the birth time name the object itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FolderIdentity = { inode: number; seconds?: number; nanos?: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FolderIdentity {
    /// The inode number.
    inode: u64,
    /// The birth time's seconds,
    /// or nothing when the filesystem reports none.
    birth: Option<(i64, u32)>,
}

impl FolderIdentity {
    /// The identity of a folder the caller holds,
    /// taking its inode and birth time and dropping its device.
    ///
    /// # Errors
    /// The folder's metadata cannot be read,
    /// or its identity has an inode of zero,
    /// which no live folder answers.
    pub fn of_folder(path: &Path) -> std::io::Result<FolderIdentity> {
        let directory: std::fs::File = std::fs::File::open(path)?;
        return FolderIdentity::of_open_folder(&directory);
    }

    /// The identity of a folder the caller holds open.
    ///
    /// # Errors
    /// The handle's metadata cannot be read.
    pub fn of_open_folder(folder: &std::fs::File) -> std::io::Result<FolderIdentity> {
        let object: crate::fs_beneath::ObjectId = crate::fs_beneath::identity_of(folder)?;
        if object.inode == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, String::from("the folder's inode is zero, which no live folder answers")));
        }
        return Ok(FolderIdentity { inode: object.inode, birth: object.birth });
    }

    /// The inode the identity names.
    pub fn inode(self) -> u64 {
        return self.inode;
    }

    /// The birth time the identity names,
    /// when the filesystem reported one.
    pub fn birth(self) -> Option<(i64, u32)> {
        return self.birth;
    }

    /// The identity as `inode:seconds:nanoseconds`,
    /// or `inode:-:-` when no birth time was reported.
    pub fn encode(self) -> String {
        return match self.birth {
            Some((seconds, nanos)) => format!("{}:{seconds}:{nanos}", self.inode),
            None => format!("{}:-:-", self.inode),
        };
    }

    /// The identity its encoding names,
    /// or nothing when the encoding is not this grammar's.
    pub fn decode(text: &str) -> Option<FolderIdentity> {
        let mut parts = text.split(':');
        // Each part is at most twenty digits, so a runaway number cannot be read as an identity.
        let inode: u64 = decimal_part(parts.next()?, 20)?;
        let seconds: &str = parts.next()?;
        let nanos: &str = parts.next()?;
        if parts.next().is_some() {
            return None;
        }
        if seconds == "-" && nanos == "-" {
            return Some(FolderIdentity { inode, birth: None });
        }
        if seconds == "-" || nanos == "-" {
            return None;
        }
        let birth_seconds: i64 = decimal_part(seconds, 20)? as i64;
        let birth_nanos: u32 = decimal_part(nanos, 10)? as u32;
        return Some(FolderIdentity { inode, birth: Some((birth_seconds, birth_nanos)) });
    }
}

/// One field of an identity,
/// refused when it is empty,
/// holds a non-digit,
/// or runs past its widest form.
fn decimal_part(text: &str, widest: usize) -> Option<u64> {
    if text.is_empty() || text.len() > widest || !text.bytes().all(|byte| return byte.is_ascii_digit()) {
        return None;
    }
    return text.parse().ok();
}

/// What:
///  The request the add was given:
///  how Git was asked,
///  and the commit-ish or branch in hexadecimal,
///  or `-` for a detached add with none.
/// Why:
///  A resumed add proves it is the same request (`T21a` of the design's tests),
///  and a refname may hold `=`,
///  which would forge a `key=value` line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Request = { form: RequestForm; argument?: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    /// The form word.
    form: RequestForm,
    /// The argument's bytes,
    /// absent only for a detached add with none.
    argument: Option<Vec<u8>>,
}

impl Request {
    /// A request with an argument.
    pub fn with_argument(form: RequestForm, argument: &[u8]) -> Request {
        return Request { form, argument: Some(argument.to_vec()) };
    }

    /// A detached request with no commit-ish.
    pub fn detached() -> Request {
        return Request { form: RequestForm::Detach, argument: None };
    }

    /// The form the add was given.
    pub fn form(&self) -> RequestForm {
        return self.form;
    }

    /// The argument the add was given,
    /// when it had one.
    pub fn argument(&self) -> Option<&[u8]> {
        return self.argument.as_deref();
    }

    /// The request as the record writes it.
    pub fn encode(&self) -> String {
        let argument: String = match self.argument.as_deref() {
            Some(bytes) => bytes_as_hexadecimal(bytes),
            None => String::from(NO_REQUEST_ARGUMENT),
        };
        return format!("{} {argument}", self.form.word());
    }

    /// The request a stored value names,
    /// or nothing when the value is not this grammar's.
    pub fn decode(value: &str) -> Option<Request> {
        let (word, argument): (&str, &str) = value.split_once(' ')?;
        let form: RequestForm = RequestForm::parse(word)?;
        if argument == NO_REQUEST_ARGUMENT {
            return if form == RequestForm::Detach { Some(Request::detached()) } else { None };
        }
        let bytes: Vec<u8> = hexadecimal_as_bytes(argument)?;
        return Some(Request { form, argument: Some(bytes) });
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "binding_encoding_tests.rs"]
mod tests;
