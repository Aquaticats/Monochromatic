//! What:
//!  The words a person reads when the service gave up on a push or a relink:
//!  the notice of one abandoned relink,
//!  where that notice is written,
//!  and what the recovery hands each record it gave up on.
//! Why:
//!  Such an item moves to `failed` by recovery and not by a handler,
//!  so nothing else records why the commit was never backed up,
//!  and a pass killed between the words and the move is told again;
//!  a relink's words go to the worktree whose links still name another
//!  checkout,
//!  which no command in another worktree answers for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const item of givenUp) if (!tell(item)) told = false;
//! class SkipNotice { tell(givenUp: WorkItem) { return tellAboutGivenUp(this.gitDir, [givenUp]) } }
//! ```

use super::service_work_kinds_and_a_wake::MAX_TRIES;
use crate::push_failure_notice::record_push_skip;
use crate::work_item::{
    WorkItem,
    WorkKind,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Write one abandoned relink's words where a command reads them,
///  and answer whether they are there.
/// Why:
///   The relinked worktree's own Git directory is where the person works,
///   so the words go there when this repository can prove that the worktree at that path is the record's
///   (`notice_target_of_a_worktree`).
///   A directory that cannot be proved,
/// or that refuses the words,
///   must not hold the record in `recovering` for ever:
///   the recovery settles nothing until the person is told,
///   so the words go to the directory the record was drained from,
///   which a command in the worktree whose add wrote it reads and which is writable whenever the recovery can move
///   records at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tell = (item, drain, text) => recordInto(noticeTarget(drain, item.repository, item.created)) || recordInto(drain, text);
/// ```
fn tell_one_abandoned_relink(
    item: &WorkItem,
    drain_git_dir: &Path,
    text: &str,
) -> bool {
    let target: Option<PathBuf> = crate::package_repair_relink_notice::notice_target_of_a_worktree(
        drain_git_dir,
        item.repository.as_path(),
        item.created.as_str(),
    );
    if let Some(worktree_git_dir) = target.as_ref() {
        match crate::package_repair_relink_notice::record_notice(
            worktree_git_dir.as_path(),
            item.id().as_str(),
            text,
        ) {
            Ok(()) => return true,
            Err(problem) => crate::service_error!(
                "the abandoned relink of {} {} could not be recorded in the worktree's own Git directory ({}), so it is recorded where the record was drained from instead: {problem}",
                item.id(),
                crate::service_timing::in_the_repository_at(item.repository.as_path()),
                worktree_git_dir.display()
            ),
        }
    }
    match crate::package_repair_relink_notice::record_notice(
        drain_git_dir,
        item.id().as_str(),
        text,
    ) {
        Ok(()) => return true,
        Err(problem) => {
            crate::service_error!(
                "the abandoned relink of {} {} could not be recorded for the person: {problem}",
                item.id(),
                crate::service_timing::in_the_repository_at(item.repository.as_path())
            );
            return false;
        }
    }
}

/// What:
///  Leaves a notice for each push the service gave up on without ever finishing it:
/// its holder was stopped,
/// or lost,
/// on every
///  try,
/// and answers whether every notice is where a command will show it.
/// Why:
///   Such an item moves to `failed` by recovery,
/// not by a handler,
/// so nothing else records why the commit was never backed up
///  and the person would not be told.
/// The notice is the one a skipped push leaves,
/// because nothing failed and nothing was sent.
///  A notice that could not be written answers that it is not there,
/// so the push is not settled in silence.
///  A relink given up on is told in the Git directory of the worktree whose links still name another checkout,
///  so the next command there shows it,
///  and no command in another worktree answers for a worktree it does not hold.
///  That directory is used only when this repository proves it is the record's worktree and it takes the words;
///  otherwise the words go to the directory the record was drained from (`tell_one_abandoned_relink`).
///  Its reason stays kept until the record is in `failed` (`GivenUpNotice::settled`),
///  because a pass killed between the words and the move is told again and must say the same thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const item of givenUp.filter(isPush)) if (!(await recordPushSkip(gitDir, item, reason))) return false; return true;
/// ```
pub(crate) fn tell_about_given_up(
    git_dir: &Path,
    given_up: &[WorkItem],
) -> bool {
    let mut told: bool = true;
    for item in given_up {
        // A relink given up on leaves the worktree's links naming the checkout the copy came from, and nothing reads `failed`;
        //  the words name what the last try failed with and the command that finishes the job, and the notice waits in that
        //  worktree's own Git directory for the next command there.
        if item.kind == WorkKind::Relink {
            match crate::package_repair_relink_notice::notice_for_given_up(item, MAX_TRIES) {
                Some(text) => {
                    // The words replace any an earlier telling of this record left: a pass killed between them and the move
                    //  to `failed` is told again, and says the same thing because the reason is still kept
                    //  (`GivenUpNotice::settled` takes it back once the record is where nothing reads it again).
                    if !tell_one_abandoned_relink(item, git_dir, text.as_str()) {
                        told = false;
                    }
                    continue;
                }
                None => {
                    crate::service_error!(
                        "the abandoned relink of {} {} names no tool this release knows, so no notice names a command to run",
                        item.id(),
                        crate::service_timing::in_the_repository_at(item.repository.as_path())
                    );
                    told = false;
                }
            }
            continue;
        }
        if item.kind != WorkKind::Push {
            continue;
        }
        let reason: String = format!(
            "monochromatic-git-policy-cli: the commit service stopped {MAX_TRIES} times before it finished pushing {} (a restart, a kill or a crash each time), so it gave up on that push.",
            item.branch_ref
        );
        if let Err(problem) = record_push_skip(
            git_dir,
            item.branch_ref.as_str(),
            item.oid.as_str(),
            reason.as_str(),
        ) {
            crate::service_error!(
                "the abandoned push of {} {} could not be recorded for the person: {problem}",
                item.oid,
                crate::service_timing::in_the_repository_at(item.repository.as_path())
            );
            told = false;
        }
    }
    return told;
}

/// What:
///  The notice a recovery leaves for a push whose holder died on every try:
/// the skipped-push record of the branch,
/// written into the directory the recovery works on.
/// Why:
///   The record moves to `failed` right after the notice,
/// and nothing scans `failed`,
/// so the notice is the person's only remaining word about that branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SkipNotice { tell(item) { recordPushSkip(this.gitDir, item) } }
/// ```
pub(crate) struct SkipNotice<'a>(pub(crate) &'a Path);

impl crate::work_item_recovery::GivenUpNotice for SkipNotice<'_> {
    fn tell(
        &self,
        given_up: &WorkItem,
    ) -> bool {
        return tell_about_given_up(self.0, std::slice::from_ref(given_up));
    }

    /// What:
    ///  Take a relink's kept reason back,
    ///  now that its record is in `failed` and no later pass will tell again,
    ///  and give up the programs and transport settings a push given up on held.
    /// Why:
    ///   The reason has to outlive the words:
    ///   a pass killed between them and the move to `failed` is told again by the next one,
    ///   and a reason forgotten at the first telling would make that second notice claim no tool ever answered.
    ///   Nothing reads a record in `failed`,
    /// so keeping it longer would leave a file nothing removes;
    /// and the programs of a push live only in this service's memory and are keyed by that record
    ///  (`service_push_programs_held_in_memory`),
    /// so a push the recovery gave up on keeps none of them either,
    ///  and one of them can hold a password.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// settled(item) { if (item.kind === 'relink') forget(item.gitDir, item.id()); if (item.kind === 'push') forgetPrograms(item.gitDir, item.id()); }
    /// ```
    fn settled(
        &self,
        given_up: &WorkItem,
    ) {
        if given_up.kind == WorkKind::Relink {
            crate::package_repair_relink_failure::forget(
                given_up.git_dir.as_path(),
                given_up.id().as_str(),
            );
        }
        if given_up.kind == WorkKind::Push {
            crate::service_push_programs_held_in_memory::forget(
                given_up.git_dir.as_path(),
                given_up.id().as_str(),
            );
        }
    }
}
