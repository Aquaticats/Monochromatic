//! What:
//!  Controls for the act's own reads and its refusal of a namespace it cannot
//!  enter:
//!  the kernel's tags become the closure's rows,
//!  the topmost mount at a path is the last row printed for it,
//!  a mount id names one row,
//!  and an act that cannot enter a namespace is refused with a word and
//!  changes nothing.
//! Why:
//!  Every other call this module makes is a mount change and is shown in the
//!  guest arm,
//!  and these are the reads the act refuses on,
//!  so they are shown here without a mount.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('an act that cannot enter is refused', () => { expect(takeTheCopyOut(...).word).toBe('holder-unendable'); });
//! ```

use super::{
    DESTINATION_MODE,
    NamespaceCopy,
    Ready,
    row_of,
    rows_of,
    take_the_copy_out,
    topmost_at,
};
use crate::worktree_clone_forced_removal::forced_removal_words::RefusalWord;
use crate::worktree_clone_forced_removal::holder_preflight::Route;
use crate::worktree_clone_forced_removal::recorded_handle::RecordedHandle;
use std::path::{
    Path,
    PathBuf,
};

/// A mount table holding the shape the design's tags are read from:
/// the checkout's own mount shared with a group of its own,
/// the clone's copy below it,
/// a slave of the checkout,
/// and a second mount stacked over the copy.
const A_TABLE: &str = "\
31 1 0:30 / /home/user/work rw,relatime shared:12 - zfs tank/checkout rw\n\
32 31 0:31 / /home/user/work/one rw,relatime - zfs tank/clones/one rw\n\
33 31 0:30 / /home/user/work/report rw,relatime master:12 - zfs tank/checkout rw\n\
34 32 0:99 / /home/user/work/one rw,relatime - tmpfs a-scratch rw\n";

#[test]
fn the_kernel_s_tags_become_the_closure_s_rows() {
    let rows = rows_of(A_TABLE);
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].mount_id, 31);
    assert_eq!(rows[0].parent_id, 1);
    assert_eq!(rows[0].shared, vec![12]);
    assert_eq!(rows[0].master, None);
    assert_eq!(rows[0].filesystem, "zfs");
    assert_eq!(rows[0].source, "tank/checkout");
    assert_eq!(rows[0].mount_point, "/home/user/work");
    assert_eq!(rows[2].master, Some(12));
    assert!(rows[2].shared.is_empty());
    assert_eq!(rows[2].root, "/");
}

#[test]
fn the_topmost_mount_at_a_path_is_the_last_row_printed_for_it() {
    let rows = rows_of(A_TABLE);
    let topmost =
        topmost_at(&rows, Path::new("/home/user/work/one")).expect("two mounts sit at that path");
    assert_eq!(
        topmost.mount_id, 34,
        "the tmpfs stacked over the copy is what an unmount there meets"
    );
    assert_eq!(topmost.filesystem, "tmpfs");
    assert!(
        topmost_at(&rows, Path::new("/home/user/work/nowhere")).is_none(),
        "a path no mount sits at answers nothing"
    );
    let below = row_of(&rows, 32).expect("the copy's own row is found by its mount id");
    assert_eq!(below.parent_id, 31);
}

#[test]
fn the_way_s_directory_is_its_owner_s_alone() {
    assert_eq!(DESTINATION_MODE, 0o700);
}

#[test]
fn an_act_that_cannot_enter_a_namespace_is_refused_with_its_word_and_changes_nothing() {
    let namespace: std::fs::File =
        std::fs::File::open("/proc/self/ns/mnt").expect("this process's own namespace is opened");
    let handle: RecordedHandle = RecordedHandle {
        handle_type: 1,
        handle: vec![1, 2, 3],
    };
    let checkout: PathBuf = PathBuf::from("/home/user/work");
    let slot: PathBuf = checkout.join("one");
    let copy: NamespaceCopy<'_> = NamespaceCopy {
        namespace: &namespace,
        identity: "mnt:[1]",
        checkout: checkout.as_path(),
        slot: slot.as_path(),
        dataset: "tank/clones/one",
        handle: &handle,
        directory: ".git-worktree-clone-removal-1",
    };
    let ready: Ready = Ready {
        route: Route::InPlaceByTheHandle,
        mount_id: 32,
        parent_id: 31,
        directory: None,
    };
    let refused = take_the_copy_out(&ready, &copy).expect_err(
        "this account holds no capability to enter a mount namespace, and the act refuses",
    );
    assert_eq!(refused.word, RefusalWord::HolderUnendable);
    assert_eq!(
        refused.errno,
        Some(libc::EPERM),
        "the kernel's own refusal reaches the act, and it read {refused}"
    );
    assert!(
        refused.said.contains("setns"),
        "the refusal names the call, and it read {}",
        refused.said
    );
}
