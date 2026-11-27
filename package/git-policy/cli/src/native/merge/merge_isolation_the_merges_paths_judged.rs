//! What:
//!  The judgement of each path a merge changes:
//!  how the worktree stands at it against the two sides of the change,
//!  and a directory the branch's own files explain read as the side the merge replaces.
//! Why:
//!   The recovery of an interrupted merge replaces a path only when the worktree still holds the old side there,
//!  and a merge that replaces a directory with a file must not take a person's files with it;
//!  one judgement answers both for every path of the merge,
//!  so the settle of a moved branch and the recovery of a killed one judge a path the same way.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const judged = await judgeChanges({ place, root, changes }); // standing(...) per path, and directory for an explained one
//! ```

/// The jobs of the blocking pool this judgement makes,
/// so a disk that makes a look or a walk wait holds no worker of the runtime.
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
/// Whether a directory standing where the merge adds a path holds only what the merge deletes.
use crate::merge_isolation_a_directory_the_branch_explains::the_directory_holds_only_what_the_merge_deletes;
/// The paths the merge changes and what the worktree holds at each.
use crate::merge_isolation_changes::{
    Change,
    FileIdentity,
    WorktreeState,
    file_identity,
    worktree_path,
    worktree_state_awaiting,
};
/// Where recovery runs Git.
use crate::merge_isolation_recovery_place::RecoveryPlace;
/// The fail-closed recovery failure.
use crate::recovery_error::RecoveryError;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};

/// What the worktree holds at a changed path,
/// against the two sides of the merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Standing = 'old' | 'new' | 'foreign';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Standing {
    /// The old content,
    /// or no file where the old side has none.
    Old,
    /// The new content,
    /// or no file where the new side has none.
    New,
    /// A directory that stands where the merge puts a file,
    /// and that holds only files of the branch,
    /// which the merge itself
    ///  deletes.
    /// Nothing of anyone's is there to set aside,
    /// and the merge's own deletions empty it.
    Directory,
    /// Anything else.
    Foreign,
}
/// What:
///  Compare a worktree state with the two sides of a change,
///  content and the mode Git records for it.
/// Why:
///   The old side wins a tie:
/// a change that does not alter the content is not evidence of a started merge.
/// A mode is read as part of the file,
/// because Git records it in the index entry:
/// a file whose bytes
///  are the branch's and whose executable bit is not has been changed by someone,
/// and replacing it would erase
///  that change
///  (a cross-vendor review of the ninth round,
/// its seventh finding).
/// A repository that records no
///  executable bit reads no mode difference at all,
/// as Git does there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function standing(change: Change, state: WorktreeState, recordsFileModes: boolean): Standing;
/// ```
pub fn standing(
    change: &Change,
    state: &WorktreeState,
    records_file_modes: bool,
) -> Standing {
    let old_matches: bool = match state {
        WorktreeState::Missing => !change.existed(),
        WorktreeState::Content {
            object,
            mode,
        } => {
            change.existed()
                && *object == change.old_oid
                && (!records_file_modes || *mode == change.old_mode)
        }
        WorktreeState::Other => false,
    };
    if old_matches {
        return Standing::Old;
    }
    let new_matches: bool = match state {
        WorktreeState::Missing => !change.exists_after(),
        WorktreeState::Content {
            object,
            mode,
        } => {
            change.exists_after()
                && *object == change.new_oid
                && (!records_file_modes || *mode == change.new_mode)
        }
        WorktreeState::Other => false,
    };
    if new_matches {
        return Standing::New;
    }
    return Standing::Foreign;
}
/// What:
///  The judgement of every changed path,
///  with a directory the branch's own files explain read as the old side.
/// Why:
///  `worktree_state` reads a directory as neither side of a change,
/// so a merge that replaces a directory with a
///  file was refused for good,
/// although the directory holds only the branch's files,
/// which the merge's own
///  deletions take
/// (a cross-vendor review of the second round):
/// a path whose old side has no entry of its own is
///  the merge's to replace when the branch stores files below it and the merge deletes them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judgeChanges({ place, root, changes }): Judged[]; // standing(...), then explainedDirectory(...) ? 'directory' : one
/// ```
pub async fn judge_changes_awaiting(
    place: &RecoveryPlace,
    root: &Path,
    changes: &[Change],
) -> Result<Vec<Judged>, RecoveryError> {
    // Whether a mode is part of the file here is one question for the whole judgement,
    // and it is the repository's answer, not this
    //  command's.
    let records_file_modes: bool =
        crate::merge_isolation_changes::the_repository_records_file_modes_awaiting(
            &place.git, root,
        )
        .await;
    // The identity of every path is looked up in one job of the blocking pool, and all of them before any content is read, so a
    //  file written in between does not match the judgement and is never replaced.
    let locations: Vec<PathBuf> = changes
        .iter()
        .map(|change| return worktree_path(root, change.path.as_slice()))
        .collect();
    let looked_up: Result<Vec<Option<FileIdentity>>, crate::blocking_job::BlockingJobError> =
        run_blocking_job_within_the_budget(move || {
            return locations
                .iter()
                .map(|location| return file_identity(location.as_path()))
                .collect();
        })
        .await;
    let identities: Vec<Option<FileIdentity>> = match looked_up {
        Ok(found) => found,
        // A look the pool never made leaves nothing proven about the worktree, so the command stops rather than judge the merge's
        //  paths on a guess and replace what it could not see.
        Err(refused) => {
            return Err(RecoveryError(format!(
                "the entries of this merge's paths could not be looked at: {refused}"
            )));
        }
    };
    let mut judgements: Vec<Judged> = Vec::with_capacity(changes.len());
    for (index, change) in changes.iter().enumerate() {
        let state: WorktreeState =
            worktree_state_awaiting(&place.git, root, change.path.as_slice())
                .await
                .map_err(RecoveryError)?;
        let content: Option<String> = match &state {
            WorktreeState::Content {
                object,
                ..
            } => Some(object.clone()),
            WorktreeState::Missing | WorktreeState::Other => None,
        };
        judgements.push(Judged {
            standing: standing(change, &state, records_file_modes),
            identity: identities[index],
            content,
        });
    }
    let candidates: Vec<usize> = (0..changes.len())
        .filter(|index| {
            let change: &Change = &changes[*index];
            return judgements[*index].standing == Standing::Foreign
                && !change.existed()
                && change.exists_after();
        })
        .collect();
    if !candidates.is_empty() {
        // Whether a directory stands where the merge adds a file, and whether it holds only the branch's own files, are a look
        //  and a walk of the worktree: one job of the blocking pool makes both, and one job covers every such path of the merge.
        let changes_for_the_job: Vec<Change> = changes.to_vec();
        let root_for_the_job: PathBuf = root.to_path_buf();
        let decided: Result<Vec<(usize, bool)>, crate::blocking_job::BlockingJobError> =
            run_blocking_job_within_the_budget(move || {
                let mut answers: Vec<(usize, bool)> = Vec::new();
                for index in candidates {
                    let change: &Change = &changes_for_the_job[index];
                    let location: PathBuf =
                        worktree_path(root_for_the_job.as_path(), change.path.as_slice());
                    let stands_as_a_directory: bool = std::fs::symlink_metadata(location.as_path())
                        .map(is_a_directory)
                        .unwrap_or(false);
                    let only_the_branchs: bool = stands_as_a_directory
                        && the_branchs_files_stand_below(change, changes_for_the_job.as_slice())
                        && the_directory_holds_only_what_the_merge_deletes(
                            change,
                            changes_for_the_job.as_slice(),
                            root_for_the_job.as_path(),
                        );
                    answers.push((index, only_the_branchs));
                }
                return answers;
            })
            .await;
        match decided {
            Ok(answers) => {
                for (index, holds_only_the_branchs) in answers {
                    if !holds_only_the_branchs {
                        continue;
                    }
                    judgements[index] = Judged {
                        standing: Standing::Directory,
                        identity: None,
                        content: None,
                    };
                }
            }
            // A walk the pool never made proves nothing about these directories, so each stays foreign and the command refuses it
            //  rather than replace a path nothing judged.
            Err(refused) => {
                crate::diagnostic_log::debug(
                    "judgeChanges",
                    format!("the directories standing where the merge adds a path could not be judged: {refused}").as_str(),
                );
            }
        }
    }
    return Ok(judgements);
}
/// Whether one file system entry is a directory.
fn is_a_directory(found: std::fs::Metadata) -> bool {
    return found.is_dir();
}
/// Whether one change deletes a file of the branch below `prefix`.
fn takes_a_file_below(
    other: &Change,
    prefix: &[u8],
) -> bool {
    return other.path.starts_with(prefix) && other.existed() && !other.exists_after();
}
/// What:
///  Whether the branch stores files below a path that the merge deletes them from,
///  which is what a directory standing
///  there means.
/// Why:
///   A directory where the merge puts a file is the branch's own only when the merge takes every file in it;
/// a
///  directory nobody's files explain stays someone else's and is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// changes.some((other) => other.path.startsWith(`${change.path}/`) && other.existed() && !other.existsAfter());
/// ```
fn the_branchs_files_stand_below(
    change: &Change,
    changes: &[Change],
) -> bool {
    let mut prefix: Vec<u8> = change.path.clone();
    prefix.push(b'/');
    for other in changes {
        if takes_a_file_below(other, prefix.as_slice()) {
            return true;
        }
    }
    return false;
}
/// What the worktree held at one changed path when recovery judged it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Judged = { standing: Standing; identity?: FileIdentity };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Judged {
    /// How the path stood against the two sides of the change.
    pub standing: Standing,
    /// The file that stood there,
    /// as the file system named it,
    /// or nothing when nothing stood there.
    pub identity: Option<FileIdentity>,
    /// The object the worktree held there,
    /// or nothing when it held no file:
    /// the bytes a take must still find when it finishes the file.
    pub content: Option<String>,
}
