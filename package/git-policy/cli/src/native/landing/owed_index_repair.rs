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
//! The question is not who owns an entry but what the commit is about to record:
//! the record names a path
//!  with the entry the landing replaced and the entry it landed,
//! and a path is left out exactly while the commit's own
//!  index still holds the entry the landing replaced.
//! That asks about content,
//! so an amend,
//! a `reset --hard`,
//! an attach from a
//!  detached head and an alternate index need no rule of their own.
//! The price is that a person's deliberate restoration of that same
//!  content is left out too,
//! with the notice that names the path and both ways on,
//! and their staging stays in the live index.
//! A path whose live entry moved after the commit copied the index is left out as well:
//! that snapshot is older
//!  than the index,
//! so what the commit would record there is not the person's current intent.
//! Nothing here writes the live index or removes a record.
//! What it reads is `owed_index_entries.rs`.
//!
//! The rule this file enforces:
//!   It may change only a path a record names,
//! and only while every fact it has about that path agrees that the index still holds the content the
//! landing replaced:
//! the commit's own index holds exactly the entry the landing's
//!  old tree held there,
//! the worktree does not match that entry (so the staging is not the person's own doing),
//! and the commit did not select the path by name.
//! On no evidence it changes nothing,
//! leaves the landed content where it belongs,
//! and says which path it left out;
//! when it cannot even make the change it decided on,
//! it stops the commit instead of letting stale content be recorded.
//! A path whose live entry moved after the commit copied the index is left out too,
//! on the same terms:
//! the copy,
//! not the staged index,
//! is what the live index is compared with.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const left = await excludeWhatALandedCommitLeft(git, gitDir, liveIndex, commitIndex);
//! ```

use super::owed_index_repair_the_commit_index_writes::{
    set_to_head_paths,
    set_to_head_with,
};
use super::owed_index_repair_the_entries_it_reads::{
    moved_since,
    worktree_object,
};
use crate::owed_index_entries::{
    Entry,
    head_entries,
    index_entries,
    is_unmerged,
    is_zero_byte,
    moved_since_listing,
    names_in,
    tree_entries,
};
use crate::service_protocol::FailureCode;
use crate::the_record_of_an_owed_index_update::Owed;
use crate::transaction_git::{
    GitContext,
    GitRequest,
    run_git,
};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// The paths one record's update covers,
/// derived from the two branch values when the batch names none.
fn names_of(
    git: &GitContext,
    repository: &Path,
    owed: &Owed,
) -> Result<Vec<Vec<u8>>, FailureCode> {
    if !owed.derived {
        return Ok(owed.paths.clone());
    }
    let names: Vec<u8> = crate::service_land_follow::derived_names(
        git,
        repository,
        owed.old.as_str(),
        owed.new.as_str(),
    )?;
    return Ok(names_in(names.as_slice()));
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
    repository: &Path,
    git_dir: &Path,
    live_index: &Path,
    commit_index: Option<&Path>,
    captured_index: Option<&Path>,
    captured: &crate::service_land_start::IndexWitness,
) -> Result<Vec<String>, String> {
    let mut note: Option<String> = None;
    let left: Vec<String> = exclude_with_note(
        git,
        repository,
        git_dir,
        live_index,
        commit_index,
        captured_index,
        captured,
        &mut note,
    )?;
    return Ok(left);
}

/// What:
///  `exclude_what_a_landed_commit_left`,
/// leaving the sentence a commit must print when an obligation could not be read,
/// and nothing otherwise.
/// Why:
///   The notice of a repair names the paths it left out;
/// when the reason is a record that cannot be read,
/// the person
///  must be told that too,
/// and the reason is only known here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// excludeWithNote(...) // [paths, note]
/// ```
pub fn exclude_with_note(
    git: &GitContext,
    repository: &Path,
    git_dir: &Path,
    live_index: &Path,
    commit_index: Option<&Path>,
    captured_index: Option<&Path>,
    captured: &crate::service_land_start::IndexWitness,
    note: &mut Option<String>,
) -> Result<Vec<String>, String> {
    let mut left: Vec<String> = Vec::new();
    let (owed_records, unreadable): (Vec<Owed>, usize) =
        crate::the_record_of_an_owed_index_update::records_and_unreadable(git_dir);
    // An obligation that cannot be read names no path, so nothing can be repaired for it: what is known is that a landed file could
    //  be recorded as deleted. The commit goes through with such a deletion left out, and says why.
    if unreadable > 0 {
        let at_risk: Vec<Vec<u8>> = match commit_index {
            None => Vec::new(),
            Some(index) => deletions_a_landing_could_own(git, repository, index)?,
        };
        if !at_risk.is_empty() {
            let index = commit_index.expect("the paths were asked of the commit's own index");
            if let Err(code) = set_to_head_paths(git, repository, index, at_risk.as_slice()) {
                return Err(format!(
                    "the entries an obligation that cannot be read may owe cannot be set to the head ({code:?})"
                ));
            }
        }
        for path in &at_risk {
            left.push(crate::path_bytes::path_message_text(path.as_slice()));
        }
        *note = Some(unreadable_notice(unreadable, left.as_slice()));
    }
    // Nothing at all is done unless this worktree has an owed record: a commit that owes nothing is never compared, so no staging of
    //  its own can be mistaken for a move.
    if owed_records.is_empty() {
        return Ok(left);
    }
    // What the commit copied is not what the index holds any more: the paths whose live entry moved since the copy are left out,
    //  because what the snapshot would record there is not the person's current intent. The copy, not the commit's staged index, is
    //  what the live index is compared with: the commit's own staging (`--all`) is the person's and must never be reported as moved.
    if let (Some(index), Some(copy), Ok(now)) = (
        commit_index,
        captured_index,
        crate::service_land_start::witness_of(live_index),
    ) && now != *captured
    {
        // Only a path a record names may be changed: a move another command made to a path no record names is that command's own
        //  staging, and the commit keeps what it selected for it.
        let mut named: Vec<Vec<u8>> = Vec::new();
        for owed in &owed_records {
            // A record of another index file (`GIT_INDEX_FILE`) names no path of this commit's index.
            if owed.index != live_index {
                continue;
            }
            if let Ok(names) = names_of(git, repository, owed) {
                named.extend(names);
            }
        }
        let moved: Vec<Vec<u8>> = match moved_since(git, repository, live_index, copy) {
            Ok(moved) => moved,
            Err(_unwritable) => match moved_since_listing(git, repository, live_index, copy) {
                Ok(moved) => moved,
                Err(code) => {
                    return Err(format!(
                        "the entries the index moved since this commit copied it cannot be read ({code:?})"
                    ));
                }
            },
        };
        let mut owned: Vec<Vec<u8>> = Vec::with_capacity(moved.len());
        for path in moved {
            if named.contains(&path) {
                owned.push(path);
            }
        }
        if !owned.is_empty() {
            if let Err(code) = set_to_head_paths(git, repository, index, owned.as_slice()) {
                return Err(format!(
                    "the entries the index moved since this commit copied it cannot be set to the head ({code:?})"
                ));
            }
            for path in &owned {
                left.push(crate::path_bytes::path_message_text(path.as_slice()));
            }
        }
    }
    for owed in owed_records {
        // The record names the index file whose update it owes; a commit of another one is another question.
        if owed.index != live_index {
            continue;
        }
        let snapshot: &Path = commit_index.unwrap_or(live_index);
        let names: Vec<Vec<u8>> = match names_of(git, repository, &owed) {
            Ok(names) => names,
            Err(code) => {
                return Err(format!(
                    "the paths of the index update batch {} owes cannot be read ({code:?})",
                    owed.seq
                ));
            }
        };
        let replaced: BTreeMap<Vec<u8>, Entry> =
            match tree_entries(git, repository, owed.old.as_str(), names.as_slice()) {
                Ok(entries) => entries,
                Err(code) => {
                    return Err(format!(
                        "the tree the landing of batch {} replaced cannot be read ({code:?})",
                        owed.seq
                    ));
                }
            };
        let held: BTreeMap<Vec<u8>, Entry> = match index_entries(
            git,
            repository,
            snapshot,
            names.as_slice(),
        ) {
            Ok(entries) => entries,
            Err(code) => {
                return Err(format!(
                    "the commit's own index cannot be read for the paths of batch {} ({code:?})",
                    owed.seq
                ));
            }
        };
        // What the head holds there: a path whose copy already holds it is not left out, and is not said, because the commit
        //  records nothing about it either way.
        let head: BTreeMap<Vec<u8>, Entry> = match head_entries(git, repository, names.as_slice()) {
            Ok(entries) => entries,
            Err(code) => {
                return Err(format!(
                    "the worktree's head cannot be read for the paths of batch {} ({code:?})",
                    owed.seq
                ));
            }
        };
        // A path the person selected is never changed by a guess.
        if commit_index.is_none() {
            for name in &names {
                if held.get(name) == replaced.get(name) && !is_unmerged(held.get(name)) {
                    left.push(crate::path_bytes::path_message_text(name.as_slice()));
                }
            }
            continue;
        }
        // The worktree is asked whether the staging is the person's own: a worktree that holds what the index holds means they
        //  staged what they have, so it is their statement and never the landing's leftover.
        let mut excluded: Vec<Vec<u8>> = Vec::new();
        for name in &names {
            let entry: Option<&Entry> = held.get(name);
            if is_unmerged(entry) {
                // A merge in progress is the person's own act.
                continue;
            }
            if entry != replaced.get(name) {
                continue;
            }
            let theirs: bool = match (entry, worktree_object(git, repository, name.as_slice())) {
                (Some((mode, object)), Ok(Some((worktree_mode, mine)))) => {
                    *object == mine && *mode == worktree_mode
                }
                // Nothing is proof of the person's staging here: a file that is not there with no entry in the copy is what a
                //  landing whose index update failed leaves, so the landed entry stays and the path is said. A deletion the person
                //  staged in that window waits for the next commit.
                _ => false,
            };
            if !theirs && head.get(name) != entry {
                excluded.push(name.clone());
            }
        }
        if excluded.is_empty() {
            continue;
        }
        if let Err(code) =
            set_to_head_with(git, repository, snapshot, excluded.as_slice(), snapshot)
        {
            return Err(format!(
                "the commit's own index cannot be set to the head for the entries batch {} owes ({code:?})",
                owed.seq
            ));
        }
        for path in &excluded {
            left.push(crate::path_bytes::path_message_text(path.as_slice()));
        }
    }
    return Ok(left);
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

/// What:
///  The sentence a commit prints when an obligation of this worktree cannot be read.
/// Why:
///   The paths it left out are those a landing could have made it delete;
/// the person is told why they were left out,
///  and
/// that the record is still there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unreadableNotice(count, paths)
/// ```
pub fn unreadable_notice(
    count: usize,
    paths: &[String],
) -> String {
    return format!(
        "monochromatic-git-policy-cli: {count} record(s) of what this worktree's index owes cannot be read, so the deletion this commit would have recorded of {} was left out, and the record is kept.",
        paths.join(", ")
    );
}

/// What:
///  The paths this commit would record as deleted while the head holds an entry and the worktree holds the file:
/// what a landing whose index update never ran leaves.
/// Why:
///   An obligation that cannot be read names no path,
/// and the only state it can have left is a copy of the index older than the landing.
/// A deletion the person staged is a file the worktree does not hold,
/// so it is not this set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// deletionsALandingCouldOwn(index) // `diff-index --cached --diff-filter=D HEAD`, kept where the worktree has the file
/// ```
fn deletions_a_landing_could_own(
    git: &GitContext,
    repository: &Path,
    index: &Path,
) -> Result<Vec<Vec<u8>>, String> {
    let mut request: GitRequest = GitRequest::new(
        repository,
        &[
            "diff-index",
            "--cached",
            "--name-only",
            "--diff-filter=D",
            "-z",
            "HEAD",
        ],
    );
    request.index = Some(index.to_path_buf());
    let output = match run_git(git, &request) {
        Ok(answer) if answer.succeeded() => answer,
        _ => {
            return Err(String::from(
                "the deletions this commit would record cannot be read",
            ));
        }
    };
    let mut at_risk: Vec<Vec<u8>> = Vec::new();
    for name in output.stdout.split(is_zero_byte) {
        if name.is_empty() {
            continue;
        }
        let held: PathBuf = repository.join(
            <OsString as std::os::unix::ffi::OsStringExt>::from_vec(name.to_vec()).as_os_str(),
        );
        if std::fs::symlink_metadata(held.as_path()).is_ok() {
            at_risk.push(name.to_vec());
        }
    }
    return Ok(at_risk);
}
