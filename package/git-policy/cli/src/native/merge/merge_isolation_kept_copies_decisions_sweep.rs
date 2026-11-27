//! What:
//!  The copies a command decides all at once:
//!  every copy this worktree's records name,
//!  the copies left beside one path of a merge that is running,
//!  and the keys of the merges whose own process still runs.
//! Why:
//!   One decision settles one copy,
//! and a command that runs the merge recovery makes it for every copy the
//!  worktree still holds,
//! so a copy nothing looks at again never becomes a file the person cannot account for.
//!  The copies of a merge whose process still runs are not a command's to decide:
//! that merge kept them because a path it is still writing was in the way,
//! and putting one back under it makes its own checkout fail.
//!  The sweep is a file of its own so the one decision it repeats stays inside the code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await sweepKeptCopiesAwaiting(place, runningKeys, out); const back = await settleTheAsidesOfAMergeAwaiting({ changes, capture });
//! ```

/// The one job of the blocking pool that makes the step's file calls,
/// so a disk that makes them wait holds no worker of the runtime.
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
/// The person's own work a merge captured,
/// which says which bytes are the merge's own.
use crate::merge_isolation_carry_parts::Capture;
/// The change one path of a merge makes,
/// and the worktree path it writes.
use crate::merge_isolation_changes::{
    Change,
    worktree_path,
};
/// The records still there,
/// and whether the merge each names still runs.
use crate::merge_isolation_journal::{
    list_merge_intents,
    parse_merge_intent,
};
/// The copies a merge kept,
/// and the record of each.
use crate::merge_isolation_kept_copies::{
    KeptCopy,
    kept_copies,
};
/// The one decision every command makes about a copy.
use crate::merge_isolation_kept_copies_decisions::settle_kept_copy_awaiting;
/// Whether an unfinished merge names one copy.
use crate::merge_isolation_kept_copies_decisions_evidence::an_unfinished_merge_names_it_of;
/// The place a command's recovery runs in.
use crate::merge_isolation_recovery_place::RecoveryPlace;
/// The private names a take leaves,
/// which say which merge took a copy.
use crate::merge_isolation_take_aside_names::{
    Aside,
    asides_of,
    live_merge_keys,
    merge_key_of_a_record,
    stale_carry_names_of,
};
/// Real Git.
use crate::transaction_git::GitContext;
/// Whether the process that owns a merge still runs.
use crate::transaction_owner::OwnerLiveness;
/// Writing the lines a person reads.
use std::io::Write;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};
/// A list of the live merges' keys shared with each path's job,
/// so no path's job copies it.
use std::sync::Arc;

/// What:
///  Decide every copy a merge kept in this worktree,
///  for a command that runs the recovery,
///  and leave alone every copy a merge that still runs owns.
/// Why:
///   A copy is left beside a path one merge touched,
/// and a later merge that touches another path says nothing
///  about it,
/// so without this step the copy is a file no command ends
///  (`merge-carry-review-round-2-findings-1`,
/// its first finding).
///  The copies of a merge whose process still runs are not this command's to decide:
/// that merge kept them because
///  a path it is still writing was in the way,
/// and putting one back under it makes Git refuse the checkout the merge was about to
///  make,
/// so a command that has nothing to do with that merge would fail it (the owner's "Agents should never
///  wait on each other on commits or merges",
/// and `SPEC.md`,
/// the record of a merge whose owner still runs).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await sweepKeptCopiesAwaiting(place, runningKeys, stderr);
/// ```
pub(crate) async fn sweep_kept_copies_awaiting<W: Write + ?Sized>(
    place: &RecoveryPlace,
    running: &[String],
    out: &mut W,
) {
    // The copies this worktree's records name and the keys of the merges that are live are read from a folder of the Git
    //  directory, so one job of the blocking pool makes both.
    let records_of: PathBuf = place.git_dir.clone();
    let listed = run_blocking_job_within_the_budget(move || {
        let copies: Vec<KeptCopy> = kept_copies(records_of.as_path());
        if copies.is_empty() {
            return (copies, Vec::new());
        }
        return (copies, live_merge_keys(records_of.as_path()));
    })
    .await;
    let (copies, live): (Vec<KeptCopy>, Vec<String>) = match listed {
        Ok(found) => found,
        // A records folder the pool never read names no copy this command could decide, so it decides none and the next command
        //  meets them.
        Err(refused) => {
            crate::diagnostic_log::debug(
                "sweepKeptCopies",
                format!(
                    "the copies kept beside this worktree's paths could not be read: {refused}"
                )
                .as_str(),
            );
            return;
        }
    };
    for copy in copies {
        // The copy's own name says which merge took it, and the record cannot: a name is the only thing that survives a release
        //  that changed the record's shape.
        let names_beside: PathBuf = copy.location.clone();
        let asides: Vec<Aside> = match run_blocking_job_within_the_budget(move || {
            return asides_of(names_beside.as_path());
        })
        .await
        {
            Ok(found) => found,
            // A folder the pool never read says nothing about this path's copies, so they wait for the next command.
            Err(_refused) => continue,
        };
        let mut named: Option<Aside> = None;
        for aside in asides {
            if aside.path == copy.aside {
                named = Some(aside);
            }
        }
        // A merge that still runs decides its own copies when it finishes them; a command beside it only reads.
        if an_unfinished_merge_names_it_of(named.as_ref(), running) {
            continue;
        }
        // A merge that wrote its record after the caller read the owners is not in `running`, and its copies are as much its
        //  own as a running merge's: putting one back under the path it cleared makes its own checkout fail. The names are read
        //  again here, beside the copy they decide, so a merge that started in between keeps what it took.
        if named.is_some() {
            let records_again: PathBuf = place.git_dir.clone();
            let owners_that_run: Vec<String> = match run_blocking_job_within_the_budget(move || {
                return merges_whose_owner_runs(records_again.as_path());
            })
            .await
            {
                Ok(found) => found,
                // A reading the pool never made proves nothing about this copy's merge, so the copy is left for that merge.
                Err(_refused) => continue,
            };
            if an_unfinished_merge_names_it_of(named.as_ref(), owners_that_run.as_slice()) {
                continue;
            }
        }
        settle_kept_copy_awaiting(
            &place.git,
            place.cwd.as_path(),
            place.git_dir.as_path(),
            &copy,
            named.as_ref(),
            None,
            None,
            live.as_slice(),
            out,
        )
        .await;
    }
    return;
}

/// What:
///  The keys of the merges whose records are there at this instant and whose owner still runs.
/// Why:
///  A copy names the merge that took it,
/// and only that merge may decide it while it runs:
/// a command beside
///  a live merge that put the copy back under the path the merge cleared would make its own checkout fail
/// (`SPEC.md`,
///  "a copy whose merge still runs is left for that merge to finish").
/// The list a caller built is a reading of the
///  records,
/// and a merge that wrote its record after it is not in it,
/// so the copy's own name is looked up against a reading taken here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergesWhoseOwnerRuns(gitDir: string): string[]; // listMergeIntents().filter(isAlive).map(nameKey)
/// ```
pub(crate) fn merges_whose_owner_runs(git_dir: &Path) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    let Ok(records) = list_merge_intents(git_dir) else {
        return keys;
    };
    for record in records {
        // A record that is a pipe is no record this command can read, so it names no merge that runs and the
        //  read answers at once rather than waiting for a writer (`crate::plain_file`).
        let Ok(bytes) = crate::plain_file::read_plain_file(
            record.as_path(),
            crate::plain_file::SMALL_FILE_LIMIT,
        ) else {
            continue;
        };
        let Some(intent) = parse_merge_intent(bytes.as_slice()) else {
            continue;
        };
        let liveness = crate::merge_isolation_journal::owner_liveness(&intent, record.as_path());
        if matches!(liveness, Ok(OwnerLiveness::Alive)) {
            keys.push(merge_key_of_a_record(record.as_path()));
        }
    }
    return keys;
}

/// What:
///  Decide every copy left beside one path of a merge that is running,
///  with the change and the capture that say which bytes are the merge's own.
/// Why:
///   The merge's own record and capture are the only evidence that says
/// whether a copy an earlier release left is the file this merge judged at that
///  path,
/// and the path must hold that file before the paths are judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const change of changes) for (const aside of asidesOf(worktreePath(root, change.path))) await settleKeptCopy({ ..., aside, change, capture, live, stderr });
/// ```
pub(crate) async fn settle_the_asides_of_a_merge_awaiting<W: Write + ?Sized>(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    changes: &[Change],
    capture: Option<&Capture>,
    transaction_id: &str,
    out: &mut W,
) -> Vec<PathBuf> {
    // The keys of the live merges are read from a folder of the Git directory, so a job of the blocking pool reads them.
    let records_of: PathBuf = git_dir.to_path_buf();
    let live: Arc<Vec<String>> = match run_blocking_job_within_the_budget(move || {
        return live_merge_keys(records_of.as_path());
    })
    .await
    {
        Ok(found) => Arc::new(found),
        // A folder the pool never read names no merge whose copies are its own, so the caller is answered none and the next
        //  command decides them.
        Err(refused) => {
            crate::diagnostic_log::debug(
                "settleTheAsidesOfAMerge",
                format!("the merges whose copies live beside this worktree's paths could not be read: {refused}").as_str(),
            );
            return Vec::new();
        }
    };
    // The paths a copy went back to, which now hold the file the merge judged there: a rename leaves that file's identity as it
    //  was and changes only its change time, so a caller that judges those paths against an older reading must read them again.
    let mut put_back: Vec<PathBuf> = Vec::new();
    for change in changes {
        let location: PathBuf = worktree_path(root, change.path.as_slice());
        // The bytes a killed carry had written under their own name are a temporary of the placing, made again by the next
        //  command and held in the object store either way, so a name nothing is waiting on goes before the copy beside it is
        //  decided: otherwise a finished merge leaves an untracked file in the worktree for good (`SPEC.md`, no half-applied
        //  result). Reading those names, removing them and reading the asides are file calls that can wait on a disk, so one job
        //  of the blocking pool makes all three.
        let (path_beside, keys, finished): (PathBuf, Arc<Vec<String>>, String) = (
            location.clone(),
            Arc::clone(&live),
            String::from(transaction_id),
        );
        let looked = run_blocking_job_within_the_budget(move || {
            for stale in
                stale_carry_names_of(path_beside.as_path(), keys.as_slice(), finished.as_str())
            {
                let _removed = std::fs::remove_file(stale.as_path());
            }
            return asides_of(path_beside.as_path());
        })
        .await;
        let Ok(asides) = looked else {
            // A path the pool never read says nothing about the copies beside it, so they wait for the next command.
            continue;
        };
        for aside in asides {
            let copy: KeptCopy = KeptCopy {
                location: location.clone(),
                aside: aside.path.clone(),
            };
            if settle_kept_copy_awaiting(
                context,
                root,
                git_dir,
                &copy,
                Some(&aside),
                Some(change),
                capture,
                live.as_slice(),
                out,
            )
            .await
            {
                put_back.push(location.clone());
            }
        }
    }
    return put_back;
}
