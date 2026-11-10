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

use crate::wrapper_names_old_and_new::COMMIT_TIMING_LOG;
/// Why this commit did not land,
/// which the record states beside how it went and what it exited with.
use crate::commit_stop_reason::{CommitStopReason, stop_json};
use crate::commit_index_builds::{IndexBuild, drain_index_builds, index_builds_json};
/// The preparation's own work,
/// by name,
/// as its sites note it.
use crate::prepare_work_spans::{PrepareSpan, drain_spans, spans_json};
use crate::commit_prepare_part_durations::{PreparePart, drain_prepare_parts, prepare_parts_json};
/// The build's own stretches,
/// by name,
/// kept apart from the preparation's so neither reading is read as the other.
use crate::commit_build_part_durations::{build_parts_json, drain_build_parts};
use crate::time_spent_outside_the_phases::{OutsideThePhases, outside_the_phases};
use crate::common_directory_of_a_git_directory::common_directory_of;
/// What the notice of the scratch `git commit` saw while the step ran,
/// one entry per build.
use crate::scratch_commit_step::{ScratchStepFacts, drain_scratch_steps, scratch_steps_json};
use crate::diagnostic_log::debug;
use crate::log_generations_kept_to_a_bound::UNPINNED_LOG;
use crate::log_lines_appended_without_a_lock::append_line;
use crate::iso_time::now_iso;
use crate::json_record::{quote, string_array};
use crate::quiet_ledger::{drain, longest_healthy, object_json};
use crate::transaction_step_durations::{StepFigure, drain_steps, steps_json};
use crate::service_instance::instance_of;
use crate::service_protocol::{LandingFigures, RELEASE};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The file inside the common Git directory that collects the timing records of every worktree of the repository.
/// A full file moves aside to the next free generation
/// (`monochromatic-git-policy-cli-commit-timing.jsonl.<number>`),
/// and the generations are kept up to `LOG_HISTORY_BYTES`.
pub const TIMING_FILE: &str = COMMIT_TIMING_LOG.current;

/// The version of the record's fields;
/// it changes only when a field's meaning changes,
/// never when a field is added.
pub const TIMING_SCHEMA_VERSION: u32 = 1;

/// Module tag for diagnostics.
const TAG: &str = "commitTiming";

/// The word written for a release a service did not announce (the services of the earliest releases sent no release).
const UNANNOUNCED: &str = "unannounced";

/// What:
///  What the client did to reach a service that speaks its protocol and was started from its release.
/// Why:
///   The 17-second commit began with a service of another release being asked to retire;
/// the tries,
/// the wait for a start,
/// the
///  retires sent and the releases that answered tell that apart from a slow landing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReachTrace = { tries: number; refused: number; startWaited: number; retiresSent: number; releasesSeen: string[]; answeredBy?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReachTrace {
    /// Connection attempts made,
    /// each one a `connect` call.
    pub tries: usize,
    /// Attempts that found nothing listening or no greeting,
    /// which is the service being started by the manager.
    pub refused: usize,
    /// Time from the first attempt of a reach to the greeting,
    /// summed over the reaches that needed more than one attempt.
    pub start_waited: Duration,
    /// Retire messages sent to services of another release.
    pub retires_sent: usize,
    /// The release of every greeting,
    /// in the order received,
    /// without repeats.
    pub releases_seen: Vec<String>,
    /// The release of the service that finally took the request.
    pub answered_by: Option<String>,
    /// Time spent reaching,
    /// retire rounds included.
    pub elapsed: Duration,
}

impl ReachTrace {
    /// Notes the release a greeting named.
    pub fn saw_release(&mut self, release: Option<&str>) {
        let word: String = release.unwrap_or(UNANNOUNCED).to_string();
        if !self.releases_seen.contains(&word) {
            self.releases_seen.push(word);
        }
    }
}

/// What:
///  What a waiting notice of the service says of the holder of the index lock.
/// Why:
///   The notice is the only thing about the holder that reaches the client,
/// and the record should say whether the wait was for a process proven to hold the lock or for one nobody could prove alive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holderKindOf(text) // 'proven-alive' | 'unproven'
/// ```
pub fn holder_kind_of(text: &str) -> &'static str {
    return if text.contains("(proven by ") { "proven-alive" } else { "unproven" };
}

/// What:
///  The kind of holder a wait was for after one more waiting notice,
/// given the kind the earlier notices gave.
/// Why:
///  A proof counts when it comes and is never undone:
/// the notices after the one that proved the holder (the way out of a wait that reached its ceiling,
/// the service's own notice that it is inside a write the moment the holder lets go) say nothing of the holder,
/// and the last notice must not decide what kind of holder the wait was for
/// (the heartbeat of the connection and the holder's release meet at the same instant in the control that releases it after a
/// whole number of beats).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holderKindAfter(earlier, text) // earlier === 'proven-alive' ? 'proven-alive' : holderKindOf(text)
/// ```
pub fn holder_kind_after(earlier: Option<&'static str>, text: &str) -> &'static str {
    return match earlier {
        Some("proven-alive") => "proven-alive",
        Some(_) | None => holder_kind_of(text),
    };
}

/// What:
///  What the client did to hand one request to the service and get its answer.
/// Why:
///   The queue wait,
/// the wait for a foreign index lock and the landing itself all happen between the send and the answer;
///  the service's own log line for the request splits that wait,
/// and this record says how long the whole of it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SubmitTrace = { total: number; reach: ReachTrace; answerWait: number; positions: number[]; waitingNotices: number; holderKind?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SubmitTrace {
    /// The whole of the sends,
    /// reaches included.
    pub total: Duration,
    /// How the service was reached.
    pub reach: ReachTrace,
    /// Time from a request being written to the answer (or the connection breaking),
    /// summed over the sends.
    pub answer_wait: Duration,
    /// The queue positions the service reported,
    /// in order,
    /// without repeats.
    pub positions: Vec<usize>,
    /// Notices the service sent while it waited for another program's hold on the index.
    pub waiting_notices: usize,
    /// What the last of those notices said of the holder:
    /// `proven-alive` when it named a process proven to hold the lock,
    /// `unproven` when it said the owner could not be proven alive.
    pub holder_kind: Option<&'static str>,
    /// Times the connection broke before an answer and the same request was sent again.
    pub reconnects: usize,
    /// Requests written.
    pub sends: usize,
    /// What the landing of this commit spent and held,
    /// as the service's answer stated it;
    /// `None` for a service of a release that sends no figures,
    /// so a reader never takes zeros for a figure nobody measured.
    pub landing: Option<LandingFigures>,
}

/// Where the record goes and which service instance it belongs to.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Place {
    /// The worktree's own Git directory.
    git_dir: PathBuf,
    /// The common Git directory,
    /// which holds the file.
    common_dir: PathBuf,
    /// The instance name that `journalctl --user --unit=cli-git@<instance>.service` takes.
    instance: String,
}

/// The whole milliseconds of a duration.
fn whole_milliseconds(elapsed: Duration) -> u128 {
    return elapsed.as_millis();
}

/// The JSON word for a field that has no value.
fn null_word() -> String {
    return String::from("null");
}

/// What:
///  The landing's own figures as one JSON object,
/// or the JSON word for no value when the service sent none.
/// Why:
///   The record of a slow commit then explains its landing on its own:
/// which read spent the time,
/// how long the landing held what others queued on,
/// how many commits queued behind it
/// and how many syncs it made with the time of each,
/// without a second file read by request identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landingJson = (l) => l === null ? 'null' : JSON.stringify({ lockHoldMs: l.lockHoldMs, queuedBehind: l.queuedBehind, readsMs: l.readsMs, reads: Object.fromEntries(l.reads), syncs: l.syncs });
/// ```
fn landing_json(landing: Option<&LandingFigures>) -> String {
    let Some(figures) = landing else { return null_word() };
    let mut reads: Vec<String> = Vec::new();
    for (name, ms) in &figures.reads {
        reads.push(format!("{}:{ms}", quote(name.as_str())));
    }
    let mut syncs: Vec<String> = Vec::new();
    for sync in &figures.syncs {
        syncs.push(format!("{{\"what\":{},\"waited\":{},\"ms\":{}}}", quote(sync.what.as_str()), sync.waited, sync.ms));
    }
    return format!(
        "{{\"lockHoldMs\":{},\"queuedBehind\":{},\"readsMs\":{},\"reads\":{{{}}},\"syncs\":[{}]}}",
        figures.lock_hold_ms,
        figures.queued_behind,
        figures.reads_ms,
        reads.join(","),
        syncs.join(","),
    );
}

impl Place {
    /// The instance as a JSON string.
    fn instance_json(&self) -> String {
        return quote(self.instance.as_str());
    }

    /// The worktree's own Git directory as a JSON string.
    fn git_dir_json(&self) -> String {
        return quote(&self.git_dir.to_string_lossy());
    }
}

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
    began: Instant,
    /// Milliseconds since the Unix epoch at the start,
    /// for finding the commit among the service's journal lines.
    start_millis: u128,
    /// Known once the worktree's Git directory was found.
    place: Option<Place>,
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
    /// a rewrite and a reload.
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
    /// Leaving the push and the upkeep for the service and waking it.
    pub handoff: Duration,
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
            quiet: Vec::new(),
            outside: OutsideThePhases::default(),
            prepare_parts: Vec::new(),
            prepare_parts_dropped: 0,
            build_parts: Vec::new(),
            build_parts_dropped: 0,
            build_began: None,
            route: Duration::ZERO,
            steps: Vec::new(),
            steps_dropped: 0,
        };
    }

    /// Names the worktree whose Git directory the command found;
    /// a command outside a repository writes no record.
    pub fn place_at(&mut self, git_dir: &Path) {
        self.place = Some(Place { git_dir: git_dir.to_path_buf(), common_dir: common_directory_of(git_dir), instance: instance_of(git_dir) });
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
        let Some(began) = self.build_began.take() else { return };
        self.build += began.elapsed();
    }

    /// What the phases leave uncovered out of `total`.
    fn other(&self, total: Duration) -> Duration {
        let covered: Duration = self.route + self.prepare + self.build + self.provision + self.submit.total + self.after_landing + self.gate + self.handoff;
        return total.saturating_sub(covered);
    }

    /// What:
    ///  The record as one line of JSON ending in a newline.
    /// Why:
    ///   One `write` call per line keeps lines of concurrent commits from interleaving;
    /// a branch name with a quote or a
    ///  newline is escaped by `quote`,
    /// so the line stays one line.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// JSON.stringify(record) + '\n'
    /// ```
    pub fn line(&self, total: Duration, exit_code: i32) -> String {
        let place: Option<&Place> = self.place.as_ref();
        let branch: String = self.reference.as_deref().map_or_else(null_word, quote);
        let answered_by: String = self.submit.reach.answered_by.as_deref().map_or_else(null_word, quote);
        let request: String = self.request.as_deref().map_or_else(null_word, quote);
        let level: &str = if self.outcome == "stopped" { "warn" } else { "info" };
        let positions: Vec<String> = self.submit.positions.iter().map(usize::to_string).collect();
        return format!(
            "{{\"schemaVersion\":{TIMING_SCHEMA_VERSION},\"time\":{},\"level\":\"{level}\",\"startMillis\":{},\"pid\":{},\"release\":{},\"instance\":{},\"gitDir\":{},\
             \"requestId\":{request},\"branch\":{branch},\"outcome\":{},\"exitCode\":{exit_code},\"stop\":{},\"totalMs\":{},\
             \"routeMs\":{},\"prepareMs\":{},\"attempts\":{},\"prepareSpans\":{},\"prepareSpansDropped\":{},\"indexBuilds\":{},\"scratchSteps\":{},\
             \"buildMs\":{},\"buildParts\":{},\"buildPartsDropped\":{},\"provisionMs\":{},\
             \"submitMs\":{},\"sends\":{},\"reachMs\":{},\"connectTries\":{},\"refusedConnects\":{},\"startWaitMs\":{},\
             \"updateRounds\":{},\"releasesSeen\":{},\"answeredByRelease\":{answered_by},\
             \"answerWaitMs\":{},\"positions\":[{}],\"waitingNotices\":{},\"lockHolderKind\":{},\"reconnects\":{},\
             \"afterLandingMs\":{},\"gateMs\":{},\"handoffMs\":{},\"otherMs\":{},\
             \"scratchRemovalMs\":{},\"removalsThatDidNotFinish\":{},\"stderrMs\":{},\
             \"prepareParts\":{},\"preparePartsDropped\":{},\
             \"longestQuietMs\":{},\"quietMs\":{},\"steps\":{},\"stepsDropped\":{},\"landing\":{}}}\n",
            quote(now_iso().as_str()),
            self.start_millis,
            std::process::id(),
            quote(RELEASE),
            place.map_or_else(null_word, Place::instance_json),
            place.map_or_else(null_word, Place::git_dir_json),
            quote(self.outcome),
            stop_json(self.stop.as_ref()),
            total.as_millis(),
            self.route.as_millis(),
            self.prepare.as_millis(),
            self.attempts,
            spans_json(self.prepare_spans.as_slice()),
            self.prepare_spans_dropped,
            index_builds_json(self.index_builds.as_slice()),
            scratch_steps_json(self.scratch_steps.as_slice()),
            self.build.as_millis(),
            build_parts_json(self.build_parts.as_slice()),
            self.build_parts_dropped,
            self.provision.as_millis(),
            self.submit.total.as_millis(),
            self.submit.sends,
            self.submit.reach.elapsed.as_millis(),
            self.submit.reach.tries,
            self.submit.reach.refused,
            self.submit.reach.start_waited.as_millis(),
            self.submit.reach.retires_sent,
            string_array(self.submit.reach.releases_seen.as_slice()),
            self.submit.answer_wait.as_millis(),
            positions.join(","),
            self.submit.waiting_notices,
            self.submit.holder_kind.map_or_else(null_word, quote),
            self.submit.reconnects,
            self.after_landing.as_millis(),
            self.gate.as_millis(),
            self.handoff.as_millis(),
            self.other(total).as_millis(),
            self.outside.scratch_removal.as_millis(),
            self.outside.removals_that_did_not_finish,
            self.outside.standard_error.as_millis(),
            prepare_parts_json(self.prepare_parts.as_slice()),
            self.prepare_parts_dropped,
            longest_healthy(self.quiet.as_slice()),
            object_json(self.quiet.as_slice()),
            steps_json(self.steps.as_slice()),
            self.steps_dropped,
            landing_json(self.submit.landing.as_ref()),
        );
    }

    /// What:
    ///  Appends the record to the repository's timing file,
    /// best effort.
    /// Why:
    ///   The command's result never depends on the record:
    /// a file that cannot be written is only a debug line.
    /// The directory
    ///  is never created,
    /// and nothing is flushed to disk,
    /// so the record costs one `write`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// recordTiming(timing, exitCode)
    /// ```
    pub fn finish(&self, exit_code: i32) {
        let Some(place) = self.place.as_ref() else { return };
        let target: PathBuf = place.common_dir.join(TIMING_FILE);
        // The ledger holds what this command's Git steps and tools showed; the record takes it and the next command starts empty.
        let mut with_quiet: CommitTiming = self.clone();
        // A build no seam closed is the phase the command ended in, so its time is stated here rather than left to the remainder.
        with_quiet.close_an_open_build();
        with_quiet.quiet = drain();
        (with_quiet.steps, with_quiet.steps_dropped) = drain_steps(self.began);
        with_quiet.index_builds = drain_index_builds();
        (with_quiet.prepare_spans, with_quiet.prepare_spans_dropped) = drain_spans();
        with_quiet.outside = outside_the_phases();
        (with_quiet.prepare_parts, with_quiet.prepare_parts_dropped) = drain_prepare_parts();
        (with_quiet.build_parts, with_quiet.build_parts_dropped) = drain_build_parts();
        with_quiet.scratch_steps = drain_scratch_steps();
        let line: String = with_quiet.line(self.began.elapsed(), exit_code);
        if let Err(error) = append_line(target.as_path(), line.as_bytes(), &UNPINNED_LOG) {
            debug(TAG, &format!("the timing record could not be written to {}: {error}", target.display()));
        }
    }
}

/// The line,
/// the sum rule,
/// and the escaping.
#[cfg(test)]
#[path = "commit_timing_tests.rs"]
mod tests;
