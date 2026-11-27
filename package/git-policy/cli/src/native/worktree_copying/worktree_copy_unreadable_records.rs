//! What:
//!  The records of an interrupted ignored-state copy that cannot be read:
//!  the command that finds one moves it aside into a folder of its own,
//!  says so in one line,
//!  and goes on.
//! Why:
//!  A copy publishes every record whole by one rename,
//!  so a record that does not read is what a crash leaves
//!  when the filesystem publishes a name before its bytes
//!  (`SPEC.md`,
//!  "The records an add writes for itself").
//! Every later command reads these records first,
//! and on this machine thirty or more agents work in linked worktrees at once,
//  so one unreadable record that stops a command stops every command of that repository,
//  in every worktree,
//  until a person removes the file.
//! A record is never deleted here:
//! it is the only trace of what the interrupted copy was doing.
//! What it named is found again from the records beside it where it can be,
//  and the words of the notice say what may still stand on disk where it cannot.
//! The records that name a worktree rather than the copy's private stage,
//! and what becomes of one whose copy gave up or whose claim is gone,
//! are in `worktree_copy_records_that_name_a_worktree.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const notices = await settleUnreadableRecords({ commonDir, verdicts });
//! ```

use crate::json_record::quote;
use crate::worktree_copy_error::{
    WorktreeCopyError,
    raw_failure,
};
use crate::worktree_copy_journal::{
    JournalFile,
    Role,
    assert_private_path,
    journal_owner_is_dead,
    journal_root,
    read_listed_journals,
};
use crate::worktree_copy_journal_encoding_and_validation::owner_token_of;
use crate::worktree_copy_owner::token_owner_is_dead;
use crate::worktree_copy_records_that_name_a_worktree::{
    destination_record_of,
    settle_unreadable_unfilled_records,
};
use crate::worktree_copy_selection::STAGE_PREFIX;
use crate::worktree_copy_stage_binding::{
    BindingFile,
    STAGE_BINDING_NAME,
    stage_binding_reading,
};
use crate::worktree_copy_stage_claim::{
    claim_names,
    rewrite_claim,
};
use crate::worktree_copy_stage_claim_owner_verdicts::StageVerdicts;
use crate::worktree_copy_stage_claim_record::{
    ClaimFile,
    claim_file_reading,
};
use crate::worktree_copy_stage_made_object::read_made;
use crate::worktree_copy_unreadable_record_aside::move_aside;
use crate::worktree_copy_unreadable_record_words::{
    settled_notice,
    what_a_lost_claim_leaves,
    what_a_lost_journal_leaves,
};
use std::ffi::{
    OsStr,
    OsString,
};
use std::path::{
    Path,
    PathBuf,
};

/// Where a record this release cannot read is kept,
/// and the readers of that folder,
/// re-exported so a caller that only needs the folder reaches it here.
pub use crate::worktree_copy_unreadable_record_aside::{
    moved_record_path,
    unreadable_record_names,
    unreadable_records_stand,
};

/// The folder under the journal root that keeps those records,
/// whose name the journal's own names file defines.
pub use crate::worktree_copy_journal_names_and_shapes::UNREADABLE_FOLDER;

/// The settling of the records that name a worktree rather than a private stage,
/// re-exported so a caller that only needs it reaches it here.
pub use crate::worktree_copy_records_that_name_a_worktree::settle_records_without_a_claim;

/// What:
///  The stage container a claim that cannot be read still names through the records beside it.
/// Why:
///   A claim is the record that names the stage folder,
/// and two records beside it name the same folder:
/// the record of the object that folder's creation made holds its path,
/// and the destination record names the worktree whose parent holds it.
/// Either one is enough to settle the folder and to let an add's records be settled,
/// and a claim with neither leaves a folder nothing finds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const container = recoveredContainer(claim); // undefined when nothing beside it names the folder
/// ```
fn recovered_container(claim: &Path) -> Option<PathBuf> {
    let identifier: String = claim.file_stem()?.to_string_lossy().into_owned();
    let expected: String = format!("{STAGE_PREFIX}{identifier}");
    // The record of the object the stage's creation made holds the folder's own path.
    if let Some(made) = read_made(claim)
        && made.container.file_name() == Some(OsStr::new(expected.as_str()))
    {
        return Some(made.container);
    }
    // The destination record names the worktree the add registered, and the folder sits beside it.
    let destination: PathBuf = destination_record_of(claim)?;
    return Some(destination.parent()?.join(expected));
}

/// What:
///  Settles every record of this repository that cannot be read,
/// moving each aside with one line,
/// after judging the writer that left it.
/// Why:
///   The command that meets an unreadable record is the only one that can end the state,
/// and it may only act on records whose writer is gone:
/// a live owner's record is its running add or copy.
/// Every move is one rename,
/// so of several commands that find one record exactly one settles it and the others find it gone.
/// The pass changes no worktree:
/// it settles the wrapper's own records and names what it cannot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = await settleUnreadableRecords({ commonDir, verdicts });
/// ```
pub fn settle_unreadable_records(
    common_dir: &Path,
    verdicts: &mut StageVerdicts,
) -> Result<Vec<String>, WorktreeCopyError> {
    let mut notices: Vec<String> = Vec::new();
    settle_unreadable_journals(common_dir, &mut notices)?;
    settle_unreadable_claims(common_dir, verdicts, &mut notices)?;
    settle_unreadable_unfilled_records(common_dir, &mut notices)?;
    return Ok(notices);
}

/// What:
///  Moves every journal that cannot be read aside,
/// where its writer is gone,
/// and settles the registration record of its stage first.
/// Why:
///   The journal's name carries its owner,
/// so the writer is judged before anything moves;
/// and a stage binding this release cannot read is settled while the journal still names the stage,
/// so the copy an earlier command would have finished proves its destination from that destination alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const listed of await readListedJournals(commonDir)) if (listed.kind === 'unreadable' && tokenOwnerIsDead(ownerOf(listed.path), listed.path)) { await settleBinding(listed); await moveAside(listed); }
/// ```
fn settle_unreadable_journals(
    common_dir: &Path,
    notices: &mut Vec<String>,
) -> Result<(), WorktreeCopyError> {
    for listed in read_listed_journals(common_dir)? {
        match listed {
            JournalFile::Pending(pending) => {
                // A copy whose writer is gone is one a later command finishes; the registration it was made for is
                // settled first, so that finishing it never stops for a record nothing can read.
                if journal_owner_is_dead(&pending)?
                    && let Some(notice) = settle_unreadable_binding(
                        common_dir,
                        pending.journal.stage_container.as_path(),
                    )?
                {
                    notices.push(notice);
                }
            }
            JournalFile::Unreadable {
                path,
                reason,
            } => {
                // A journal with no owner in its name has none to wait for, which is what an interrupted add leaves.
                if !token_owner_is_dead(owner_token_of(path.as_path()).as_deref(), path.as_path())?
                {
                    continue;
                }
                let Some(name) = path.file_name() else {
                    continue;
                };
                let Some(moved) = move_aside(common_dir, path.as_path(), name)? else {
                    continue;
                };
                notices.push(settled_notice(
                    "worktree-copy journal",
                    path.as_path(),
                    moved.as_path(),
                    reason.as_str(),
                    what_a_lost_journal_leaves().as_str(),
                ));
            }
        }
    }
    return Ok(());
}

/// What:
///  Moves every stage claim that cannot be read aside,
/// writes it again where its stage folder can be recovered,
/// and settles the registration record inside that folder.
/// Why:
///   A claim names the stage folder an add may still owe a copy from,
/// so the folder is found again from the records beside it where they name it;
/// writing the claim again leaves the sweeps that settle the add and make its copy with something to read,
/// and a claim that names no owner has none to wait for.
/// A claim whose folder nothing names is moved aside with the words that say what may stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of await claimNames(root)) { const read = await claimFileReading(join(root, name)); if (read.kind !== 'unreadable') continue; ... }
/// ```
fn settle_unreadable_claims(
    common_dir: &Path,
    verdicts: &mut StageVerdicts,
    notices: &mut Vec<String>,
) -> Result<(), WorktreeCopyError> {
    let root: PathBuf = journal_root(common_dir);
    for name in claim_names(root.as_path())? {
        let claim: PathBuf = root.join(name.as_str());
        let Some(ClaimFile::Unreadable {
            reason,
        }) = claim_file_reading_tolerant(claim.as_path())?
        else {
            continue;
        };
        let container: Option<PathBuf> = recovered_container(claim.as_path());
        // The registration record of the folder is the copy's own, and its writer is the claim's: settling it here keeps
        // the copy the sweeps make from stopping at a record nothing can read.
        if let Some(found) = container.as_ref()
            && let Some(notice) = settle_unreadable_binding(common_dir, found.as_path())?
        {
            notices.push(notice);
        }
        let Some(moved) = move_aside(common_dir, claim.as_path(), OsStr::new(name.as_str()))?
        else {
            continue;
        };
        match container {
            Some(found) => {
                // `match` on the write: a claim written again leaves the sweeps something to settle; one that could not be
                // written is named with what stands instead.
                match rewrite_claim(claim.as_path(), found.as_path()) {
                    Ok(()) => {
                        // The claim names no owner, which is what an interrupted add is, so this pass settles it too.
                        verdicts.record_dead_claim(name.as_str());
                        notices.push(settled_notice(
                            "worktree-copy stage claim",
                            claim.as_path(),
                            moved.as_path(),
                            reason.as_str(),
                            format!(
                                "It was written again beside the claim name from the record of the object its stage's creation made or from the destination record beside it, naming the stage folder {}, so the sweeps that settle that folder and make the ignored-state copy the add still owes find it.",
                                quote(found.to_string_lossy().as_ref())
                            )
                            .as_str(),
                        ));
                    }
                    Err(refusal) => notices.push(settled_notice(
                        "worktree-copy stage claim",
                        claim.as_path(),
                        moved.as_path(),
                        reason.as_str(),
                        format!(
                            "It could not be written again ({}), so the stage folder {} stays and nothing of the wrapper's settles it.",
                            refusal.0,
                            quote(found.to_string_lossy().as_ref())
                        )
                        .as_str(),
                    )),
                }
            }
            None => notices.push(settled_notice(
                "worktree-copy stage claim",
                claim.as_path(),
                moved.as_path(),
                reason.as_str(),
                what_a_lost_claim_leaves().as_str(),
            )),
        }
    }
    return Ok(());
}

/// What:
///  `claim_file_reading` with a claim that another command settled in between read as gone.
/// Why:
///   The pass acts on what a listing found,
/// and a claim taken by the add that owns it is nobody's failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return await claimFileReading(claim); } catch (error) { if (error.code === 'ENOENT') return undefined; throw error; }
/// ```
fn claim_file_reading_tolerant(claim: &Path) -> Result<Option<ClaimFile>, WorktreeCopyError> {
    // `match` on the metadata first: the strict reader refuses a path that is not a private file, and a vanish is not that.
    return match std::fs::symlink_metadata(claim) {
        Ok(_) => claim_file_reading(claim),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(raw_failure(&error)),
    };
}

/// What:
///  Settles the registration record inside one private stage container when it cannot be read.
/// Why:
///   The record says which registration the copy was made for,
/// and a copy that cannot read it proves its destination from that destination's own registration,
/// which is what a stage an earlier release made does.
/// The record is moved into the journal root's folder for records that cannot be read,
/// under a name that keeps the stage container it belonged to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notice = await settleUnreadableBinding(commonDir, container);
/// ```
fn settle_unreadable_binding(
    common_dir: &Path,
    container: &Path,
) -> Result<Option<String>, WorktreeCopyError> {
    // `match` on the container: a stage another command removed holds no record of its own to settle.
    match std::fs::symlink_metadata(container) {
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_not_a_directory) => return Ok(None),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(raw_failure(&error)),
    }
    assert_private_path(container, Role::PrivateStage)?;
    let Some(BindingFile::Unreadable {
        reason,
    }) = stage_binding_reading(container)?
    else {
        return Ok(None);
    };
    let record: PathBuf = container.join(STAGE_BINDING_NAME);
    let Some(container_name) = container.file_name() else {
        return Ok(None);
    };
    let mut moved_name: OsString = container_name.to_owned();
    moved_name.push(format!(".{STAGE_BINDING_NAME}"));
    let Some(moved) = move_aside(common_dir, record.as_path(), moved_name.as_os_str())? else {
        return Ok(None);
    };
    return Ok(Some(settled_notice(
        "registration recorded for a private stage",
        record.as_path(),
        moved.as_path(),
        reason.as_str(),
        "The copy that stage belongs to proves its destination from that destination's own registration, as a stage an earlier release made does, and nothing in any worktree was changed by this.",
    )));
}

/// Unreadable-record controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_unreadable_records_tests.rs"]
mod tests;
