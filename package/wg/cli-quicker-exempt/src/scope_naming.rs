//! Names the scope that holds the moved processes of one origin cgroup,
//! and finds the cgroup directory and path of a scope.
//! The name is made from the origin's path below the user's service manager,
//! escaped the way `systemd-escape --path` does,
//! so two origins with the same last component under different parents never share a scope,
//! and a name that would pass the unit name limit ends in a hash of the whole path.
//! The name ends in exactly one `.scope`:
//! an origin that is itself a unit (`.../run-p1-i2.scope`) keeps its own suffix escaped as `\x2e`,
//! because `app.slice/foo` and `app.slice/foo.scope` would otherwise both be named
//! `...-foo.scope` and the second origin's processes would share the first origin's limits.
//! The spellings earlier builds produced are still found,
//! so a scope of one origin that is already running receives the origin's later processes
//! instead of a second scope for the same origin.

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
/// The unit types a cgroup name can end in.
/// A dot that would be the last separator of such a suffix is escaped,
/// so the name the scope itself gets never repeats it.
const UNIT_TYPE_SUFFIXES: &[&str] = &["service", "socket", "device", "mount", "automount", "swap", "target", "path", "timer", "slice", "scope"];

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

/// The part of an origin's cgroup path that the name spells:
/// what lies below `user@<uid>.service`,
/// or the whole path without its leading slash for a cgroup outside the manager's subtree.
fn origin_relative(origin: &str, uid: u32) -> &str {
    let prefix = format!("/user.slice/user-{uid}.slice/user@{uid}.service/");
    return origin.strip_prefix(prefix.as_str()).unwrap_or(origin.trim_start_matches('/'));
}

/// Escapes the dot of a trailing unit-type suffix as `\x2e`,
/// so the `.scope` this module appends never doubles it
/// and a cgroup named `foo` and one named `foo.scope` keep distinct scopes.
fn escape_trailing_unit_suffix(escaped: &mut String) {
    for unit_type in UNIT_TYPE_SUFFIXES {
        let tail = format!(".{unit_type}");
        if let Some(at) = escaped.len().checked_sub(tail.len())
            && escaped.ends_with(&tail)
        {
            escaped.replace_range(at..at.saturating_add(1), "\\x2e");
            return;
        }
    }
}

/// The unit name made from an escaped origin path,
/// cut and hashed when it would pass the unit name limit.
fn name_from_escaped(escaped: &str, relative: &str) -> String {
    let plain = format!("{SCOPE_PREFIX}{escaped}{SCOPE_SUFFIX}");
    if plain.len() <= UNIT_NAME_MAX_BYTES {
        return plain;
    }
    let kept = cut_before_partial_escape(escaped, KEPT_BYTES_BEFORE_HASH);
    return format!("{SCOPE_PREFIX}{kept}-{:016x}{SCOPE_SUFFIX}", fingerprint(relative));
}

/// The name of the scope that holds the moved processes of `origin`,
/// the cgroup path the processes leave,
/// for the user `uid`.
/// The part of the path inside `user@<uid>.service` is escaped,
/// so `/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash`
/// gives `wg-quicker-update-app.slice-claude\x2dcode\x2dbash.scope`,
/// and `/.../app.slice/run-p1-i2.scope` gives
/// `wg-quicker-update-app.slice-run\x2dp1\x2di2\x2escope.scope`.
pub fn origin_scope_name(origin: &str, uid: u32) -> String {
    let relative = origin_relative(origin, uid);
    let mut escaped = escape_unit_name(relative);
    escape_trailing_unit_suffix(&mut escaped);
    return name_from_escaped(&escaped, relative);
}

/// The names earlier builds of this watcher gave the scope of `origin`,
/// current one last:
/// that build appended `.scope` to the escaped path whatever it ended in,
/// so an origin that is a unit itself got two of them.
/// A scope still running under such a name holds processes of this origin,
/// and the origin's later processes join it instead of making a second scope with the same cap.
pub fn legacy_origin_scope_names(origin: &str, uid: u32) -> Vec<String> {
    let relative = origin_relative(origin, uid);
    let escaped = escape_unit_name(relative);
    let earlier = name_from_escaped(&escaped, relative);
    if earlier == origin_scope_name(origin, uid) {
        return Vec::new();
    }
    return vec![earlier];
}

/// Every name the scope of `origin` may run under right now,
/// the current spelling first,
/// so the scope that exists is found before a new one is created.
pub fn origin_scope_names(origin: &str, uid: u32) -> Vec<String> {
    let mut names = vec![origin_scope_name(origin, uid)];
    names.extend(legacy_origin_scope_names(origin, uid));
    return names;
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
