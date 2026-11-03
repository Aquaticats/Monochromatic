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

use super::repo_read_config_list::ListedEntry;
use super::repo_read_config_text::parse_bool;

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

/// The settings of a configuration, or `None` when a key or value is outside what this update reproduces.
pub fn read_settings(listed: &[ListedEntry]) -> Option<IndexSettings> {
    let last = |key: &str| -> Option<&super::repo_read_config_text::ConfigEntry> {
        return listed.iter().rev().map(|item| { return &item.entry; }).find(|entry| { return entry.key == key; });
    };
    if listed.iter().any(|item| { return DECLINED.contains(&item.entry.key.as_str()); }) {
        return None;
    }
    for key in DECLINED_WHEN_TRUE {
        if let Some(entry) = last(key)
            && parse_bool(entry.value.as_deref()) != Some(false)
        {
            return None;
        }
    }
    let monitor: bool = match last("core.fsmonitor") {
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
    let untracked: UntrackedMode = match last("core.untrackedcache") {
        None => UntrackedMode::Keep,
        Some(entry) => match entry.value.as_deref() {
            Some(text) if text.eq_ignore_ascii_case(b"keep") => UntrackedMode::Keep,
            other => match parse_bool(other)? {
                true => UntrackedMode::Write,
                false => UntrackedMode::Remove,
            },
        },
    };
    return Some(IndexSettings { monitor, untracked });
}

#[cfg(test)]
#[path = "index_settings_tests.rs"]
mod tests;
