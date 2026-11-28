//! What:
//! How the route to a delegated control group is discovered:
//! the cgroup mount,
//! the user manager,
//! the slice,
//! and the limits the scope inherits.
//! Why:
//! Split out of `control_group.rs`,
//! which holds the containment of a run in a control group,
//! to keep that file under the code-line limit.
//! The items are re-exported from `control_group`,
//! so their paths there are unchanged.
//! The mount view a route records is `control_group_route_view.rs`,
//! and the programs the discovery runs are `control_group_route_child_runs.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control_group_route_discovery.ts: detectRoute(settings), the route types and the refusals
//! ```

/// What:
/// `use crate::control_group_route_child_runs::run_plain;` imports the short program the discovery asks its manager with.
/// Why:
/// The capability of this host is discovered by asking its manager,
/// and that question runs under one deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { run_plain } from 'crate/control_group_route_child_runs';
/// ```
use crate::control_group_route_child_runs::run_plain;
/// What:
/// `use crate::control_group_route_manager_reach::a_refusal_that_ends_the_asking;` imports the read that says nothing on this host
/// could carry an ask to a user manager.
/// Why:
/// A host with no manager refuses every ask at once,
/// so the one failure that host gives ends the discovery instead of spending the whole span of the asking again (issue 6629).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { aRefusalThatEndsTheAsking } from 'crate/control_group_route_manager_reach';
/// ```
use crate::control_group_route_manager_reach::a_refusal_that_ends_the_asking;
/// What:
/// `use crate::control_group_route_one_ask::{FailedAsk, WhereTheSliceIs, ask_where_the_slice_is_once, spent_its_bound,
/// the_manager_answered_a_start};` imports one
/// ask of the manager,
/// the words it may fail with,
/// and the reading that tells the manager's own refusal among them.
/// Why:
/// The asking again is a loop of that one ask,
/// and the file it lives in is split out to keep this one under the code-line limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { FailedAsk, WhereTheSliceIs, askWhereTheSliceIsOnce, spentItsBound, theManagerAnsweredAStart } from 'crate/control_group_route_one_ask';
/// ```
pub(crate) use crate::control_group_route_one_ask::{
    FailedAsk,
    WhereTheSliceIs,
    ask_where_the_slice_is_once,
    spent_its_bound,
    the_manager_answered_a_start,
};
/// What:
/// `use crate::inherited_limits::{InheritedLimits, caller_limits};` imports the limits a scope keeps from its caller and the read of them.
/// Why:
/// A scope that did not carry the caller's limits would loosen the run it bounds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { InheritedLimits, caller_limits } from 'crate/inherited_limits';
/// ```
use crate::inherited_limits::{
    InheritedLimits,
    caller_limits,
};
/// What:
/// `use std::fmt;` imports the standard library's formatting module,
/// which holds `Display` (a type that can be printed with `{}`) and `Formatter` (the printer it writes into).
/// Why:
/// A refusal is printed as the reason the delegated scope is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // String(value) and template literals
/// ```
use std::fmt;
/// What:
/// `use std::fs;` imports the standard library's file-system module so this file can name it unqualified.
/// Why:
/// The slice the manager reports must exist as a directory before it can be used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { statSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::io;` imports the standard library's input/output module so this file can name it unqualified.
/// Why:
/// Whether the cgroup tree is a v2 mount,
/// and which namespace it is,
/// are both answered as an `io::Result` that may fail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // an I/O failure is an ordinary thrown Error in TS
/// ```
use std::io;
/// What:
/// `use std::os::unix::fs::MetadataExt;` imports the trait that adds the inode number to file metadata.
/// Why:
/// The cgroup namespace of this process is named by the inode of `/proc/self/ns/cgroup`,
/// which is all this host offers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { statSync } from 'node:fs';   // statSync(...).ino
/// ```
use std::os::unix::fs::MetadataExt;
/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed and owned path types.
/// Why:
/// The slice directory is produced here and stored in the route,
/// so it is owned,
/// while the paths tested on the way are lent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::{
    Path,
    PathBuf,
};
/// What:
/// `use std::sync::Arc;` imports the atomically counted pointer type (its sibling `Rc` counts without atomics,
/// for one thread only).
/// Why:
/// One route's limits are read by every run that plans a scope on it,
/// so they are shared rather than copied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // shared object references in TS need no count
/// ```
use std::sync::Arc;
/// What:
/// `use std::time::{Duration, Instant};` imports a span of time and a reading of the monotonic clock (sibling of `Instant`:
/// `SystemTime`,
/// the wall clock,
/// which can jump).
/// Why:
/// A route carries the bound of its discovery question and the interval of its reaping check,
/// and an ask of the manager is timed so a failure that came before its own deadline is told from one that spent it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a number of milliseconds, and Date.now()
/// ```
use std::time::{
    Duration,
    Instant,
};

/// The check that a group directory can be killed through `cgroup.kill` (split out of this module).
pub(crate) use crate::control_group_route_child_runs::kill_file_present;
/// The mount view a route records (split out of this module).
pub(crate) use crate::control_group_route_view::{
    View,
    view_of,
};
/// Where the unified control group hierarchy is mounted.
pub(crate) const CGROUP_ROOT: &str = "/sys/fs/cgroup";

/// The file that names the control group of this process.
pub(crate) const OWN_CGROUP_FILE: &str = "/proc/self/cgroup";

/// The file system type number of a cgroup v2 mount (`CGROUP2_SUPER_MAGIC`,
/// ASCII "cgrp"),
/// from `statfs(2)`.
pub(crate) const CGROUP2_MAGIC: u64 = 0x6367_7270;

/// How long the stop waits for a change of `cgroup.events` before it looks for members to signal again.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// measurement beside it:
/// on 2026-11-04 `populated 0` followed `cgroup.kill` within 5 ms for ordinary members and 1 ms after a signal to a zombie leader,
/// so a wake-up every 50 ms only matters for a member that appears or survives the first signals.
pub(crate) const STRAGGLER_INTERVAL: Duration = Duration::from_millis(50);

/// How long the discovery of the manager's slice may take.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// measurement beside it:
/// `systemctl --user show app.slice` answered in a few milliseconds on 2026-11-04,
/// so ten seconds only ends a manager that does
/// not answer.
/// It bounds two things since 2026-11-27:
/// the ask of where the slice is,
/// and the start that waits a queued job out.
/// The start was measured against a container's own manager at 3.68 s,
/// 3.71 s,
/// 3.76 s and 3.78 s,
/// each the four seconds of the planted `TimeoutStopSec=4` stop it was queued behind,
/// so a slice whose stop is given the manager's default of ninety seconds is refused by this bound rather than waited for,
/// and the refusal names the program and this bound.
/// Since 2026-11-28 that refusal stops the call:
/// a start that spent this bound and left a job on the slice leaves no route,
/// because a scope made there is refused while the job stands and a scope launcher would wait for that job with no bound of its
/// own,
/// so the call stops and changes nothing rather than running the command behind another unit's stop (Medium 1 of the alpha.18
/// read).
pub(crate) const DISCOVERY_BOUND: Duration = Duration::from_secs(10);

/// How often the discovery asks whether `systemctl` has ended.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// not a measurement.
pub(crate) const DISCOVERY_POLL: Duration = Duration::from_millis(5);

/// How many random bytes make a scope name unique.
/// 16 bytes are 128 bits,
/// so a name repeated by chance is not a case to plan for.
pub(crate) const TOKEN_BYTES: usize = 16;

/// Why no control group could be had,
/// in words that name the missing capability.
/// Two kinds of refusal are told apart by `stops_the_call`:
/// one that means this host has no capability, which the run stands in for with a process group of its own,
/// and one that is a moment no wait of this tool ends, which stops the call instead.
#[derive(Clone, Debug)]
pub(crate) struct GroupUnavailable {
    /// The missing capability and what was tried.
    pub(crate) reason: String,
    /// Whether this refusal stops the call instead of taking the weaker path.
    pub(crate) stops_the_call: bool,
}

/// How the reason reads to a person.
impl fmt::Display for GroupUnavailable {
    /// What:
    /// `fn fmt(&self, formatter)` writes the reason.
    /// Why:
    /// The one-line notice of the weaker path is this text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        return write!(formatter, "{}", self.reason);
    }
}

/// What the discovery of the route uses,
/// so a test can make the host look like one that lacks a capability.
#[derive(Clone, Debug)]
pub(crate) struct RouteSettings {
    /// The program that makes the scope.
    pub(crate) systemd_run: String,
    /// The program that asks the manager where the slice is.
    pub(crate) systemctl: String,
    /// Where the cgroup v2 hierarchy is mounted.
    pub(crate) cgroup_root: PathBuf,
    /// The slice the scopes are made in.
    pub(crate) slice: String,
    /// Variables set (`Some`) or removed (`None`) for the program that asks the manager.
    pub(crate) environment: Vec<(String, Option<String>)>,
    /// The arguments of the question to the manager when a test replaces the ordinary ones (`None` is the ordinary question).
    pub(crate) question: Option<Vec<String>>,
    /// How long the question may take.
    pub(crate) discovery_bound: Duration,
    /// The file that names this process's own control group.
    pub(crate) own_cgroup_file: PathBuf,
}

/// The settings of an ordinary host.
impl Default for RouteSettings {
    /// What:
    /// `fn default()` is the ordinary settings:
    /// `systemd-run`,
    /// `systemctl`,
    /// `/sys/fs/cgroup` and `app.slice`.
    /// Why:
    /// `app.slice` is where the user manager puts application scopes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const defaults: RouteSettings = { ... }
    /// ```
    fn default() -> RouteSettings {
        return RouteSettings {
            systemd_run: "systemd-run".to_string(),
            systemctl: "systemctl".to_string(),
            cgroup_root: PathBuf::from(CGROUP_ROOT),
            slice: "app.slice".to_string(),
            environment: Vec::new(),
            question: None,
            discovery_bound: DISCOVERY_BOUND,
            own_cgroup_file: PathBuf::from(OWN_CGROUP_FILE),
        };
    }
}

/// What a host can give:
/// the way to a delegated scope.
#[derive(Clone, Debug)]
pub(crate) struct Route {
    /// The program that makes the scope.
    pub(crate) systemd_run: String,
    /// The program that asks the manager to stop a scope unit after its run ended.
    pub(crate) systemctl: String,
    /// The slice the scopes are made in.
    pub(crate) slice: String,
    /// The directory of the slice in the cgroup file system,
    /// as the manager reports it.
    pub(crate) slice_dir: PathBuf,
    /// The inode of this process's cgroup namespace.
    pub(crate) namespace: u64,
    /// How the slice directory is seen now.
    pub(crate) slice_view: View,
    /// The limits of the caller's control group that every scope of this route carries.
    /// Shared,
    /// not inline,
    /// so a `Route` stays small enough to sit in the run's containment value (`clippy::large_enum_variant`).
    pub(crate) limits: Arc<InheritedLimits>,
}

/// What:
/// `fn is_cgroup2(path: &Path) -> io::Result<bool>` says whether the directory is on a cgroup v2 file system.
/// Why:
/// Only the file system type shows that a directory is part of the control group hierarchy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isCgroup2(path: string): boolean
/// ```
pub(crate) fn is_cgroup2(path: &Path) -> io::Result<bool> {
    let found = rustix::fs::statfs(path).map_err(io::Error::from)?;
    return Ok(u64::try_from(found.f_type).unwrap_or(0) == CGROUP2_MAGIC);
}

/// What:
/// `fn namespace_id() -> io::Result<u64>` is the inode number of this process's cgroup namespace.
/// Why:
/// A recorded path means the same thing only inside the same namespace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namespaceId(): number
/// ```
pub(crate) fn namespace_id() -> io::Result<u64> {
    return Ok(fs::metadata("/proc/self/ns/cgroup")?.ino());
}

/// What:
/// `fn on_path(program: &str) -> bool` says whether an executable of that name is on `PATH` (or is itself a path to one).
/// Why:
/// A missing `systemd-run` is a missing capability to name,
/// found without starting anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function onPath(program: string): boolean
/// ```
pub(crate) fn on_path(program: &str) -> bool {
    if program.contains('/') {
        return is_executable_file(Path::new(program));
    }
    let search: String = std::env::var("PATH").unwrap_or_default();
    for directory in search.split(':') {
        if !directory.is_empty() && is_executable_file(&Path::new(directory).join(program)) {
            return true;
        }
    }
    return false;
}

/// What:
/// `fn is_executable_file(candidate: &Path) -> bool` says whether a path is a regular file with an execute bit.
/// Why:
/// A function replaces the closure that `on_path` called for each search directory.
pub(crate) fn is_executable_file(candidate: &Path) -> bool {
    /// The permission bits that tell an executable file from a plain one.
    use std::os::unix::fs::PermissionsExt;
    match fs::metadata(candidate) {
        Ok(found) => return found.is_file() && found.permissions().mode() & 0o111 != 0,
        Err(_) => return false,
    }
}

/// What:
/// `fn unavailable(reason: String) -> GroupUnavailable` wraps a reason in the refusal that route discovery returns.
/// Why:
/// A function replaces the closure that `detect_route` called at every refusal.
pub(crate) fn unavailable(reason: String) -> GroupUnavailable {
    return GroupUnavailable {
        reason,
        stops_the_call: false,
    };
}

/// What:
/// `fn refuses_the_call(reason: String) -> GroupUnavailable` wraps a reason in the refusal that stops the call,
/// instead of the one the run stands in for with a process group of its own.
/// Why:
/// The reason it exists for is a route the manager reports beside a job a start could not wait out:
/// a scope planned there is refused while that job stands,
/// and waiting for another unit's stop has no bound this tool sets,
/// so a run that took the weaker path there would go on uncontained behind a stop of unknown length (rule "nothing waits forever")
/// while running the command it was asked to check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusesTheCall(reason: string): GroupUnavailable  // the kind that throws instead of falling back
/// ```
pub(crate) fn refuses_the_call(reason: String) -> GroupUnavailable {
    return GroupUnavailable {
        reason,
        stops_the_call: true,
    };
}

/// How many times the discovery asks the manager where its slice is before a failed ask is read as a capability this host has not
/// got.
/// Why:
/// One failed ask is not a host without a control group.
/// Everything that needs a delegated scope asks this before it does anything,
/// so on 2026-11-27 a run of the whole suite failed ten controls in nine milliseconds each,
/// which is the shape of this question being answered with a failure and of nothing else those controls do (issue 6619).
/// A program this discovery starts can fail for a moment that is not the host's answer about itself,
/// and asking again is what tells the two apart.
/// What it rests on:
/// the manager's own reexecution,
/// measured 2026-11-27 in a container of its own (Linux 7.3.5,
/// systemd 262)
/// and in a guest (Linux 6.8,
/// systemd 255):
/// asks fail while the manager reexecutes and answer again as soon as it has,
/// and the longest reexec across which an ask failed was 0.588 s.
/// Five attempts `ASKING_AGAIN_PAUSE` apart span 0.800 s,
/// which covers that,
/// and a reload of 0.415 s to 0.654 s refused no ask at all.
/// What it does not rest on:
/// the event it was asked about,
/// ten controls whose asks began about half a second after a reexec had ended (issue 6623):
/// no ask failed in the five seconds after any reexec measured here,
/// so nothing is sized to that event and it stays open.
/// Where no user manager can be reached at all,
/// which is not the case these attempts are for:
/// the first failed ask is not put again,
/// because the discovery reads in its own process that nothing here could carry another one
/// (`control_group_route_manager_reach`).
/// Measured 2026-11-27 in a container with no user manager,
/// both ways of having none,
/// nine rounds of this branch's binary beside the released `0.1.5-alpha.15` one and a byte-identical copy of each:
/// one ask per call for all 72 calls,
/// and this branch's call 1.01 to 1.28 times the released one per round where neither variable is set (median 1.11),
/// and 0.87 to 1.05 times it where the runtime directory holds no socket (median 1.00),
/// where the same harness measured 12.90 to 29.68 times before this change (README,
/// "Development").
pub(crate) const ASKING_AGAIN_ATTEMPTS: u32 = 5;

/// How long the discovery waits before it asks the same question again,
/// where a manager could be reached and did not answer for a moment.
/// What it rests on:
/// the longest reexec measured across which asks fail (0.588 s,
/// 2026-11-27),
/// so that four waits of this length hold the question open past the end of one however early in it the first ask fell,
/// and a copy of each figure is in `doc/the-route-question-that-is-asked-again-when-one-ask-fails.md`.
/// This wait is spent where a manager can be reached and did not answer for a moment,
/// and that includes a host whose socket file or manager directory is still there with nothing behind it:
/// at the instant an ask fails,
/// that is what a manager in a reexec looks like.
/// An ask that spent its own deadline inside its bound is not made again (see `ask_where_the_slice_is`),
/// and an ask that failed where no user manager can be reached is not made again either,
/// because nothing here could carry a second one (`control_group_route_manager_reach`),
/// which is what keeps a call there at one ask and no wait.
pub(crate) const ASKING_AGAIN_PAUSE: Duration = Duration::from_millis(200);

/// What:
/// `fn ask_once(settings: &RouteSettings) -> Result<String, FailedAsk>` asks the manager where the slice is,
/// starts the slice where it has no control group yet or has a job queued on it,
/// and answers the path a manager that could be reached reported.
/// Why:
/// The ask of where the slice is comes first because it is also the read that says the manager answered at all:
/// a manager that is there answers it whether or not the slice was ever started,
/// so a failure of the start that follows is the manager's own answer about the slice rather than a transport that is not
/// there (measured 2026-11-27 against a masked slice in a container:
/// the ask of where the slice is answered at once with no
/// path,
/// the start answered `Failed to start app.slice: Unit app.slice is masked.`,
/// and one ask after it answered again,
/// which is the manager saying the same thing twice).
/// Two answers of that ask are not the manager's refusal and are asked through instead:
/// a path it reports after a failed start,
/// which is a start that ran and lost its reply,
/// and no control group after a start whose own failure was a transport rather than the manager's answer.
/// A path reported beside a job queued on the slice is a slice being stopped,
/// which the start waits out;
/// where that start spent the bound it was given, the job is still there and the path beside it is not the route,
/// because no wait of this tool ends another unit's stop (see `ask_once`).
/// The words are the refusal the caller names,
/// so they say which ask failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function askOnce(settings: RouteSettings): string  // throws FailedAsk
/// ```
fn ask_once(settings: &RouteSettings) -> Result<String, FailedAsk> {
    match ask_where_the_slice_is_once(settings)? {
        // What:     `WhereTheSliceIs::APath { a_job_is_queued: false, .. }` is the route, found by the first ask.
        // Why:      An active slice with no job queued on it is the ordinary host, and this is the whole discovery there: one
        //           program start.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return path
        // ```
        WhereTheSliceIs::APath {
            path,
            a_job_is_queued: false,
        } => return Ok(path),
        // What:     `WhereTheSliceIs::APath { a_job_is_queued: true, .. }` means the manager reports the slice's path while a job
        //           stands on the slice, which is what a slice being stopped answers.
        // Why:      A scope made in the slice while that job stands is refused as destructive
        //           (`Transaction for cargo-shadow-….scope/start is destructive (app.slice has 'stop' job queued, but 'start' is
        //           included in transaction).`, measured 2026-11-27 in a container),
        //           and the start below is the manager's own way to wait the job out: measured at 3.68 s to 3.78 s against the
        //           planted `TimeoutStopSec=4`, with nothing queued on the slice afterwards.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // fall through to the start
        // ```
        WhereTheSliceIs::APath {
            a_job_is_queued: true,
            ..
        } => {}
        // What:     `WhereTheSliceIs::NoControlGroup` means the slice is not started yet, which the start below makes good.
        // Why:      An inactive slice has no control group (a fresh login, a runner whose manager has run no application), and
        //           the discovery is still the discovery of a capability.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // fall through to the start
        // ```
        WhereTheSliceIs::NoControlGroup => {}
    }
    if settings.question.is_none() {
        // What:     `started` is the manager's answer to `systemctl --user start <slice>`, or the reason it failed.
        // Why:      Starting a slice that is inactive is idempotent, and a manager that cannot start it is a missing
        //           capability.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const started = runPlain(settings.systemctl, ['--user', 'start', settings.slice], settings.environment, settings.discoveryBound)
        // ```
        let began: Instant = Instant::now();
        let started: Result<String, String> = run_plain(
            &settings.systemctl,
            &["--user", "start", &settings.slice],
            &settings.environment,
            settings.discovery_bound,
        );
        if let Err(reason) = started {
            // What:     `asked_where` is the refusal's own words: the start failed, and this names the program's answer.
            // Why:      A person reading the weaker path's notice has to see what the manager said about the slice.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const askedWhere = `the user manager cannot start ${settings.slice} (${reason})`
            // ```
            let asked_where: String = format!(
                "the user manager cannot start {} ({reason})",
                settings.slice
            );
            // What:     `spent` says whether that program used the whole bound the ask gave it.
            // Why:      A manager that answered nothing at all is a host without one, and the ask after this is not made to
            //           find that out twice.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const spent = spentItsBound(settings, began.elapsed())
            // ```
            let spent: bool = spent_its_bound(settings, began.elapsed());
            // What:     `answered_about_the_slice` says whether the start's own words are the manager's answer about the slice, which
            //           is the one kind of failure a wait cannot change.
            // Why:      A start that failed to reach the manager, or whose reply was lost, is a moment the asking again is for; a
            //           start the manager refused is its answer, and asking four more times spent 895 ms on the same sentence
            //           (measured 2026-11-27 against a masked slice).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const answeredAboutTheSlice = theManagerAnsweredAStart(reason, settings.slice)
            // ```
            let answered_about_the_slice: bool =
                the_manager_answered_a_start(&reason, &settings.slice);
            return match ask_where_the_slice_is_once(settings) {
                // What:     `Ok(WhereTheSliceIs::APath { path, a_job_is_queued })` is the manager reporting the slice's control group
                //           after the start, so the start ran and the path is the route, whatever the reply about it said.
                // Why:      A reply lost in flight leaves a slice that was really started, and the path is the whole point of the
                //           ask: throwing it away ran the call without a control group over a lost reply (measured 2026-11-27 in a
                //           container's own manager).
                //           A job beside that path after a start that spent its bound is the stop the start was waiting out, and
                //           the path is then no route: a scope made there is refused while the job stands, so the call stops with
                //           the words rather than waiting for another unit's stop (`DISCOVERY_BOUND` names the bound).
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (spent && aJobIsQueued) throw { reason, mayBePutAgain: false, stopsTheCall: true }
                // return path
                // ```
                Ok(WhereTheSliceIs::APath {
                    path,
                    a_job_is_queued,
                }) => {
                    if spent && a_job_is_queued {
                        return Err(FailedAsk {
                            reason: format!(
                                "{asked_where}, and a job is still queued on {}: a scope made there while that job stands is refused, so no delegated control group can be had until it ends, and this tool does not wait for another unit's stop",
                                settings.slice
                            ),
                            may_be_put_again: false,
                            stops_the_call: true,
                        });
                    }
                    Ok(path)
                }
                // What:     `Ok(WhereTheSliceIs::NoControlGroup)` after a failed start is the manager's refusal where the start's own
                //           failure was its answer about the slice, and a moment to ask through where it was a transport.
                // Why:      This is the refusal of a manager that is there: asking four more times spent 895 ms on the same
                //           sentence (measured 2026-11-27 against a masked slice), and a start that never reached the manager is
                //           read the other way (measured 2026-11-27 in the same container: alpha.16 delegated in every call of
                //           three such states).
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return { reason: askedWhere, mayBePutAgain: !spent && !answeredAboutTheSlice }
                // ```
                Ok(WhereTheSliceIs::NoControlGroup) => Err(FailedAsk {
                    reason: asked_where,
                    may_be_put_again: !spent && !answered_about_the_slice,
                    stops_the_call: false,
                }),
                // What:     `Err(after)` is that ask failing too, which is a transport that is not there rather than the
                //           manager's answer.
                // Why:      A moment of a manager that is there must still be asked through, which is what the asking again is
                //           for (issue 6619).
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return { reason: askedWhere, mayBePutAgain: !spent && after.mayBePutAgain }
                // ```
                Err(after) => Err(FailedAsk {
                    reason: asked_where,
                    may_be_put_again: !spent && after.may_be_put_again,
                    stops_the_call: false,
                }),
            };
        }
    }
    return match ask_where_the_slice_is_once(settings)? {
        // What:     `WhereTheSliceIs::APath { path, .. }` is where the manager just put the slice's control group.
        // Why:      The slice was started for this answer, so it is the route.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return path
        // ```
        WhereTheSliceIs::APath {
            path,
            ..
        } => Ok(path),
        // What:     `WhereTheSliceIs::NoControlGroup` after a start is the manager reporting no control group for a slice it
        //           was just asked to start.
        // Why:      The manager answered twice, so this is its answer and no wait changes it (measured 2026-11-27 through a
        //           stand-in that answers every ask of where the slice is with an empty value: two asks spent 895 ms saying
        //           the same thing).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // throw { reason: `the user manager reports no control group for ${settings.slice}`, mayBePutAgain: false }
        // ```
        WhereTheSliceIs::NoControlGroup => Err(FailedAsk {
            reason: format!(
                "the user manager reports no control group for {}",
                settings.slice
            ),
            may_be_put_again: false,
            stops_the_call: false,
        }),
    };
}

/// What:
/// `fn names_the_slice(reported: &str, slice: &str) -> bool` says whether the control group the manager reported is the
/// slice's own, which is the group named after the unit that was asked about.
/// Why:
/// A unit's own control group carries the unit's name,
/// so an answer whose last component is another name is an answer about another unit,
/// and a scope planned under it would be made somewhere else than the group this run shows empty
/// (measured 2026-11-28 against a stand-in: `0.1.5-alpha.18` took `/system.slice` and `/init.scope` as `app.slice`'s route and
/// then reported quiescence it never showed).
/// What it does not decide:
/// which account's slice the path belongs to,
/// because the manager's own slice is spelled differently on different hosts
/// (`user-1000.slice`, `user@1000.service` and a container's own service were seen beside this check)
/// and a guess at that spelling would refuse routes that are right.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namesTheSlice(reported: string, slice: string): boolean {
///   return reported.replace(/\/+$/, '').split('/').at(-1) === slice
/// }
/// ```
fn names_the_slice(
    reported: &str,
    slice: &str,
) -> bool {
    // What:     `without_trailing_slashes` is the reported path with any trailing separators taken off, so the last component
    //           is the unit's name whether or not the answer ended with one.
    // Why:      A comparison against the slice's name must not depend on a slash the manager may or may not print.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const withoutTrailingSlashes = reported.replace(/\/+$/, '')
    // ```
    let without_trailing_slashes: &str = reported.trim_end_matches('/');
    // What:     `rsplit('/').next()` reads the path from its end and takes the first piece it meets, which is the last
    //           component (`split` walks from the front, `pop` on that list is the TS shape of it).
    // Why:      The unit's name is the last component of its own control group's path.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return withoutTrailingSlashes.split('/').at(-1) === slice
    // ```
    return without_trailing_slashes.rsplit('/').next() == Some(slice);
}

/// What:
/// `fn ask_where_the_slice_is(settings: &RouteSettings) -> Result<String, FailedAsk>` asks until the manager reports the path of the
/// slice,
/// and answers that path or the last failure it left.
/// Why:
/// A run and every control that needs a delegated scope ask this before they may do anything,
/// and one failed ask is not the host's answer about itself:
/// the same host answers a moment later (issue 6619).
/// Two failures are not put again.
/// An ask whose program spent its own deadline is one,
/// because a manager that answered nothing inside its bound is what a host without one looks like,
/// and a second ask would only spend a second bound.
/// A host where nothing could carry an ask to a manager at all is the other,
/// which the discovery reads in its own process rather than from the words of the refusal (issue 6629).
/// And an answer that is a refusal of the manager's own is the end of the asking too,
/// which `ask_once` reads from the manager answering one more ask after it:
/// a masked slice answered `start` once and the question after it once,
/// in 3 asks and about 0.1 s,
/// where five asks spent 895 ms (measured 2026-11-27 in a container whose own manager had `app.slice` masked and stopped).
/// What that start's failure says decides whether that answer ends the asking:
/// where the words are the manager's answer about the slice,
/// the asking ends;
/// where they are a transport that did not reach it or a reply that was lost,
/// the question is put again
/// (`the_manager_answered_a_start`),
/// and the path such an answer reports is the route and is used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function askWhereTheSliceIs(settings: RouteSettings): string  // throws the last reason
/// ```
fn ask_where_the_slice_is(settings: &RouteSettings) -> Result<String, FailedAsk> {
    // What:     `last` is the failure of the last attempt put, kept so the refusal names the words of the last one rather than
    //           the first.
    // Why:      A refusal that named the first attempt's words would hide why the last attempt failed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let last: FailedAsk = { reason: '', mayBePutAgain: true, stopsTheCall: false }
    // ```
    let mut last: FailedAsk = FailedAsk {
        reason: String::new(),
        may_be_put_again: true,
        stops_the_call: false,
    };
    for attempt in 1..=ASKING_AGAIN_ATTEMPTS {
        if attempt > 1 {
            // What:     `std::thread::sleep(ASKING_AGAIN_PAUSE)` gives the moment the failure met time to pass.
            // Why:      An ask put again with no wait meets the same condition again and answers nothing new.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // await new Promise((resolve) => setTimeout(resolve, ASKING_AGAIN_PAUSE))
            // ```
            std::thread::sleep(ASKING_AGAIN_PAUSE);
        }
        match ask_once(settings) {
            Ok(reported) => return Ok(reported),
            Err(failed) => {
                if !failed.may_be_put_again {
                    return Err(failed);
                }
                // What:     `a_refusal_that_ends_the_asking(settings, &failed.reason)` says what to refuse with where nothing on
                //           this host could carry an ask to a user manager.
                // Why:      The same failure is that host's whole answer, so putting the question again only spends the span the
                //           asking again is sized to (issue 6629).
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const refusal = aRefusalThatEndsTheAsking(settings, failed.reason)
                // if (refusal !== undefined) throw { reason: refusal, mayBePutAgain: false }
                // ```
                if let Some(refusal) = a_refusal_that_ends_the_asking(settings, &failed.reason) {
                    return Err(FailedAsk {
                        reason: refusal,
                        may_be_put_again: false,
                        stops_the_call: false,
                    });
                }
                last = failed;
            }
        }
    }
    return Err(last);
}

/// What:
/// `pub(crate) fn detect_route(settings: &RouteSettings) -> Result<Route, GroupUnavailable>` finds out whether this host can give a
/// delegated scope,
/// and where the manager keeps its scopes.
/// Why:
/// The manager's real hierarchy is asked for,
/// never assumed from this process's own path (contract,
/// "The spawn route").
/// Every failure names the missing capability.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function detectRoute(settings: RouteSettings): Route  // throws GroupUnavailable
/// ```
pub(crate) fn detect_route(settings: &RouteSettings) -> Result<Route, GroupUnavailable> {
    match is_cgroup2(&settings.cgroup_root) {
        Ok(true) => {}
        Ok(false) => {
            return Err(unavailable(format!(
                "{} is not a cgroup v2 mount",
                settings.cgroup_root.display()
            )));
        }
        Err(error) => {
            return Err(unavailable(format!(
                "{} cannot be examined ({error})",
                settings.cgroup_root.display()
            )));
        }
    }
    if !on_path(&settings.systemd_run) {
        return Err(unavailable(format!(
            "{} is not installed",
            settings.systemd_run
        )));
    }
    let reported: String = match ask_where_the_slice_is(settings) {
        Ok(found) => found,
        // What:     `Err(failed) if failed.stops_the_call` is a refusal of the moment no wait of this tool ends, which is the one
        //           the caller stops the call for.
        // Why:      The run cannot fall back to a process group there without running the command it was asked to check behind a
        //           stop of unknown length.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (failed.stopsTheCall) throw (refusal that stops the command)
        // ```
        Err(failed) if failed.stops_the_call => return Err(refuses_the_call(failed.reason)),
        Err(failed) => return Err(unavailable(failed.reason)),
    };
    // What:     `names_the_slice(&reported, &settings.slice)` says whether the manager's answer is the slice's own control group.
    // Why:      A path of another unit is no route to this one: the scope this run plans under it would not be the scope the
    //           manager makes, so the run's end would read an absent directory as an empty scope (Medium 2 of the alpha.18 read).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!namesTheSlice(reported, settings.slice)) throw new GroupUnavailable(...)
    // ```
    if !names_the_slice(&reported, &settings.slice) {
        return Err(unavailable(format!(
            "the user manager reports {reported} for {}, and that group is another unit's, so a scope planned there would never be the scope this run makes",
            settings.slice
        )));
    }
    let slice_dir: PathBuf = settings.cgroup_root.join(reported.trim_start_matches('/'));
    if slice_dir.to_str().is_none() {
        return Err(unavailable(format!(
            "{} is not UTF-8, which the journal record cannot hold",
            slice_dir.display()
        )));
    }
    let slice_is_directory: bool = match fs::metadata(&slice_dir) {
        Ok(found) => found.is_dir(),
        Err(_) => false,
    };
    if !slice_is_directory {
        return Err(unavailable(format!(
            "{} does not exist in the cgroup file system",
            slice_dir.display()
        )));
    }
    if !kill_file_present(&slice_dir) {
        return Err(unavailable(format!(
            "{} has no cgroup.kill; Linux 5.14 or later is needed",
            slice_dir.display()
        )));
    }
    let namespace: u64 = match namespace_id() {
        Ok(found) => found,
        Err(error) => {
            return Err(unavailable(format!(
                "the cgroup namespace cannot be identified ({error})"
            )));
        }
    };
    let slice_view: View = match view_of(&slice_dir) {
        Ok(found) => found,
        Err(error) => {
            return Err(unavailable(format!(
                "{} cannot be examined ({error})",
                slice_dir.display()
            )));
        }
    };
    let limits: InheritedLimits = match caller_limits(
        &settings.own_cgroup_file,
        &settings.cgroup_root,
        &slice_dir,
    ) {
        Ok(found) => found,
        Err(error) => {
            return Err(unavailable(format!(
                "the resource limits of the caller's control group cannot be read ({error}), and a scope without them would loosen the run"
            )));
        }
    };
    return Ok(Route {
        systemd_run: settings.systemd_run.clone(),
        systemctl: settings.systemctl.clone(),
        slice: settings.slice.clone(),
        slice_dir,
        namespace,
        slice_view,
        limits: Arc::new(limits),
    });
}

/// The controls of the question to the manager:
/// that a failure which came back at once is put again,
/// and that a manager which answered nothing is asked once (issue 6619).
#[cfg(test)]
#[path = "control_group_route_discovery_asking_again_tests.rs"]
mod asking_again_tests;

/// The controls of the span the asking again covers:
/// a moment as long as the longest reexec measured here,
/// and a real reexec of a container's own user manager (issue 6623).
#[cfg(test)]
#[path = "control_group_route_discovery_reexec_tests.rs"]
mod reexec_tests;
