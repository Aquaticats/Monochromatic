//! What:
//!  A child waited for by the thread that owns it,
//! with a ceiling,
//! so a query that never answers is ended by its owner.
//! Why:
//!  A number is a safe address only while its process is this process's own unreaped child.
//! A caller that
//!  stopped waiting at its ceiling and signalled the number later could reach a successor of the child,
//! because the
//!  thread that waits for the child reaps it as soon as it ends
//! (`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
//!  the numeric-signal findings of the second and third reviews).
//! Here the thread that waits is the thread that kills:
//! while it waits it has not
//!  reaped the child,
//! so the number still names it,
//! and nothing else can have it.
//! A child that cannot be ended
//!  therefore cannot exist on this path,
//! and no descriptor and no pin is needed to end one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const status = await withDeadline(child.exited, ceiling) ?? (child.kill(), await child.exited, throw ceilingReached());
//! ```

use crate::bounded_push::STOP_GRACE;
use std::process::{Child, ExitStatus};
use std::time::{Duration, Instant};

// What:
//  A control that refuses the next kill, which is how a child the kernel will not let end looks from this wait's side.
// Why:
//   No test on this host can hold a process in an uninterruptible call,
//   and the state that matters to this wait is a child that is not ended by its signal.
#[cfg(test)]
thread_local! {
    /// Whether the next kill is refused.
    static A_KILL_IS_REFUSED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Ask a child to end,
/// unless a control refuses the signal.
fn kill(child: &mut Child) -> std::io::Result<()> {
    #[cfg(test)]
    if A_KILL_IS_REFUSED.with(|refused| { return refused.replace(false); }) {
        return Ok(());
    }
    return child.kill();
}

/// How often a waiting thread looks at its child.
///  A child that ends is noticed within this,
/// and the wait costs one look per interval.
pub const LOOK_INTERVAL: Duration = Duration::from_millis(10);

/// What:
///  What a wait that met its ceiling came to:
/// whether the child was ended and reaped,
/// or left for the kernel to release.
/// Why:
///   A query in an uninterruptible call cannot be ended,
/// so the person is told that it was left running rather than
///  that it was ended,
/// and the two sentences are the difference between "it is gone" and "it is still there".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AtTheCeiling = { leftRunning: boolean };
/// ```
#[derive(Debug)]
pub struct AtTheCeiling {
    /// Whether the child could not be ended and was left running.
    pub left_running: bool,
}

impl std::fmt::Display for AtTheCeiling {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str("the child did not answer within its ceiling");
    }
}

impl std::error::Error for AtTheCeiling {}

/// Whether a wait that met its ceiling left the child running,
/// for the sentence the caller prints.
pub fn left_running_when_the_ceiling_was_reached(error: &std::io::Error) -> bool {
    let Some(reached) = error.get_ref().and_then(ceiling_reached_in) else {
        return false;
    };
    return reached.left_running;
}

/// The ceiling answer inside an error,
/// when it is this error's cause.
fn ceiling_reached_in<'cause>(inner: &'cause (dyn std::error::Error + Send + Sync + 'static)) -> Option<&'cause AtTheCeiling> {
    return inner.downcast_ref::<AtTheCeiling>();
}

/// What:
///  The sentence for a query that did not answer within its ceiling,
/// ended or left running.
/// Why:
///  The person has to know the query is what was stopped,
/// the limit it met,
/// whether it is gone,
/// and that nothing of the repository was changed by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${program} did not answer within ${seconds} seconds ... and was ${leftRunning ? 'left running' : 'ended'} ...`
/// ```
pub fn ceiling_reached_text(program: &str, ceiling: Duration, left_running: bool) -> String {
    let ending: &str = if left_running {
        "and could not be ended: it is in a call the kernel has not returned from (a stalled disk or a network file system), so it \
         was left running, and the system reaps it when that call returns"
    } else {
        "and was ended"
    };
    return format!(
        "{program} did not answer within {} seconds, the longest a read-only query of this wrapper waits, {ending}. Such a query \
         changes nothing of the repository. Hint: check with `ps` what that program waits for, look at the disk it reads, and run \
         the command again.",
        ceiling.as_secs()
    );
}

/// What:
///  Waits for `child` to end,
/// but no longer than `ceiling`;
/// past it the child is killed,
/// given `STOP_GRACE` to go,
/// and `TimedOut` is returned.
/// Why:
///   The caller owns the child and has not reaped it,
/// so the kill reaches the child and never a number that was reused
///  after it was reaped.
///  A child in an uninterruptible call cannot end until the kernel lets it go,
/// so the wait after the kill is
///  the stop's grace and no more:
/// a child that is still there then is left to be reaped by the thread that watches it,
///  and the sentence says which program was left
/// (the third review's finding 3).
/// The owner's question about what a *commit* should do
///  there is unanswered in `doc/handover/questions-for-the-owner.md`,
/// and this is a read-only query,
/// which says what it left and
///  changes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function waitWithin(child: Child, ceiling: Duration): ExitStatus; // throws and hands a stuck child to a reaper
/// ```
pub fn wait_within(mut child: Child, ceiling: Duration) -> std::io::Result<ExitStatus> {
    let began: Instant = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        let left: Duration = ceiling.saturating_sub(began.elapsed());
        if left.is_zero() {
            break;
        }
        std::thread::sleep(left.min(LOOK_INTERVAL));
    }
    // The ceiling is the end of waiting for it: the query is asked to go, and given the stop's grace to be reaped in.
    let _asked: std::io::Result<()> = kill(&mut child);
    let grace: Instant = Instant::now();
    loop {
        if let Some(_status) = child.try_wait()? {
            return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, AtTheCeiling { left_running: false }));
        }
        if grace.elapsed() >= STOP_GRACE {
            // Nothing here can end it: it is in a call the kernel has not returned from. A thread of this command reaps it when
            //  the kernel lets it go, and keeps it if the system refuses that thread, so the child is never lost.
            crate::stuck_children_reaped_when_the_kernel_lets_go::reap_soon(child);
            return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, AtTheCeiling { left_running: true }));
        }
        std::thread::sleep(LOOK_INTERVAL);
    }
}

/// Controls for a child its own thread waits for stay out of the release executable.
#[cfg(test)]
#[path = "child_waited_for_within_a_ceiling_tests.rs"]
mod tests;
