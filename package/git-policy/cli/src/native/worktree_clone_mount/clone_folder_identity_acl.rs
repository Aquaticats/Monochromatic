//! What:
//!  The two ACLs of a folder,
//!  read through its descriptor in the kernel's own extended-attribute encoding,
//!  and their form in a report and in the store.
//! Why:
//!   The route compares an ACL byte for byte,
//! and the design's measurements found an ACL drift passing a mode-only check
//! (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//! "Descriptor mount and unmount");
//! the raw value is the only form that comparison can rest on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const access = getxattr(fd, 'system.posix_acl_access'); // Uint8Array or null
//! const token = access ? hex(access) : '-';
//! ```

use std::os::fd::AsRawFd;

/// The extended attribute holding a folder's access ACL
/// (`system.posix_acl_access`, `acl(5)`).
pub(super) const ACCESS_ACL_ATTRIBUTE: &[u8] = b"system.posix_acl_access\0";

/// The extended attribute holding a folder's default ACL
/// (`system.posix_acl_default`, `acl(5)`).
pub(super) const DEFAULT_ACL_ATTRIBUTE: &[u8] = b"system.posix_acl_default\0";

/// The largest ACL blob a read accepts,
/// the kernel's own bound on one extended attribute value
/// (`XATTR_SIZE_MAX`, `xattr(7)`).
pub const MAXIMUM_ACL_BYTES: usize = 65536;

/// An ACL's words in a report:
/// its length in bytes,
/// or `none`.
pub(super) fn acl_words(acl: Option<&[u8]>) -> String {
    return match acl {
        Some(bytes) => format!("{} bytes", bytes.len()),
        None => String::from("none"),
    };
}

/// One ACL as a store token:
/// lowercase hexadecimal,
/// or `-` for none.
pub(super) fn acl_token(acl: Option<&[u8]>) -> String {
    return match acl {
        Some(bytes) if !bytes.is_empty() => bytes.iter().map(|byte| { return format!("{byte:02x}") }).collect::<String>(),
        _ => String::from("-"),
    };
}

/// What:
///  An ACL from its store token,
///  or nothing when the token is not one `acl_token` wrote.
/// Why:
///   A line this repository did not write must refuse rather than decode,
///  and `acl_token` writes `-` for no ACL and lowercase hexadecimal for an ACL of at least one byte,
///  so an empty field and an uppercase digit are not tokens this module wrote:
///  `u8::from_str_radix` alone accepts both,
///  which made a line nobody wrote read as an empty ACL.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const acl = aclFromToken('-') ?? throwError('no token'); // undefined when the text is not lowercase hex
/// ```
pub(super) fn acl_from_token(token: &str) -> Option<Option<Vec<u8>>> {
    if token == "-" {
        return Some(None);
    }
    let digits: &[u8] = token.as_bytes();
    if tokens_is_lowercase_hex(digits).is_err() {
        return None;
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(token.len() / 2);
    for pair in digits.as_chunks::<2>().0 {
        let text: &str = std::str::from_utf8(pair).ok()?;
        bytes.push(u8::from_str_radix(text, 16).ok()?);
    }
    return Some(Some(bytes));
}

/// What:
///  Nothing when `digits` is lowercase hexadecimal of at least one byte,
///  and the reason it is not otherwise.
/// Why:
///   The writer's form is the reader's grammar:
///  `acl_token` writes only `0`-`9` and `a`-`f`,
///  an even number of them,
///  and never none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tokensIsLowercaseHex(digits: Uint8Array): void  // throws on an empty or odd-length field, or an uppercase digit
/// ```
fn tokens_is_lowercase_hex(digits: &[u8]) -> Result<(), ()> {
    if digits.is_empty() {
        return Err(());
    }
    if digits.len() % 2 != 0 {
        return Err(());
    }
    for byte in digits {
        if !(byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)) {
            return Err(());
        }
    }
    return Ok(());
}

/// What:
///  Read one ACL extended attribute of an open folder,
///  up to `MAXIMUM_ACL_BYTES`.
/// Why:
///   The value is compared byte for byte,
/// so the raw kernel encoding is kept rather than an interpretation of it;
/// the size is read first so a value larger than the bound is refused instead of truncated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const size = fgetxattr(fd, name, null, 0); const bytes = fgetxattr(fd, name, size);
/// ```
///
/// # Errors
/// The read failure,
/// or a refusal naming `field` when its value is larger than the bound.
pub(super) fn read_acl(folder: &std::fs::File, attribute: &[u8], field: &str) -> std::io::Result<Option<Vec<u8>>> {
    let descriptor: libc::c_int = folder.as_raw_fd();
    // SAFETY: `attribute` is a NUL-terminated string that outlives the call, `descriptor` is an open file, and a null
    // value pointer asks the kernel for the value's size.
    let size: libc::ssize_t = unsafe { libc::fgetxattr(descriptor, attribute.as_ptr().cast::<libc::c_char>(), std::ptr::null_mut(), 0) };
    if size < 0 {
        let error: std::io::Error = std::io::Error::last_os_error();
        return match error.raw_os_error() {
            Some(libc::ENODATA) | Some(libc::ENOTSUP) => Ok(None),
            _ => Err(error),
        };
    }
    let length: usize = usize::try_from(size).map_err(|_error| return std::io::Error::new(std::io::ErrorKind::InvalidData, format!("the {field} is {size} bytes, which does not fit")))?;
    if length > MAXIMUM_ACL_BYTES {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("the {field} is {length} bytes, larger than the {MAXIMUM_ACL_BYTES} bytes a read accepts")));
    }
    if length == 0 {
        return Ok(None);
    }
    let mut bytes: Vec<u8> = vec![0; length];
    // SAFETY: `attribute` is NUL-terminated and outlives the call, `bytes` holds the `length` bytes the call may write,
    // and `descriptor` is an open file.
    let read: libc::ssize_t = unsafe { libc::fgetxattr(descriptor, attribute.as_ptr().cast::<libc::c_char>(), bytes.as_mut_ptr().cast::<libc::c_void>(), length) };
    if read < 0 {
        return Err(std::io::Error::last_os_error());
    }
    bytes.truncate(usize::try_from(read).unwrap_or(0));
    return Ok(Some(bytes));
}
