//! Controls for the standard error descriptor:
//! it writes,
//! and it does not wait for the lock the standard library's shared handle takes.

use super::*;

/// What:
///  A write to the descriptor finishes while another thread holds standard error's shared lock.
/// Why:
///  That lock is the one a commit's own synchronous write to standard error takes;
/// a warning worker abandoned at its
///  ceiling must not hold it,
/// or the commit waits for it with no end.
///  The bytes go to this process's real standard error,
/// so this control also proves the descriptor answers.
#[cfg(unix)]
#[test]
fn a_write_to_the_descriptor_does_not_wait_for_the_shared_lock() {
    let held = std::io::stderr().lock();
    let delivered: Option<std::io::Result<()>> = crate::work_awaited_within_a_ceiling::write_within(
        std::time::Duration::from_secs(5),
        StandardErrorDescriptor,
        String::from("a descriptor write holds no lock another writer needs\n"),
    );
    drop(held);
    assert!(
        matches!(delivered, Some(Ok(()))),
        "the write needed the lock a synchronous writer holds, so an abandoned worker would keep it: {delivered:?}"
    );
}
