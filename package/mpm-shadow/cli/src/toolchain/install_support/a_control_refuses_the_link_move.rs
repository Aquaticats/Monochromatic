//! What:
//! The one rename of a link write that a build made for controls can refuse:
//! the rename that gives a link its final name.
//! Why:
//! A control has to show what a run leaves when the move of `current` fails,
//! and no disposable fixture can make one `rename` fail on demand;
//! a shipped build holds no way to refuse it:
//! every read below is compiled out of a build that is not a test,
//! the shape `doc/decision/cli-git-test-only-switches-compiled-out-of-release.md` chose,
//! as the publish's own plant does (`archive_staging`,
//! `a_control_plants_these_steps`).
//! Nothing here is read from the environment at all,
//! so no `Environment=` line and no shell can plant it:
//! a control plants one for its own thread and for the link move it is about.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a-control-refuses-the-link-move.ts
//! type HowOften = 'once' | 'every-time';
//! function refusingTheNextLinkMove(code: number, howOften: HowOften): RefusalGuard;  // tests only
//! function refusedLinkMove(): Error | undefined;  // always undefined in a shipped build
//! ```

/// Imports the error type a planted refusal answers with.
use std::io;

/// Imports the cell a control's refusal lives in.
#[cfg(test)]
use std::cell::Cell;

/// What:
/// How many of a run's link moves a control's refusal answers.
/// Why:
/// A control of the rule that nothing is removed under a link that cannot be moved needs the link to stay where it is,
/// while a control of what a later step of the same run does needs the first move alone refused;
/// a shipped build plants neither,
/// so the type only says what a control may ask for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HowOften = 'once' | 'every-time';
/// ```
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HowOften {
    /// The next link move.
    Once,
    /// Every link move until the control's guard is dropped.
    EveryTime,
}

/// What:
/// The refusal one control planted:
/// the operating system's number for the error,
/// and how many moves it answers.
/// Why:
/// A control names a failure no disposable fixture can make on demand,
/// and the two counts are what tell a rule from the step that repairs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { code: number; howOften: HowOften };
/// ```
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Refusal {
    /// The operating system's number for the error the move answers with.
    code: i32,
    /// How many moves the refusal answers.
    how_often: HowOften,
}

#[cfg(test)]
thread_local! {
    /// The refusal this thread's next link move answers with.
    static THE_REFUSAL: Cell<Option<Refusal>> = const { Cell::new(None) };
}

/// What:
/// Forgets a refusal,
/// so a link move after a control's own is the real one.
/// Why:
/// A refusal that outlived its control would decide another control's link move on the same thread,
/// and one armed for every move would keep a later test's link from being written at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function forgetTheRefusal(): void
/// ```
#[cfg(test)]
fn forget_the_refusal() {
    THE_REFUSAL.with(|cell| {
        cell.set(None);
    });
}

/// What:
/// The release of a refusal,
/// taken by the guard `refusing_the_next_link_move` answers.
/// Why:
/// A control's failing assertion raises a panic,
/// which skips the line that would forget the refusal,
/// and a refusal left armed for every move would decide the next run on this thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RefusalGuard { [Symbol.dispose]() { forgetTheRefusal(); } }
/// ```
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct LinkMoveRefusalGuard;

/// The release of a refusal.
#[cfg(test)]
impl Drop for LinkMoveRefusalGuard {
    /// What:
    /// Forgets the refusal this thread planted,
    /// which is the one release a control's body must reach however it leaves.
    /// Why:
    /// The drop is the only place that runs on the way out of a panic,
    /// and a refusal left behind would keep the next run's link from being written.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose]() { forgetTheRefusal(); }
    /// ```
    fn drop(&mut self) {
        forget_the_refusal();
    }
}

/// What:
/// Plants `refusal` for this thread's next link move,
/// and answers the guard that forgets it when the control's body leaves,
/// however it leaves.
/// Why:
/// A control asks for the failure it means and nothing else,
/// the thread is where its own run writes the link,
/// and the guard means a failed assertion cannot arm a refusal for another test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusingTheNextLinkMove(code: number, howOften: HowOften): RefusalGuard {
///   REFUSALS[threadId] = { code, howOften };
///   return new RefusalGuard();
/// }
/// ```
#[cfg(test)]
pub(crate) fn refusing_the_next_link_move(
    code: i32,
    how_often: HowOften,
) -> LinkMoveRefusalGuard {
    THE_REFUSAL.with(|cell| {
        cell.set(Some(Refusal {
            code,
            how_often,
        }));
    });
    return LinkMoveRefusalGuard;
}

/// What:
/// The error a control planted for the next link move,
/// or nothing when that move is to be made.
/// Why:
/// A control shows what a run leaves when the link cannot be placed,
/// which is one `rename` of the link write,
/// and the failure arm after that move is the one the control is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusedLinkMove(): Error | undefined {
///   const refusal = plantedRefusal();
///   if (refusal === undefined) return undefined;
///   if (refusal.howOften === 'once') forgetTheRefusal();
///   return Object.assign(new Error('planted'), { errno: refusal.code });
/// }
/// ```
#[cfg(test)]
pub(super) fn refused_link_move() -> Option<io::Error> {
    let refusal: Refusal = THE_REFUSAL.with(|cell| return cell.get())?;
    if refusal.how_often == HowOften::Once {
        forget_the_refusal();
    }
    return Some(io::Error::from_raw_os_error(refusal.code));
}

/// The answer a shipped build gives:
/// every link move is made,
/// because nothing can ask for a refusal.
#[cfg(not(test))]
pub(super) fn refused_link_move() -> Option<io::Error> {
    return None;
}
