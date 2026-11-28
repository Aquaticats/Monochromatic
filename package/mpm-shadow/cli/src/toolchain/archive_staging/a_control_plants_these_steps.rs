//! What:
//! The steps of a publish that a build made for controls can plant:
//! the exchange answered as a file system without `RENAME_EXCHANGE` answers it,
//! the exchange answered with an error a control names,
//! one or two of the three renames answered with an error a control names,
//! the process ended in the instant between the two renames of the fallback,
//! and what another install does while this one stands at one of its steps,
//! before that copy is moved aside as well as after it.
//! Why:
//! A control has to show every arm of the replace step and of the single rename,
//! and the instructions a second install interleaves with,
//! and a shipped build must hold no way to plant any of them:
//! every read below is compiled out of a build that is not a test,
//! the shape `doc/decision/cli-git-test-only-switches-compiled-out-of-release.md` chose
//! for the git wrapper's own switches.
//! The reason is the one recorded there:
//! a switch that can end the process or refuse a publish turns a working install into a failing one,
//! and a build a release ships holds no such switch.
//! Nothing here is read from the environment at all,
//! so no `Environment=` line and no shell can plant a step:
//! a control plants one for its own thread and for one publish.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a-control-plants-these-steps.ts
//! type RenameRefused = 'the-finished-copy-goes-in' | 'the-copy-there-goes-aside' | 'the-copy-there-comes-back';
//! type Plant = {
//!   refuseTheExchange: boolean;
//!   answerTheExchangeWith?: number;
//!   refuseTheRename?: [RenameRefused, number];
//!   killBetweenTheTwoRenames: boolean;
//!   anotherInstallActs?: (staging: string, finalDir: string) => void;
//! };
//! function plantTheseSteps(plant: Plant): void;    // tests only
//! function exchangeIsRefused(): boolean;           // always false in a shipped build
//! function theExchangesError(): Error | undefined; // always undefined in a shipped build
//! function refusedOrNone(what: RenameRefused): Error | undefined;
//! function reachedTheInstantBetweenTheTwoRenames(staging: string, finalDir: string): void;
//! function reachedTheInstantBeforeTheNameIsGiven(staging: string, finalDir: string): void;
//! ```

/// Imports the error type a planted step answers with.
use std::io;
/// Imports the borrowed path type the two names arrive as.
use std::path::Path;

/// Imports the cell a control's plant lives in.
#[cfg(test)]
use std::cell::Cell;

/// What:
/// Which of a publish's renames a control answers with an error instead of making it.
/// Why:
/// The three renames have three different failure arms,
/// and a control names the one whose arm it shows;
/// a shipped build plants none of them and never builds this value,
/// so the type only says what a control may ask for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RenameRefused = 'the-finished-copy-goes-in' | 'the-copy-there-goes-aside' | 'the-copy-there-comes-back';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RenameRefused {
    /// The rename that gives the version's name the finished copy this publish holds.
    GivesTheVersionItsName,
    /// The rename that moves the copy at the version's name to that copy's aside name,
    /// the first of the two renames a file system without the exchange needs.
    MovesTheCopyAside,
    /// The rename that puts the copy at the aside name back at the version's name.
    PutsTheCopyBack,
}

/// What:
/// The steps of one publish a control plants.
/// Why:
/// The exchange refusal picks the arm a publish takes,
/// the exchange's own error picks what the arm that meets it does,
/// a refused rename picks one of the three failure arms of the rename itself,
/// the kill names the instant only the two-rename arm has,
/// and each action is what a second install does at the instant its own field names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Plant = {
///   refuseTheExchange: boolean;
///   answerTheExchangeWith?: number;
///   refuseTheRename?: [RenameRefused, number];
///   refuseThePutBackWith?: number;
///   killBetweenTheTwoRenames: boolean;
///   anotherInstallActs?: (staging: string, finalDir: string) => void;
///   anotherInstallPublishesBeforeTheAsideRename?: (staging: string, finalDir: string) => void;
/// };
/// ```
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Plant {
    /// Whether the exchange answers as a file system without `RENAME_EXCHANGE` answers it.
    pub(crate) refuse_the_exchange: bool,
    /// The error the exchange answers with instead of changing the two names,
    /// as the operating system's number for it.
    pub(crate) answer_the_exchange_with: Option<i32>,
    /// One rename of this publish,
    /// answered with an operating system error's number instead of being made.
    pub(crate) refuse_the_rename: Option<(RenameRefused, i32)>,
    /// The put-back that follows the refused rename,
    /// answered with this operating system error's number,
    /// which is the arm whose message names both errors:
    /// the two-rename fallback whose second rename fails and whose put-back fails too.
    /// It is a field of its own because that arm needs two of the three renames refused at once,
    /// and `refuse_the_rename` names one and is spent by the answer it gives.
    pub(crate) refuse_the_put_back_with: Option<i32>,
    /// Whether the process ends in the instant between the two renames.
    pub(crate) kill_between_the_two_renames: bool,
    /// What another install does while this publish stands at its step,
    /// with the name this publish holds the finished copy at and the version's name.
    pub(crate) another_install_acts: Option<fn(&Path, &Path)>,
    /// What another install publishes in the instant before this publish moves the copy at the version's name aside,
    /// with the name this publish holds the finished copy at and the version's name.
    pub(crate) another_install_publishes_before_the_aside_rename: Option<fn(&Path, &Path)>,
}

#[cfg(test)]
thread_local! {
    /// The steps this thread's next publish takes.
    static PLANTED: Cell<Option<Plant>> = const { Cell::new(None) };
}

/// What:
/// Plants `plant` for the next publish of this thread.
/// Why:
/// A test asks for the arm it means and nothing else,
/// and the thread is where its own publish runs,
/// so two controls in one test binary never plant each other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function plantTheseSteps(plant: Plant): void
/// ```
#[cfg(test)]
pub(crate) fn plant_these_steps(plant: Plant) {
    PLANTED.with(|cell| {
        cell.set(Some(plant));
    });
}

/// What:
/// Forgets a plant,
/// so a publish made after a control's own takes the shipped steps.
/// Why:
/// A plant that outlived its control would decide another control's publish on the same thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clearThePlant(): void
/// ```
#[cfg(test)]
pub(crate) fn clear_the_plant() {
    PLANTED.with(|cell| {
        cell.set(None);
    });
}

/// What:
/// The steps this thread's next publish takes,
/// or nothing when no control planted any.
/// Why:
/// Every step below asks this one cell,
/// so a control plants once and every arm it names reads the same plant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function plantedSteps(): Plant | undefined
/// ```
#[cfg(test)]
fn planted_steps() -> Option<Plant> {
    return PLANTED.with(|cell| return cell.get());
}

/// What:
/// Whether the exchange answers as a file system without `RENAME_EXCHANGE` answers it,
/// which is what a control's plant says.
/// Why:
/// A control needs the two-rename arm on a machine whose file systems all exchange the two names,
/// and this is the only way to ask for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exchangeIsRefused(): boolean { return plantedSteps()?.refuseTheExchange === true; }
/// ```
#[cfg(test)]
pub(super) fn exchange_is_refused() -> bool {
    let planted: Option<Plant> = planted_steps();
    return planted.is_some_and(|plant| return plant.refuse_the_exchange);
}

/// The answer a shipped build gives:
/// the exchange is attempted,
/// because nothing can ask it not to be.
#[cfg(not(test))]
pub(super) fn exchange_is_refused() -> bool {
    return false;
}

/// What:
/// The error the exchange answers with instead of changing the two names,
/// which is what a control's plant says,
/// or nothing when the exchange is to be made.
/// Why:
/// A control shows what the arm that meets an error outside the "cannot exchange" list does,
/// which no file system on this machine answers on its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theExchangesError(): Error | undefined {
///   const code = plantedSteps()?.answerTheExchangeWith;
///   return code === undefined ? undefined : Object.assign(new Error('planted'), { errno: code });
/// }
/// ```
#[cfg(test)]
pub(super) fn the_exchanges_error() -> Option<io::Error> {
    let planted: Option<Plant> = planted_steps();
    // What: `planted?.answer_the_exchange_with?` takes the number through two optional steps (TS: `?.` twice).
    // Why: a publish with no plant, or one whose plant answers no error, makes the exchange itself.
    let code: i32 = planted?.answer_the_exchange_with?;
    forget_the_exchange_answer();
    return Some(io::Error::from_raw_os_error(code));
}

/// What:
/// Takes the error a control planted for the exchange out of the plant,
/// so the exchange a later install in the same test makes is the real one.
/// Why:
/// A control that shows a failed publish leaves a run that installs afterwards,
/// and that install's publish must not meet the same planted answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function forgetTheExchangeAnswer(): void { if (planted) planted.answerTheExchangeWith = undefined; }
/// ```
#[cfg(test)]
fn forget_the_exchange_answer() {
    PLANTED.with(|cell| {
        if let Some(mut plant) = cell.get() {
            plant.answer_the_exchange_with = None;
            cell.set(Some(plant));
        }
    });
}

/// The answer a shipped build gives:
/// the exchange is attempted and answers its own error,
/// if any.
#[cfg(not(test))]
pub(super) fn the_exchanges_error() -> Option<io::Error> {
    return None;
}

/// What:
/// The error a control planted for the rename that does `what`,
/// as the operating system's number for it,
/// or nothing when that rename is to be made.
/// Why:
/// A control shows each failure arm of the three renames,
/// and every one of them fails for a reason no disposable fixture can make on demand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusedOrNone(what: RenameRefused): Error | undefined {
///   const refused = plantedSteps()?.refuseTheRename;
///   return refused?.[0] === what ? Object.assign(new Error('planted'), { errno: refused[1] }) : undefined;
/// }
/// ```
#[cfg(test)]
pub(super) fn refused_or_none(what: RenameRefused) -> Option<io::Error> {
    // What: this thread's plant, which a control set and which one answer takes the refusal out of.
    // Why: a publish with no plant makes every rename itself, and the plant is read once per question.
    let planted: Plant = planted_steps()?;
    // What: the put-back that fails after the refused rename, which is the sixth arm of a failed publish.
    // Why: that arm is the one whose message names two errors, so a plant that refuses one rename cannot reach it;
    //  the second refusal is asked for by a field of its own and is not spent by the first answer.
    if what == RenameRefused::PutsTheCopyBack
        && let Some(restore_code) = planted.refuse_the_put_back_with
    {
        return Some(io::Error::from_raw_os_error(restore_code));
    }
    let (refused, code): (RenameRefused, i32) = planted.refuse_the_rename?;
    if refused != what {
        return None;
    }
    // What: `cell.set(Some(plant))` writes the plant back with the refusal taken out.
    // Why: the answer is the next rename that does this, and a run that installs after a failed publish makes another
    //  rename of the same kind, which must be the real one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // planted.refuseTheRename = undefined;
    // ```
    PLANTED.with(|cell| {
        if let Some(mut rest) = cell.get() {
            rest.refuse_the_rename = None;
            cell.set(Some(rest));
        }
    });
    return Some(io::Error::from_raw_os_error(code));
}

/// The answer a shipped build gives:
/// every rename is made,
/// because nothing can ask for a refusal.
#[cfg(not(test))]
pub(super) fn refused_or_none(_what: RenameRefused) -> Option<io::Error> {
    return None;
}

/// What:
/// Runs what a control planted as another install at the step this publish stands at,
/// with the name it holds the finished copy at and the version's name.
/// Why:
/// The window a kill between the two renames leaves is also the window in which a second install acts,
/// and a control picks that step exactly,
/// so what it shows never depends on timing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function anotherInstallActs(staging: string, finalDir: string): void {
///   plantedSteps()?.anotherInstallActs?.(staging, finalDir);
/// }
/// ```
#[cfg(test)]
fn another_install_acts(
    staging: &Path,
    final_dir: &Path,
) {
    let planted: Option<Plant> = planted_steps();
    if let Some(acts) = planted.and_then(|plant| return plant.another_install_acts) {
        acts(staging, final_dir);
    }
}

/// What:
/// Runs what another install may do,
/// and then ends the process when a control planted the kill,
/// at the instant between the two renames of the fallback.
/// Why:
/// It is the one instant a publish that cannot exchange the two names has,
/// and a control shows what a kill in it leaves behind and what a second install puts there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reachedTheInstantBetweenTheTwoRenames(staging: string, finalDir: string): void {
///   anotherInstallActs(staging, finalDir);
///   if (plantedSteps()?.killBetweenTheTwoRenames) process.kill(process.pid, 'SIGKILL');
/// }
/// ```
#[cfg(test)]
pub(super) fn reached_the_instant_between_the_two_renames(
    staging: &Path,
    final_dir: &Path,
) {
    another_install_acts(staging, final_dir);
    let planted: Option<Plant> = planted_steps();
    if planted.is_some_and(|plant| return plant.kill_between_the_two_renames) {
        end_this_process_as_a_kill_does();
    }
}

/// The same instant in a shipped build:
/// nothing else acts there and nothing ends the process.
#[cfg(not(test))]
pub(super) fn reached_the_instant_between_the_two_renames(
    _staging: &Path,
    _final_dir: &Path,
) {
}

/// What:
/// Runs what another install publishes in the instant before this publish moves the copy there aside,
/// with the name it holds the finished copy at and the version's name.
/// Why:
/// A second install can give the version its name between this publish's check that the name holds no finished copy
/// and the rename that moves whatever is there aside,
/// and the copy moved aside then reads as finished;
/// a control picks that instant exactly,
/// it is the only way the arm that puts such a copy back is reached,
/// and it publishes once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reachedTheInstantBeforeTheAsideRename(staging: string, finalDir: string): void {
///   plantedSteps()?.anotherInstallPublishesBeforeTheAsideRename?.(staging, finalDir);
/// }
/// ```
#[cfg(test)]
pub(super) fn reached_the_instant_before_the_aside_rename(
    staging: &Path,
    final_dir: &Path,
) {
    let planted: Option<Plant> = planted_steps();
    let Some(publishes) = planted.and_then(|plant| {
        return plant.another_install_publishes_before_the_aside_rename;
    }) else {
        return;
    };
    // What: the publisher is taken out of the plant before it runs, so a second publish of this run meets the real step.
    // Why: the install that follows a failed recovery publishes too,
    //  and its copy must not be the last thing this control put at the version's name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // planted.anotherInstallPublishesBeforeTheAsideRename = undefined;
    // ```
    PLANTED.with(|cell| {
        if let Some(mut plant) = cell.get() {
            plant.another_install_publishes_before_the_aside_rename = None;
            cell.set(Some(plant));
        }
    });
    publishes(staging, final_dir);
}

/// The same instant in a shipped build:
/// nothing else acts there.
#[cfg(not(test))]
pub(super) fn reached_the_instant_before_the_aside_rename(
    _staging: &Path,
    _final_dir: &Path,
) {
}

/// What:
/// Runs what another install may do at the instant after a publish decided the version's name is absent,
/// before the one rename that gives it the finished copy.
/// Why:
/// A second install can put a copy at that name in this instant,
/// and a control picks the step exactly to show what this publish does with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reachedTheInstantBeforeTheNameIsGiven(staging: string, finalDir: string): void {
///   anotherInstallActs(staging, finalDir);
/// }
/// ```
#[cfg(test)]
pub(super) fn reached_the_instant_before_the_name_is_given(
    staging: &Path,
    final_dir: &Path,
) {
    another_install_acts(staging, final_dir);
}

/// The same instant in a shipped build:
/// nothing else acts there.
#[cfg(not(test))]
pub(super) fn reached_the_instant_before_the_name_is_given(
    _staging: &Path,
    _final_dir: &Path,
) {
}

/// What:
/// Ends this process with `SIGKILL`,
/// the signal no handler can catch and after which no cleanup runs.
/// Why:
/// The control shows what a killed install leaves,
/// so nothing of the installer may run after the instant:
/// no destructor and no cleanup of the directories it made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.kill(process.pid, 'SIGKILL');
/// ```
#[cfg(all(test, unix))]
fn end_this_process_as_a_kill_does() -> ! {
    // The wrapper is safe, and nothing follows a `SIGKILL` delivered to this process.
    rustix::process::kill_process(rustix::process::getpid(), rustix::process::Signal::KILL).ok();
    // A signal that the system refused to deliver leaves this process alive, so it ends the other way.
    std::process::abort()
}

/// What:
/// Ends this process at once on a platform without `SIGKILL`.
/// Why:
/// A control that cannot use that signal still must not run any cleanup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.abort();
/// ```
#[cfg(all(test, not(unix)))]
fn end_this_process_as_a_kill_does() -> ! {
    std::process::abort()
}
