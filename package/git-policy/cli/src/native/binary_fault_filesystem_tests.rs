//! What:
//!  Slow flushes and filesystems that refuse something the wrapper uses: hard links, symbolic links, a rename that
//!  crosses devices.
//! Why:
//!  The README promises that slow fsync and unusual filesystems do not strand the repository.
//!  A slow flush is a delay on the return of every `fsync` of the wrapper and of Git (`strace --inject=...:delay_exit`),
//!  so the commit must still land, with the same result, inside its bound.
//!  A refusal is the error such a filesystem returns from the call (`EPERM` for a link on a filesystem without
//!  links, `EXDEV` for a rename across mounts), kept up from the `n`th call on as the filesystem keeps refusing; the
//!  commit may fail, but it must say so, and the repository must recover (`binary_fault_support.rs`).
//!  Real filesystems with these properties need privilege to mount; the system calls are where their behavior shows.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await injectAndExpectRecovery({ call: 'linkat', error: 'EPERM', from: n });
//! ```

use super::commit::remove_settled;
use super::fault::{Effect, Injection, Ran, When, run_injected, staged_repository, sweep_persistent, verify_recovery};
use super::fork_failure::strace_available;
use super::support::{Fixture, fixture};
use std::path::PathBuf;

/// How long every flush takes to return, in microseconds: more than a healthy disk, far below a failing one.
const SLOW_FLUSH_MICROSECONDS: u64 = 30_000;

/// Every `fsync` of the wrapper and of Git returns late, and the commit lands with the same result.
#[test]
fn a_slow_flush_does_not_change_what_a_commit_does() {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no flush was slowed");
        return;
    }
    let fixture: Fixture = fixture("slow-fsync");
    let repo: PathBuf = staged_repository(&fixture, "repo");
    let fault: Injection = Injection {
        syscall: "fsync",
        effect: Effect::SlowReturn(SLOW_FLUSH_MICROSECONDS),
        when: When::FromNth(1),
        follow_forks: true,
    };
    let ran: Ran = run_injected(&fixture, repo.as_path(), "slow-fsync", &fault);
    assert!(ran.injected(), "no flush was delayed, so nothing was tested: {}", ran.text());
    assert_eq!(ran.code(), 0, "a slow disk is not a failed one: {}", ran.text());
    assert_eq!(verify_recovery(&fixture, repo.as_path(), "slow-fsync", ran.text().as_str()), Ok(()));
    remove_settled(&fixture);
}

/// A filesystem without hard links refuses the link that installs the index: the commit says so, and the next
///  command recovers.
#[test]
fn a_filesystem_that_refuses_hard_links_never_strands_a_commit() {
    sweep_persistent("fs-linkat", "linkat", Effect::Fail("EPERM"), false, 1);
}

/// A filesystem without symbolic links refuses the links of the shadow repository: the commit says so, and nothing stays.
#[test]
fn a_filesystem_that_refuses_symbolic_links_never_strands_a_commit() {
    sweep_persistent("fs-symlink", "symlink", Effect::Fail("EPERM"), false, 1);
}

/// A rename across devices, as when the registry and the repository are on different mounts, is refused: the commit
///  says so, and nothing stays.
#[test]
fn a_rename_across_devices_never_strands_a_commit() {
    sweep_persistent("fs-rename", "rename", Effect::Fail("EXDEV"), false, 1);
}

/// A read-only filesystem refuses every directory creation from the `n`th on.
#[test]
fn a_filesystem_that_turns_read_only_never_strands_a_commit() {
    sweep_persistent("fs-mkdir", "mkdir", Effect::Fail("EROFS"), false, 1);
}
