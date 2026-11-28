//! What:
//! One ask of the user manager where a slice's control group is:
//! the words it failed with,
//! the answer it gave (the slice's path and the job queued on it),
//! and the rule that decides whether the program that failed used the bound the ask gave it.
//! Why:
//! Split out of `control_group_route_discovery.rs`,
//! which holds the discovery that puts one ask after another,
//! to keep that file under the code-line limit.
//! The discovery re-exports what it uses,
//! so a reader of the discovery still finds these names beside it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control_group_route_one_ask.ts: FailedAsk, WhereTheSliceIs, askWhereTheSliceIsOnce, spentItsBound
//! ```

/// What:
/// `use std::time::{Duration, Instant};` imports a span of time and a reading of the monotonic clock (its sibling is
/// `SystemTime`,
/// the wall clock,
/// which can jump).
/// Why:
/// One ask is timed on its own,
/// so a program that ran out of the bound the ask gave it is told from one that answered at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a number of milliseconds, and performance.now()
/// ```
use std::time::{
    Duration,
    Instant,
};

/// What:
/// `use crate::control_group_route_child_runs::run_plain;` imports the short program the ask is made with.
/// Why:
/// The ask is one program start under one deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { runPlain } from 'crate/control_group_route_child_runs';
/// ```
use crate::control_group_route_child_runs::run_plain;
/// What:
/// `use crate::control_group_route_discovery::DISCOVERY_POLL;` imports the interval at which the wait notices that a program
/// ended.
/// Why:
/// The rule that judges whether a program used its bound allows for that interval,
/// so the constant is read here rather than written a second time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DISCOVERY_POLL } from 'crate/control_group_route_discovery';
/// ```
use crate::control_group_route_discovery::DISCOVERY_POLL;
/// What:
/// `use crate::control_group_route_discovery::RouteSettings;` imports the settings an ask is made with.
/// Why:
/// A control plants a host and a stand-in program through them,
/// and the ask has to ask what the settings name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RouteSettings } from 'crate/control_group_route_discovery';
/// ```
use crate::control_group_route_discovery::RouteSettings;

/// What one ask of the manager left when it answered nothing:
/// the words of the failure,
/// and whether putting the question again could get another answer.
/// Why:
/// Two failures are the end of the asking.
/// A manager that answered nothing inside its bound is what a host without one looks like,
/// and a second ask would only spend a second bound;
/// and a manager that answered with a refusal of its own about the slice (a masked slice,
/// a policy that denies the start) answers the same way
/// a moment later,
/// which is asked once and not five times (measured 2026-11-27:
/// one ask of `start app.slice` against a masked slice
/// answered `Failed to start app.slice: Unit app.slice is masked.`,
/// and five asks spent 895 ms where the answer was the first one).
/// Everything else can be a moment of pressure,
/// and asking again is what tells the two apart;
/// whether the words are that refusal is decided in the discovery (`the_manager_answered_a_start`).
/// Each program is timed on its own,
/// because the discovery starts one program per ask and a slow one followed by a fast failure must still be put again.
pub(crate) struct FailedAsk {
    /// The words the refusal names.
    pub(crate) reason: String,
    /// Whether asking again could answer differently:
    /// false where a program spent the bound the ask gave it,
    /// and false where the manager itself answered.
    pub(crate) may_be_put_again: bool,
}

/// What one ask of where the slice is answered:
/// the control group's path and whether a job is queued for the slice,
/// or the manager's answer that the slice has no control group.
/// Why:
/// The two are different facts about the host,
/// and only the first one is a route:
/// a slice with no control group is one that has not been started,
/// and a manager that answers at all is one that can be reached.
/// Whether a job is queued is the fact a slice being stopped is told by:
/// the manager still reports the slice's path while a stop is queued on it,
/// and a scope made in the slice while that job stands is refused as destructive
/// (measured 2026-11-27 against a container's own manager,
/// and read from `transaction_is_destructive` in systemd's `src/core/transaction.c`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhereTheSliceIs = { path: string; aJobIsQueued: boolean } | { noControlGroup: true }
/// ```
pub(crate) enum WhereTheSliceIs {
    /// What:
    /// the manager's answer that is a control group path.
    /// Why:
    /// The path is what the route is built on.
    APath {
        /// The path the manager reported,
        /// which is where the slice's control group is.
        path: String,
        /// Whether the manager also reported a job queued for the slice,
        /// which is what a slice with a stop job queued answers.
        a_job_is_queued: bool,
    },
    /// The manager answered,
    /// and its answer is no control group path at all.
    NoControlGroup,
}

/// What:
/// `fn what_the_manager_reported(answered: &str) -> WhereTheSliceIs` reads the answer of one ask.
/// Why:
/// The question asks for two properties,
/// so an answer is the named lines `systemctl show` prints;
/// a control may replace the question with a program that answers one value and no names,
/// and that value is read as the path it stands for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whatTheManagerReported(answered: string): WhereTheSliceIs
/// ```
fn what_the_manager_reported(answered: &str) -> WhereTheSliceIs {
    // What:     `reported` and `a_job_is_queued` are the two facts the named lines carry, and `a_name_was_read` says whether an answer
    //           named anything at all.
    // Why:      An answer with no name is one value, which is the shape a control's own program answers with.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let reported = ''; let aJobIsQueued = false; let aNameWasRead = false
    // ```
    let mut reported: String = String::new();
    let mut a_job_is_queued: bool = false;
    let mut a_name_was_read: bool = false;
    for line in answered.lines() {
        let Some((name, written)) = line.split_once('=') else {
            continue;
        };
        a_name_was_read = true;
        let value: &str = written.trim();
        match name.trim() {
            "ControlGroup" => reported = value.to_string(),
            "Job" => a_job_is_queued = !value.is_empty(),
            _ => {}
        }
    }
    if !a_name_was_read {
        // What:     `alone` is the whole answer, which is the one value a program that replaced the question wrote.
        // Why:      A value that is not a path is no route, and one that spans lines is not a path either.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const alone = answered.trim()
        // ```
        let alone: &str = answered.trim();
        if alone.is_empty() || !alone.starts_with('/') || alone.contains('\n') {
            return WhereTheSliceIs::NoControlGroup;
        }
        return WhereTheSliceIs::APath {
            path: alone.to_string(),
            a_job_is_queued: false,
        };
    }
    if reported.is_empty() || !reported.starts_with('/') || reported.contains('\n') {
        return WhereTheSliceIs::NoControlGroup;
    }
    return WhereTheSliceIs::APath {
        path: reported,
        a_job_is_queued,
    };
}

/// What:
/// `fn ask_where_the_slice_is_once(settings) -> Result<WhereTheSliceIs, FailedAsk>` makes one ask of the manager about the slice,
/// and answers the path it reported or what the failure left.
/// Why:
/// This ask is also the read that says whether the manager can be reached:
/// a transport that is not there fails before the manager sees the request,
/// and a manager that is there always answers this one.
/// A failure is put again (see `ask_where_the_slice_is`),
/// and an answer is judged here,
/// so the two callers below share one reading of the manager's words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function askWhereTheSliceIsOnce(settings: RouteSettings): WhereTheSliceIs  // throws FailedAsk
/// ```
pub(crate) fn ask_where_the_slice_is_once(
    settings: &RouteSettings
) -> Result<WhereTheSliceIs, FailedAsk> {
    // What:     `ordinary` is the question `systemctl --user show <slice> --property=ControlGroup --property=Job`, whose answer is
    //           the path the manager has the slice's control group at (nothing for a slice it has none for) and the job it has
    //           queued for the slice (nothing where it has none).
    // Why:      The path is what the route is built on and what a manager that is there answers to, and the job is the one fact
    //           that tells a slice being stopped from a slice at rest: both report the same path, and a scope made in the slice
    //           while the stop stands is refused as destructive. Both are read in one program start, which is what keeps an
    //           ordinary call at one program.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ordinary = ['--user', 'show', settings.slice, '--property=ControlGroup', '--property=Job']
    // ```
    let ordinary: Vec<String> = vec![
        "--user".to_string(),
        "show".to_string(),
        settings.slice.clone(),
        "--property=ControlGroup".to_string(),
        "--property=Job".to_string(),
    ];
    // What:     `question` is that question, or the one a control put in its place.
    // Why:      A control asks through a stand-in whose answer it chose, so it replaces the words of the ask.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const question = settings.question ?? ordinary
    // ```
    let question: Vec<&str> = settings
        .question
        .as_ref()
        .unwrap_or(&ordinary)
        .iter()
        .map(String::as_str)
        .collect();
    // What:     `began` is the clock reading the ask is timed from, so a program that ran out of its bound is told from one
    //           that answered at once.
    // Why:      A manager that answered nothing inside the bound is a host without one, and no second ask changes that.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const began = performance.now()
    // ```
    let began: Instant = Instant::now();
    // What:     `answered` is what the program wrote, or the reason it failed.
    // Why:      Both are needed: an answer is read for a path, and a failure is what a refusal or a moment is built from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let answered: string
    // try { answered = runPlain(settings.systemctl, question, settings.environment, settings.discoveryBound) }
    // catch (reason) { throw { reason: `the user manager cannot be asked where ${settings.slice} is (${reason})`, mayBePutAgain: elapsed < bound } }
    // ```
    let answered: String = match run_plain(
        &settings.systemctl,
        &question,
        &settings.environment,
        settings.discovery_bound,
    ) {
        Ok(text) => text,
        Err(reason) => {
            // What:     `FailedAsk { .. }` is the failure this ask leaves: the words name which ask failed, and `may_be_put_again`
            //           is false only where the program used the whole bound it was given.
            // Why:      The caller puts the question again on a failure that came back at once, and ends where the manager
            //           answered nothing at all.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // throw { reason: `the user manager cannot be asked where ${settings.slice} is (${reason})`, mayBePutAgain: !spentItsBound(began) }
            // ```
            return Err(FailedAsk {
                reason: format!(
                    "the user manager cannot be asked where {} is ({reason})",
                    settings.slice
                ),
                may_be_put_again: !spent_its_bound(settings, began.elapsed()),
            });
        }
    };
    // What:     `what_the_manager_reported(&answered)` is the answer read as the path the slice has and the job queued on it.
    // Why:      The answer is two named lines, and the reading of them is one function so both callers share it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return whatTheManagerReported(answered)
    // ```
    return Ok(what_the_manager_reported(&answered));
}

/// The words systemd's own client prints where the connection to the manager went away under a request,
/// which are the words an answer of the manager's cannot carry.
/// What it rests on,
/// each form copied from a capture of 2026-11-27:
/// `Transport endpoint is not connected`
/// (a start in flight,
/// `Failed to start app.slice: Transport endpoint is not connected See user logs and 'systemctl --user status
/// app.slice' for details.`,
/// read in a container whose manager reexecuted under the request),
/// `Connection refused`
/// (a manager reexecuting,
/// `Failed to connect to user scope bus via local transport: Connection refused`),
/// `Connection reset by peer`
/// (the same in the package document's container measurement),
/// and `Message recipient disconnected from message bus without replying`
/// (the guest's session bus,
/// same document).
/// What it does not rest on:
/// any list of the manager's own answers:
/// those are recognised by naming the slice,
/// not by their reason,
/// and a reason no one has seen is read as a moment rather than as a refusal.
pub(crate) const THE_WORDS_OF_A_TRANSPORT_THAT_WENT_AWAY: [&str; 4] = [
    "Transport endpoint is not connected",
    "Connection reset by peer",
    "Connection refused",
    "Message recipient disconnected from message bus without replying",
];

/// What:
/// `fn the_manager_answered_a_start(reason: &str, slice: &str) -> bool` says whether the words of a failed start are the manager's
/// own answer about the slice.
/// Why:
/// A start can fail three ways and the words are what tells them apart:
/// it can fail before the manager sees it (`Failed to connect to user scope bus via local transport: Connection refused`),
/// its reply can be lost after the manager accepted it (`Failed to start app.slice: Transport endpoint is not connected ...`),
/// or the manager can answer about the slice (`Failed to start app.slice: Unit app.slice is masked.`).
/// Only the third is a refusal a wait cannot change,
/// so it is recognised positively:
/// the answer names the slice after the verb,
/// and carries none of the transport's words.
/// The first two are what the asking again is for (measured 2026-11-27 in a container's own manager:
/// `0.1.5-alpha.16` delegated in every call of the three states,
/// `0.1.5-alpha.17` in none).
/// Where the words cannot tell:
/// a manager that refuses with the slice's name and a transport's words in one sentence,
/// a refusal that does not name the slice (`Job canceled`),
/// and a transport failure that names neither the slice nor anything this list holds,
/// are all read as moments,
/// which spends the span of the asking on a refusal rather than refusing a call over a moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theManagerAnsweredAStart(reason: string, slice: string): boolean
/// ```
pub(crate) fn the_manager_answered_a_start(
    reason: &str,
    slice: &str,
) -> bool {
    // What:     `after_the_verb` is what the manager said after `Failed to start <slice>: `, or nothing where its words have no such
    //           opening.
    // Why:      The slice's name appears in the opening of every start failure, so only what comes after it says whether the
    //           manager was speaking about the slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const afterTheVerb = reason.split(`Failed to start ${slice}: `)[1] ?? ''
    // ```
    let opening: String = format!("Failed to start {slice}: ");
    let after_the_verb: &str = match reason.split_once(&opening) {
        Some((_, rest)) => rest,
        None => return false,
    };
    let carries_a_transports_words: bool = THE_WORDS_OF_A_TRANSPORT_THAT_WENT_AWAY
        .iter()
        .any(|words| return reason.contains(words));
    return after_the_verb.contains(slice) && !carries_a_transports_words;
}

/// What:
/// `fn spent_its_bound(settings: &RouteSettings, took: Duration) -> bool` says whether one program that failed used the whole bound
/// the ask gave it.
/// Why:
/// Only the program's own time says it ran out of its bound:
/// the discovery starts one program per ask and a moment of a manager that is there must still be asked through,
/// so the time around a whole ask tells neither.
/// `DISCOVERY_POLL` is the interval the wait notices an end at,
/// and twice it is the most an ended program can be noticed after its deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function spentItsBound(settings: RouteSettings, took: number): boolean
/// ```
pub(crate) fn spent_its_bound(
    settings: &RouteSettings,
    took: Duration,
) -> bool {
    return took + DISCOVERY_POLL * 2 >= settings.discovery_bound;
}
