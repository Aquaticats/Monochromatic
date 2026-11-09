//! What:
//!  The repair of an index update a landed batch owes:
//! which of its paths still hold what the batch started from,
//! and the reset that sets those to the
//!  landed commit.
//! Why:
//!  The record of the owed update is `owed_index_update.rs`;
//! this is what the next commit and the landing's own step do with it.
//!  Only entries that still hold what the batch started from are reset,
//! so staging that is the program's own is kept and named.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const left = await leftovers(owed, index); await resetEntries(left); const said = restatedNotice(left);
//! ```

use crate::owed_index_update::{Owed, records_of, remove};
use crate::service_protocol::FailureCode;
use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::collections::BTreeMap;
use std::path::Path;

/// One entry of a tree or an index:
/// its mode and its object.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Entry {
    mode: String,
    object: String,
}

/// What:
///  The entries a tree holds at `names`,
/// each as the mode and object a comparison with an index needs.
/// Why:
///   A repair resets a path only while the index still holds what the batch started from,
/// and that is this value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// treeEntries(tree, names) // `ls-tree -z --full-tree <tree> -- :(top,literal)<name>...`
/// ```
fn tree_entries(git: &GitContext, repository: &Path, tree: &str, names: &[Vec<u8>]) -> Result<BTreeMap<Vec<u8>, Entry>, FailureCode> {
    let specs: Vec<std::ffi::OsString> = names.iter().map(path_spec).collect();
    let mut arguments: Vec<&std::ffi::OsStr> = vec![std::ffi::OsStr::new("ls-tree"), std::ffi::OsStr::new("-z"), std::ffi::OsStr::new("--full-tree"), std::ffi::OsStr::new(tree), std::ffi::OsStr::new("--")];
    arguments.extend(specs.iter().map(std::ffi::OsString::as_os_str));
    let output = run(git, repository, arguments.as_slice(), None)?;
    return Ok(entries_of(output.stdout.as_slice(), TabShape::Tree));
}

/// The path the record's paths become,
/// which is where the repair is told which entries to look at.
/// What:
///  Whether a byte is the NUL that ends one record of a `-z` listing.
/// Why:
///  A listing's records are split by this byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isZeroByte = (byte: number) => byte === 0;
/// ```
fn is_zero_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// Whether a byte is the tab that separates the entry's words from its path.
fn is_tab_byte(byte: &u8) -> bool {
    return *byte == b'\t';
}

/// Whether a byte is the space between the entry's words.
fn is_space_byte(byte: &u8) -> bool {
    return *byte == b' ';
}

/// Whether an entry from a listing is not the empty name two NULs in a row leave.
fn is_not_empty(name: &&[u8]) -> bool {
    return !name.is_empty();
}

/// Whether a stage column names an unmerged entry,
/// which is a program's own and not the landing's leftover.
fn is_unmerged(stage: &&[u8]) -> bool {
    return *stage != b"0";
}

/// A path of a record as the bytes of a pathspec.
fn path_bytes(path: &String) -> Vec<u8> {
    return path.as_bytes().to_vec();
}

/// A path of a record as a literal pathspec.
fn path_spec(name: &Vec<u8>) -> std::ffi::OsString {
    return literal_spec(name.as_slice());
}

/// What:
///  The one line the wrapper prints when it set the entries a landed commit left,
/// and nothing when it set none.
/// Why:
///   The command the person runs must say why its index entries were changed before it read them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// restatedNotice(paths)
/// ```
pub fn restated_notice(paths: &[String]) -> Option<String> {
    if paths.is_empty() {
        return None;
    }
    return Some(format!(
        "monochromatic-git-policy-cli: a commit that landed did not finish updating this worktree's index.
\
The entries of {} were set to that commit for this one, so this commit does not record them against it.
",
        paths.join(", ")
    ));
}

/// What:
///  Sets the index entries a landed commit left in this worktree to that commit,
/// and gives the paths it set.
/// Why:
///   A commit that landed but did not finish its index update leaves the index describing the branch before it,
/// so a plain commit of that index records the landed commit's own paths as their old content,
/// and a path it added as a staged deletion.
/// Only the entries that still hold what the landed batch started from are set:
/// an entry a program staged since holds something else and is left,
/// with the pieces of its obligation forgotten.
/// Nothing here can fail a commit:
/// a repair that cannot be made is said,
/// the record stays,
/// and the next commit tries it again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const restated = restateLeftovers(git, gitDir, realIndex); // the paths set
/// ```
pub fn restate_leftovers(git: &GitContext, git_dir: &Path, real_index: &Path) -> Vec<String> {
    let mut restated: Vec<String> = Vec::new();
    for owed in records_of(git_dir) {
        // A batch of another index file (`GIT_INDEX_FILE`) is not this commit's to settle.
        if owed.index != real_index {
            continue;
        }
        match settle_leftovers(git, &owed, real_index) {
            Ok(set) => {
                for path in set {
                    restated.push(crate::path_bytes::path_message_text(path.as_slice()));
                }
                remove(git_dir, owed.seq);
            }
            Err(code) => crate::diagnostic_log::warn(
                "servicePrepare",
                format!("the index update batch {} still owes in {} could not be made ({code:?}), so this commit records none of it and the record stays", owed.seq, real_index.display()).as_str(),
            ),
        }
    }
    return restated;
}
/// A path of a record as a literal pathspec.
fn literal_spec(name: &[u8]) -> std::ffi::OsString {
    let mut spec: Vec<u8> = b":(top,literal)".to_vec();
    spec.extend_from_slice(name);
    return <std::ffi::OsString as std::os::unix::ffi::OsStringExt>::from_vec(spec);
}

/// Which of the two listings a line belongs to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TabShape {
    /// `<mode> SP <type> SP <object> TAB <path>`,
    /// as `ls-tree -z` writes it.
    Tree,
    /// `<mode> SP <object> SP <stage> TAB <path>`,
    /// as `ls-files -s -z` writes it;
    /// a stage other than zero is an unmerged entry,
    /// which is the person's own.
    Index,
}

/// The entries a NUL-terminated listing holds,
/// split at its words and its tab.
fn entries_of(listing: &[u8], shape: TabShape) -> BTreeMap<Vec<u8>, Entry> {
    let mut found: BTreeMap<Vec<u8>, Entry> = BTreeMap::new();
    for record in listing.split(is_zero_byte) {
        if record.is_empty() {
            continue;
        }
        let Some(tab) = record.iter().position(is_tab_byte) else {
            continue;
        };
        let words: Vec<&[u8]> = record[..tab].split(is_space_byte).collect();
        let name: Vec<u8> = record[tab + 1..].to_vec();
        // An unmerged path is not the landing's leftover, whatever else the index holds for it.
        if shape == TabShape::Index && words.last().is_some_and(is_unmerged) {
            let _dropped = found.remove(name.as_slice());
            continue;
        }
        // A tree line is `<mode> <type> <object>` and an index line is `<mode> <object> <stage>`.
        let object: usize = if shape == TabShape::Tree { 2 } else { 1 };
        found.insert(name, Entry { mode: String::from_utf8_lossy(words.first().copied().unwrap_or_default()).into_owned(), object: String::from_utf8_lossy(words.get(object).copied().unwrap_or_default()).into_owned() });
    }
    return found;
}

/// The entries one index file holds at `names`.
fn index_entries(git: &GitContext, repository: &Path, index: &Path, names: &[Vec<u8>]) -> Result<BTreeMap<Vec<u8>, Entry>, FailureCode> {
    let specs: Vec<std::ffi::OsString> = names.iter().map(path_spec).collect();
    let mut arguments: Vec<&std::ffi::OsStr> = vec![std::ffi::OsStr::new("ls-files"), std::ffi::OsStr::new("-s"), std::ffi::OsStr::new("-z"), std::ffi::OsStr::new("--")];
    arguments.extend(specs.iter().map(std::ffi::OsString::as_os_str));
    let mut request: GitRequest = GitRequest::new(repository, &arguments);
    request.index = Some(index.to_path_buf());
    let output = match run_git(git, &request) {
        Ok(answer) if answer.succeeded() => answer,
        _ => return Err(FailureCode::Unavailable),
    };
    return Ok(entries_of(output.stdout.as_slice(), TabShape::Index));
}

/// The paths a record's update covers,
/// derived from the two branch values when the batch names none.
fn names_of(git: &GitContext, owed: &Owed) -> Result<Vec<Vec<u8>>, FailureCode> {
    if !owed.derived {
        return Ok(owed.paths.iter().map(path_bytes).collect());
    }
    let names: Vec<Vec<u8>> = crate::service_land_follow::derived_names(git, owed.repository.as_path(), owed.old.as_str(), owed.new.as_str())?
        .split(is_zero_byte)
        .filter(is_not_empty)
        .map(<[u8]>::to_vec)
        .collect();
    return Ok(names);
}

/// What:
///  The paths of a record whose entries in `index` still hold what the batch started from,
/// which are the entries the landing left.
/// Why:
///   Only those may be reset:
/// a path the person staged since the landing holds something else,
/// and resetting it
///  would discard their staging.
/// A path whose old entry does not exist is the batch's own addition,
/// and an index with no entry
///  there matches it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// leftovers(owed, index)
/// ```
pub fn leftovers(git: &GitContext, owed: &Owed, index: &Path) -> Result<Vec<Vec<u8>>, FailureCode> {
    let names: Vec<Vec<u8>> = names_of(git, owed)?;
    if names.is_empty() {
        return Ok(names);
    }
    let started: BTreeMap<Vec<u8>, Entry> = tree_entries(git, owed.repository.as_path(), owed.old.as_str(), names.as_slice())?;
    let current: BTreeMap<Vec<u8>, Entry> = index_entries(git, owed.repository.as_path(), index, names.as_slice())?;
    let mut left: Vec<Vec<u8>> = Vec::new();
    for name in names {
        if current.get(name.as_slice()) == started.get(name.as_slice()) {
            left.push(name);
        }
    }
    return Ok(left);
}

/// What:
///  Settles one owed record for one index:
/// the entries the landing left are reset to the commit the batch landed,
/// and the ones a program staged since are left,
/// with the ones reset given back.
/// Why:
///   The reset is the landing's own idempotent step (`service_land_follow.rs`),
/// made here through Git's own lock,
/// so a concurrent
///  staging of a path whose entry is not the landing's is never a candidate,
/// and a concurrent staging of one that is ends
///  in the one thing Git's own `reset` does with a lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reset = await settleLeftovers(owed, realIndex); // the paths this settled
/// ```
pub fn settle_leftovers(git: &GitContext, owed: &Owed, index: &Path) -> Result<Vec<Vec<u8>>, FailureCode> {
    let left: Vec<Vec<u8>> = leftovers(git, owed, index)?;
    if !left.is_empty() {
        crate::service_land_follow::reset_entries(git, owed.repository.as_path(), Some(index), owed.new.as_str(), left.as_slice())?;
    }
    return Ok(left);
}

/// Runs one Git step,
/// naming no answer as a failure the caller can classify.
fn run(git: &GitContext, repository: &Path, arguments: &[&std::ffi::OsStr], input: Option<Vec<u8>>) -> Result<crate::transaction_git::GitOutput, FailureCode> {
    let mut request: GitRequest = GitRequest::new(repository, arguments);
    request.input = input;
    return match run_git(git, &request) {
        Ok(output) if output.succeeded() => Ok(output),
        _ => Err(FailureCode::Unavailable),
    };
}
