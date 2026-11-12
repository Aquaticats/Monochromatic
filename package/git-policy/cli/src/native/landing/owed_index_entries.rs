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
//! Each read has two faces:
//! the synchronous one a caller on a thread asks,
//! and the awaited one a task of the runtime asks
//!  (`tree_entries_awaiting`,
//! `index_entries_awaiting`,
//! `whole_listing_awaiting`,
//! `head_entries_awaiting`,
//! `moved_since_listing_awaiting`),
//! which awaits its Git child on the runtime.
//! Both faces ask
//!  the one request and read Git's answer through one function,
//! so neither can answer what the other does not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [mode, object, stage] = frontFields(record); const entries = await indexEntries(index, names);
//! ```

use crate::service_protocol::{FailureCode, is_zero_object};
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::transaction_git_awaiting::run_git_awaiting;
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

/// What:
///  The question `with_specs` asks Git:
///  the words,
///  a `--`,
///  and the names as literal pathspecs from the top of the worktree,
///  against one index file.
/// Why:
///  Both faces ask the one question,
/// so neither can read another listing than the other,
/// and the pathspec mode the caller
///  inherited reaches neither of them (`own_pathspecs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: [...words, '--', ...specs], index, ownPathspecs: true };
/// ```
fn specs_request(repository: &Path, index: Option<&Path>, words: &[&str], names: &[Vec<u8>]) -> GitRequest {
    let specs: Vec<OsString> = specs_of(names);
    let mut arguments: Vec<&OsStr> = words.iter().map(OsStr::new).collect();
    arguments.push(OsStr::new("--"));
    arguments.extend(specs.iter().map(OsString::as_os_str));
    let mut request: GitRequest = GitRequest::new(repository, arguments.as_slice()).own_pathspecs();
    request.index = index.map(Path::to_path_buf);
    return request;
}

/// What:
///  Git's answer to one of these questions,
/// when it gave one this module can read.
/// Why:
///  Both faces read every answer through this one function,
/// so a step Git refused and a listing it wrote cannot be told
///  apart differently by the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function answered(asked: GitOutput | Error): GitOutput; // throws FailureCode.Unavailable
/// ```
fn answered(asked: std::io::Result<GitOutput>) -> Result<GitOutput, FailureCode> {
    return match asked {
        Ok(output) if output.succeeded() => Ok(output),
        _ => Err(FailureCode::Unavailable),
    };
}

/// One Git question with `names` as literal pathspecs,
/// answered or not.
pub(crate) fn with_specs(git: &GitContext, repository: &Path, index: Option<&Path>, words: &[&str], names: &[Vec<u8>]) -> Result<GitOutput, FailureCode> {
    return answered(run_git(git, &specs_request(repository, index, words, names)));
}

/// `with_specs` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function withSpecsAwaiting(git, repository, index, words, names): Promise<GitOutput>;
/// ```
async fn with_specs_awaiting(git: &GitContext, repository: &Path, index: Option<&Path>, words: &[&str], names: &[Vec<u8>]) -> Result<GitOutput, FailureCode> {
    return answered(run_git_awaiting(git, &specs_request(repository, index, words, names)).await);
}

/// What:
///  The entries one tree listing holds,
/// by path.
/// Why:
///  Both faces read a tree listing through this one function,
/// so a path that holds a tab and an entry whose kind is
///  not a blob are read the same by the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeEntriesOf(stdout: Uint8Array): Map<path, Entry>;
/// ```
fn tree_entries_of(stdout: &[u8]) -> BTreeMap<Vec<u8>, Entry> {
    let mut found: BTreeMap<Vec<u8>, Entry> = BTreeMap::new();
    for record in stdout.split(is_zero_byte) {
        // A tree line is `<mode> <type> <object> TAB <path>`.
        let Some(([mode, _kind, object], path)) = front_fields(record) else { continue };
        found.insert(path.to_vec(), (String::from_utf8_lossy(mode).into_owned(), String::from_utf8_lossy(object).into_owned()));
    }
    return found;
}

/// What:
///  The entries one index listing holds,
/// by path,
/// with a path that holds unmerged stages marked as a conflict's.
/// Why:
///  Both faces read an index listing through this one function,
/// so a conflict is marked,
/// and a stage-zero entry
///  replaced by a later one,
/// the same way for the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexEntriesOf(stdout: Uint8Array): Map<path, Entry>; // a stage above zero marks the path 'unmerged'
/// ```
fn index_entries_of(stdout: &[u8]) -> BTreeMap<Vec<u8>, Entry> {
    let mut found: BTreeMap<Vec<u8>, Entry> = BTreeMap::new();
    for record in stdout.split(is_zero_byte) {
        // An index line is `<mode> <object> <stage> TAB <path>`; a stage other than zero is a conflict.
        let Some(([mode, object, stage], path)) = front_fields(record) else { continue };
        if is_conflict_stage(&stage) {
            let _dropped = found.remove(path);
            found.insert(path.to_vec(), (String::from("unmerged"), String::new()));
            continue;
        }
        found.insert(path.to_vec(), (String::from_utf8_lossy(mode).into_owned(), String::from_utf8_lossy(object).into_owned()));
    }
    return found;
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
    return Ok(tree_entries_of(output.stdout.as_slice()));
}

/// `tree_entries` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function treeEntriesAwaiting(git, repository, old, names): Promise<Map<path, Entry>>;
/// ```
///
/// # Errors
///
/// As `tree_entries`.
pub(crate) async fn tree_entries_awaiting(git: &GitContext, repository: &Path, old: &str, names: &[Vec<u8>]) -> Result<BTreeMap<Vec<u8>, Entry>, FailureCode> {
    if names.is_empty() || is_zero_object(old) {
        return Ok(BTreeMap::new());
    }
    let output = with_specs_awaiting(git, repository, None, &["ls-tree", "-z", "--full-tree", old], names).await?;
    return Ok(tree_entries_of(output.stdout.as_slice()));
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
    return Ok(index_entries_of(output.stdout.as_slice()));
}

/// `index_entries` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function indexEntriesAwaiting(git, repository, index, names): Promise<Map<path, Entry>>;
/// ```
///
/// # Errors
///
/// As `index_entries`.
pub(crate) async fn index_entries_awaiting(git: &GitContext, repository: &Path, index: &Path, names: &[Vec<u8>]) -> Result<BTreeMap<Vec<u8>, Entry>, FailureCode> {
    if names.is_empty() {
        return Ok(BTreeMap::new());
    }
    let output = with_specs_awaiting(git, repository, Some(index), &["ls-files", "-s", "-z", "--full-name"], names).await?;
    return Ok(index_entries_of(output.stdout.as_slice()));
}

/// Whether an index entry is a conflict's,
/// which is left alone.
pub(crate) fn is_unmerged(entry: Option<&Entry>) -> bool {
    let Some((mode, _)) = entry else { return false };
    return mode == "unmerged";
}

/// What:
///  The question `whole_listing` asks Git:
///  every entry of one index,
///  in full names from the top of the worktree.
/// Why:
///  Both faces ask the one question,
/// so neither lists another index or shortens another name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: ['ls-files', '-s', '-z', '--full-name'], index };
/// ```
fn whole_listing_request(repository: &Path, index: &Path) -> GitRequest {
    // `--full-name`: the names of both listings are compared and then named to `git rm` and `git reset` as
    // `:(top,literal)<path>` specs, which a caller's `-C` into a subdirectory would otherwise shorten.
    let mut request: GitRequest = GitRequest::new(repository, &["ls-files", "-s", "-z", "--full-name"]);
    request.index = Some(index.to_path_buf());
    return request;
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
    let output = answered(run_git(git, &whole_listing_request(repository, index)))?;
    return Ok(index_entries_of(output.stdout.as_slice()));
}

/// `whole_listing` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function wholeListingAwaiting(git, repository, index): Promise<Map<path, Entry>>;
/// ```
///
/// # Errors
///
/// As `whole_listing`.
pub(crate) async fn whole_listing_awaiting(git: &GitContext, repository: &Path, index: &Path) -> Result<BTreeMap<Vec<u8>, Entry>, FailureCode> {
    let output = answered(run_git_awaiting(git, &whole_listing_request(repository, index)).await)?;
    return Ok(index_entries_of(output.stdout.as_slice()));
}

/// What:
///  The question `head_entries` asks first:
///  whether the worktree's head names any commit at all.
/// Why:
///  Both faces ask the one question,
/// so a branch that does not exist yet stops neither of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: ['rev-parse', '--verify', '--quiet', 'HEAD'] };
/// ```
fn unborn_request(repository: &Path) -> GitRequest {
    return GitRequest::new(repository, &["rev-parse", "--verify", "--quiet", "HEAD"]);
}

/// What:
///  Whether Git's answer to `unborn_request` says the head names no commit.
/// Why:
///   An unborn branch has no head to hold an entry:
/// nothing is there,
/// and a commit that owes an index update must not stop for it.
/// Only `rev-parse --verify --quiet`'s own answer for a name that is no object means that:
///  exit 1 with nothing on the standard error.
/// A probe that failed for any other reason is a failure to know the head,
///  and the caller restores what it had.
/// Both faces read the answer through this one function,
/// so neither takes a failure for an unborn branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unbornAnswered(asked: GitOutput | Error): boolean; // throws FailureCode.Unavailable
/// ```
fn unborn_answered(asked: std::io::Result<GitOutput>) -> Result<bool, FailureCode> {
    return match asked {
        Ok(answer) if answer.succeeded() => Ok(false),
        Ok(answer) if answer.code == Some(NAME_IS_NO_OBJECT) && answer.stderr.is_empty() => Ok(true),
        _ => Err(FailureCode::Unavailable),
    };
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
    if unborn_answered(run_git(git, &unborn_request(repository)))? {
        return Ok(BTreeMap::new());
    }
    let output = with_specs(git, repository, None, &["ls-tree", "-z", "--full-tree", "HEAD"], paths)?;
    return Ok(tree_entries_of(output.stdout.as_slice()));
}

/// `head_entries` with its Git children awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function headEntriesAwaiting(git, repository, paths): Promise<Map<path, Entry>>;
/// ```
///
/// # Errors
///
/// As `head_entries`.
pub(crate) async fn head_entries_awaiting(git: &GitContext, repository: &Path, paths: &[Vec<u8>]) -> Result<BTreeMap<Vec<u8>, Entry>, FailureCode> {
    if unborn_answered(run_git_awaiting(git, &unborn_request(repository)).await)? {
        return Ok(BTreeMap::new());
    }
    let output = with_specs_awaiting(git, repository, None, &["ls-tree", "-z", "--full-tree", "HEAD"], paths).await?;
    return Ok(tree_entries_of(output.stdout.as_slice()));
}

/// What:
///  The paths whose entry differs between two listings,
/// in the live listing's order and then the held one's.
/// Why:
///  Both faces compare two listings through this one function,
/// so a path only one side holds and a path whose entry
///  moved are named the same way for the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function movedBetween(live, held): path[];
/// ```
fn moved_between(live: &BTreeMap<Vec<u8>, Entry>, held: &BTreeMap<Vec<u8>, Entry>) -> Vec<Vec<u8>> {
    let mut moved: Vec<Vec<u8>> = Vec::new();
    for (path, entry) in live {
        if held.get(path) != Some(entry) {
            moved.push(path.clone());
        }
    }
    for path in held.keys() {
        if !live.contains_key(path) {
            moved.push(path.clone());
        }
    }
    return names_in(&moved.join(&0));
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
    return Ok(moved_between(&live, &held));
}

/// `moved_since_listing` with both of its Git children awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function movedSinceListingAwaiting(git, repository, liveIndex, commitIndex): Promise<path[]>;
/// ```
///
/// # Errors
///
/// As `moved_since_listing`.
pub(crate) async fn moved_since_listing_awaiting(git: &GitContext, repository: &Path, live_index: &Path, commit_index: &Path) -> Result<Vec<Vec<u8>>, FailureCode> {
    let live: BTreeMap<Vec<u8>, Entry> = whole_listing_awaiting(git, repository, live_index).await?;
    let held: BTreeMap<Vec<u8>, Entry> = whole_listing_awaiting(git, repository, commit_index).await?;
    return Ok(moved_between(&live, &held));
}

/// Controls for the reads,
///  against real Git.
#[cfg(test)]
#[path = "owed_index_entries_tests.rs"]
mod tests;

/// Controls for the awaited faces of these reads stay out of the release executable.
#[cfg(test)]
#[path = "owed_index_entries_awaiting_tests.rs"]
mod awaiting_tests;
