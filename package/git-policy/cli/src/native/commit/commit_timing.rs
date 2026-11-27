//! What:
//!  The record of where one wrapped `git commit` spent its time,
//! appended to a size-capped file in the common Git directory
//!  beside the discard audit:
//! preparation,
//! building,
//! setting up the service,
//! reaching it,
//! waiting for its answer,
//! the work
//!  after landing,
//! the post-commit gate and the hand-over of the push.
//! Why:
//!  A commit that took seventeen seconds could not be split afterwards,
//! because nothing outside verbose mode said which phase
//!  held the time.
//! The numbers are taken from the monotonic clock as the commit runs (one `Instant` read per phase edge,
//! no
//!  Git start,
//! no flush to disk) and written once,
//! in one append,
//! when the command ends.
//! The record goes to the file only:
//!  the person's terminal never shows it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await appendFile(join(commonDir, 'monochromatic-git-policy-cli-commit-timing.jsonl'), JSON.stringify(timing) + '\n');
//! ```

/// How much of a command's total the record names,
/// which the record's own methods answer with.
pub use crate::commit_accounting::Accounting;
use crate::commit_index_builds::IndexBuild;
use crate::commit_prepare_part_durations::PreparePart;
/// Why this commit did not land,
/// which the record states beside how it went and what it exited with.
use crate::commit_stop_reason::CommitStopReason;
/// A duration in whole milliseconds,
///  which the record's start stamp is taken in.
use crate::commit_timing_json_words::whole_milliseconds;
/// Where the record goes and which service instance it belongs to.
use crate::commit_timing_place::Place;
/// What the client did to hand one request to the service and get its answer,
/// kept in a file of its own,
/// re-exported where every caller of it already looks.
pub use crate::commit_timing_submit::SubmitTrace;
use crate::common_directory_of_a_git_directory::common_directory_of;
/// The preparation's own work,
/// by name,
/// as its sites note it.
use crate::prepare_work_spans::PrepareSpan;
/// What the notice of the scratch `git commit` saw while the step ran,
/// one entry per build.
use crate::scratch_commit_step::ScratchStepFacts;
use crate::service_instance::instance_of;
/// What the command paid outside its phases.
use crate::time_spent_outside_the_phases::OutsideThePhases;
use crate::transaction_step_durations::StepFigure;
use crate::wrapper_names_old_and_new::COMMIT_TIMING_LOG;
/// `Path` is a borrowed filesystem path;
/// `Duration` is an elapsed stretch and `Instant` a monotonic reading.
use std::path::Path;
use std::time::{
    Duration,
    Instant,
};
/// The file inside the common Git directory that collects the timing records of every worktree of the repository.
/// A full file moves aside to the next free generation
/// (`monochromatic-git-policy-cli-commit-timing.jsonl.<number>`),
/// and the generations are kept up to `LOG_HISTORY_BYTES`.
pub const TIMING_FILE: &str = COMMIT_TIMING_LOG.current;

/// The version of the record's fields;
/// it changes only when a field's meaning changes,
/// never when a field is added.
pub const TIMING_SCHEMA_VERSION: u32 = 1;

/// Module tag for diagnostics,
///  which the append of this record reports a failure under.
pub(crate) const TAG: &str = "commitTiming";

/// What the client did to reach a service that speaks this release,
/// with the waits that reaching cost,
/// the words a waiting notice said of a foreign index holder,
/// and the word this release writes for a service that announced no release.
pub use crate::commit_reach_trace::{
    ReachTrace,
    UNANNOUNCED,
    holder_kind_after,
    holder_kind_of,
};

/// What:
///  The phases of one wrapped commit,
/// filled as the commit runs and written when it ends.
/// Why:
///   Each field is one disjoint stretch of the command,
/// so the phases add up to no more than the total,
/// and what they do not
///  cover is reported as `otherMs` instead of being hidden.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommitTiming = { began: number; prepare: number; build: number; provision: number; submit: SubmitTrace; /* ... */ };
/// ```
#[derive(Clone, Debug)]
pub struct CommitTiming {
    /// The monotonic start.
    pub(crate) began: Instant,
    /// Milliseconds since the Unix epoch at the start,
    /// for finding the commit among the service's journal lines.
    pub(crate) start_millis: u128,
    /// Known once the worktree's Git directory was found.
    pub(crate) place: Option<Place>,
    /// The branch (or `HEAD`) the first preparation found.
    pub reference: Option<String>,
    /// The identity the commit's request carries,
    /// which the service's `answered` line states as `request` and as the
    /// journal field `CLI_GIT_REQUEST_ID`;
    /// the key that joins this record to the service's.
    pub request: Option<String>,
    /// How the command ended:
    /// `landed`,
    /// `ended` (blocked or declined,
    /// nothing landed) or `stopped` (a failure).
    pub outcome: &'static str,
    /// What:
    ///  Why the commit did not land,
    ///  by the route that stopped it:
    /// the policy that refused the content,
    /// the transform that rejected the command,
    /// the engine failure,
    /// the Git step and its status,
    /// or real Git's own exit.
    /// Why:
    ///  `outcome` and `exitCode` say that nothing landed and what the process returned,
    ///  never whether a policy correctly refused content or a machine failed,
    ///  and the stopped records of the live log could not be told apart for exactly that reason.
    /// `None` for a commit that landed.
    pub stop: Option<CommitStopReason>,
    /// Preparations made:
    /// one,
    /// and one more for each time the service sent the commit back or Git built it again.
    pub attempts: usize,
    /// How each build of the private commit index was made and whether it kept the real index's cached stat data,
    /// in the order built,
    /// taken from the ledger when the record is written (`commit_index_builds.rs`):
    /// empty for a commit that stopped before it built an index.
    pub index_builds: Vec<IndexBuild>,
    /// What the preparation's own work did,
    /// by name,
    /// with how often,
    /// the whole of its time and its longest run,
    /// taken from the ledger when the record is written (`prepare_work_spans.rs`):
    /// empty for a commit that noted no step.
    /// Each name is a literal of this crate:
    /// the index's stat check,
    /// its read,
    /// its copies,
    /// the answer of whether it already holds the base,
    /// the writes of corrected entries,
    /// the candidate set a pass's content policies share,
    /// and every policy with the lifecycle point whose pass ran it.
    /// The record's `steps` hold the Git children these steps started,
    /// so a step that starts Git is stated twice:
    /// once as the work and once as the program it ran.
    pub prepare_spans: Vec<PrepareSpan>,
    /// How many step names came after the list reached its bound.
    pub prepare_spans_dropped: usize,
    /// What the waiting notice of the scratch `git commit` saw in each build of it,
    /// in the order built
    /// (`scratch_commit_step.rs`):
    /// empty for a commit that ran no scratch step,
    /// and one entry per build otherwise.
    /// The record's own `git commit` step figure is the child alone,
    /// so the notice's join and the time a program below the step ran are here and never inside it.
    pub scratch_steps: Vec<ScratchStepFacts>,
    /// Policies,
    /// hooks,
    /// the private index,
    /// the tree and the objects of the commit,
    /// summed over the attempts.
    pub prepare: Duration,
    /// What the command spent before its first preparation began:
    /// finding the worktree's route to a service,
    /// this commit's identity,
    /// the standard input a pathspec file is read from,
    /// and the client's settings.
    /// Why:
    ///  No other phase covers that stretch,
    ///  so without this field its time is reported as time no phase covers.
    pub route: Duration,
    /// The message,
    /// the identities,
    /// the tree check and Git's own build of the commit,
    /// with the scratch hand-over.
    pub build: Duration,
    /// Setting up socket activation,
    /// which is the manager's unit comparison and,
    /// after an upgrade,
    /// a rename of the program link and the unloading of this worktree's unit.
    pub provision: Duration,
    /// Sending the request and waiting for the answer.
    pub submit: SubmitTrace,
    /// The summary line,
    /// the names it needs,
    /// and the files a policy corrected,
    /// after the branch moved.
    pub after_landing: Duration,
    /// The post-commit gate.
    pub gate: Duration,
    /// Leaving the push and the upkeep for the service and waking it,
    /// with the queue's own records of that push made and its held record promoted.
    pub handoff: Duration,
    /// What:
    ///  What the command spent after its last phase:
    /// returning from the commit and letting go of everything it held.
    /// Why:
    ///  The record's total is read when the record is written,
    ///  which is after the command has unwound,
    ///  so without this field that stretch was reported as time no phase covers,
    ///  and on a machine where one file call takes a second it reached eight and fourteen seconds
    ///  (`doc/planning/cli-git-commit-latency-alpha-11-against-alpha-10.md`).
    pub closing: Duration,
    /// The longest quiet gap of every Git step and tool this command ran,
    /// by step name,
    /// taken from the ledger when the record
    ///  is written;
    /// a step that was ended for being quiet is listed apart from the steps that ended by themselves.
    pub quiet: Vec<(String, Duration)>,
    /// What the command paid outside its phases,
    /// taken from the ledger when the record is written:
    ///  removing scratch trees,
    /// and writing to standard error.
    pub outside: OutsideThePhases,
    /// How long each stretch of the preparation held no Git child,
    /// by name,
    /// in the order the names first appeared,
    /// taken from the ledger when the record is written.
    pub prepare_parts: Vec<PreparePart>,
    /// How many stretches of the preparation came after the list reached its bound.
    pub prepare_parts_dropped: usize,
    /// How long each stretch of the build phase held no Git child,
    /// by name,
    /// in the order the names first appeared,
    /// taken from the ledger when the record is written.
    pub build_parts: Vec<PreparePart>,
    /// How many stretches of the build phase came after the list reached its bound.
    pub build_parts_dropped: usize,
    /// When the build phase began,
    /// while it is running.
    /// Why:
    ///  Every ending of the build must state the time it spent,
    ///  and a stop inside it leaves the phase before the figure is taken,
    ///  so the open build is closed where the command stops instead.
    ///  A build that no seam closed is closed when the record is written,
    ///  so its time is never counted as time no phase covers.
    build_began: Option<Instant>,
    /// When the closing stretch began,
    /// while the command is ending.
    /// Why:
    ///  The end of the work is the last phase edge,
    ///  and the record is written after the command has unwound,
    ///  so the stretch between them is measured here rather than counted as time no phase covers.
    closing_began: Option<Instant>,
    /// Every Git step this command started,
    /// each with its program and subcommand,
    /// when it began (from the command's start) and how
    ///  long it ran,
    /// taken from the step ledger when the record is written (`transaction_step_durations.rs`).
    pub steps: Vec<StepFigure>,
    /// How many steps came after the list reached its bound.
    pub steps_dropped: usize,
}

impl CommitTiming {
    /// Starts the clocks:
    /// the monotonic one for the durations and the wall clock for the stamp.
    pub fn start() -> Self {
        let start_millis: u128 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(whole_milliseconds)
            .unwrap_or_default();
        return Self {
            began: Instant::now(),
            start_millis,
            place: None,
            reference: None,
            request: None,
            outcome: "stopped",
            stop: None,
            attempts: 0,
            index_builds: Vec::new(),
            prepare_spans: Vec::new(),
            prepare_spans_dropped: 0,
            scratch_steps: Vec::new(),
            prepare: Duration::ZERO,
            build: Duration::ZERO,
            provision: Duration::ZERO,
            submit: SubmitTrace::default(),
            after_landing: Duration::ZERO,
            gate: Duration::ZERO,
            handoff: Duration::ZERO,
            closing: Duration::ZERO,
            quiet: Vec::new(),
            outside: OutsideThePhases::default(),
            prepare_parts: Vec::new(),
            prepare_parts_dropped: 0,
            build_parts: Vec::new(),
            build_parts_dropped: 0,
            build_began: None,
            closing_began: None,
            route: Duration::ZERO,
            steps: Vec::new(),
            steps_dropped: 0,
        };
    }

    /// Names the worktree whose Git directory the command found;
    /// a command outside a repository writes no record.
    pub fn place_at(
        &mut self,
        git_dir: &Path,
    ) {
        self.place = Some(Place {
            git_dir: git_dir.to_path_buf(),
            common_dir: common_directory_of(git_dir),
            instance: instance_of(git_dir),
        });
    }

    /// What:
    ///  States what the command spent before its first preparation,
    ///  once.
    /// Why:
    ///   A rebuild prepares again,
    ///  and the stretch before the first preparation is the one that belongs to no phase;
    ///  a later attempt must not overwrite the figure with the time of the attempt before it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// if (timing.route === 0) timing.route = now() - timing.began;
    /// ```
    pub fn note_route(&mut self) {
        if self.route.is_zero() {
            self.route = self.began.elapsed();
        }
    }

    /// What:
    ///  Starts the build phase's clock.
    /// Why:
    ///   The build is closed by `end_build`,
    ///  by `close_an_open_build` at a stop,
    ///  and by the record's own writer if neither ran,
    ///  so the phase states its time however the command ends.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// timing.beginBuild();
    /// ```
    pub fn begin_build(&mut self) {
        self.close_an_open_build();
        self.build_began = Some(Instant::now());
    }

    /// What:
    ///  Ends the build phase and adds what it spent.
    /// Why:
    ///   The normal seam,
    ///  the phase being over when the commit is ready to be sent.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// timing.endBuild();
    /// ```
    pub fn end_build(&mut self) {
        self.close_an_open_build();
    }

    /// What:
    ///  Adds the time of a build that is still open,
    ///  and forgets it.
    /// Why:
    ///   A stop inside the build leaves the phase before its figure was taken,
    ///  and the record used to report that whole stretch as time no phase covers.
    ///  `doc/planning/cli-git-commit-latency-under-many-agents.md` holds the reading
    ///  in its section on what the build phase waits on.
    ///  Closing it where the command stops keeps the stop's own printing and removal out of it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// timing.closeAnOpenBuild();
    /// ```
    pub fn close_an_open_build(&mut self) {
        let Some(began) = self.build_began.take() else {
            return;
        };
        self.build += began.elapsed();
    }

    /// What:
    ///  Starts the clock of the stretch after the command's last phase,
    ///  once.
    /// Why:
    ///  Every wait a commit makes must be inside a named phase,
    ///  and the command's own unwinding is a wait like any other:
    ///  the record's total is read after it,
    ///  so naming it is what keeps the record's remainder at zero near the end of the command.
    ///  A command that stopped inside a phase never begins this stretch and states no closing time.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// if (timing.closingBegan === undefined) timing.closingBegan = now();
    /// ```
    pub fn begin_closing(&mut self) {
        if self.closing_began.is_none() {
            self.closing_began = Some(Instant::now());
        }
    }

    /// What:
    ///  Adds the closing stretch to its phase and forgets its clock,
    ///  so the record states it once.
    /// Why:
    ///   The record's writer is the last place that can close it:
    ///  the command that stopped inside a phase never began the closing stretch,
    ///  and a record written twice must not count the stretch twice.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// timing.closeTheClosing();
    /// ```
    pub fn close_the_closing(&mut self) {
        let Some(began) = self.closing_began.take() else {
            return;
        };
        self.closing += began.elapsed();
    }

    /// What:
    ///  How the phases account for the command's total:
    ///  the time their names cover and the time no name covers.
    /// Why:
    ///  A record whose remainder is large cannot split a slow commit,
    ///  which is the one thing it is written for;
    ///  the control that reads a real commit's record states a tolerance
    ///  and fails when the remainder passes it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const { named, unnamed } = timing.accounting(total);
    /// ```
    pub fn accounting(
        &self,
        total: Duration,
    ) -> Accounting {
        let unnamed: Duration = self.other(total);
        return Accounting {
            named: total.saturating_sub(unnamed),
            unnamed,
        };
    }

    /// What the phases leave uncovered out of `total`.
    pub(crate) fn other(
        &self,
        total: Duration,
    ) -> Duration {
        let covered: Duration = self.route
            + self.prepare
            + self.build
            + self.provision
            + self.submit.total
            + self.after_landing
            + self.gate
            + self.handoff
            + self.closing;
        return total.saturating_sub(covered);
    }
}

/// The line,
/// the sum rule,
/// and the escaping.
#[cfg(test)]
#[path = "commit_timing_tests.rs"]
mod tests;
