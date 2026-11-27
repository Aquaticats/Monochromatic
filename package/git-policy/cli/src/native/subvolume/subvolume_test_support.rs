//! What:
//!  The stand-in platform and the fixture checkouts of the subvolume migration controls.
//! Why:
//!  The migration's journal,
//! resume,
//! rollback,
//! refusal and index logic does not need btrfs,
//! only the five operations
//!  of `Platform` do.
//! The stand-in answers them on a plain directory tree:
//! a "subvolume" is a directory registered by
//!  its device and inode (an identity that survives the exchange and a rename,
//! as a real subvolume root's does),
//! file
//!  clones are refused so the byte-copy path runs,
//! and the exchange is the real `renameat2`.
//! These controls therefore
//!  run on the container gate's tmpfs fixture root too.
//! The controls that need the real ioctls use `btrfs_fixture`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const platform = new PlainPlatform({ removalAllowed: true });
//! const root = makeCheckout(parent, 'checkout');
//! ```
//!
//! The code of this module is split by role,
//! and this file re-exports every name those files hold,
//! so that a control writes `subvolume_test_support::PlainPlatform` as it did before:
//! `subvolume_test_support_plain_platform` is the stand-in platform,
//! `subvolume_test_support_hooked_platform` the one that can fail a clone or a hard link,
//! `subvolume_test_support_flushing_platform` the one that says it flushes,
//! and `subvolume_test_support_fixtures` the fixture checkouts and run helpers.

pub(crate) use crate::subvolume_test_support_fixtures::{
    backdate,
    checkout_moved_behind_a_link,
    checkout_with_worktree,
    index_inode,
    make_checkout,
    migrate_arguments,
    run_command,
};
pub(crate) use crate::subvolume_test_support_flushing_platform::Flushing;
pub(crate) use crate::subvolume_test_support_hooked_platform::Hooked;
pub(crate) use crate::subvolume_test_support_plain_platform::PlainPlatform;
