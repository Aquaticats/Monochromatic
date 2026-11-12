//! What:
//!  The one line a commit prints when the service manager has not answered the provisioning of its worktree's commit
//!  service within a moment.
//! Why:
//!   Provisioning asks the manager to read its units again after a template changed,
//! and that read takes every unit the manager
//!  holds:
//! on 2026-11-11 it took 13.3 seconds on this host with its load average at 120,
//! and the commit that asked said nothing at all
//!  until it was over.
//! A wait nobody can see is indistinguishable from a hang,
// and the owner's rules are that nothing waits without saying
//  so and that a wait says what it waits for and on whom.
//! The line is written from a thread of its own because the ask itself blocks the commit's thread;
//! the thread is ended as soon
//!  as the manager answered, so a commit that waited no time prints nothing.
//! It is written to the wrapper's log at the warning priority,
//! as every other provisioning step that did not go as
//!  intended is,
//! so `journalctl --priority=warning` holds it beside them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const notice = SlowManagerNotice.arm('monochromatic-git-policy-cli@abc.socket'); /* the calls */ notice.finish();
//! ```

use std::sync::mpsc::{RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

/// How long the manager may take to answer one provisioning step before the commit says so.
///  Measured:
/// the manager reloaded its units in 50 to 300 ms on an idle machine,
/// and the slowest of 52 cold service starts took 1,041 ms
///  (`startWaitMs` of this repository's commit timing log),
/// so one second is the wait the other notices use (`START_NOTICE_GRACE`)
/// and longer than
///  either step takes when the machine is not strained.
pub const SLOW_MANAGER_NOTICE_AFTER: Duration = Duration::from_secs(1);

/// What:
///  The sentence that says what provisioning is waiting for,
/// on whom,
/// and what happens to the commit meanwhile.
/// Why:
///   The unit names the worktree whose service is being set up,
/// and the person can ask the manager about that very unit;
/// the commit is never lost by the wait,
/// which
///  the sentence says so nobody thinks it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// slowManagerNotice('monochromatic-git-policy-cli@abc.socket', 1) // 'monochromatic-git-policy-cli: the service manager ...'
/// ```
pub fn slow_manager_notice(unit: &str, waited: Duration) -> String {
    return format!(
        "monochromatic-git-policy-cli: the service manager has not answered for {} seconds while the commit service of this worktree is set up ({unit}); a busy machine reads every unit again after a release moves, and this commit goes on as soon as it answers.",
        waited.as_secs()
    );
}

/// What:
///  The line about a slow manager,
/// armed around the calls that ask one and printed once,
/// from a thread of its own,
/// when they outlast [`SLOW_MANAGER_NOTICE_AFTER`].
/// Why:
///   The ask blocks the commit's thread inside one read of the manager's answer,
/// so nothing on that thread can count the seconds
///  it takes.
/// Ending the wait of the thread is what silence is for:
/// a commit whose manager answered at once prints nothing,
/// and
///  nothing outlives the calls it was armed for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { using notice = SlowManagerNotice.arm(unit); await askTheManager(); }
/// ```
pub(crate) struct SlowManagerNotice {
    /// Dropped to end the thread's wait,
    /// which is what tells it the manager answered.
    stop: Option<Sender<()>>,
    /// The thread that prints the line,
    /// joined by the guard so no thread of a finished commit is left.
    thread: Option<JoinHandle<()>>,
}

impl SlowManagerNotice {
    /// Arms the line about a slow manager for the calls that follow until the guard ends.
    pub(crate) fn arm(unit: &str) -> Self {
        let named: String = unit.to_string();
        return Self::arm_saying(move || crate::service_warn!("{}", slow_manager_notice(named.as_str(), SLOW_MANAGER_NOTICE_AFTER)));
    }

    /// Arms what is said for those calls,
    /// so a control reads it on the thread that says it.
    pub(crate) fn arm_saying(saying: impl FnOnce() + Send + 'static) -> Self {
        let (stop, waiting): (Sender<()>, std::sync::mpsc::Receiver<()>) = std::sync::mpsc::channel();
        let thread: Option<JoinHandle<()>> = std::thread::Builder::new()
            .name(String::from("provision notice"))
            .spawn(move || {
                if matches!(waiting.recv_timeout(SLOW_MANAGER_NOTICE_AFTER), Err(RecvTimeoutError::Timeout)) {
                    saying();
                }
            })
            .ok();
        return Self { stop: Some(stop), thread };
    }
}

impl Drop for SlowManagerNotice {
    fn drop(&mut self) {
        // The sender is dropped rather than sent to, so a thread that has not printed yet reads the disconnection and ends
        //  without saying anything.
        drop(self.stop.take());
        if let Some(thread) = self.thread.take() {
            let _joined = thread.join();
        }
    }
}

/// The line and its guard stay out of the release executable.
#[cfg(test)]
#[path = "service_provision_slow_manager_notice_tests.rs"]
mod tests;
