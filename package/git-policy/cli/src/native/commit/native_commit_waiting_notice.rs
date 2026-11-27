//! What:
//!  The waiting notice and the ceiling of the commit Git builds in the scratch repository:
//! while that `git commit` runs,
//! a line on standard error
//! says what the command waits for,
//! how long it has waited,
//! and when the step is stopped;
//! when the ceiling comes,
//! that `git commit` and everything it started is ended.
//! Why:
//!  Every other wait of the wrapper says what it waits for (issue 1811),
//! but this step runs with the terminal attached and no watch,
//! because an editor may open in it,
//! so a commit that spent four minutes in it printed nothing.
//! The line names the `git commit` the command started (its process),
//! says the commit is signed when `commit.gpgsign` or `-S` asks
//! for it,
//! and names the signing program while one runs below the `git commit`.
//! The owner's rule is that nothing waits for ever (2026-11-03),
//! and his answer on this step (2026-11-07,
//! "limit unless editor"):
//! a ceiling when the message came with the command,
//! none when an editor may open,
//! where the person is the wait and a stop would lose what they typed.
//! The ceiling is `SCRATCH_COMMIT_STEP_CEILING` (`service/wait_ceilings.rs`),
//! with its measurement.
//!  A step that reaches its ceiling while the kernel holds it in a disk wait is slow and not stuck,
//!  and ending it there failed two commits on 2026-11-11,
//!  so the ceiling stands aside once for such a step and the step is waited for to
//!  `SCRATCH_COMMIT_STEP_STALL_CEILING` instead.
//!  That stand-aside is one decision this thread shares with the wait that owns the child
//!  (`scratch_step_stall_decided_once.rs`):
//!  either enforcer may make it,
//!  both then honour it,
//!  and neither ends the step at the usual ceiling afterwards.
//!  The line that says so belongs to the wait that owns the child
//!  (`process/child_awaited_until_it_exits_the_stop_or_the_ceiling.rs`),
//!  which always runs,
//!  so this thread prints no line of its own about a stand-aside;
//!  it says the later bound in every waiting line it prints after that.
//!  A look that could not be made decides nothing:
//!  the ceiling stays armed and the look is made again,
//!  and a step nothing can be learned about is ended at the bound it is waited for rather than at the usual ceiling.
//! A stop ends that `git commit` and what it started with `SIGKILL`:
//! the step runs in a scratch repository on a private copy of the index,
//! and the objects it wrote are content-addressed files the next build writes again,
//! so nothing of the person's is in the step.
//!  A step the kernel has not released cannot act on that signal,
//!  and the command waits for the kernel rather than for its ceiling:
//!  the owner chose that wait and asked for it to be said
//!  (2026-11-10,
//!  "Keep waiting and say so"),
//!  so the wait that keeps waiting says it,
//!  once and again for as long as the wait lasts,
//!  in a line that names the process the kernel is holding,
//!  says the stop was sent and the kernel has not released it,
//!  and says nothing of the person's is at risk
//!  (`process/wait_after_the_stop_saying_the_kernel_holds_it.rs`,
//!  `scratch_step_held_by_the_kernel.rs`).
//!  That line is the wait's because the wait is the one that lasts:
//!  a notice thread the system refused at its thread limit cannot say anything,
//!  and this notice's thread ends at the stop.
//!  What this notice keeps for it is the writer thread and the single queue every line goes through,
//!  so the line is written by one thread and never by a reader a stopped terminal holds.
//!  It ends nothing else:
//! the landing service runs landings of different repositories on worker threads of one process,
//! so another landing's Git is a child of the same process at the same moment,
//! and cutting its write is what the owner's answer of 2026-11-09 rules out
//! ("B:
//! let a branch write finish").
//! The step's own number is what the notice watches (`WatchedStep`):
//! `SignalGuard` relays signals but names no child,
//! so the number is handed over the moment the step starts,
//! and every look and every stop is rooted at it.
//! A commit that opens the person's editor prints nothing and is never stopped,
//! because the person is the wait.
//! The ceiling is counted from the first look that found the step below this process,
//! never from when the notice started (`LOOK_INTERVAL`):
//! a start this host held up would otherwise pass the ceiling before the step existed,
//! and a look that finds nothing would then end the notice and the ceiling with it,
//! which is how a commit landed after a 36,651 ms step under a 3,006 ms ceiling on 2026-11-08.
//! Once the step has been seen,
//! a look that finds nothing means it ended by itself,
//! and the commit lands.
//! A look that could not be made at all decides nothing:
//! the ceiling stays armed and the look is made again,
//! because reading a failed look as an empty one is how a ceiling is lost
//! (`blindLooks` counts them).
//! A stop is recorded only when a signal was really taken for the step:
//! a step that finished between the look and the signal landed,
//! and refusing it would fail a commit that succeeded.
//!  The lines are written by a thread of their own (`LINE_QUEUE`),
//! so a reader that stopped reading holds that thread and never the ceiling,
//! and `finish` waits for the notice and for the line it handed over no longer than `NOTICE_BOUND`.
//!
//! This file holds the notice itself:
//! the value a commit starts and ends,
//! the thread that writes its lines,
//! and the queue both the person and the wait that keeps waiting read.
//! The ceiling a person sets is in `native_commit_waiting_notice_ceiling.rs`,
//! the signing question in `native_commit_waiting_notice_signing.rs`,
//! and the thread that watches the step in `native_commit_waiting_notice_watch.rs`;
//! each is re-exported here so no caller changes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const notice = startNotice({ signs, cadence, ceiling }); try { await runGitCommit(); } finally { const stoppedAfter = notice.finish(); }
//! ```

/// The ceiling of the step a person sets,
/// and the seconds it names.
pub use crate::native_commit_waiting_notice_ceiling::{
    SCRATCH_CEILING_VARIABLE,
    scratch_ceiling,
};
/// Whether the commit is signed,
/// read from the configuration and the command line.
pub use crate::native_commit_waiting_notice_signing::signs_commit;

/// The number of the step,
/// published the moment it starts.
use crate::child_number_told_at_start::ChildNumber;
/// The configuration the commit is built under.
use crate::config_snapshot::ConfigSnapshot;
/// The thread that watches the step,
/// and what one watching thread is given.
use crate::native_commit_waiting_notice_watch::{
    TheWatch,
    watch_the_step,
};
/// The one decision this ceiling shares with the wait that owns the step.
use crate::scratch_step_stall_decided_once::TheStallDecidedOnce;
/// Whether the commit's arguments open an editor.
use crate::service_commit_args_editor_opening::opens_editor;
/// The cadence of the lines.
use crate::wait_progress::Cadence;
/// The notice's sink.
use std::ffi::OsString;
use std::io::Write;
/// A decision two enforcers of one ceiling share, kept across the thread that watches the step.
use std::sync::Arc;
/// The writer thread's own time,
/// kept where the notice thread can read it.
use std::sync::Mutex;
/// The channel that ends the notice thread,
/// and the queue its writer thread takes whole lines from.
use std::sync::mpsc::{
    Sender,
    SyncSender,
    channel,
    sync_channel,
};
/// Time spans and the monotonic clock.
use std::time::{
    Duration,
    Instant,
};

/// What:
///  How many lines the writer may hold before the notice stops handing it more.
/// Why:
///   A reader that stopped reading holds one line and nothing else:
/// the line before it is dropped and counted,
/// so the notice thread never waits for a reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lineQueue = 1;
/// ```
const LINE_QUEUE: usize = 1;

/// What:
///  The longest `finish` waits for the notice's answer,
/// and then for the writer to write the line it was handed.
/// Why:
///   A line is advisory:
/// a reader that stopped reading must never hold the commit,
/// and neither may a notice thread that cannot answer.
/// Twice `LOOK_INTERVAL` is longer than the thread's own wake and one look,
/// which is what an answer can honestly need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noticeBound = 500; // ms
/// ```
const NOTICE_BOUND: Duration = Duration::from_millis(500);

// What:
//  A control that refuses the thread the ceiling is enforced on,
//  as a process at its thread limit does with `EAGAIN` from `pthread_create`.
// Why:
//   The refusal cannot be produced on demand here:
//   a thread limit low enough to refuse this thread refuses the test's own threads too,
//   so the system's answer is injected
//   (`member_signalled_through_its_pidfd.rs` injects a refused descriptor the same way).
//  The planter runs on the thread that starts the notice,
//  and the notice thread is spawned from it.
#[cfg(test)]
thread_local! {
    /// How many notice threads the next starts are refused.
    static THREAD_REFUSALS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Refuse the next `count` notice threads.
#[cfg(test)]
pub(crate) fn refuse_the_next_notice_thread(count: usize) {
    THREAD_REFUSALS.with(|left| {
        left.set(count);
    });
}

/// What:
///  The refusal a control armed,
///  if any,
///  which stands for a system that cannot make another thread.
/// Why:
///   The start of the notice answers a refused thread the way it answers any other,
/// and only a control tells the two apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusedNow = () => armedRefusal-- > 0 ? new Error('refused') : undefined;
/// ```
fn a_thread_is_refused_now() -> Option<std::io::Error> {
    #[cfg(test)]
    if THREAD_REFUSALS.with(|left| {
        let armed: usize = left.get();
        left.set(armed.saturating_sub(1));
        return armed > 0;
    }) {
        return Some(std::io::Error::other("a control refused the notice thread"));
    }
    return None;
}

/// What:
///  A control's window on the notice:
///  the ceiling it plants,
///  the stretch it states between two looks of one step,
///  and the looks that found the step it waits for.
/// Why:
///  It is the watching thread's own state,
///  so it lives beside that thread (`native_commit_waiting_notice_watch.rs`),
///  and this is the path every control already reads it at.
///  Test-only,
/// so a release build reads no atomic here.
#[cfg(all(test, target_os = "linux"))]
pub(crate) use crate::native_commit_waiting_notice_watch::the_controls_window;

/// What a started notice is given.
pub struct NoticeStart<'a> {
    /// The configuration the commit is built under.
    pub snapshot: Option<&'a ConfigSnapshot>,
    /// The commit command's own arguments,
    /// each with its own bytes.
    pub commit_arguments: &'a [OsString],
    /// The cadence of the lines.
    pub cadence: Cadence,
    /// How long the step lasts before it is stopped,
    /// when no editor may open in it.
    pub ceiling: Duration,
    /// Where the lines go.
    pub sink: Box<dyn Write + Send>,
}

/// What the notice saw while the step ran,
/// and the ledger the commit's timing record takes it from.
use crate::scratch_commit_step::ScratchStepFacts;

/// The running notice:
/// the thread that watches the step,
/// the thread that writes its lines,
/// and the sender that ends the watching.
pub struct NativeCommitNotice {
    /// Ends the thread when it is sent to or dropped.
    stop: Option<Sender<()>>,
    /// The facts the thread answers when it ends,
    /// waited for no longer than `NOTICE_BOUND`.
    answers: std::sync::mpsc::Receiver<ScratchStepFacts>,
    /// The writer's end,
    /// waited for no longer than `NOTICE_BOUND` after the thread answered.
    written: std::sync::mpsc::Receiver<()>,
    /// How long the writer spent on the lines it finished.
    wrote: std::sync::Arc<Mutex<Duration>>,
    /// Set once the command is over,
    /// so the writer stops instead of printing a line about a step that ended.
    abandoned: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// What is reported when no thread ran,
    /// so a record still says that no ceiling was armed and why.
    unarmed: ScratchStepFacts,
    /// The step the thread watches.
    step: ChildNumber,
    /// Where a wait that must say why it still lasts hands its line,
    /// and nothing when no writer thread runs;
    /// it is the same queue every other line of this notice goes through,
    /// so one thread writes and two lines never come inside one another.
    said: Option<SyncSender<String>>,
    /// The one decision this ceiling shares with the wait that owns the step.
    ///
    /// It outlives both enforcers' own answers:
    /// the notice's thread reads it at every look,
    /// the wait reads and writes it when its ceiling comes,
    /// and the commit takes it for the words of a stop.
    the_stall: Arc<TheStallDecidedOnce>,
}

impl NativeCommitNotice {
    /// A notice that prints nothing:
    /// the commit opens an editor,
    /// or no thread could be started.
    fn silent(ceiling: Option<Duration>) -> Self {
        let (_no_lines, written) = channel::<()>();
        let (_no_answer, answers) = channel::<ScratchStepFacts>();
        return Self {
            stop: None,
            answers,
            written,
            wrote: std::sync::Arc::default(),
            abandoned: std::sync::Arc::default(),
            unarmed: ScratchStepFacts::unarmed(ceiling),
            step: ChildNumber::not_started(),
            said: None,
            the_stall: Arc::new(TheStallDecidedOnce::new()),
        };
    }

    /// What:
    ///  Keep the number of the `git commit` this notice watches.
    /// Why:
    ///   A ceiling ends the step it watches and nothing else this process started:
    /// the landing service runs one process for every landing,
    /// so another landing's Git is a child of the same process at the same moment.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// notice.stepStarted(pid);
    /// ```
    pub fn step_started(
        &self,
        pid: i32,
    ) {
        self.step.publish(pid);
    }

    /// What:
    ///  The spot the step's number is published into,
    ///  for the call that starts the step.
    /// Why:
    ///   The code that starts a child takes no callback from the code that watches it:
    /// the start publishes the number into this,
    /// and the notice reads it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// guard.runChildNaming(command, notice.stepNumber());
    /// ```
    pub fn step_number(&self) -> &ChildNumber {
        return &self.step;
    }

    /// What:
    ///  The ceiling this step is under,
    ///  counted from the moment it starts,
    ///  or nothing when a message may be typed into an editor in it.
    /// Why:
    ///   The wait that owns the step enforces this same bound,
    /// so it needs the number even when no notice thread runs,
    /// and an editor-opening commit has none at all,
    /// because the person is the wait.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// notice.ceilingThatApplies(); // number | undefined
    /// ```
    pub fn ceiling_that_applies(&self) -> Option<Duration> {
        return self.unarmed.ceiling;
    }

    /// What:
    ///  Where a wait that must say why it still lasts hands its line.
    /// Why:
    ///   The wait that stops a step the kernel then holds is the only code that keeps waiting,
    /// and the lines that explain that wait must reach the person even when no notice thread was started.
    ///  The writer thread is this notice's,
    ///  and this is the queue it takes whole lines from.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// notice.linesForTheWait(); // Queue<string> | undefined
    /// ```
    pub fn lines_for_the_wait(&self) -> Option<&SyncSender<String>> {
        return self.said.as_ref();
    }

    /// What:
    ///  The one decision this ceiling shares with the wait that owns the step.
    /// Why:
    ///   Two enforcers of one ceiling must not decide the stand-aside separately
    ///  (`scratch_step_stall_decided_once.rs`),
    ///  and the commit takes the decision for the words of a stop.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// notice.theStallDecidedOnce(); // TheStallDecidedOnce
    /// ```
    pub fn the_stall_decided_once(&self) -> &TheStallDecidedOnce {
        return self.the_stall.as_ref();
    }

    /// What:
    ///  Starts the notice for a `git commit` that is about to run.
    /// Why:
    ///   A commit that opens the editor gets none;
    /// a failure to start the thread leaves the step to the wait that owns it,
    /// which ends the child at this ceiling itself,
    /// and prints no line of its own,
    /// because there is no thread to print one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const notice = NativeCommitNotice.start({ snapshot, commitArguments, cadence });
    /// ```
    pub fn start(
        snapshot: Option<&ConfigSnapshot>,
        commit_arguments: &[OsString],
        cadence: Cadence,
        sink: Box<dyn Write + Send>,
    ) -> Self {
        return Self::start_within(NoticeStart {
            snapshot,
            commit_arguments,
            cadence,
            ceiling: crate::wait_ceilings::SCRATCH_COMMIT_STEP_CEILING,
            sink,
        });
    }

    /// Starts the notice with the ceiling given.
    pub fn start_within(start: NoticeStart<'_>) -> Self {
        let NoticeStart {
            snapshot,
            commit_arguments,
            cadence,
            ceiling,
            sink,
        } = start;
        if opens_editor(commit_arguments) {
            // The person is the wait, and a message typed into an editor would be lost by a stop.
            return Self::silent(None);
        }
        let signs: bool = signs_commit(snapshot, commit_arguments);
        let (stop, ended) = channel::<()>();
        let (lines, waiting) = sync_channel::<String>(LINE_QUEUE);
        let (told_written, written) = channel::<()>();
        let (told_answers, answers) = channel::<ScratchStepFacts>();
        let wrote: std::sync::Arc<Mutex<Duration>> = std::sync::Arc::default();
        // The writer's own copy of the time it spends, so the notice thread's figures can say it without waiting for a write.
        let spending: std::sync::Arc<Mutex<Duration>> = wrote.clone();
        let abandoned: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::default();
        let stand_down: std::sync::Arc<std::sync::atomic::AtomicBool> = abandoned.clone();
        let the_one_decision: Arc<TheStallDecidedOnce> = Arc::new(TheStallDecidedOnce::new());
        let watching_the_stall: Arc<TheStallDecidedOnce> = the_one_decision.clone();
        let began: Instant = Instant::now();
        let step: ChildNumber = ChildNumber::not_started_pinned();
        let watched: ChildNumber = step.clone();
        // The lines are written here and never on the thread that enforces the ceiling:
        // a reader that stopped reading then holds this thread, and the commit does not wait for it.
        let writer = std::thread::Builder::new()
            .name(String::from("native-commit-notice-writer"))
            .spawn(move || {
                let mut to_write: Box<dyn Write + Send> = sink;
                for line in waiting {
                    // The command is over: the line it was handed and every line behind it say nothing a reader can act on.
                    if stand_down.load(std::sync::atomic::Ordering::SeqCst) {
                        break;
                    }
                    let writing: Instant = Instant::now();
                    let _written = to_write.write_all(line.as_bytes());
                    let _flushed = to_write.flush();
                    let mut spent = spending
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    *spent = spent.saturating_add(writing.elapsed());
                }
                let _told = told_written.send(());
            });
        // What: A control refuses the notice thread here, as a process at its thread limit answers `pthread_create` with `EAGAIN`.
        // Why:  The refusal cannot be produced on demand,
        //       and a notice with no thread answers exactly what it answers when the system refuses one:
        //       it enforces nothing,
        //       so the wait that owns the step ends it at the ceiling
        //       (`child_awaited_until_it_exits_the_stop_or_the_ceiling.rs`).
        //       The writer thread is made either way, so the wait that keeps waiting can still say why it does.
        let no_notice_thread: bool = if a_thread_is_refused_now().is_some() {
            // No thread kept this sender, and a wait that ends the step itself answers what it saw.
            drop(told_answers);
            true
        } else {
            let sending: SyncSender<String> = lines.clone();
            let spawned = std::thread::Builder::new()
                .name(String::from("native-commit-notice"))
                .spawn(move || {
                    return watch_the_step(TheWatch {
                        cadence,
                        ceiling,
                        signs,
                        sending,
                        ended,
                        told_answers,
                        watched,
                        the_stall: watching_the_stall,
                        began,
                    });
                });
            spawned.is_err()
        };
        // The writer runs either way: the wait that keeps waiting says why through it, and a notice thread the system refused
        //  leaves that wait as the only voice left (`process/wait_after_the_stop_saying_the_kernel_holds_it.rs`).
        let said: Option<SyncSender<String>> = match writer {
            Ok(_writes) => Some(lines),
            // The system refused the writer too, so no line of this notice can be written and none is handed over.
            Err(_refused) => None,
        };
        return Self {
            stop: if no_notice_thread { None } else { Some(stop) },
            answers,
            written,
            wrote,
            abandoned,
            unarmed: ScratchStepFacts::unarmed(Some(ceiling)),
            step,
            said,
            the_stall: the_one_decision,
        };
    }

    /// Ends the notice once the step has ended,
    /// and answers what it saw:
    /// how long the step had lasted when the notice stopped it,
    /// how long the notice's own end took,
    /// and what ran below the step.
    /// Why:
    ///   Both waits here are bounded (`NOTICE_BOUND`):
    /// the line is advisory,
    /// so neither a reader that stopped reading nor a notice thread that cannot answer may hold the commit,
    /// which is finding 3 of the 2026-11-09 process and child lifetime audit.
    ///  A thread that does not answer leaves a record that says a ceiling was armed and no figure of it,
    /// rather than a record of zeroes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const began = now(); notice.stop(); const seen = notice.answerWithin(bound); seen.notice = now() - began; return seen;
    /// ```
    pub fn finish(mut self) -> ScratchStepFacts {
        let began: Instant = Instant::now();
        if let Some(stop) = self.stop.take() {
            let _sent = stop.send(());
        }
        let mut facts: ScratchStepFacts = match self.answers.recv_timeout(NOTICE_BOUND) {
            Ok(heard) => heard,
            // No thread ran: the commit may open an editor, or the system refused the thread.
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => self.unarmed,
            // The thread did not answer in time: a ceiling was armed and no figure of it is known,
            // which is a different record from one whose counters are zero.
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                let mut unanswered: ScratchStepFacts = self.unarmed;
                unanswered.armed = unanswered.ceiling.is_some();
                unanswered
            }
        };
        // The line already handed to the writer is written if the reader takes it within the same bound, and no longer.
        // The wait's own sender goes here too, so the writer's queue ends with the command rather than with the last field of this value.
        drop(self.said.take());
        let _written = self.written.recv_timeout(NOTICE_BOUND);
        // The command is over, so the writer prints nothing more: a line it still holds is dropped when its write returns,
        // and the thread ends with it instead of living on beside a reader that stopped reading.
        self.abandoned
            .store(true, std::sync::atomic::Ordering::SeqCst);
        facts.wrote = *self
            .wrote
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        facts.notice = began.elapsed();
        return facts;
    }
}

/// The line and the signing question are controlled in the test module.
#[cfg(test)]
#[path = "native_commit_waiting_notice_tests.rs"]
mod tests;
