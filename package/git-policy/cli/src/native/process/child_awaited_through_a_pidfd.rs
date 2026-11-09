//! What:
//!  A child started with the standard library and awaited on the tokio runtime through a pidfd (a file descriptor that the
//!  kernel makes readable when the process it names exits),
//! with no thread and no signal handler of its own.
//! Why:
//!  The owner chose "std start + pidfd" (`doc/planning/git-policy-cli-full-tokio.md`,
//! choice 7).
//! `tokio::process` can report a
//!  failed start after the child is running (`src/process/unix/mod.rs`,
//! `build_child`,
//! tokio 1.54.1),
//! which the start retry of
//!  `push/start_retry.rs` cannot undo and which would leave an unsupervised Git beside a cleanup that removes what Git writes.
//!  Here the standard library starts the child (so a failed start leaves no child),
//! and every failure after the start leaves
//!  the child owned by this value until it is reaped or deliberately handed over.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const child = spawn('git', args);            // started elsewhere
//! const owned = adopt(child);                  // never throws: failures after the start only change how the wait is done
//! const status = await owned.wait();           // never blocks the event loop
//! ```

use crate::bounded_push::STOP_GRACE;
use crate::member_signalled_through_its_pidfd::{Delivered, kill_members};
use crate::process_tree::{TreeMember, descendants_of_read};
use std::os::fd::OwnedFd;
use std::process::{Child, ExitStatus};
use std::time::{Duration, Instant};
use tokio::io::Interest;
use tokio::io::unix::AsyncFd;

/// How often a child with no pidfd is looked at:
/// the fallback is a timer,
/// not a blocking wait and not a thread.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// How often a child that was killed is looked at while the stop's grace runs.
const LOOK_INTERVAL: Duration = POLL_INTERVAL;

/// A failure to inject after the start,
/// so the invariant "none of them is a failed start" can be tested.
/// Production uses `None`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InjectedFailure {
    /// No failure:
    /// use the pidfd.
    None,
    /// `pidfd_open` fails,
    /// as on a kernel without it or at a descriptor limit.
    PidfdOpen,
    /// The pidfd cannot be registered with the reactor (`epoll_ctl`),
    /// as under a watch or memory limit.
    Registration,
}

/// A running child and,
/// when the system gave one,
/// the pidfd that says when it exits.
pub struct OwnedChild {
    /// The child;
    /// `None` once it was handed over or reaped.
    child: Option<Child>,
    /// The pidfd registered with the reactor,
    /// when `pidfd_open` and the registration both worked.
    watch: Option<AsyncFd<OwnedFd>>,
}

/// What:
///  Takes ownership of a child the standard library already started.
/// It cannot fail:
/// whatever goes wrong afterwards only
///  changes how the wait is done (a timer instead of the pidfd).
/// Why:
///  Once the start succeeded,
/// no later step may report a failed start that the retry could repeat.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owned = adopt(child);
/// ```
pub fn adopt(child: Child) -> OwnedChild {
    return adopt_with(child, InjectedFailure::None);
}

/// `adopt` with a failure injected after the start,
/// for the controls of the invariant.
pub fn adopt_with(child: Child, injected: InjectedFailure) -> OwnedChild {
    let watch: Option<AsyncFd<OwnedFd>> = open_watch(&child, injected);
    return OwnedChild { child: Some(child), watch };
}

/// The pidfd of `child` registered with the reactor,
/// or nothing when either step fails (or `injected` says it does).
fn open_watch(child: &Child, injected: InjectedFailure) -> Option<AsyncFd<OwnedFd>> {
    if injected == InjectedFailure::PidfdOpen {
        return None;
    }
    let pid: rustix::process::Pid = rustix::process::Pid::from_raw(i32::try_from(child.id()).ok()?)?;
    let descriptor: OwnedFd = rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()).ok()?;
    if injected == InjectedFailure::Registration {
        return None;
    }
    // Registering needs a reactor, and `AsyncFd::new` panics rather than fails outside one, so outside the runtime there is
    //  nothing to register with and the timer takes over.
    if tokio::runtime::Handle::try_current().is_err() {
        return None;
    }
    // SAFETY: an `OwnedFd` holds an open descriptor for its whole life and always returns the same number from `as_raw_fd`, and the
    //  `AsyncFd` takes the `OwnedFd` itself, so nothing else can close it while it is registered.
    return unsafe { AsyncFd::register_with_interest(descriptor, Interest::READABLE) }.ok();
}

impl OwnedChild {
    /// The child's process id.
    pub fn id(&self) -> u32 {
        return self.child.as_ref().map_or(0, Child::id);
    }

    /// Whether the wait uses the pidfd;
    /// `false` means the timer does it,
    /// as after a refused `pidfd_open`.
    pub fn waits_on_a_pidfd(&self) -> bool {
        return self.watch.is_some();
    }

    /// What:
    ///  Waits for the child to exit and reaps it.
    /// The pidfd becomes readable at exit;
    /// with none,
    /// the child is looked at every
    ///  `POLL_INTERVAL` by a timer.
    /// Why:
    ///  No thread waits and no signal handler is installed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const status = await owned.wait();
    /// ```
    pub async fn wait(&mut self) -> std::io::Result<ExitStatus> {
        loop {
            let Some(child) = self.child.as_mut() else {
                return Err(std::io::Error::other("the child was already reaped or handed over"));
            };
            if let Some(status) = child.try_wait()? {
                self.child = None;
                return Ok(status);
            }
            match self.watch.as_ref() {
                Some(watch) => {
                    // Readable means exited; a spurious wake only goes round the loop again.
                    let mut ready = watch.readable().await?;
                    ready.clear_ready();
                }
                None => tokio::time::sleep(POLL_INTERVAL).await,
            }
        }
    }

    /// What:
    ///  Sends `SIGKILL` to the child without waiting for it;
    /// the exit is still collected by `wait`.
    /// Why:
    ///  A stop that kills a child in an uninterruptible call must not wait for the kernel to release it,
    /// so the kill and the wait
    ///  are separate.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// owned.kill(); // does not wait
    /// ```
    pub fn kill(&mut self) -> std::io::Result<()> {
        return match self.child.as_mut() {
            Some(child) => child.kill(),
            None => Ok(()),
        };
    }

    /// What:
    ///  Ends the child with `SIGKILL` and reaps it;
    /// nothing happens when it is already reaped or handed over,
    /// and answers whether a child
    ///  the kernel would not let go was handed to a reaper,
    /// which is what the caller reports as left running.
    /// Why:
    ///   The stop's grace is the whole of the wait for a child,
    /// so a caller must be able to say whether the child it ended is gone;
    /// a
    ///  child in an uninterruptible call is handed to a reaper,
    /// which is the route `SPEC.md` states for a leader that does not end in
    ///  `STOP_GRACE`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function endAndReap(child): { leftRunning: boolean };
    /// ```
    pub fn end_and_reap(&mut self) -> bool {
        if let Some(mut child) = self.child.take() {
            // A kill of an exited child fails harmlessly; the wait reaps either way.
            let _asked: std::io::Result<()> = crate::child_waited_for_within_a_ceiling::kill(&mut child);
            let began: Instant = Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(_status)) => return false,
                    Ok(None) => {}
                    // A record that cannot be read is not the child's end, so the grace decides, not this answer.
                    Err(_unreadable) => break,
                }
                if began.elapsed() >= STOP_GRACE {
                    crate::stuck_children_reaped_when_the_kernel_lets_go::reap_soon(child);
                    return true;
                }
                std::thread::sleep(LOOK_INTERVAL);
            }
            crate::stuck_children_reaped_when_the_kernel_lets_go::reap_soon(child);
            return true;
        }
        return false;
    }

    /// What:
    ///  Ends the child and every process it started,
    /// and reaps the child.
    /// Why:
    ///  A ceiling that ends only the leader leaves what the leader started running:
    /// a helper that holds the
    ///  step's streams open,
    ///  or one that writes into the repository after the step was reported ended.
    /// The owner's rule is that a step ends with its whole process tree (`StepBound::EndAt`),
    ///  so the tree is read here,
    /// while the child is this process's own unreaped one and no reuse can have taken its number,
    ///  and every member is killed through a descriptor opened while its number still named it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const tree = descendantsOf(owned.id); for (const member of tree) member.kill('SIGKILL'); owned.endAndReap();
    /// ```
    pub fn end_with_the_processes_it_started(&mut self) -> bool {
        // The tree is read before the leader is signalled: a child's link to its parent is gone once the parent has exited.
        let walked: Option<Vec<TreeMember>> = i32::try_from(self.id()).ok().and_then(descendants_of_read);
        // The leader is signalled by `end_and_reap`, which owns the grace after that signal and answers whether it took.
        let _below_ended: Vec<Delivered> = kill_members(walked.as_deref().unwrap_or_default());
        return self.end_and_reap();
    }

    /// What:
    ///  Gives the standard-library child back to the caller,
    /// which then owns ending and reaping it.
    /// Why:
    ///  A child that must outlive this value (a push left running on purpose) is handed over,
    /// never ended by the drop.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const child = owned.intoStd();
    /// ```
    pub fn into_std(mut self) -> Child {
        return self.child.take().expect("the child is still owned: it was neither reaped nor handed over");
    }
}

impl OwnedChild {
    /// What:
    ///  Gives the standard-library child back when it has not been reaped,
    /// and nothing when it was.
    /// Why:
    ///  A stop hands a child that is still not collected to the task that waits for it,
    /// and has nothing to hand over once the
    ///  wait has collected it;
    /// `into_std` would refuse the second case.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const child = owned.intoStdWhenUnreaped(); // undefined once reaped
    /// ```
    pub fn into_std_when_unreaped(mut self) -> Option<Child> {
        return self.child.take();
    }
}

impl Drop for OwnedChild {
    /// A live child is ended and reaped,
    /// so the caller's cleanup never runs beside a child that is still writing.
    fn drop(&mut self) {
        self.end_and_reap();
    }
}

/// Helper controls stay out of the release executable.
#[cfg(test)]
#[path = "child_awaited_through_a_pidfd_tests.rs"]
mod tests;

/// Descriptor-limit controls stay out of the release executable.
#[cfg(test)]
#[path = "child_awaited_through_a_pidfd_descriptor_limit_tests.rs"]
mod descriptor_limit_tests;
