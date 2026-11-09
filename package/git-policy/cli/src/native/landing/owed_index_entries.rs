//! What:
//!  How the repair reads Git's entry listings:
//! one entry of a tree or one index,
//! by its fixed fields from the front,
//! and the listings
//!  it compares.
//! Why:
//!   A path may hold a tab,
//! so an entry is read from the front:
//! three words,
//! the tab that follows them,
//! and the rest as the path.
//! The repair asks what an entry is,
//! so the reading of entries and the deciding about them are separate files.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [mode, object, stage] = frontFields(record); const entries = await indexEntries(index, names);
//! ```

use crate::service_protocol::{FailureCode, is_zero_object};
use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStringExt as _;
use std::path::Path;

/// The exit status `git rev-parse --verify --quiet` gives for a name that is no object:
/// the one answer that means "not there".
const NAME_IS_NO_OBJECT: i32 = 1;

/// One entry of a tree or an index:
/// its mode and its object.
pub type Entry = (String, String);

/// What:
///  The names of a NUL-separated listing,
/// each as the bytes Git wrote.
/// Why:
///  A `-z` listing of paths needs no quoting and no character set,
/// so the bytes are kept as they are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namesIn(bytes) // split on NUL, drop the empty tail
/// ```
pub(crate) fn names_in(listing: &[u8]) -> Vec<Vec<u8>> {
    let mut names: Vec<Vec<u8>> = Vec::new();
    for name in listing.split(is_zero_byte) {
        if !name.is_empty() {
            names.push(name.to_vec());
        }
    }
    return names;
}

/// Whether a byte is the NUL that ends one record of a `-z` listing.
pub(crate) fn is_zero_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// Whether a byte is the tab that separates an entry's words from its path.
fn is_tab_byte(byte: &u8) -> bool {
    return *byte == b'\t';
}

/// Whether a byte is the space between one entry's words.
fn is_space_byte(byte: &u8) -> bool {
    return *byte == b' ';
}

/// Whether an index entry's stage column names a conflict.
fn is_conflict_stage(stage: &&[u8]) -> bool {
    return *stage != b"0";
}

/// What:
///  One line of a NUL-delimited index or tree listing,
/// read by its fields from the front:
/// three words,
/// a tab,
/// and the path.
/// Why:
///   A path may hold a tab,
/// so the tab that ends an entry's words is the first one,
/// never the last:
///   `100644 <object> 0<TAB>z<TAB>b` is the entry `z<TAB>b` at stage zero,
/// and reading it from the back would
///  take the file `b` for a conflict of a file named `0<TAB>z`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [mode, second, third, path] = frontFields(record); // three words, a tab, then the path
/// ```
fn front_fields(record: &[u8]) -> Option<([&[u8]; 3], &[u8])> {
    let first: usize = record.iter().position(is_space_byte)?;
    let second: usize = record[first + 1..].iter().position(is_space_byte)? + first + 1;
    let third: usize = record[second + 1..].iter().position(is_tab_byte)? + second + 1;
    return Some(([&record[..first], &record[first + 1..second], &record[second + 1..third]], &record[third + 1..]));
}

/// The paths as literal pathspecs from the top of the worktree.
pub(crate) fn specs_of(names: &[Vec<u8>]) -> Vec<OsString> {
    let mut specs: Vec<OsString> = Vec::with_capacity(names.len());
    for name in names {
        let mut spec: Vec<u8> = b":(top,literal)".to_vec();
        spec.extend_from_slice(name.as_slice());
        specs.push(OsString::from_vec(spec));
    }
    return specs;
}

/// One Git question with `names` as literal pathspecs,
/// answered or not.
pub(crate) fn with_specs(git: &GitContext, repository: &Path, index: Option<&Path>, words: &[&str], names: &[Vec<u8>]) -> Result<crate::transaction_git::GitOutput, FailureCode> {
    let specs: Vec<OsString> = specs_of(names);
    let mut arguments: Vec<&OsStr> = words.iter().map(OsStr::new).collect();
    arguments.push(OsStr::new("--"));
    arguments.extend(specs.iter().map(OsString::as_os_str));
    let mut request: GitRequest = GitRequest::new(repository, arguments.as_slice()).own_pathspecs();
    request.index = index.map(Path::to_path_buf);
    return match run_git(git, &request) {
        Ok(output) if output.succeeded() => Ok(output),
        _ => Err(FailureCode::Unavailable),
    };
}

/// What:
///  The entries a tree holds at `names`.
/// Why:
///   The entry the landing replaced is read from the tree before it,
/// and an unborn branch has none at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// treeEntries(old, names) // `ls-tree -z --full-tree <old> -- :(top,literal)<name>...`
/// ```
pub(crate) fn tree_entries(git: &GitContext, repository: &Path, old: &str, names: &[Vec<u8>]) -> Result<BTreeMap<Vec<u8>, Entry>, FailureCode> {
    if names.is_empty() || is_zero_object(old) {
        return Ok(BTreeMap::new());
    }
    let output = with_specs(git, repository, None, &["ls-tree", "-z", "--full-tree", old], names)?;
    let mut found: BTreeMap<Vec<u8>, Entry> = BTreeMap::new();
    for record in output.stdout.split(is_zero_byte) {
        // A tree line is `<mode> <type> <object> TAB <path>`.
        let Some(([mode, _kind, object], path)) = front_fields(record) else { continue };
        found.insert(path.to_vec(), (String::from_utf8_lossy(mode).into_owned(), String::from_utf8_lossy(object).into_owned()));
    }
    return Ok(found);
}

/// What:
///  The entries one index file holds at `names`,
/// leaving out every path with unmerged stages,
/// which is a merge's own and never a landing's leftover.
/// Why:
///   What the commit is about to record for a path is this entry.
///
///  The question is asked with `--full-name`,
///  because every name here is written from the top of the
/// worktree and looked up again by that same path:
///  without it `ls-files` answers relative to the
/// directory Git runs in,
///  which a caller's own `-C` can move below the top.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// indexEntries(index, names) // `ls-files -s -z --full-name -- :(top,literal)<name>...`
/// ```
pub(crate) fn index_entries(git: &GitContext, repository: &Path, index: &Path, names: &[Vec<u8>]) -> Result<BTreeMap<Vec<u8>, Entry>, FailureCode> {
    if names.is_empty() {
        return Ok(BTreeMap::new());
    }
    let output = with_specs(git, repository, Some(index), &["ls-files", "-s", "-z", "--full-name"], names)?;
    let mut found: BTreeMap<Vec<u8>, Entry> = BTreeMap::new();
    for record in output.stdout.split(is_zero_byte) {
        // An index line is `<mode> <object> <stage> TAB <path>`; a stage other than zero is a conflict.
        let Some(([mode, object, stage], path)) = front_fields(record) else { continue };
        if is_conflict_stage(&stage) {
            let _dropped = found.remove(path);
            found.insert(path.to_vec(), (String::from("unmerged"), String::new()));
            continue;
        }
        found.insert(path.to_vec(), (String::from_utf8_lossy(mode).into_owned(), String::from_utf8_lossy(object).into_owned()));
    }
    return Ok(found);
}

/// Whether an index entry is a conflict's,
/// which is left alone.
pub(crate) fn is_unmerged(entry: Option<&Entry>) -> bool {
    let Some((mode, _)) = entry else { return false };
    return mode == "unmerged";
}

/// What:
///  Every entry of one index,
/// as a map from the path to its mode and object.
/// Why:
///   The whole listing is what `moved_since` falls back to when a tree cannot be written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wholeListing(index) // `ls-files -s -z`
/// ```
pub(crate) fn whole_listing(git: &GitContext, repository: &Path, index: &Path) -> Result<BTreeMap<Vec<u8>, Entry>, FailureCode> {
    // `--full-name`: the names of both listings are compared and then named to `git rm` and `git reset` as
    // `:(top,literal)<path>` specs, which a caller's `-C` into a subdirectory would otherwise shorten.
    let mut request: GitRequest = GitRequest::new(repository, &["ls-files", "-s", "-z", "--full-name"]);
    request.index = Some(index.to_path_buf());
    let output = match run_git(git, &request) {
        Ok(answer) if answer.succeeded() => answer,
        _ => return Err(FailureCode::Unavailable),
    };
    let mut found: BTreeMap<Vec<u8>, Entry> = BTreeMap::new();
    for record in output.stdout.split(is_zero_byte) {
        let Some(([mode, object, stage], path)) = front_fields(record) else { continue };
        if is_conflict_stage(&stage) {
            let _dropped = found.remove(path);
            found.insert(path.to_vec(), (String::from("unmerged"), String::new()));
            continue;
        }
        found.insert(path.to_vec(), (String::from_utf8_lossy(mode).into_owned(), String::from_utf8_lossy(object).into_owned()));
    }
    return Ok(found);
}

/// What:
///  The entries the worktree's head holds at `paths`.
/// Why:
///   A refused `reset` leaves the entries to be written by index-info from the head's tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// headEntries(paths) // `ls-tree -z --full-tree HEAD -- :(top,literal)<path>...`
/// ```
pub(crate) fn head_entries(git: &GitContext, repository: &Path, paths: &[Vec<u8>]) -> Result<BTreeMap<Vec<u8>, Entry>, FailureCode> {
    // An unborn branch has no head to hold an entry: nothing is there, and a commit that owes an index update must not stop for it.
    // Only `rev-parse --verify --quiet`'s own answer for a name that is no object means that:
    //  exit 1 with nothing on the standard error.
    // A probe that failed for any other reason is a failure to know the head,
    //  and the caller restores what it had.
    let unborn: GitRequest = GitRequest::new(repository, &["rev-parse", "--verify", "--quiet", "HEAD"]);
    match run_git(git, &unborn) {
        Ok(answer) if answer.succeeded() => {}
        Ok(answer) if answer.code == Some(NAME_IS_NO_OBJECT) && answer.stderr.is_empty() => return Ok(BTreeMap::new()),
        _ => return Err(FailureCode::Unavailable),
    }
    let output = with_specs(git, repository, None, &["ls-tree", "-z", "--full-tree", "HEAD"], paths)?;
    let mut found: BTreeMap<Vec<u8>, Entry> = BTreeMap::new();
    for record in output.stdout.split(is_zero_byte) {
        let Some(([mode, _kind, object], path)) = front_fields(record) else { continue };
        found.insert(path.to_vec(), (String::from_utf8_lossy(mode).into_owned(), String::from_utf8_lossy(object).into_owned()));
    }
    return Ok(found);
}

/// What:
///  `moved_since` by whole listings,
/// for an index a tree cannot be written from (a path in conflict).
/// Why:
///   A listing of both indexes needs no tree,
/// so the check that protects a stale snapshot is never lost to an
///  unmerged entry somewhere else in the index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// movedSinceByListing(liveIndex, commitIndex) // every path whose entry differs, from `ls-files -s -z`
/// ```
pub(crate) fn moved_since_listing(git: &GitContext, repository: &Path, live_index: &Path, commit_index: &Path) -> Result<Vec<Vec<u8>>, FailureCode> {
    let live: BTreeMap<Vec<u8>, Entry> = whole_listing(git, repository, live_index)?;
    let held: BTreeMap<Vec<u8>, Entry> = whole_listing(git, repository, commit_index)?;
    let mut moved: Vec<Vec<u8>> = Vec::new();
    for (path, entry) in &live {
        if held.get(path) != Some(entry) {
            moved.push(path.clone());
        }
    }
    for path in held.keys() {
        if !live.contains_key(path) {
            moved.push(path.clone());
        }
    }
    return Ok(names_in(&moved.join(&0)));
}

/// Controls for the reads,
///  against real Git.
#[cfg(test)]
#[path = "owed_index_entries_tests.rs"]
mod tests;
