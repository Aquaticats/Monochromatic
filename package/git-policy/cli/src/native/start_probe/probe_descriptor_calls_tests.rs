//! What:
//!  Controls of the descriptor calls on a machine that attaches nothing:
//!  a context for a filesystem type no kernel has is refused,
//!  and a move of a mount between two paths that are not mounts is refused
//!  with the call's own errno.
//! Why:
//!  The attach's calls are covered by the guest arms,
//!  and what a host can hold without authority is that a refused call answers
//!  its errno rather than a made-up success.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(fsopen('no-such-filesystem').errno).toBe(ENODEV);
//! ```

use super::{fsopen, move_mount_between_paths};
use crate::testing::test_support::fixture;
use std::path::PathBuf;

/// A context for a filesystem type the kernel does not have is refused,
/// and its call is `fsopen`.
#[test]
fn a_context_for_an_unknown_filesystem_is_refused() {
    let refusal = fsopen("no-such-filesystem-type").expect_err("the context is refused");
    assert_eq!(refusal.call, "fsopen");
}

/// A move between two plain paths is refused,
/// and its call is `move_mount`.
#[test]
fn a_move_between_plain_paths_is_refused() {
    let root: PathBuf = fixture("start-probe-descriptor-move");
    std::fs::create_dir(root.join("from")).expect("the fixture folder is made");
    std::fs::create_dir(root.join("to")).expect("the fixture folder is made");
    let refusal = move_mount_between_paths(root.join("from").as_path(), root.join("to").as_path()).expect_err("the move is refused");
    assert_eq!(refusal.call, "move_mount");
}
