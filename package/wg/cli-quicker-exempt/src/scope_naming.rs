//! Names the scope that holds the moved processes of one origin cgroup,
//! and finds the cgroup directory and path of a scope.
//! The name is made from the origin's path below the user's service manager,
//! escaped the way `systemd-escape --path` does,
//! so two origins with the same last component under different parents never share a scope,
//! and a name that would pass the unit name limit ends in a hash of the whole path.

/// Lowercase hexadecimal spelling shared with the path components.
use crate::bytes_as_lowercase_hex::push_lowercase_hex_byte;
/// Directories and paths.
use std::path::{Path, PathBuf};

/// Start of every scope the watcher creates.
const SCOPE_PREFIX: &str = "wg-quicker-update-";
/// End of every scope name.
const SCOPE_SUFFIX: &str = ".scope";
/// Longest unit name the service manager accepts,
/// in bytes.
const UNIT_NAME_MAX_BYTES: usize = 255;
/// Bytes of the escaped origin kept ahead of the hash when the name would be too long.
const KEPT_BYTES_BEFORE_HASH: usize = 160;
/// FNV-1a offset basis for 64 bits.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a prime for 64 bits.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Appends one byte of a path to `escaped` as `systemd-escape --path` writes it:
/// a slash becomes a dash,
/// letters,
/// digits,
/// colon and underscore stay,
/// a dot stays unless it is the first byte,
/// and every other byte,
/// the dash and the backslash included,
/// becomes `\xNN` in lowercase hex.
fn push_escaped(escaped: &mut String, byte: u8, first: bool) {
    match byte {
        b'/' => escaped.push('-'),
        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b':' | b'_' => escaped.push(char::from(byte)),
        b'.' if !first => escaped.push('.'),
        other => {
            escaped.push_str("\\x");
            push_lowercase_hex_byte(escaped, other);
        }
    }
}

/// Escapes a relative path for use inside a unit name.
/// Distinct paths give distinct names:
/// the dash that stands for a slash is the only unescaped dash.
pub fn escape_unit_name(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for (index, byte) in text.bytes().enumerate() {
        push_escaped(&mut escaped, byte, index == 0);
    }
    return escaped;
}

/// A 64-bit FNV-1a hash of the text,
/// to tell apart two origins whose escaped names were cut at the same place.
fn fingerprint(text: &str) -> u64 {
    let mut hash = FNV_OFFSET;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    return hash;
}

/// Cuts `escaped` to at most `limit` bytes without ending inside a four-byte `\xNN` escape.
/// The text is plain ASCII,
/// because every other byte was escaped.
fn cut_before_partial_escape(escaped: &str, limit: usize) -> &str {
    let mut end = limit.min(escaped.len());
    if let Some(position) = escaped[..end].rfind('\\')
        && end - position < 4
    {
        end = position;
    }
    return &escaped[..end];
}

/// The name of the scope that holds the moved processes of `origin`,
/// the cgroup path the processes leave,
/// for the user `uid`.
/// The part of the path inside `user@<uid>.service` is escaped,
/// so `/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash`
/// gives `wg-quicker-update-app.slice-claude\x2dcode\x2dbash.scope`.
pub fn origin_scope_name(origin: &str, uid: u32) -> String {
    let prefix = format!("/user.slice/user-{uid}.slice/user@{uid}.service/");
    let relative = origin.strip_prefix(prefix.as_str()).unwrap_or(origin.trim_start_matches('/'));
    let escaped = escape_unit_name(relative);
    let plain = format!("{SCOPE_PREFIX}{escaped}{SCOPE_SUFFIX}");
    if plain.len() <= UNIT_NAME_MAX_BYTES {
        return plain;
    }
    let kept = cut_before_partial_escape(&escaped, KEPT_BYTES_BEFORE_HASH);
    return format!("{SCOPE_PREFIX}{kept}-{:016x}{SCOPE_SUFFIX}", fingerprint(relative));
}

/// The cgroup path of a scope below `slice`,
/// as `/proc/<pid>/cgroup` spells it.
pub fn scope_cgroup_path(uid: u32, slice: &str, scope_name: &str) -> String {
    return format!("/user.slice/user-{uid}.slice/user@{uid}.service/app.slice/{slice}/{scope_name}");
}

/// The directory of a scope in the cgroup filesystem rooted at `cgroup_root`.
/// It exists while the scope is active.
pub fn scope_cgroup_directory(cgroup_root: &Path, uid: u32, slice: &str, scope_name: &str) -> PathBuf {
    return cgroup_root.join(scope_cgroup_path(uid, slice, scope_name).trim_start_matches('/'));
}
