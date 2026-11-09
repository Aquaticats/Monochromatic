//! What:
//!  What a commit does about the index entries a landed batch still owes:
//! it records nothing about those paths,
//! and it says so.
//! Why:
//!   A batch's branch moves before its index is installed,
//! so the worktree's index can describe the branch before a landing.
//! A commit of that index would record the landed commit's own paths as their old content,
//! and a path the
//!  commit added as a staged deletion.
//! Which entries are the landing's own cannot be decided from what they hold:
//! a person can stage the
//!  batch's old content back,
//! another branch can hold the same bytes,
//! a staged child of an added path leaves no entry at that path,
//! an unmerged path is
//!  represented by its stages,
//! and a program can stage between any comparison and any write.
//! A repair that wrote the live index would
//!  therefore be able to discard staging that is the person's,
//! which is the worse outcome
//!  (`doc/planning/cli-git-commit-and-landing-route-findings-on-main.md`,
//! findings 2 to 6 of the round-2 review).
//!   So nothing here writes the live index.
//! The commit's own private index is set to the worktree's head for each path whose entry
//!  disagrees with that head,
//! so the commit records nothing about it,
//! and the notice names the paths and both ways on:
//! `git reset -- <paths>` catches the index up with the branch,
//! and `git commit -- <paths>` takes the worktree's content for them.
//! A path with unmerged stages is left to the person:
//! a merge in progress is their own act.
//! The obligation
//!  goes when the live index agrees with the head for every path it names,
//! or when the worktree has left the branch that landed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const left = await excludeWhatALandedCommitLeft(git, gitDir, liveIndex, commitIndex, explicitPaths);
//! ```

use crate::owed_index_update::{Owed, records_of, remove};
use crate::service_protocol::FailureCode;
use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStringExt as _;
use std::path::Path;

/// The exit status `git merge-base --is-ancestor` gives when the first commit is not below the second.
const NOT_AN_ANCESTOR: i32 = 1;

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
    return Ok(names_in(crate::service_land_follow::derived_names(git, owed.repository.as_path(), owed.old.as_str(), owed.new.as_str())?.as_slice()));
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

/// The offset of the tab that ends one entry's words,
/// or nothing when it holds none.
fn last_tab(record: &[u8]) -> Option<usize> {
    return record.iter().rposition(is_tab_byte);
}

/// Whether a byte is the NUL that ends one record of a `-z` listing.
fn is_zero_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// Whether a byte is the tab that separates an entry's words from its path.
fn is_tab_byte(byte: &u8) -> bool {
    return *byte == b'\t';
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
fn with_specs(git: &GitContext, repository: &Path, index: &Path, arguments: &[&str], names: &[Vec<u8>]) -> Result<crate::transaction_git::GitOutput, FailureCode> {
    let specs: Vec<OsString> = specs_of(names);
    let mut words: Vec<&OsStr> = arguments.iter().map(OsStr::new).collect();
    words.push(OsStr::new("--"));
    words.extend(specs.iter().map(OsString::as_os_str));
    let mut request: GitRequest = GitRequest::new(repository, words.as_slice());
    request.index = Some(index.to_path_buf());
    return match run_git(git, &request) {
        Ok(output) if output.succeeded() => Ok(output),
        _ => Err(FailureCode::Unavailable),
    };
}

/// What:
///  The paths of the record whose entry in the live index differs from the worktree's head.
/// Why:
///   Those are the entries a commit of that index would record against the head,
/// and the commit that
///  follows records nothing about them:
/// what they hold may be the unfinished landing's or a program's,
///  and only the person can say which.
/// A deletion,
/// an addition,
/// a change and a staged child all appear in
///  this listing;
/// an unmerged path does too,
/// and is left out here because a merge in progress is the person's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// differingFromHead(owed, liveIndex) // `git diff --cached --name-only -z HEAD -- <specs>`
/// ```
fn differing_from_head(git: &GitContext, owed: &Owed, live_index: &Path, names: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, FailureCode> {
    if names.is_empty() {
        return Ok(Vec::new());
    }
    let output: crate::transaction_git::GitOutput =
        with_specs(git, owed.repository.as_path(), live_index, &["diff", "--cached", "--name-only", "-z", "HEAD"], names)?;
    let differing: Vec<Vec<u8>> = names_in(output.stdout.as_slice());
    let unmerged: Vec<Vec<u8>> = unmerged_of(git, owed, live_index, names)?;
    let mut kept: Vec<Vec<u8>> = Vec::with_capacity(differing.len());
    for path in differing {
        if !unmerged.contains(&path) {
            kept.push(path);
        }
    }
    return Ok(kept);
}

/// The paths that hold unmerged entries in the live index,
/// which are a merge's own and never a landing's leftover.
fn unmerged_of(git: &GitContext, owed: &Owed, live_index: &Path, names: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, FailureCode> {
    let output: crate::transaction_git::GitOutput = with_specs(git, owed.repository.as_path(), live_index, &["ls-files", "-u", "-z"], names)?;
    let mut unmerged: Vec<Vec<u8>> = Vec::new();
    for record in output.stdout.split(is_zero_byte) {
        // Each record is `<mode> SP <object> SP <stage> TAB <path>`.
        let Some(tab) = last_tab(record) else { continue };
        let name: &[u8] = &record[tab + 1..];
        if !name.is_empty() && !unmerged.contains(&name.to_vec()) {
            unmerged.push(name.to_vec());
        }
    }
    return Ok(unmerged);
}

/// What a record's applicability came to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Applies {
    /// The worktree names the reference the landing recorded.
    Yes,
    /// The worktree names another reference:
    /// its index describes that one,
    /// and nothing is owed here.
    No,
    /// A question could not be answered,
    /// so nothing is decided and the record stays.
    Unknown,
}

/// What:
///  The reference the worktree names now,
/// `HEAD` when its head is detached.
/// Why:
///   An index belongs to the reference the worktree has checked out.
/// A detached head is a different reference from any
///  branch,
/// so a record of a branch applies only while that branch is checked out,
/// and a record made on a detached head only
///  while the head stays detached.
/// Nothing else is asked:
/// an amend
///  rewrites the ancestry of the same branch and takes no index obligation away,
/// so ancestry is no reason to forget
///  a record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// referenceNow(owed) // 'refs/heads/main', or 'HEAD' when detached
/// ```
fn reference_now(git: &GitContext, owed: &Owed) -> Option<String> {
    let request: GitRequest = GitRequest::new(owed.repository.as_path(), &["symbolic-ref", "--quiet", "HEAD"]);
    return match run_git(git, &request) {
        Ok(output) if output.succeeded() => Some(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string()),
        // Exit status 1 is a detached head: a reference no branch names.
        Ok(output) if output.code == Some(NOT_AN_ANCESTOR) => Some(String::from("HEAD")),
        _ => None,
    };
}

/// Whether the record's reference is the one the worktree names.
fn applies(git: &GitContext, owed: &Owed) -> Applies {
    return match reference_now(git, owed) {
        Some(named) if named == owed.reference => Applies::Yes,
        Some(_) => Applies::No,
        None => Applies::Unknown,
    };
}

/// What:
///  Sets the commit's own index to the worktree's head for the paths the commit it is making would record against that
///  head,
///  and gives the paths it left out.
/// Why:
///   This is the whole of what a commit does about a landed batch's owed index update.
/// The live index is never
///  written:
/// nothing that may be a program's staging is replaced,
/// so a deliberate reversal,
/// a staged child,
/// a
///  conflict's stages and a stage made while the commit prepares all stand.
/// The comparison is made against the commit's own
///  index when it has one,
/// because that snapshot is what the commit records,
/// and the live index is asked in the same
///  breath whether anything is still owed:
/// a program that catches the live index up between the copy and this step
///  removes the record and leaves the snapshot's stale entry,
/// so the snapshot is what the commit leaves out.
/// The paths are
///  set to the head rather than to the landed commit,
/// so the commit records nothing about them whatever the branch has
///  done since.
/// An explicit-path commit names its own selection,
/// which is the person's statement,
/// so nothing is taken
///  from it and the notice alone is given.
/// A record whose reference this worktree no longer names is forgotten,
/// and one
///  nothing is owed under any more is settled and forgotten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// excludeWhatALandedCommitLeft(git, gitDir, liveIndex, commitIndex) // the paths this commit leaves out
/// ```
pub fn exclude_what_a_landed_commit_left(git: &GitContext, git_dir: &Path, live_index: &Path, commit_index: Option<&Path>) -> Vec<String> {
    let mut left: Vec<String> = Vec::new();
    for owed in records_of(git_dir) {
        // A batch of another index file (`GIT_INDEX_FILE`) is not this commit's to settle.
        if owed.index != live_index {
            continue;
        }
        match applies(git, &owed) {
            Applies::Yes => {}
            Applies::No => {
                crate::service_log!("the record of the index update batch {} owes in {} is forgotten: the worktree names another reference", owed.seq, live_index.display());
                remove(git_dir, owed.seq);
                continue;
            }
            Applies::Unknown => {
                crate::diagnostic_log::warn(
                    "servicePrepare",
                    format!("the reference of the index update batch {} owes in {} cannot be read, so this commit leaves the record and its paths as they are", owed.seq, live_index.display()).as_str(),
                );
                continue;
            }
        }
        let names: Vec<Vec<u8>> = match names_of(git, &owed) {
            Ok(names) => names,
            Err(code) => {
                crate::diagnostic_log::warn("servicePrepare", format!("the paths of the index update batch {} owes cannot be read ({code:?}), so the record stays", owed.seq).as_str());
                continue;
            }
        };
        // The commit records its own snapshot; the live index says whether anything is still owed.
        let snapshot: &Path = commit_index.unwrap_or(live_index);
        let snapshot_differing: Vec<Vec<u8>> = match differing_from_head(git, &owed, snapshot, names.as_slice()) {
            Ok(differing) => differing,
            Err(code) => {
                crate::diagnostic_log::warn(
                    "servicePrepare",
                    format!("the entries the index update of batch {} owes cannot be compared ({code:?}), so this commit leaves them as they are and the record stays", owed.seq).as_str(),
                );
                continue;
            }
        };
        let live_differing: Vec<Vec<u8>> = match commit_index {
            None => snapshot_differing.clone(),
            Some(_) => match differing_from_head(git, &owed, live_index, names.as_slice()) {
                Ok(differing) => differing,
                Err(code) => {
                    crate::diagnostic_log::warn(
                        "servicePrepare",
                        format!("the entries the index update of batch {} owes cannot be compared with the live index ({code:?}), so the record stays", owed.seq).as_str(),
                    );
                    continue;
                }
            },
        };
        if snapshot_differing.is_empty() && live_differing.is_empty() {
            // Nothing is owed any more: the index holds the head's entry at every path the batch named.
            remove(git_dir, owed.seq);
            continue;
        }
        match commit_index {
            Some(index) => match set_to_head(git, &owed, index, snapshot_differing.as_slice()) {
                Ok(()) => {
                    for path in &snapshot_differing {
                        left.push(crate::path_bytes::path_message_text(path.as_slice()));
                    }
                }
                Err(code) => crate::diagnostic_log::warn(
                    "servicePrepare",
                    format!("the commit's own index could not be set to the head for the entries batch {} owes ({code:?}), so this commit records them as its index holds them", owed.seq).as_str(),
                ),
            },
            None => {
                for path in &live_differing {
                    left.push(crate::path_bytes::path_message_text(path.as_slice()));
                }
            }
        }
        if live_differing.is_empty() {
            remove(git_dir, owed.seq);
        }
    }
    return left;
}

/// What:
///  Sets an index to the worktree's head at `paths`,
/// removing whatever it held there and below them first.
/// Why:
///   A path the landing added can stand beside entries a program staged below it
/// (`p` and `p/child`),
/// and the head holds one of
///  the two:
/// an index that holds both cannot be written to a tree,
/// so the commit's own index takes the head's
///  shape for the path.
/// The removal is forced,
/// because the entry may hold neither the head's content nor the worktree's:
///  a landing whose index update failed leaves exactly that,
/// and a refusal would record the old content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await setToHead(owed, index, paths) // git rm --cached -r --force; git reset HEAD
/// ```
fn set_to_head(git: &GitContext, owed: &Owed, index: &Path, paths: &[Vec<u8>]) -> Result<(), FailureCode> {
    if paths.is_empty() {
        return Ok(());
    }
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
    let mut prune: GitRequest = GitRequest::new(owed.repository.as_path(), prune_words.as_slice());
    prune.index = Some(index.to_path_buf());
    match run_git(git, &prune) {
        Ok(output) if output.succeeded() => {}
        _ => return Err(FailureCode::Unavailable),
    }
    return crate::service_land_follow::reset_entries(git, owed.repository.as_path(), Some(index), "HEAD", paths);
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
        "monochromatic-git-policy-cli: a commit that landed did not finish updating this worktree's index, and these entries differ from the branch. They were left out of this commit: {}. To take the branch's content: git reset -- <paths>. To commit the worktree's content for them: git commit -- <paths>.\n",
        paths.join(", ")
    ));
}
