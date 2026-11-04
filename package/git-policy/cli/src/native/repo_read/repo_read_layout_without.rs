//! What:
//!  The layout reader's answer for an invocation with some global options and variables left out.
//! Why:
//!  A conflict-side line under a pathspec mode is rejected because the reader declines every global option but `-C` and
//!  every variable it does not know.
//! To tell a person that the same line without the mode would be allowed,
//! the reader must
//!  be asked the question with the mode left out,
//! and only then.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const layout = readLayoutWithout(globalPrefix, overlay, ['--icase-pathspecs'], ['GIT_ICASE_PATHSPECS']);
//! ```

use crate::repo_read_config_scan::ScopeEnvironment;
use crate::repo_read_config_typed::environment_is_accepted;
use crate::repo_read_hook::enabled;
use crate::repo_read_layout::{RepoLayout, effective_environment, environment_is_plain, start_directory, walk_up};
use std::ffi::OsString;
use std::path::PathBuf;

/// Whether `name` spells one of `spellings`.
fn is_one_of(name: &OsString, spellings: &[&str]) -> bool {
    for spelling in spellings {
        if name.as_encoded_bytes() == spelling.as_bytes() {
            return true;
        }
    }
    return false;
}

/// What:
///  The layout `read_layout` would find if the global options in `options` and the variables in `variables` were absent.
/// Why:
///   The question is asked only for a line that qualifies,
/// and the answer only decides whether a sentence is added to a
///  rejection that stands either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readLayoutWithout(globalPrefix: string[], overlay: [string, string][], options: string[], variables: string[]): RepoLayout | undefined;
/// ```
pub fn read_layout_without(global_prefix: &[OsString], overlay: &[(OsString, OsString)], options: &[&str], variables: &[&str]) -> Option<RepoLayout> {
    if !enabled(overlay) {
        return None;
    }
    let mut prefix: Vec<OsString> = Vec::new();
    for each in global_prefix {
        if !is_one_of(each, options) {
            prefix.push(each.clone());
        }
    }
    let mut kept: Vec<(OsString, OsString)> = Vec::new();
    for (name, value) in effective_environment(overlay) {
        if !is_one_of(&name, variables) {
            kept.push((name, value));
        }
    }
    if !environment_is_plain(kept.as_slice()) || !environment_is_accepted(kept.as_slice()) {
        return None;
    }
    let start: PathBuf = start_directory(prefix.as_slice())?;
    return walk_up(start.as_path(), &ScopeEnvironment::from_variables(kept.as_slice()), true);
}
