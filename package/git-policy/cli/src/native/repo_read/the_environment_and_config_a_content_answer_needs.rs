//! What:
//!  The environment and the configuration a content answer needs to be plain:
//! the environment Git would see,
//!  and the proof that no configuration file Git reads sets a key that could change the answer.
//! Why:
//!  A content command's answer depends on more than the index and the object database:
//! a configuration key can change what Git prints,
//!  and two environment variables name the index and the object directory themselves.
//!  The entry that dispatches the answers asks these first,
//!  and declines an answer whose configuration it cannot prove plain,
//!  so these checks are one file and the dispatch is `repo_read_query.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [kept, indexPath, objectsPath] = environment(base, added); // the environment Git would see
//! const plain = contentConfigIsPlain(layout, scope); // false: start Git for this answer
//! ```

#![cfg(unix)]

use crate::repo_read_config_scan::{
    ScopeEnvironment,
    content_key,
    entries_avoid_keys,
    file_avoids_keys,
    last_variable,
    outer_scopes_avoid_keys,
};
use crate::repo_read_config_text::{
    CONFIG_TEXT_LIMIT,
    parse_config_text,
};
use crate::repo_read_layout::RepoLayout;
use std::ffi::OsString;
use std::path::PathBuf;

/// Whether a configuration key,
/// in any letter case,
/// is one that could change the output of a content command.
pub(crate) fn content_key_of_any_case(key: &str) -> bool {
    return content_key(key.to_ascii_lowercase().as_str());
}

/// The environment Git would see,
/// without the two variables this reader handles itself.
pub(crate) fn environment(
    base: Vec<(OsString, OsString)>,
    variables: &[(OsString, OsString)],
) -> (
    Vec<(OsString, OsString)>,
    Option<OsString>,
    Option<OsString>,
) {
    let mut all: Vec<(OsString, OsString)> = Vec::new();
    for (name, value) in base {
        let mut replaced: bool = false;
        for (added, _) in variables {
            if *added == name {
                replaced = true;
                break;
            }
        }
        if !replaced {
            all.push((name, value));
        }
    }
    all.extend(variables.iter().cloned());
    // The last assignment wins, as it does for the child's environment.
    let index: Option<OsString> = last_variable(all.as_slice(), "GIT_INDEX_FILE");
    let objects: Option<OsString> = last_variable(all.as_slice(), "GIT_OBJECT_DIRECTORY");
    let mut kept: Vec<(OsString, OsString)> = Vec::new();
    for (name, value) in all {
        if name != "GIT_INDEX_FILE" && name != "GIT_OBJECT_DIRECTORY" {
            kept.push((name, value));
        }
    }
    return (kept, index, objects);
}

/// Whether no configuration file Git reads for these queries sets a key that could change the output.
pub(crate) fn content_config_is_plain(
    layout: &RepoLayout,
    scope: &ScopeEnvironment,
) -> Option<bool> {
    let path: PathBuf = layout.common_dir.join("config");
    let bytes: Vec<u8> =
        crate::private_storage::read_regular_file(path.as_path(), CONFIG_TEXT_LIMIT as u64).ok()?;
    let entries = parse_config_text(bytes.as_slice())?;
    if !entries_avoid_keys(
        entries.as_slice(),
        path.as_path(),
        scope.home.as_deref(),
        0,
        &content_key,
    )? {
        return Some(false);
    }
    // The per-worktree file is where `git sparse-checkout` writes `core.sparseCheckout`; it is scanned whenever it
    // exists, also when the extension that makes Git read it is off, which only makes the readers decline more often.
    if !file_avoids_keys(
        layout.git_dir.join("config.worktree").as_path(),
        scope.home.as_deref(),
        0,
        &content_key,
    )? {
        return Some(false);
    }
    return outer_scopes_avoid_keys(scope, &content_key);
}
