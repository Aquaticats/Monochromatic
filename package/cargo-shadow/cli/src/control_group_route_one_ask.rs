//! What:
//! One ask of the user manager where a slice's control group is:
//! the words it failed with,
//! the answer it gave,
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
/// Three failures are the end of the asking.
/// A manager that answered nothing inside its bound is what a host without one looks like,
/// and a second ask would only spend a second bound;
/// a manager that answered with a refusal of its own (a masked slice,
/// a policy that denies the start) answers the same way
/// a moment later,
/// which is asked once and not five times (measured 2026-11-27:
/// one ask of `start app.slice` against a masked slice
/// answered `Failed to start app.slice: Unit app.slice is masked.`,
/// and five asks spent 895 ms where the answer was the first one);
/// and everything else can be a moment of pressure,
/// and asking again is what tells the two apart.
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
/// the control group's path,
/// or the manager's answer that the slice has no control group.
/// Why:
/// The two are different facts about the host,
/// and only the first one is a route:
/// a slice with no control group is one that has not been started,
/// and a manager that answers at all is one that can be reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhereTheSliceIs = { path: string } | { noControlGroup: true }
/// ```
pub(crate) enum WhereTheSliceIs {
    /// What:
    /// the manager's answer that is a control group path.
    /// Why:
    /// The path is what the route is built on.
    APath(
        /// The path the manager reported,
        /// which is where the slice's control group is.
        String,
    ),
    /// The manager answered,
    /// and its answer is no control group path at all.
    NoControlGroup,
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
    // What:     `ordinary` is the question `systemctl --user show <slice> --property=ControlGroup --value`, whose answer is the
    //           path the manager has the slice's control group at, and nothing at all for a slice it has none for.
    // Why:      The path is what the route is built on, and what a manager that is there answers to.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ordinary = ['--user', 'show', settings.slice, '--property=ControlGroup', '--value']
    // ```
    let ordinary: Vec<String> = vec![
        "--user".to_string(),
        "show".to_string(),
        settings.slice.clone(),
        "--property=ControlGroup".to_string(),
        "--value".to_string(),
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
    // What:     `reported` is that answer with the newline `systemctl` ends it with taken off.
    // Why:      The path is compared as a path, so the answer's own trailing whitespace is not part of it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const reported = answered.trim()
    // ```
    let reported: &str = answered.trim();
    if reported.is_empty() || !reported.starts_with('/') || reported.contains('\n') {
        // What:     `WhereTheSliceIs::NoControlGroup` is the manager's answer that this slice has no control group.
        // Why:      The manager answered, so the transport works and the answer is its own: an answer that is not a path is
        //           no route, and no wait makes it one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { noControlGroup: true }
        // ```
        return Ok(WhereTheSliceIs::NoControlGroup);
    }
    // What:     `WhereTheSliceIs::APath(...)` is the manager's answer, wrapped as the path the route is built on.
    // Why:      The caller needs the path and not the answer's words.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { path: reported }
    // ```
    return Ok(WhereTheSliceIs::APath(reported.to_string()));
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
