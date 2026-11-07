//! What:
//!  Ends a child that `spawn_owned` started together with every process of its group,
//! and waits until the group is empty.
//! Why:
//!  `Child::kill` signals only the leader,
//! so a background subshell it started outlives it and can write into the fixture while the
//!  test removes it;
//! this file holds the one function that ends the whole group,
//! apart from `test_support.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.kill(-child.pid, 'SIGKILL'); await child.exited; while (groupHasProcesses(child.pid)) await sleep(5);
//! ```

/// How long `end_with_its_group` waits for the processes of an ended group to be gone before it stops looking.
const GROUP_GONE_PATIENCE: std::time::Duration = std::time::Duration::from_secs(5);

/// What:
///  Kill a child that `spawn_owned` started together with every process of its group,
/// reap the child,
/// and look until the group has
///  no process left.
/// Why:
///   `Child::kill` signals only the leader,
/// so a background subshell it started outlives it and can write into the fixture while the
///  test removes it (`DirectoryNotEmpty`);
/// `spawn_owned` ends the group only when the test thread's fixtures drop,
/// after the removal.
///  A killed process ends asynchronously,
/// so the group is looked at until `kill(-group, 0)` says it is empty.
pub(crate) fn end_with_its_group(mut child: std::process::Child) {
    let group: libc::pid_t = libc::pid_t::try_from(child.id()).expect("a pid fits the pid type");
    // SAFETY: `killpg` takes two integers and reads no memory; the group is the one `spawn_owned` made for this child.
    let _signalled: libc::c_int = unsafe { libc::killpg(group, libc::SIGKILL) };
    child.wait().expect("reap the child");
    let began: std::time::Instant = std::time::Instant::now();
    // SAFETY: as above; signal 0 only asks whether a process of the group exists.
    while unsafe { libc::kill(-group, 0) } == 0 && began.elapsed() < GROUP_GONE_PATIENCE {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
