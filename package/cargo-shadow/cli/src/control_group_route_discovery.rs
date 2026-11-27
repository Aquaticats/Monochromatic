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
#[derive(Debug)]
pub(crate) struct GroupUnavailable {
    /// The missing capability and what was tried.
    pub(crate) reason: String,
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
/// and this branch's call as long as the released one
/// (0.63 to 1.56 times per round where neither variable is set,
/// 0.29 to 1.11 times where the runtime directory holds no socket),
/// where the same harness measured 12.90 to 29.68 times before this change (README,
/// "Development").
const ASKING_AGAIN_ATTEMPTS: u32 = 5;

/// How long the discovery waits before it asks the same question again,
/// where a manager could be reached and did not answer for a moment.
/// What it rests on:
/// the longest reexec measured across which asks fail (0.588 s,
/// 2026-11-27),
/// so that four waits of this length hold the question open past the end of one however early in it the first ask fell,
/// and a copy of each figure is in `doc/the-route-question-that-is-asked-again-when-one-ask-fails.md`.
/// This wait is spent on one case only:
/// a manager that can be reached and did not answer for a moment.
/// An ask that spent its own deadline inside its bound is not made again (see `ask_where_the_slice_is`),
/// and an ask that failed where no user manager can be reached is not made again either,
/// because nothing here could carry a second one (`control_group_route_manager_reach`),
/// which is what keeps a call there at one ask and no wait.
pub(crate) const ASKING_AGAIN_PAUSE: Duration = Duration::from_millis(200);

/// What one ask of the manager left when it answered nothing:
/// the words of the failure,
/// and whether the program ran out of the bound that ask gave it.
/// Why:
/// The two are different failures.
/// A manager that answered nothing inside its bound is what a host without one looks like,
/// and a second ask would only spend a second bound;
/// a program that failed at once can be a moment of pressure,
/// and asking again is what tells the two apart.
/// Each program is timed on its own,
/// because the discovery starts two of them and a slow one followed by a fast failure must still be put again.
struct FailedAsk {
    /// The words the refusal names.
    reason: String,
    /// Whether the program this ask ran ran out of the bound the ask gave it.
    spent_its_deadline: bool,
}

/// What:
/// `fn ask_once(settings: &RouteSettings) -> Result<String, FailedAsk>` starts the slice when the ordinary question is asked,
/// asks the manager where the slice is,
/// and answers the path it reported or what the failure left.
/// Why:
/// The two asks are one answer about one thing:
/// a slice that could not be started cannot be asked about.
/// The words are the refusal the caller names,
/// so they say which of the two failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function askOnce(settings: RouteSettings): string  // throws FailedAsk
/// ```
fn ask_once(settings: &RouteSettings) -> Result<String, FailedAsk> {
    // An inactive slice has no control group yet (a fresh login, a runner whose manager has run no application): starting it is
    // idempotent, and it is still the discovery of a capability, so a manager that cannot start it is a missing capability.
    if settings.question.is_none() {
        let began: Instant = Instant::now();
        let started: Result<String, String> = run_plain(
            &settings.systemctl,
            &["--user", "start", &settings.slice],
            &settings.environment,
            settings.discovery_bound,
        );
        if let Err(reason) = started {
            return Err(FailedAsk {
                reason: format!(
                    "the user manager cannot start {} ({reason})",
                    settings.slice
                ),
                spent_its_deadline: spent_its_bound(settings, began.elapsed()),
            });
        }
    }
    let ordinary: Vec<String> = vec![
        "--user".to_string(),
        "show".to_string(),
        settings.slice.clone(),
        "--property=ControlGroup".to_string(),
        "--value".to_string(),
    ];
    let question: Vec<&str> = settings
        .question
        .as_ref()
        .unwrap_or(&ordinary)
        .iter()
        .map(String::as_str)
        .collect();
    let began: Instant = Instant::now();
    let answered: String = match run_plain(
        &settings.systemctl,
        &question,
        &settings.environment,
        settings.discovery_bound,
    ) {
        Ok(text) => text,
        Err(reason) => {
            return Err(FailedAsk {
                reason: format!(
                    "the user manager cannot be asked where {} is ({reason})",
                    settings.slice
                ),
                spent_its_deadline: spent_its_bound(settings, began.elapsed()),
            });
        }
    };
    let reported: &str = answered.trim();
    if reported.is_empty() || !reported.starts_with('/') || reported.contains('\n') {
        return Err(FailedAsk {
            reason: format!(
                "the user manager reports no control group for {}",
                settings.slice
            ),
            // An answer that is not a path is an answer: the program ran and ended inside its bound, so it is put again.
            spent_its_deadline: false,
        });
    }
    return Ok(reported.to_string());
}

/// What:
/// `fn spent_its_bound(settings: &RouteSettings, took: Duration) -> bool` says whether one program that failed used the whole bound
/// the ask gave it.
/// Why:
/// Only the program's own time says it ran out of its bound:
/// the discovery starts two programs per ask,
/// so the time around both tells neither.
/// `DISCOVERY_POLL` is the interval the wait notices an end at,
/// and twice it is the most an ended program can be noticed after its deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function spentItsBound(settings: RouteSettings, took: number): boolean
/// ```
fn spent_its_bound(
    settings: &RouteSettings,
    took: Duration,
) -> bool {
    return took + DISCOVERY_POLL * 2 >= settings.discovery_bound;
}

/// What:
/// `fn ask_where_the_slice_is(settings: &RouteSettings) -> Result<String, String>` asks until the manager reports the path of the
/// slice,
/// and answers that path or the words of the last failure.
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function askWhereTheSliceIs(settings: RouteSettings): string  // throws the last reason
/// ```
fn ask_where_the_slice_is(settings: &RouteSettings) -> Result<String, String> {
    let mut last: String = String::new();
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
                if failed.spent_its_deadline {
                    return Err(failed.reason);
                }
                // What:     `a_refusal_that_ends_the_asking(settings, &failed.reason)` says what to refuse with where nothing on
                //           this host could carry an ask to a user manager.
                // Why:      The same failure is that host's whole answer, so putting the question again only spends the span the
                //           asking again is sized to (issue 6629).
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const refusal = aRefusalThatEndsTheAsking(settings, failed.reason)
                // if (refusal !== undefined) throw new Error(refusal)
                // ```
                if let Some(refusal) = a_refusal_that_ends_the_asking(settings, &failed.reason) {
                    return Err(refusal);
                }
                last = failed.reason;
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
        Err(reason) => return Err(unavailable(reason)),
    };
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
