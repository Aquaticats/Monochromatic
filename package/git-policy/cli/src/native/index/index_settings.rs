//! What:
//!  The configuration that decides what Git does to the extensions of an index it reads and writes, and whether the
//!  in-process index update may stand in for Git at all (`repo-settings.c`, `fsmonitor-settings.c`, `read-cache.c`).
//! Why:
//!  `core.untrackedCache` makes Git keep, add or remove the untracked cache; `core.fsmonitor` makes it keep the fsmonitor
//!  extension and clear the untracked cache of replaced entries, and drop the extension when it is off; split and sparse
//!  indexes, the offset-table extensions, skipped hashes and a case-insensitive worktree change the file layout or the
//!  checks. Any key this module cannot reproduce makes the update decline, so Git answers it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const settings: IndexSettings | undefined = readIndexSettings(listedConfig); // undefined: ask Git
//! ```

use crate::repo_read_config_list::ListedEntry;
use crate::repo_read_config_text::parse_bool;

/// What Git does with the untracked-cache extension while reading an index (`core.untrackedCache`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UntrackedMode {
    /// Leave it as the file has it.
    Keep,
    /// Add one when the file has none; this update declines that case.
    Write,
    /// Drop it.
    Remove,
}

/// The settings the in-process update follows.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IndexSettings {
    /// Whether fsmonitor is on (`core.fsmonitor` a true value or a hook name).
    pub monitor: bool,
    /// What happens to the untracked cache.
    pub untracked: UntrackedMode,
    /// `core.sharedRepository` as Git reads it (`git_config_perm`): zero for the umask alone, a positive permission bit set that
    ///  is added to the file's mode, a negative one whose absolute value replaces the permission bits.
    pub shared_repository: i32,
}

/// The keys whose mere presence makes the update decline.
const DECLINED: [&str; 8] = [
    "core.splitindex",
    "index.sparse",
    "index.skiphash",
    "index.recordendofindexentries",
    "index.recordoffsettable",
    "index.threads",
    "core.hookspath",
    "core.worktree",
];

/// The keys that decline when their value is true (or not a recognised boolean).
const DECLINED_WHEN_TRUE: [&str; 3] = ["core.sparsecheckout", "core.ignorecase", "feature.manyfiles"];

/// The mode bits `core.sharedRepository = group` adds.
const SHARED_GROUP: i32 = 0o660;

/// The mode bits `core.sharedRepository = all` adds.
const SHARED_EVERYBODY: i32 = 0o664;

/// What:
///  The value Git makes of `core.sharedRepository` (`git_config_perm` in `setup.c`), or `None` for a value Git dies on or this
///  reader cannot classify.
/// Why:
///   The index Git writes goes through a lock file whose mode `adjust_shared_perm` widens by this value; the in-process update
///  must publish a file with the same mode.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shared: number | undefined = sharedPermission(value); // undefined: ask Git
/// ```
pub fn shared_permission(value: Option<&[u8]>) -> Option<i32> {
    let Some(text) = value else {
        return Some(SHARED_GROUP);
    };
    match text {
        b"" | b"umask" => return Some(0),
        b"group" => return Some(SHARED_GROUP),
        b"all" | b"world" | b"everybody" => return Some(SHARED_EVERYBODY),
        _ => {}
    }
    let digits: &str = std::str::from_utf8(text).ok()?;
    // `strtol` with base 8 reads an optional sign and octal digits; anything left over makes the value a boolean.
    if let Ok(number) = i32::from_str_radix(digits, 8) {
        return match number {
            0 => Some(0),
            1 => Some(SHARED_GROUP),
            2 => Some(SHARED_EVERYBODY),
            // The owner must keep read and write access, or Git dies.
            other if (other & 0o600) != 0o600 || other < 0 => None,
            other => Some(-(other & 0o666)),
        };
    }
    return match parse_bool(Some(text)) {
        Some(true) => Some(SHARED_GROUP),
        Some(false) => Some(0),
        None => None,
    };
}

/// The last entry of `listed` whose key is `key`, or `None`; later lines of a configuration win.
fn last_entry<'a>(listed: &'a [ListedEntry], key: &str) -> Option<&'a crate::repo_read_config_text::ConfigEntry> {
    for item in listed.iter().rev() {
        if item.entry.key == key {
            return Some(&item.entry);
        }
    }
    return None;
}

/// Whether the configuration holds a key that this update does not reproduce.
fn has_declined_key(listed: &[ListedEntry]) -> bool {
    for item in listed {
        if DECLINED.contains(&item.entry.key.as_str()) {
            return true;
        }
    }
    return false;
}

/// The settings of a configuration, or `None` when a key or value is outside what this update reproduces.
pub fn read_settings(listed: &[ListedEntry]) -> Option<IndexSettings> {
    if has_declined_key(listed) {
        return None;
    }
    for key in DECLINED_WHEN_TRUE {
        if let Some(entry) = last_entry(listed, key)
            && parse_bool(entry.value.as_deref()) != Some(false)
        {
            return None;
        }
    }
    let monitor: bool = match last_entry(listed, "core.fsmonitor") {
        None => false,
        Some(entry) => match entry.value.as_deref() {
            // A bare name is a true boolean to Git.
            None => true,
            Some(text) => match parse_bool(Some(text)) {
                Some(truth) => truth,
                // Any other text is the path of a hook, which also turns the monitor on unless it is empty.
                None => !text.is_empty(),
            },
        },
    };
    let untracked: UntrackedMode = match last_entry(listed, "core.untrackedcache") {
        None => UntrackedMode::Keep,
        Some(entry) => match entry.value.as_deref() {
            Some(text) if text.eq_ignore_ascii_case(b"keep") => UntrackedMode::Keep,
            other => match parse_bool(other)? {
                true => UntrackedMode::Write,
                false => UntrackedMode::Remove,
            },
        },
    };
    let shared_repository: i32 = match last_entry(listed, "core.sharedrepository") {
        None => 0,
        Some(entry) => shared_permission(entry.value.as_deref())?,
    };
    return Some(IndexSettings { monitor, untracked, shared_repository });
}

#[cfg(test)]
#[path = "index_settings_tests.rs"]
mod tests;
