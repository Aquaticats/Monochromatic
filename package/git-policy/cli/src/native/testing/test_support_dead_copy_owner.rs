//! What:
//!  The owner of a process that really started,
//! whose birth identity was read while it ran,
//! and which was then killed and
//!  reaped.
//! Why:
//!   An interrupted copy is a journal or a claim whose owner is gone;
//! a control builds one with this owner instead of waiting
//!  for a real crash.
//! It sits beside `test_support.rs` rather than in it,
//! because that file's helpers pass the line limit and this one
//!  would put it over.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const dead = await deadCopyOwner(); // { pid, birthIdentity, pidNamespace } of a killed child
//! ```

use crate::test_support::spawn_owned;
use std::process::Command;

/// What:
///  The owner of a process that really started,
/// had its birth identity read while it ran,
/// and was then killed and reaped.
/// Why:
///   An interrupted copy is a journal or a claim whose owner is gone;
/// a control builds one with this owner instead of
///  waiting for a real crash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dead = await deadCopyOwner(); // { pid, birthIdentity, pidNamespace } of a killed child
/// ```
pub(crate) fn dead_copy_owner() -> crate::worktree_copy_owner::CopyOwner {
    use std::process::Stdio;
    // Owned, so an assertion between the start and the kill does not leave the sleeper for a minute.
    let mut child: std::process::Child = spawn_owned(
        Command::new("sleep")
            .arg("60")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null()),
    )
    .expect("sleep starts");
    let pid: i64 = i64::from(child.id());
    let birth: String = crate::process_identity::process_birth_identity(pid)
        .expect("identity readable")
        .expect("the child runs");
    child.kill().expect("child is killed");
    child.wait().expect("child is reaped");
    return crate::worktree_copy_owner::CopyOwner {
        pid,
        birth_identity: birth,
        pid_namespace: crate::pid_namespace::current_pid_namespace(),
    };
}
