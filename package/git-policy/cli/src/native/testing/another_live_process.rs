//! What:
//!  A process that is alive while a control holds it,
//! and is no process of the test.
//! Why:
//!   A note names the process that owns an instance,
//!  and the rules about a live owner and a dead owner's leftovers are told apart only by a second process being alive.
//! It lives beside `test_support.rs` rather than in it,
//! because that file is at its line bound.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const other = aLiveProcessThatIsNotThisOne(); // ends when `other` is dropped
//! ```

/// What:
///  A process of this control's own that is alive while the control holds it,
///  and that is no process of this test.
/// Why:
///   A note names the process that owns an instance,
/// and the rules about a live owner and a dead owner's leftovers are told apart only by a second process being alive.
/// The child waits on a pipe this control holds,
/// so it costs nothing while it waits,
/// and it ends when the control drops it,
/// even when an assertion panics first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const other = aLiveProcessThatIsNotThisOne(); // ends when `other` is dropped
/// ```
pub(crate) struct AnotherLiveProcess {
    /// The child itself,
    /// whose standard input is the pipe that keeps it waiting.
    child: std::process::Child,
}

impl AnotherLiveProcess {
    /// The process number the note is written with.
    pub(crate) fn pid(&self) -> i32 {
        return i32::try_from(self.child.id()).expect("a process number fits");
    }
}

impl Drop for AnotherLiveProcess {
    fn drop(&mut self) {
        // Closing the pipe ends the child's read, so it exits by itself; a child that is slow to go is killed.
        let _killed = self.child.kill();
        let _reaped = self.child.wait();
    }
}

/// Starts a process that is alive until the value it is given back in is dropped.
pub(crate) fn a_live_process_that_is_not_this_one() -> AnotherLiveProcess {
    let mut command: std::process::Command = std::process::Command::new("/bin/sh");
    command.arg("-c").arg("read -r line").stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    let child: std::process::Child = command.spawn().expect("a stand-in process that waits on its input");
    return AnotherLiveProcess { child };
}
