//! What:
//!  What a commit does about the index entries a landed batch still owes:
//! it records nothing about a path that still holds exactly the content the landing replaced,
//! and nothing
//!  about a path whose index entry moved after the commit copied it.
//! Why:
//!   A batch's branch moves before its index is installed,
//! so the worktree's index can describe the branch before a landing,
//! and a
//!  commit of that index would record the landed commit's own paths as their old content,
//! or as a staged deletion.
//!   The question is not who owns an entry but what the commit is about to record:
//! the record names a path with the entry the
//!  landing replaced (`old`) and the entry it landed (`new`),
//! and a path is left out exactly while the commit's own index still
//!  holds `old` there.
//! That asks about content,
//! so an amend,
//! a `reset --hard`,
//! an attach from a detached head and an
//!  alternate index need no rule of their own.
//! The price is that a person's deliberate restoration of that same content is left out
//!  too,
//! with the notice that names the path and both ways on,
//! and their staging stays in the live index.
//! A path whose
//!  live entry moved after the commit copied the index is left out as well:
//! that snapshot is older than the index,
//! so what the
//!  commit would record there is not the person's current intent.
//! Nothing here writes the live index or removes a record:
//! a record that no
//!  longer matches anything costs a comparison,
//! and the service settles it when it installs the index.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const left = await excludeWhatALandedCommitLeft(git, gitDir, liveIndex, commitIndex);
//! ```

use crate::owed_index_update::{Owed, records_of};
use crate::service_protocol::{FailureCode, is_zero_object};
use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStringExt as _;
use std::path::{Path, PathBuf};

/// One entry of a tree or an index:
/// its mode and its object.
type Entry = (String, String);

/// The paths one record's update covers,
/// derived from the two branch values when the batch names none.
fn names_of(git: &GitContext, owed: &Owed) -> Result<Vec<Vec<u8>>, FailureCode> {
    if !owed.derived {
        let mut listed: Vec<Vec<u8>> = Vec::with_capacity(owed.paths.len());
        for path in &owed.paths {
            listed.push(path.clone().into_bytes());
        }
        return Ok(listed);
    }
    let names: Vec<u8> = crate::service_land_follow::derived_names(git, owed.repository.as_path(), owed.old.as_str(), owed.new.as_str())?;
    return Ok(names_in(names.as_slice()));
}

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
fn names_in(listing: &[u8]) -> Vec<Vec<u8>> {
    let mut names: Vec<Vec<u8>> = Vec::new();
    for name in listing.split(is_zero_byte) {
        if !name.is_empty() {
            names.push(name.to_vec());
        }
    }
    return names;
}

/// Whether a byte is the NUL that ends one record of a `-z` listing.
fn is_zero_byte(byte: &u8) -> bool {
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
fn specs_of(names: &[Vec<u8>]) -> Vec<OsString> {
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
fn with_specs(git: &GitContext, repository: &Path, index: Option<&Path>, words: &[&str], names: &[Vec<u8>]) -> Result<crate::transaction_git::GitOutput, FailureCode> {
    let specs: Vec<OsString> = specs_of(names);
    let mut arguments: Vec<&OsStr> = words.iter().map(OsStr::new).collect();
    arguments.push(OsStr::new("--"));
    arguments.extend(specs.iter().map(OsString::as_os_str));
    let mut request: GitRequest = GitRequest::new(repository, arguments.as_slice());
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
fn tree_entries(git: &GitContext, owed: &Owed, names: &[Vec<u8>]) -> Result<BTreeMap<Vec<u8>, Entry>, FailureCode> {
    if names.is_empty() || is_zero_object(owed.old.as_str()) {
        return Ok(BTreeMap::new());
    }
    let output = with_specs(git, owed.repository.as_path(), None, &["ls-tree", "-z", "--full-tree", owed.old.as_str()], names)?;
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
/// In TS you'd write (pseudocode):
/// ```ts
/// indexEntries(index, names) // `ls-files -s -z -- :(top,literal)<name>...`
/// ```
fn index_entries(git: &GitContext, owed: &Owed, index: &Path, names: &[Vec<u8>]) -> Result<BTreeMap<Vec<u8>, Entry>, FailureCode> {
    if names.is_empty() {
        return Ok(BTreeMap::new());
    }
    let output = with_specs(git, owed.repository.as_path(), Some(index), &["ls-files", "-s", "-z"], names)?;
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
fn is_unmerged(entry: Option<&Entry>) -> bool {
    let Some((mode, _)) = entry else { return false };
    return mode == "unmerged";
}

/// What:
///  Sets the commit's own index to the worktree's head for the paths whose entry it would record against that head,
///  and gives those paths.
/// Why:
///   This is the whole of what a commit does about a landed batch's owed index update.
/// The live index is never written,
///  so nothing that may be a program's staging is replaced,
/// and nothing here removes a record,
/// so no other command's
///  snapshot is left unprotected.
/// A path is left out while the commit's own index still holds the entry the landing
///  replaced,
/// or while its live entry moved after the copy:
/// one can be the landing's leftover,
/// the other is someone
///  else's newer intent,
/// and in both cases what the commit would record there is not the person's.
/// The paths are set to the
///  head rather than to the landed commit,
/// so the commit records nothing about them whatever the branch has done since.
/// An explicit-path commit names its own selection,
/// which is the person's statement,
/// so nothing is
///  taken from it and the notice alone is given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// excludeWhatALandedCommitLeft(git, gitDir, liveIndex, commitIndex) // the paths this commit leaves out
/// ```
pub fn exclude_what_a_landed_commit_left(
    git: &GitContext,
    git_dir: &Path,
    live_index: &Path,
    commit_index: Option<&Path>,
    captured: &crate::service_land_start::IndexWitness,
) -> Vec<String> {
    let mut left: Vec<String> = Vec::new();
    // What the commit copies is not what the index holds any more: the paths whose entry moved since the copy are left out,
    //  because what the snapshot would record there is not the person's current intent. No record is needed for this, so a record
    //  another command removed, or one the service settled, cannot expose the snapshot.
    if let (Some(index), Ok(now)) = (commit_index, crate::service_land_start::witness_of(live_index))
        && now != *captured
    {
        match moved_since(git, git_dir.parent().unwrap_or(git_dir), live_index, index) {
            Ok(moved) => {
                if !moved.is_empty() {
                    match set_to_head_paths(git, git_dir.parent().unwrap_or(git_dir), index, moved.as_slice()) {
                        Ok(()) => {
                            for path in &moved {
                                left.push(crate::path_bytes::path_message_text(path.as_slice()));
                            }
                        }
                        Err(code) => crate::diagnostic_log::warn(
                            "servicePrepare",
                            format!("the commit's own index could not be set to the head for the entries the index moved since its copy ({code:?}), so this commit records them as its index holds them").as_str(),
                        ),
                    }
                }
            }
            Err(_unwritable) => match moved_since_listing(git, git_dir.parent().unwrap_or(git_dir), live_index, index) {
                Ok(moved) if moved.is_empty() => {}
                Ok(moved) => match set_to_head_paths(git, git_dir.parent().unwrap_or(git_dir), index, moved.as_slice()) {
                    Ok(()) => {
                        for path in &moved {
                            left.push(crate::path_bytes::path_message_text(path.as_slice()));
                        }
                    }
                    Err(code) => crate::service_error!(
                        "the entries the index moved since the commit's copy cannot be set to the head ({code:?}), so this commit records them as its index holds them"
                    ),
                },
                Err(code) => crate::service_error!(
                    "the entries the index moved since the commit's copy cannot be read either way ({code:?}), so this commit records them as its index holds them"
                ),
            },
        }
    }
    for owed in records_of(git_dir) {
        // The record names the index file whose update it owes; a commit of another one is another question.
        if owed.index != live_index {
            continue;
        }
        let snapshot: &Path = commit_index.unwrap_or(live_index);
        let names: Vec<Vec<u8>> = match names_of(git, &owed) {
            Ok(names) => names,
            Err(code) => {
                crate::diagnostic_log::warn("servicePrepare", format!("the paths of the index update batch {} owes cannot be read ({code:?}), so this commit leaves them as its index holds them", owed.seq).as_str());
                continue;
            }
        };
        let replaced: BTreeMap<Vec<u8>, Entry> = match tree_entries(git, &owed, names.as_slice()) {
            Ok(entries) => entries,
            Err(code) => {
                crate::diagnostic_log::warn("servicePrepare", format!("the tree the landing of batch {} replaced cannot be read ({code:?}), so this commit leaves its paths as its index holds them", owed.seq).as_str());
                continue;
            }
        };
        let held: BTreeMap<Vec<u8>, Entry> = match index_entries(git, &owed, snapshot, names.as_slice()) {
            Ok(entries) => entries,
            Err(code) => {
                crate::diagnostic_log::warn("servicePrepare", format!("the commit's own index cannot be read for the paths of batch {} ({code:?}), so it leaves them as it holds them", owed.seq).as_str());
                continue;
            }
        };
        let mut excluded: Vec<Vec<u8>> = Vec::new();
        for name in &names {
            let entry: Option<&Entry> = held.get(name);
            if is_unmerged(entry) {
                // A merge in progress is the person's own act.
                continue;
            }
            // What the command staged itself (`--all`, `--include`) is its own: only the entry the landing replaced is left out.
            //  Entries that moved in the live index are caught above, where the file the commit copied is compared with the file the
            //  index is now.
            if entry == replaced.get(name) {
                excluded.push(name.clone());
            }
        }
        if excluded.is_empty() {
            continue;
        }
        if let Some(index) = commit_index {
            match set_to_head(git, &owed, index, excluded.as_slice()) {
                Ok(()) => {
                    for path in &excluded {
                        left.push(crate::path_bytes::path_message_text(path.as_slice()));
                    }
                }
                Err(code) => crate::diagnostic_log::warn(
                    "servicePrepare",
                    format!("the commit's own index could not be set to the head for the entries batch {} owes ({code:?}), so this commit records them as its index holds them", owed.seq).as_str(),
                ),
            }
        } else {
            for path in &excluded {
                left.push(crate::path_bytes::path_message_text(path.as_slice()));
            }
        }
    }
    return left;
}

/// What:
///  The paths in which a commit's own index differs from the live index.
/// Why:
///   A writer replaces the index file whole,
/// so a live index that is another file than the one the commit copied may
///  hold entries the commit's copy does not:
/// the paths that differ are where the copy is older.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// movedSince(liveIndex, commitIndex) // `git write-tree` then `diff-index --cached <tree>` on the copy
/// ```
fn moved_since(git: &GitContext, repository: &Path, live_index: &Path, commit_index: &Path) -> Result<Vec<Vec<u8>>, FailureCode> {
    let mut tree: GitRequest = GitRequest::new(repository, &["write-tree"]);
    tree.index = Some(live_index.to_path_buf());
    let written = match run_git(git, &tree) {
        Ok(answer) if answer.succeeded() => answer,
        _ => return Err(FailureCode::Unavailable),
    };
    let held: String = String::from_utf8_lossy(written.stdout.as_slice()).trim().to_string();
    let mut moved: GitRequest = GitRequest::new(repository, &["diff-index", "--cached", "--name-only", "-z", held.as_str()]);
    moved.index = Some(commit_index.to_path_buf());
    let listing = match run_git(git, &moved) {
        Ok(output) if output.succeeded() => output,
        _ => return Err(FailureCode::Unavailable),
    };
    return Ok(names_in(listing.stdout.as_slice()));
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
fn moved_since_listing(git: &GitContext, repository: &Path, live_index: &Path, commit_index: &Path) -> Result<Vec<Vec<u8>>, FailureCode> {
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
fn whole_listing(git: &GitContext, repository: &Path, index: &Path) -> Result<BTreeMap<Vec<u8>, Entry>, FailureCode> {
    let mut request: GitRequest = GitRequest::new(repository, &["ls-files", "-s", "-z"]);
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

/// `set_to_head` for paths whose old entry is no record's,
/// with the worktree's head as the entry to take.
fn set_to_head_paths(git: &GitContext, repository: &Path, index: &Path, paths: &[Vec<u8>]) -> Result<(), FailureCode> {
    return set_to_head_with(git, repository, index, paths, index);
}

/// What:
///  Sets an index to the worktree's head at `paths`,
/// removing whatever it held there and below them first,
/// and leaves the file exactly as it was when any step fails.
/// Why:
///   A path the landing added can stand beside entries a program staged below it
/// (`p` and `p/child`),
/// and the head holds one of
///  the two:
/// an index that holds both cannot be written to a tree,
/// so the commit's own index takes the head's shape for
///  the path.
/// The removal is forced,
/// because the entry may hold neither the head's content nor the worktree's,
/// which is
///  exactly what a failed index update leaves.
/// The two steps are not atomic,
/// so the file is copied first and put back when either
///  fails:
/// a removal that outlived a refused reset would make the commit record a deletion the person never made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await setToHead(owed, index, paths) // copy aside; rm --cached -r --force; reset HEAD; restore on failure
/// ```
fn set_to_head(git: &GitContext, owed: &Owed, index: &Path, paths: &[Vec<u8>]) -> Result<(), FailureCode> {
    return set_to_head_with(git, owed.repository.as_path(), index, paths, index);
}

/// `set_to_head` with the repository given,
/// for a caller whose index is not the record's own.
fn set_to_head_with(git: &GitContext, repository: &Path, index: &Path, paths: &[Vec<u8>], _named: &Path) -> Result<(), FailureCode> {
    if paths.is_empty() {
        return Ok(());
    }
    let kept: PathBuf = index.with_extension("before-head");
    if let Err(error) = std::fs::copy(index, kept.as_path()) {
        crate::diagnostic_log::warn("servicePrepare", format!("the commit's own index cannot be copied aside before it is set to the head ({error})").as_str());
        return Err(FailureCode::Unavailable);
    }
    let restored: Result<(), FailureCode> = set_to_head_unchecked(git, repository, index, paths);
    if restored.is_err() && let Err(error) = std::fs::copy(kept.as_path(), index) {
        crate::diagnostic_log::warn("servicePrepare", format!("the commit's own index could not be put back after a refused step ({error})").as_str());
    }
    let _gone: std::io::Result<()> = std::fs::remove_file(kept.as_path());
    return restored;
}

/// The two steps of `set_to_head`,
/// each of which may refuse.
fn set_to_head_unchecked(git: &GitContext, repository: &Path, index: &Path, paths: &[Vec<u8>]) -> Result<(), FailureCode> {
    let specs: Vec<OsString> = specs_of(paths);
    let mut prune_words: Vec<&OsStr> = vec![
        OsStr::new("rm"),
        OsStr::new("--cached"),
        OsStr::new("-r"),
        OsStr::new("--force"),
        OsStr::new("--ignore-unmatch"),
        OsStr::new("--quiet"),
        OsStr::new("--"),
    ];
    prune_words.extend(specs.iter().map(OsString::as_os_str));
    let mut prune: GitRequest = GitRequest::new(repository, prune_words.as_slice());
    prune.index = Some(index.to_path_buf());
    match run_git(git, &prune) {
        Ok(output) if output.succeeded() => {}
        _ => return Err(FailureCode::Unavailable),
    }
    if crate::service_land_follow::reset_entries(git, repository, Some(index), "HEAD", paths).is_ok() {
        return Ok(());
    }
    // A `reset` this wrapper refuses is not the only way to the head: the entries are written where the head's tree has them.
    let head: BTreeMap<Vec<u8>, Entry> = match head_entries(git, repository, paths) {
        Ok(entries) => entries,
        Err(code) => return Err(code),
    };
    let mut listing: Vec<u8> = Vec::new();
    for path in paths {
        let (mode, object) = match head.get(path) {
            Some((mode, object)) => (mode.clone(), object.clone()),
            // The head holds nothing there, so the entry goes.
            None => (String::from("0"), String::from("0".repeat(40))),
        };
        listing.extend_from_slice(format!("{mode} {object} 0\t").as_bytes());
        listing.extend_from_slice(path.as_slice());
        listing.push(0);
    }
    let mut write: GitRequest = GitRequest::new(repository, &["update-index", "-z", "--index-info"]);
    write.index = Some(index.to_path_buf());
    write.input = Some(listing);
    return match run_git(git, &write) {
        Ok(answer) if answer.succeeded() => Ok(()),
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
fn head_entries(git: &GitContext, repository: &Path, paths: &[Vec<u8>]) -> Result<BTreeMap<Vec<u8>, Entry>, FailureCode> {
    let output = with_specs(git, repository, None, &["ls-tree", "-z", "--full-tree", "HEAD"], paths)?;
    let mut found: BTreeMap<Vec<u8>, Entry> = BTreeMap::new();
    for record in output.stdout.split(is_zero_byte) {
        let Some(([mode, _kind, object], path)) = front_fields(record) else { continue };
        found.insert(path.to_vec(), (String::from_utf8_lossy(mode).into_owned(), String::from_utf8_lossy(object).into_owned()));
    }
    return Ok(found);
}

/// What:
///  The one line a commit prints about the index entries it left out,
/// and nothing when it left none.
/// Why:
///   A commit whose content differs from the index must say why,
/// and name both ways on:
/// catching the index up with the
///  branch,
/// and taking the worktree's content for those paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// excludedNotice(paths)
/// ```
pub fn excluded_notice(paths: &[String]) -> Option<String> {
    if paths.is_empty() {
        return None;
    }
    return Some(format!(
        "monochromatic-git-policy-cli: a commit that landed did not finish updating this worktree's index, and these entries differ from the branch. They were left out of this commit: {}. To take the branch's content: git reset -- <paths>. To commit the worktree's content for them: git add -- <paths>, then git commit.\n",
        paths.join(", ")
    ));
}
