//! What:
//!  What Git's status lists as changed or untracked in a worktree:
//! how many paths,
//! the first,
//! and whether a `.gitattributes` file
//!  is among them.
//! Why:
//!  Only Git's own status tells a real change from a stale record of file times.
//! The person is told how many paths and one name,
//!  and the restore refuses a changed `.gitattributes` file.
//! The read sits in a file of its own so `worktree_snapshot_source.rs`
//!  stays under the file-size limit and this file needs nothing from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const changed = await changedPaths(context, root); // throws the reason Git could not say
//! ```

use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::ffi::OsString;
use std::path::Path;

/// What:
///  What Git's status lists as changed or untracked in a worktree.
/// Why:
///   The person is told how many paths and one name;
/// the restore refuses a changed `.gitattributes` file,
/// whose rules could
///  change how the restore writes other files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChangedPaths = { count: number; first?: string; attributesChanged: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangedPaths {
    /// How many paths `git status` lists;
    /// none when Git finds every file equal to its commit by content although the stat data
    ///  of the index cannot vouch for it (files as new as the index,
    /// or touched without a change).
    pub count: usize,
    /// The first of them,
    /// with control characters shown as `?`;
    /// nothing when `count` is zero.
    pub first: Option<String>,
    /// Whether a `.gitattributes` file is changed or untracked.
    pub attributes_changed: bool,
}

/// What:
///  The text with every control character replaced by `?`.
/// Why:
///   A path in a sentence on a terminal or in the journal must not carry an escape sequence or start a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.replace(/\p{Cc}/gu, '?')
/// ```
pub fn plain(text: &str) -> String {
    let mut shown: String = String::new();
    for character in text.chars() {
        shown.push(if character.is_control() { '?' } else { character });
    }
    return shown;
}

impl ChangedPaths {
    /// Whether Git lists any change;
    /// false for a worktree whose files equal its commit by content.
    pub fn has_changes(&self) -> bool {
        return self.count > 0;
    }
}

/// What:
///  Whether a byte is NUL,
/// the end of each entry of Git's `-z` output.
/// Why:
///   Status entries are split on it,
/// and a path may hold any other byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// byte === 0
/// ```
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  A changed path as printable text,
/// with control characters shown as `?`.
/// Why:
///   The first changed path goes into a log line and must not carry a terminal escape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plain(path.toString())
/// ```
fn plain_path_text(path: &&[u8]) -> String {
    return plain(String::from_utf8_lossy(path).as_ref());
}

/// What:
///  The subcommand word of one listing's arguments:
/// the first token that is not a global option.
/// Why:
///   The failure sentence names the Git run,
///  and every listing here writes one or more global options before its subcommand,
///  so `arguments[1]` is not the subcommand any more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// args.find((word) => !word.startsWith('--'))
/// ```
fn subcommand_word(arguments: &[OsString]) -> String {
    // `global_layout` is the crate's one reader of Git's global options, so no second list here can drift from it.
    let layout: crate::global_arguments::GlobalLayout = crate::global_arguments::global_layout(arguments);
    return arguments.get(layout.prefix_len).map_or_else(String::new, word_text);
}

/// What:
///  One argument as the text a failure sentence shows.
/// Why:
///  A named function keeps the conversion out of a closure the linter cannot keep.
fn word_text(word: &OsString) -> String {
    return word.to_string_lossy().into_owned();
}

/// What:
///  Run Git with `arguments` in `root` and give its standard output;
/// the error says what Git said or why it did not start.
/// Why:
///   Both listings of the changed paths ask the same way and word a failure the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(root, arguments) // throws a sentence
/// ```
fn ask_git(context: &GitContext, root: &Path, arguments: &[&str]) -> Result<Vec<u8>, String> {
    let owned: Vec<OsString> = arguments.iter().copied().map(OsString::from).collect();
    let words: String = subcommand_word(owned.as_slice());
    let request: GitRequest = GitRequest { cwd: root.to_path_buf(), arguments: owned, without_prefix: true, ..GitRequest::default() };
    return match run_git(context, &request) {
        Ok(found) if found.code == Some(0) => Ok(found.stdout),
        Ok(found) => {
            let said: String = String::from_utf8_lossy(found.stderr.as_slice()).trim().to_string();
            let code: String = match found.code {
                Some(number) => format!("ended with exit code {number}"),
                None => String::from("was stopped by a signal"),
            };
            crate::diagnostic_log::debug("changedPaths", format!("Git could not list the changes (`git {words}` {code}: {said})").as_str());
            Err(if said.is_empty() { format!("`git {words}` {code} and said nothing") } else { format!("`git {words}` {code}: {}", plain(said.as_str())) })
        }
        Err(error) => {
            crate::diagnostic_log::debug("changedPaths", format!("Git could not list the changes ({error})").as_str());
            Err(format!("`git {words}` could not be started ({error})"))
        }
    };
}

/// What:
///  The paths `git status` lists as changed or untracked in the worktree at `root`,
/// counted,
/// with the first,
/// and whether a
///  `.gitattributes` file is among them (also one inside an untracked directory);
/// or the words that say which Git run failed
///  and how.
/// Why:
///   Only Git's own status tells a real change from a stale record of file times.
/// The read takes no optional lock,
/// so it never
///  disturbs the checkout a person works in.
/// A listing with nothing in it is an answer (every file equals its commit by content),
///  never a failure:
/// the stat data of an index that was just written,
/// or of a file that was touched,
/// proves nothing,
/// and Git
///  settles those by content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const changed = await changedPaths(context, root); // throws the reason Git could not say
/// ```
pub fn changed_paths(context: &GitContext, root: &Path) -> Result<ChangedPaths, String> {
    let listing: Vec<u8> = ask_git(context, root, &["--no-optional-locks", "status", "--porcelain=v1", "-z", "--untracked-files=normal", "--no-renames"])?;
    // `:(glob)**/.gitattributes` is pathspec magic, so literal pathspec mode, from `GIT_LITERAL_PATHSPECS` or
    // `--literal-pathspecs`, is switched off here: under it the pattern would match nothing and an untracked
    // attributes file deep in the tree would go unnoticed.
    let untracked_attributes: Vec<u8> = ask_git(context, root, &["--no-optional-locks", "--no-literal-pathspecs", "ls-files", "--others", "--exclude-standard", "-z", "--", ":(glob)**/.gitattributes"])?;
    // Each status entry is two status letters, a space, then the path.
    let mut paths: Vec<&[u8]> = Vec::new();
    for entry in listing.split(is_nul_byte) {
        if entry.is_empty() {
            continue;
        }
        if let Some(path) = entry.get(3..) {
            paths.push(path);
        }
    }
    let mut names_attributes: bool = false;
    for path in &paths {
        if *path == b".gitattributes" || path.ends_with(b"/.gitattributes") {
            names_attributes = true;
            break;
        }
    }
    let attributes_changed: bool = !untracked_attributes.is_empty() || names_attributes;
    let first: Option<String> = paths.first().map(plain_path_text);
    return Ok(ChangedPaths { count: paths.len(), first, attributes_changed });
}

/// Controls for the two listings.
#[cfg(test)]
#[path = "worktree_snapshot_source_changed_paths_tests.rs"]
mod tests;
