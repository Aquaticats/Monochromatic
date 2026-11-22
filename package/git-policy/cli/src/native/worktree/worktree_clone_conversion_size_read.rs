//! What:
//!  The conversion fence's one `git ls-tree` size read:
//! the map every written path is proved by,
//! the thread that can start it before the source check on rung 1,
//! and the join that takes its answer at the fence.
//! Why:
//!  On rung 1 (the snapshot) the read sat on the add's waiting path after `checkSource` and after `git check-attr`.
//! The read needs only what the ref files answer in process (the worktree root,
//! the commit `HEAD` names and the object format),
//!  so the ladder starts it before the check and the fence takes its answer when it listed the commit the check proved
//! (`doc/decision/cli-git-fence-size-read-keeps-git-ls-tree.md`,
//! its section "The overlap,
//! answered 2026-11-22").
//! The read never fetches from a server,
//!  and it is joined with a deadline,
//!  so one that never answers refuses the rung with a failed read's own sentence rather than holding the add.
//! It runs on a thread of its own and not on the process's runtime,
//!  because every Git run registers its child's pipes with that runtime
//!  (`step_output_read_whole_through_pipes::attach_pipes`),
//!  which refuses a caller already inside it:
//! a task and a job of the blocking pool both are,
//!  and both failed this read with that refusal when it was built that way.
//! Rung 2 is untouched:
//!  its fence reads beside the copies (`worktree_clone_conversion_beside_the_clone.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const started = startBesideTheCheck(context, identity); // rung 1 only, null when it cannot start
//! const fence = fenceTheCloneWithTheReadBesideTheCheck(source, context, started); // takes it, or reads here
//! ```

use crate::add_timing::{begin_step, end_step, note_work};
use crate::diagnostic_log::debug;
use crate::repo_read_layout::ObjectFormat;
use crate::subvolume_index_files::repository_hash_length;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::worktree_clean::head_commit;
use crate::worktree_clone_conversion_fence::{AttributeFence, Fence, decide_the_paths_with_sizes, fence_the_attributes};
use crate::worktree_clone_conversion_git_reads::{BlobSizes, a_git_run_that_did_not_answer, blob_sizes, parse_sizes, unreadable_sizes};
use crate::worktree_identity::WorktreeIdentity;
use crate::scoped_start::{refused_thread_sentence, spawn_named};
use crate::worktree_snapshot_source::SnapshotSource;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeCloneConversionSizeRead";

/// The name the read's own duration is noted under in the add's record.
/// Why:
///   The read runs on another thread,
/// which the add's own steps cannot see,
///  so its time is the record's work and the join's wait is a step.
const WORK: &str = "fenceSizesBesideTheCheck";

/// How long the fence waits for the read started beside the check.
/// Why:
///   The read is one plumbing step,
/// which the runner ends at `PLUMBING_STEP_CEILING` and gives `SHORT_EXTRA_WAIT` to end
/// (`transaction_git_step_bounds.rs`),
///  so a read that has not answered by then has already failed inside its own runner;
/// this deadline only catches a read whose task died or is held outside it,
/// and the numbers are that step's own,
/// not a new limit.
const THE_JOIN_DEADLINE: Duration = crate::wait_ceilings::PLUMBING_STEP_CEILING.saturating_add(crate::wait_ceilings::SHORT_EXTRA_WAIT);

/// The name the read's thread carries in the process.
/// Why:
///   The read is the one step of an add that runs on a thread of its own,
/// so a diagnostic that lists threads names it for what it does.
const THREAD: &str = "clone-size-read-beside-the-check";

/// The variable that stops Git from fetching the objects a partial clone is missing,
///  and the option Git 2.45.0 takes for the same thing.
/// Why:
///   `git ls-tree -l` needs every blob's size.
/// On a partial clone,
/// a missing blob is fetched from the promisor remote to answer it,
///  which wrote packs into a source the check was about to refuse
///  (the gate review's High 1 of 2026-11-22).
/// Git 2.45.0 is the first release that honours the variable
///  (`environment.c`,
/// `NO_LAZY_FETCH_ENVIRONMENT`,
/// sets `fetch_if_missing = 0`),
///  and it takes `--no-lazy-fetch` as the same switch before any subcommand;
/// an older Git refuses that option and reads nothing,
///  so it cannot fetch here either.
/// With either switch,
///  an unavailable size prints as `BAD`,
///  which `parse_sizes` skips.
/// A path whose size cannot be listed therefore only ever goes to Git's writing.
const NO_LAZY_FETCH: &str = "GIT_NO_LAZY_FETCH";

/// What the read's job answers:
/// how long it took,
/// and the map or the reason it cannot be read.
type ReadAnswer = (Duration, Result<BlobSizes, String>);

/// What:
///  What the join got:
/// the read's own cost with its answer,
///  or the sentence for a read that answered nothing.
/// Why:
///   A read that answers and a read that does not are both facts the record states,
/// and the fence only uses the first;
/// one value carries them apart so no caller mistakes a missing answer for a map.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Joined = { spent: number; outcome: Result<Map<string, number>, string> } | { silent: string };
/// ```
enum Joined {
    /// The read answered,
    /// with how long it spent.
    Answered {
        /// The read's own time.
        spent: Duration,
        /// The map,
        /// or the reason it could not be read.
        outcome: Result<BlobSizes, String>,
    },
    /// The read answered nothing,
    /// in the words a failed run of the fence gives.
    Silent(String),
}

/// What:
///  The size read one add started before its source check:
/// the commit it listed and the thread that will answer.
/// Why:
///   The fence may use the answer only for the commit its source proved,
/// so the commit travels with the thread and a different one leaves the read unused;
/// the handle is the whole of this add's own state,
/// so no other command and no other record can reach it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SizeReadBesideTheCheck = { commit: string; answer: Channel<Result<Map<string, number>, string>> };
/// ```
pub struct SizeReadBesideTheCheck {
    /// The commit the read listed.
    commit: String,
    /// The one answer the read's thread sends.
    answer: std::sync::mpsc::Receiver<ReadAnswer>,
    /// The process id of the read's Git child while it runs,
    /// which an add that abandons the read ends.
    child: std::sync::Arc<std::sync::atomic::AtomicI32>,
}

/// What:
///  Whether this source may fetch a missing object from a promisor remote.
/// Why:
///   `GIT_NO_LAZY_FETCH` and `--no-lazy-fetch` are Git 2.45.0's;
///  an older Git refuses the option and reads nothing,
///  so the fence may retry the serial read only where nothing could be fetched.
///  A promisor pack or a `promisor` word in the common config is proof enough to keep the read's failure;
///  anything it cannot prove is treated as a source that may fetch,
/// which is the safe direction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mayFetch = (commonDir) => readdir(join(commonDir, 'objects/pack')).some((name) => name.endsWith('.promisor')) || readFile(join(commonDir, 'config')).toLowerCase().includes('promisor');
/// ```
fn may_fetch(common_dir: &Path) -> bool {
    let packs: std::path::PathBuf = common_dir.join("objects").join("pack");
    if std::fs::read_dir(packs).is_ok_and(|entries| return entries.flatten().any(|entry| return entry.path().extension() == Some(std::ffi::OsStr::new("promisor")))) {
        return true;
    }
    let config: Vec<u8> = std::fs::read(common_dir.join("config")).unwrap_or_default();
    return String::from_utf8_lossy(config.as_slice()).to_lowercase().contains("promisor");
}

/// What:
///  The worktree root and its two Git directories,
/// or nothing for an identity without a worktree.
/// Why:
///   The read lists the commit `HEAD` names in the source's own worktree,
/// and the ref files and the configuration are read from its directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directoriesOf = (identity) => identity.kind === 'worktree' ? [identity.root, identity.gitDir, identity.commonDir] : undefined;
/// ```
fn directories_of(identity: &WorktreeIdentity) -> Option<(&Path, &Path, &Path)> {
    return match identity {
        WorktreeIdentity::MainWorktree { worktree_root, git_dir, common_dir } | WorktreeIdentity::LinkedWorktree { worktree_root, git_dir, common_dir } => Some((worktree_root.as_path(), git_dir.as_path(), common_dir.as_path())),
        WorktreeIdentity::BareRepository { .. } | WorktreeIdentity::OutsideWorktree => None,
    };
}

/// What:
///  Start the fence's size read now,
///  before the source check the ladder runs next.
/// Why:
///   The ladder starts it on rung 1 alone,
///  where the read would otherwise run after the check and after `git check-attr`;
///  the read needs only the worktree root,
///  the commit `HEAD` names and the object format,
///  all of which are answered here without a Git run.
/// It is a plain thread and not a task of the process's runtime,
///  nor a job of its blocking pool:
/// a Git run registers its child's pipes with that runtime and refuses a caller already inside it
///  (`step_output_read_whole_through_pipes::attach_pipes`),
///  and both of those callers are inside it,
/// so neither can run this read at all.
/// A start that cannot happen (no worktree,
///  a ref store this reader does not know,
///  a system that refuses the thread)
///  loses only the overlap:
/// the fence then reads where it stands,
/// as it always has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startBesideTheCheck(context, identity): SizeReadBesideTheCheck | undefined;
/// ```
pub fn start_beside_the_check(context: &GitContext, identity: &WorktreeIdentity) -> Option<SizeReadBesideTheCheck> {
    let (root, git_dir, common_dir): (&Path, &Path, &Path) = directories_of(identity)?;
    let hash_length: usize = repository_hash_length(common_dir)?;
    let format: ObjectFormat = if hash_length == 32 { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
    let Some(commit): Option<String> = head_commit(git_dir, common_dir, format) else {
        debug(TAG, "the commit HEAD names is not a plain ref, so the size read does not start beside the check");
        return None;
    };
    let may_fetch: bool = may_fetch(common_dir);
    let child: std::sync::Arc<std::sync::atomic::AtomicI32> = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));
    let (sender, answer): (Sender<ReadAnswer>, std::sync::mpsc::Receiver<ReadAnswer>) = std::sync::mpsc::channel();
    let state: (GitContext, PathBuf, String, Sender<ReadAnswer>, bool, std::sync::Arc<std::sync::atomic::AtomicI32>) = (context.clone(), root.to_path_buf(), commit.clone(), sender, may_fetch, child.clone());
    if let Err(error) = spawn_named(THREAD, move || {
        return read_the_sizes_where_nothing_is_fetched(state);
    }) {
        debug(TAG, refused_thread_sentence("read the fence's blob sizes beside the source check", &error).as_str());
        return None;
    }
    return Some(SizeReadBesideTheCheck { commit, answer, child });
}

/// What:
///  The read's thread:
/// the sizes of the commit's tree with the time they took,
///  sent once to the add that started it.
/// Why:
///   The thread owns what it reads,
/// so it takes the context,
///  the worktree root and the commit by value and answers both the cost and the map;
/// a send nobody waits for any more is dropped,
///  because the add moved past its deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTheSizesWhereNothingIsFetched([context, root, commit, sender]) { const began = performance.now(); sender.send([performance.now() - began, sizesOf(context, root, commit)]); }
/// ```
fn read_the_sizes_where_nothing_is_fetched((context, root, commit, sender, may_fetch, child): (GitContext, PathBuf, String, Sender<ReadAnswer>, bool, std::sync::Arc<std::sync::atomic::AtomicI32>)) {
    let began: Instant = Instant::now();
    let sizes: Result<BlobSizes, String> = sizes_of_the_commit_where_nothing_is_fetched(&context, root.as_path(), commit.as_str(), may_fetch, &child);
    // The child is reaped by its runner here, so an add that abandons this read afterwards must not end a reused process id.
    child.store(0, std::sync::atomic::Ordering::SeqCst);
    let _sent: Result<(), std::sync::mpsc::SendError<ReadAnswer>> = sender.send((began.elapsed(), sizes));
}

/// What:
///  The one request the early read makes:
/// `git ls-tree -r -l -z` of `commit`,
///  with the no-fetch switches when the source may hold a promisor remote,
///  which leaves the process-wide failed-step note alone,
///  and whose child ends with the thread that starts it.
/// Why:
///   Both of the last facts are conditions of starting before the source check:
/// the add's own steps may fail while this read runs,
///  so the read must not take their note (`GitRequest::beside_another_step`),
///  and an add that ends with the read still running must leave no child (`end_with_the_starting_thread`).
/// The no-fetch switches are there only where a fetch is possible:
///  `GIT_NO_LAZY_FETCH` and `--no-lazy-fetch` are Git 2.45.0's,
///  and Git before that refuses the option before it reads anything.
///  A plain source has no promisor remote,
/// so today's read is safe there and an older Git keeps its overlap;
///  a source that may fetch gets the option,
/// so an older Git reads nothing and the fence falls back to Git's checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theEarlyReadRequest = (root, commit, mayFetch) => ({ cwd: root, args: [...(mayFetch ? ['--no-lazy-fetch'] : []), 'ls-tree', '-r', '-l', '-z', commit], env: mayFetch ? { GIT_NO_LAZY_FETCH: '1' } : {}, besideAnotherStep: true, endWithTheStartingThread: true });
/// ```
fn the_early_read_request(root: &Path, commit: &str, may_fetch: bool) -> GitRequest {
    let mut arguments: Vec<OsString> = Vec::with_capacity(6);
    let mut environment: Vec<(OsString, OsString)> = Vec::new();
    if may_fetch {
        arguments.push(OsString::from("--no-lazy-fetch"));
        environment.push((OsString::from(NO_LAZY_FETCH), OsString::from("1")));
    }
    arguments.extend([OsString::from("ls-tree"), OsString::from("-r"), OsString::from("-l"), OsString::from("-z"), OsString::from(commit)]);
    return GitRequest {
        cwd: root.to_path_buf(),
        arguments,
        without_prefix: true,
        environment,
        ..GitRequest::default()
    }.beside_another_step().ends_with_the_starting_thread();
}

/// What:
///  One `git ls-tree -r -l -z` of `commit`,
/// run where a partial clone may not fetch.
/// Why:
///   This is the early read's own run;
/// its output is parsed by the same `parse_sizes` the fence's serial read uses,
///  and a run that fails says what the fence's own read says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sizesOfTheCommitWhereNothingIsFetched(context, root, commit): Map<string, number>;
/// ```
fn sizes_of_the_commit_where_nothing_is_fetched(context: &GitContext, root: &Path, commit: &str, may_fetch: bool, child: &std::sync::Arc<std::sync::atomic::AtomicI32>) -> Result<BlobSizes, String> {
    #[cfg(test)]
    hold_the_read_where_a_control_asked();
    #[cfg(test)]
    die_where_a_control_asked();
    let output: GitOutput = run_git(context, &the_early_read_request(root, commit, may_fetch).publishing_its_child(child.clone())).map_err(|error| return a_git_run_that_did_not_answer(&error))?;
    if output.code != Some(0) {
        return Err(unreadable_sizes(&output));
    }
    return Ok(parse_sizes(output.stdout.as_slice()));
}

/// What:
///  Whether this read listed the commit the fence needs.
/// Why:
///   The guard is the whole of the proof the answer is used on:
/// it compares the commit the read listed with the commit the source check proved,
///  and the map is a function of that commit and of nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lists = (read, commit) => read.commit === commit;
/// ```
fn lists(read: &SizeReadBesideTheCheck, commit: &str) -> bool {
    return read.commit == commit;
}

/// What:
///  Both halves of the fence for the snapshot rung,
/// with the sizes taken from the read the ladder started when it listed the proved commit,
///  and read here when it did not.
/// Why:
///   The attributes and the decline are decided first,
/// as everywhere,
/// and the sizes are wanted only after a decline was refused;
/// a read of another commit is left in `started` for the add's own end to state its cost,
///  and this fence reads the proved commit itself,
/// which is today's serial order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fenceTheCloneWithTheReadBesideTheCheck(source, context, started): Fence;
/// ```
pub fn fence_the_clone_with_the_read_beside_the_check(source: &SnapshotSource, context: &GitContext, started: &mut Option<SizeReadBesideTheCheck>) -> Result<Fence, String> {
    let read: AttributeFence = fence_the_attributes(source, context)?;
    let sizes: Option<BlobSizes> = match read.needs_sizes() {
        true => Some(sizes_of_the_paths(source, context, started)?),
        false => None,
    };
    return decide_the_paths_with_sizes(read, sizes);
}

/// What:
///  The sizes the fence decides with:
/// the started read's answer when it listed the proved commit,
///  and one `git ls-tree` of that commit otherwise.
/// Why:
///   A started read that listed another commit is not this fence's answer,
/// and it is not waited for here:
///  the add's own end settles it without waiting,
///  so a settled `HEAD` is the only case that spends the overlap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sizesOfThePaths(source, context, started): Map<string, number>;
/// ```
fn sizes_of_the_paths(source: &SnapshotSource, context: &GitContext, started: &mut Option<SizeReadBesideTheCheck>) -> Result<BlobSizes, String> {
    let commit: &str = source.clean.head_commit.as_str();
    match started.as_ref() {
        Some(proved) if lists(proved, commit) => {
            let taken: SizeReadBesideTheCheck = started.take().expect("the read that lists the proved commit is the one just matched");
            return taken.join();
        }
        Some(read) => debug(TAG, format!("the read started beside the check listed {} and the fence needs {commit}, so it is not used and the fence reads for itself", read.commit).as_str()),
        None => {}
    }
    return blob_sizes(context, source.root.as_path(), commit);
}

impl SizeReadBesideTheCheck {
    /// What:
    ///  The read's answer,
    /// waited for within `THE_JOIN_DEADLINE`,
    /// with the wait recorded as a step of the adding thread and the read's own time as work.
    /// Why:
    ///   The wait is what the overlap saves,
    /// so it is the number the record has to show;
    /// a read that answers nothing (a deadline or a dead thread)
    /// refuses the rung here,
    /// which is the sentence a read that failed gives in the serial order.
    /// A thread the system refused never reaches this method:
    /// the start answers nothing and the fence reads where it stands.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// join() { const entered = beginStep('fenceSizesWaitingForTheRead'); const joined = this.answerWithinTheDeadline(); endStep(entered); return joined; }
    /// ```
    fn join(self) -> Result<BlobSizes, String> {
        let entered: bool = begin_step("fenceSizesWaitingForTheRead");
        let joined: Joined = self.answer_within_the_deadline();
        end_step(entered);
        return match joined {
            Joined::Answered { spent, outcome } => {
                note_work(WORK, spent);
                match outcome {
                    Ok(sizes) => Ok(sizes),
                    Err(reason) => Err(reason),
                }
            }
            Joined::Silent(reason) => Err(reason),
        };
    }

    /// What:
    ///  Take the read this add is not going to use only if it has already answered;
    /// never wait for it.
    /// Why:
    ///   An add the source check refuses,
    ///   or the fence declines,
    ///  may pay this read in the background
    /// (the decision `doc/decision/cli-git-fence-size-read-keeps-git-ls-tree.md`,
    /// its section "The overlap,
    /// answered 2026-11-22"),
    ///  but it never waits for it:
    /// a read longer than the add's own Git run would otherwise make the add slower than on `main`,
    ///  which is the integration pass's High 1 of 2026-11-22.
    /// A read that has answered is stated as the record's work;
    /// one that has not is stated as started and not joined,
    /// and the add ends without it.
    /// The read's thread and its child end with this process
    /// (`GitRequest::ends_with_the_starting_thread`),
    ///  so nothing of the add's outlives it.
    /// Nothing here changes the add's answer:
    ///  a read that failed is only a debug line.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// settle() { const answer = this.answer.tryRecv(); if (answer) noteWork('fenceSizesBesideTheCheck', answer.spent); else note('fenceSizesBesideTheCheck', 'started and not joined'); }
    /// ```
    pub fn settle(self) {
        match self.answer.try_recv() {
            Ok((spent, outcome)) => {
                note_work(WORK, spent);
                if let Err(reason) = outcome {
                    debug(TAG, format!("the size read started beside the check answers {reason}, and the add no longer waits for it").as_str());
                }
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                crate::add_timing::note(WORK, "started and not joined");
                debug(TAG, "the size read started beside the check has not answered, so the add states it as started and not joined; its thread and child end with this process");
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                crate::add_timing::note(WORK, "started and not joined");
                debug(TAG, "the size read started beside the check ended without answering, so the add states it as started and not joined");
            }
        }
    }

    /// What:
    ///  Stop the read this add is not going to use:
    ///  state it as started and not joined,
    ///  and end its Git child here.
    /// Why:
    ///   The source check can refuse the source before the fence,
    ///  and the read then has nothing to answer;
    ///  leaving it to run beside Git's own checkout makes the refused add slower than `main`
    ///  (the integration pass's cone shape of 2026-11-22),
    ///  so the child is ended as soon as the add knows it will not use it.
    /// The child id is the one its runner published;
    ///  a read that has not started a child,
    /// or one whose runner already reaped it,
    /// has zero here.
    /// The thread answers into a receiver this value drops,
    /// and its send is ignored.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// abandon() { note('fenceSizesBesideTheCheck', 'started and not joined'); const pid = this.child.swap(0); if (pid > 0) kill(pid, 'SIGKILL'); }
    /// ```
    pub fn abandon(self) {
        crate::add_timing::note(WORK, "started and not joined");
        let pid: i32 = self.child.swap(0, std::sync::atomic::Ordering::SeqCst);
        if pid > 0 {
            // SAFETY: `pid` is this add's own Git child, published by its runner while it ran;
            // a signal to a process that ended in between reaches nothing.
            let _ = unsafe { libc::kill(pid, libc::SIGKILL) };
            debug(TAG, format!("the size read started beside the check is not used, so its child {pid} is ended and the add does not wait for it").as_str());
        } else {
            debug(TAG, "the size read started beside the check is not used and has no child to end, so the add does not wait for it");
        }
    }

    /// What:
    ///  The read's answer within the deadline,
    ///  or the sentence for a read that answered nothing.
    /// Why:
    ///   One wait answers both facts:
    /// a task that died and a read that outlived the deadline are both "no output to decide with",
    ///  and the sentence is the one a failed run of the fence gives,
    /// not a new one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// answerWithinTheDeadline() { const waited = await timeout(deadline, this.answer); return read(waited); }
    /// ```
    fn answer_within_the_deadline(self) -> Joined {
        let deadline: Duration = the_deadline();
        return match self.answer.recv_timeout(deadline) {
            Ok((spent, outcome)) => Joined::Answered { spent, outcome },
            Err(RecvTimeoutError::Timeout) => Joined::Silent(a_git_run_that_did_not_answer(&std::io::Error::new(std::io::ErrorKind::TimedOut, format!("`git ls-tree` did not answer within {deadline:?}")))),
            Err(RecvTimeoutError::Disconnected) => Joined::Silent(a_git_run_that_did_not_answer(&std::io::Error::other("the read ended without answering"))),
        };
    }
}

/// The deadline in force for this join:
/// the shipped one,
/// or one a control asked for on this thread.
/// Why:
///   The shipped deadline is two minutes long,
/// so a control that shows the deadline's own sentence cannot spend it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deadline = THE_JOIN_DEADLINE_FOR_THIS_THREAD.get() ?? THE_JOIN_DEADLINE;
/// ```
fn the_deadline() -> Duration {
    #[cfg(test)]
    if let Some(asked) = THE_JOIN_DEADLINE_FOR_THIS_THREAD.with(std::cell::Cell::get) {
        return asked;
    }
    return THE_JOIN_DEADLINE;
}

#[cfg(test)]
thread_local! {
    /// A control's own deadline for the join on this thread;
    /// nothing outside a control.
    static THE_JOIN_DEADLINE_FOR_THIS_THREAD: std::cell::Cell<Option<Duration>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
static THE_READ_IS_HELD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
static THE_READ_DIES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
static THE_HELD_READ: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
static THE_HELD_READ_LETS_GO: std::sync::Condvar = std::sync::Condvar::new();

/// What:
///  Hold the read until a control lets it go,
/// so the join meets its deadline with a read that never answered.
/// Why:
///   No fixture can make a real `git ls-tree` hang,
/// and the sentence a deadline gives is the one a failed read gives,
///  which is the fact a control has to see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (THE_READ_IS_HELD) await new Promise(() => {}); // never answers
/// ```
#[cfg(test)]
fn hold_the_read_where_a_control_asked() {
    if !THE_READ_IS_HELD.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let held = THE_HELD_READ.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let _lets_go = THE_HELD_READ_LETS_GO.wait_timeout(held, Duration::from_secs(30)).unwrap_or_else(std::sync::PoisonError::into_inner);
}

/// What:
///  Make the read's thread die before it answers.
/// Why:
///   A dead thread is the other way the join answers nothing,
/// and no fixture makes a real read die.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (THE_READ_DIES) throw new Error('a control makes the read die');
/// ```
#[cfg(test)]
fn die_where_a_control_asked() {
    if THE_READ_DIES.load(std::sync::atomic::Ordering::SeqCst) {
        panic!("a control makes the size read die before it answers");
    }
}

/// What:
///  A shortened join deadline for this thread,
/// cleared when the value is dropped.
/// Why:
///   The shipped deadline is two minutes;
/// a control that shows its sentence needs a moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deadline = aControlShortensTheDeadlineTo(50); // deadline === 50 ms until dropped
/// ```
#[cfg(test)]
pub(crate) fn a_control_shortens_the_deadline_to(within: Duration) -> TheDeadlineForTheControl {
    THE_JOIN_DEADLINE_FOR_THIS_THREAD.with(|deadline| { deadline.set(Some(within)); });
    return TheDeadlineForTheControl;
}

/// The stand-in of `a_control_shortens_the_deadline_to`,
/// cleared when it is dropped.
#[cfg(test)]
pub(crate) struct TheDeadlineForTheControl;

/// Clears the shortened deadline.
#[cfg(test)]
impl Drop for TheDeadlineForTheControl {
    fn drop(&mut self) {
        THE_JOIN_DEADLINE_FOR_THIS_THREAD.with(|deadline| { deadline.set(None); });
    }
}

/// What:
///  A control that holds the read until this value is dropped,
/// with the read's own start of it.
/// Why:
///   The seams are process-wide because the read runs on a thread of its own,
/// and nextest gives every control its own process,
///  so one control never reaches another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hold = aControlHoldsTheRead(); // the read waits at its start until hold.release()
/// ```
#[cfg(test)]
pub(crate) fn a_control_holds_the_read() -> TheHeldReadForTheControl {
    THE_READ_IS_HELD.store(true, std::sync::atomic::Ordering::SeqCst);
    return TheHeldReadForTheControl;
}

/// The hold of `a_control_holds_the_read`,
/// released when it is dropped.
#[cfg(test)]
pub(crate) struct TheHeldReadForTheControl;

/// Lets the held read go,
/// so the process ends without it.
#[cfg(test)]
impl Drop for TheHeldReadForTheControl {
    fn drop(&mut self) {
        THE_READ_IS_HELD.store(false, std::sync::atomic::Ordering::SeqCst);
        THE_HELD_READ_LETS_GO.notify_all();
    }
}

/// What:
///  A control that makes the read's task die before it answers.
/// Why:
///   See `die_where_a_control_asked`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dies = aControlMakesTheReadDie(); // the next read throws until dies is dropped
/// ```
#[cfg(test)]
pub(crate) fn a_control_makes_the_read_die() -> TheDeadReadForTheControl {
    THE_READ_DIES.store(true, std::sync::atomic::Ordering::SeqCst);
    return TheDeadReadForTheControl;
}

/// The stand-in of `a_control_makes_the_read_die`,
/// cleared when it is dropped.
#[cfg(test)]
pub(crate) struct TheDeadReadForTheControl;

/// Clears the control's stand-in.
#[cfg(test)]
impl Drop for TheDeadReadForTheControl {
    fn drop(&mut self) {
        THE_READ_DIES.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Size-read controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clone_conversion_size_read_tests.rs"]
mod tests;
