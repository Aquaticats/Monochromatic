//! What:
//!  Read the two members every rule kind spells the same way:
//! a repo-relative path and an octal mode.
//! Why:
//!  A path that leaves the worktree and a mode that is not octal both fail at publish time,
//! so one reader per shape reports them at the value that caused them while the rest of the config still decodes.
//!  Split out of `config_decode_rules`,
//! which keeps the dispatcher.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // config_decode_rules_path_and_mode_readers.ts
//! function repoPath(reader: Reader, object: Map<string, Value>, key: string, pointer: string): string
//! function modeText(reader: Reader, object: Map<string, Value>, key: string, pointer: string): string
//! ```

/// Imports `is_octal_mode`,
/// `is_repo_relative` from `crate::config_decode`.
use crate::config_decode::{
    is_octal_mode,
    is_repo_relative,
};
/// Imports `Reader` from `crate::config_reader`.
use crate::config_reader::Reader;
/// Imports `Map`,
/// `Value` from `serde_json`.
use serde_json::{
    Map,
    Value,
};

/// What:
///  Read a repo-relative path member and check it stays inside the root.
/// Why:
///  An absolute path or a `..` segment would let a rule write outside the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repoPath(object, key, pointer): string
/// ```
pub(crate) fn repo_path(
    reader: &mut Reader,
    object: &Map<String, Value>,
    key: &str,
    pointer: &str,
) -> String {
    let path = reader.text(object, key, pointer);
    if !path.is_empty() && !is_repo_relative(&path) {
        let message = format!("{path:?} must be a relative path without empty, . or .. segments");
        reader.error(&format!("{pointer}/{key}"), &message);
    }
    return path;
}

/// What:
///  Read a required mode member and check it is octal.
/// Why:
///  A mode like `rw` would fail at publish time otherwise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function modeText(object, key, pointer): string
/// ```
pub(crate) fn mode_text(
    reader: &mut Reader,
    object: &Map<String, Value>,
    key: &str,
    pointer: &str,
) -> String {
    let mode = reader.text(object, key, pointer);
    if !mode.is_empty() && !is_octal_mode(&mode) {
        reader.error(
            &format!("{pointer}/{key}"),
            &format!("{mode:?} must be three or four octal digits such as 0600"),
        );
    }
    return mode;
}
