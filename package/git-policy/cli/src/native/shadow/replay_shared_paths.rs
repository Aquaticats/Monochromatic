//! What:
//!  The paths both the prepared commit and the landed history changed since the preparation base,
//!       with the three tree entries subsumption compares.
//! Why:
//!  Subsumption decides per shared path;
//! the listing is two `git diff-tree` runs whatever the history length,
//!       and a path stays the bytes Git listed,
//!       so every name reaches the Git that writes the synthetic base unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await listSharedPaths({ gitPath, shadowPath, mergeBase, current, prepared });
//! ```

/// Shadow runs for the two listings.
use crate::replay_shadow::run_shadow_checked;
/// The shadow runner's context and failure.
use crate::transaction_git::{GitContext, GitFailure};
/// `Path` names the shadow and the working directory.
use std::path::Path;

/// Git's absent-entry mode in raw diff output.
const ABSENT_MODE: &str = "000000";

/// Whether one byte is the NUL byte that ends each field of a `-z` listing.
/// `&u8` borrows the byte,
/// as `split` hands it over.
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  One tree entry:
/// its mode and object ID.
/// Why:
///   Subsumption compares modes as well as objects,
/// and a regular file's mode decides the synthetic base.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TreeEntry = { mode: string; oid: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeEntry {
    /// Git mode,
    /// such as `100644`.
    pub mode: String,
    /// Object ID.
    pub oid: String,
}

/// What:
///  One change of a raw diff record:
/// the path and its entries before and after.
/// Why:
///   Both listings produce these;
/// a path missing from one side has an absent entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RawChange = { path; fromMode; fromOid; toMode; toOid };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawChange {
    /// Path bytes,
    /// exactly as Git listed them.
    pub path: Vec<u8>,
    /// Entry before the change,
    /// absent for an addition.
    pub from: Option<TreeEntry>,
    /// Entry after the change,
    /// absent for a deletion.
    pub to: Option<TreeEntry>,
}

/// What:
///  One path both sides changed,
/// with its three entries.
/// Why:
///   The subsumption rules read the base,
/// landed,
/// and prepared entries together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SharedPath = { path; base: SideEntry; landed: SideEntry; prepared: SideEntry };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedPath {
    /// Repository path bytes,
    /// exactly as Git listed them.
    pub path: Vec<u8>,
    /// Preparation base entry,
    /// absent when the path was added after it.
    pub base: Option<TreeEntry>,
    /// Landed entry,
    /// absent when the landed history deleted the path.
    pub landed: Option<TreeEntry>,
    /// Prepared entry,
    /// absent when the prepared commit deleted the path.
    pub prepared: Option<TreeEntry>,
}

/// What:
///  Raw diff output that does not have the expected shape.
/// Why:
///   A malformed record cannot be attributed,
/// so the replay fails rather than guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new TypeError('malformed raw diff record');
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawFormatError(pub String);

/// What:
///  Converts one raw mode and object ID into an entry,
/// absent for the all-zero mode.
/// Why:
///   Git writes `000000` for an absent side of a raw record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entryOf({ mode, oid }); // ABSENT_ENTRY for mode 000000
/// ```
fn entry_of(mode: &str, oid: &str) -> Option<TreeEntry> {
    if mode == ABSENT_MODE {
        return None;
    }
    return Some(TreeEntry {
        mode: String::from(mode),
        oid: String::from(oid),
    });
}

/// What:
///  Parses `git diff-tree -r -z --raw --no-renames --no-abbrev` output:
///       a `:` header of five fields followed by the path,
/// both NUL-terminated.
/// Why:
///   Each path appears once because renames are off,
/// so the parser keeps records in output order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseRawChanges(bytes); // RawChange[]
/// ```
pub fn parse_raw_changes(output: &[u8]) -> Result<Vec<RawChange>, RawFormatError> {
    // `mut` lets the records be collected as the output is walked.
    let mut changes: Vec<RawChange> = Vec::new();
    let tokens: Vec<&[u8]> = output.split(is_nul_byte).collect();
    let mut cursor: usize = 0;
    while cursor < tokens.len() {
        let header: &[u8] = tokens[cursor];
        cursor += 1;
        if header.is_empty() {
            continue;
        }
        // The header and the entries are ASCII, and only a message keeps their text.
        let text: String = String::from_utf8_lossy(header).into_owned();
        let fields: Vec<&str> = text.trim_start_matches(':').split(' ').collect();
        let path: &[u8] = match tokens.get(cursor) {
            Some(found) => found,
            None => return Err(RawFormatError(format!("raw record {text} has no path"))),
        };
        cursor += 1;
        if fields.len() != 5 || !header.starts_with(b":") {
            return Err(RawFormatError(format!("malformed raw record {text}")));
        }
        changes.push(RawChange {
            path: path.to_vec(),
            from: entry_of(fields[0], fields[2]),
            to: entry_of(fields[1], fields[3]),
        });
    }
    return Ok(changes);
}

/// What:
///  Lists the raw changes between two commits in the shadow.
/// Why:
///   Two runs,
/// one per side of the replay,
/// keep the process count fixed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rawChanges({ gitPath, shadowPath, from, to }); // ReadonlyMap<path, RawChange>
/// ```
fn raw_changes(
    context: &GitContext,
    shadow: &Path,
    cwd: &Path,
    from: &str,
    to: &str,
) -> Result<Vec<RawChange>, SharedPathError> {
    let output = run_shadow_checked(
        context,
        shadow,
        cwd,
        &[
            "diff-tree",
            "-r",
            "-z",
            "--raw",
            "--no-renames",
            "--no-abbrev",
            from,
            to,
        ],
        None,
        None,
        &[],
    )
    .map_err(SharedPathError::Git)?;
    return parse_raw_changes(output.stdout.as_slice()).map_err(SharedPathError::Format);
}

/// What:
///  A shared-path listing failure:
///       a Git command failed,
/// or its output had an unexpected shape.
/// Why:
///   Replay reports either as `transaction-failed`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new CommitTransactionGitError(...);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SharedPathError {
    /// A Git command failed.
    Git(GitFailure),
    /// The raw output had an unexpected shape.
    Format(RawFormatError),
}

/// What:
///  Lists the paths both the landed history and the prepared commit changed since the preparation base,
///       in the landed listing's order,
/// with the three entries.
/// Why:
///   The landed listing decides the order subsumption and the synthetic base follow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await listSharedPaths({ gitPath, shadowPath, mergeBase, current, prepared });
/// ```
pub fn list_shared_paths(
    context: &GitContext,
    shadow: &Path,
    cwd: &Path,
    merge_base: &str,
    current: &str,
    prepared: &str,
) -> Result<Vec<SharedPath>, SharedPathError> {
    let landed: Vec<RawChange> = raw_changes(context, shadow, cwd, merge_base, current)?;
    let mine: Vec<RawChange> = raw_changes(context, shadow, cwd, merge_base, prepared)?;
    // A map answers "did the prepared side change this path" by naming the one change it
    //  compares with, so building the intersection is one pass over each listing
    //  (`AGENTS.md` `RG2`); the first change a path gets is the one the walk took.
    let mut mine_by_path: std::collections::HashMap<&[u8], &RawChange> = std::collections::HashMap::new();
    for candidate in &mine {
        crate::collected_elements_looked_at::note_looked_at(1);
        mine_by_path.entry(candidate.path.as_slice()).or_insert(candidate);
    }
    let mut shared: Vec<SharedPath> = Vec::new();
    for change in landed {
        // A landed change counts only when the prepared side changed the same path.
        crate::collected_elements_looked_at::note_looked_at(1);
        let Some(own_change) = mine_by_path.get(change.path.as_slice()) else {
            continue;
        };
        shared.push(SharedPath {
            path: change.path,
            base: change.from,
            landed: change.to,
            prepared: own_change.to.clone(),
        });
    }
    return Ok(shared);
}

/// What:
///  Whether two entries name the same mode and object,
/// or are both absent.
/// Why:
///   An identical prepared and landed entry is subsumed without any text check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sameEntry({ left, right });
/// ```
pub fn same_entry(left: &Option<TreeEntry>, right: &Option<TreeEntry>) -> bool {
    return left == right;
}

/// Whether one side of a shared path is present and a regular file.
/// An absent side is not.
fn is_regular_entry(side: &Option<TreeEntry>) -> bool {
    return match side {
        Some(entry) => is_regular_mode(entry.mode.as_str()),
        None => false,
    };
}

/// What:
///  Whether all three entries are regular files,
/// so only their bytes can decide the path.
/// Why:
///   Symbolic links and submodules never take the text check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isRegularTriple(shared);
/// ```
pub fn is_regular_triple(shared: &SharedPath) -> bool {
    for side in [&shared.base, &shared.landed, &shared.prepared] {
        if !is_regular_entry(side) {
            return false;
        }
    }
    return true;
}

/// What:
///  Whether the text check can decide a path:
///       landed and prepared are regular files,
/// and the base is one too or absent.
/// Why:
///   An addition on both sides is read against an empty base,
/// so it is a candidate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isTextCandidate(shared);
/// ```
pub fn is_text_candidate(shared: &SharedPath) -> bool {
    let changed_regular: bool = is_regular_entry(&shared.landed) && is_regular_entry(&shared.prepared);
    let base_regular: bool = match &shared.base {
        None => true,
        Some(entry) => is_regular_mode(entry.mode.as_str()),
    };
    return changed_regular && base_regular;
}

/// What:
///  Whether a mode is a regular file mode,
/// `100644` or `100755`.
/// Why:
///   Only these modes are text candidates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// REGULAR_MODES.has(mode);
/// ```
fn is_regular_mode(mode: &str) -> bool {
    return mode == "100644" || mode == "100755";
}

#[cfg(test)]
#[path = "replay_shared_paths_tests.rs"]
mod tests;
