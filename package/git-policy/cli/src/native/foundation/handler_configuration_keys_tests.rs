//! What:
//!  Controls of the configuration keys the wrapper and the root modes name.
//! Why:
//!  A key two readers share is one fact,
//!  and a control that the two spellings are these constants is what keeps a
//!  later edit from splitting them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(CLONE_MOUNTS_KEY).toBe('cloneMounts');
//! ```

use super::{
    CHECKOUT_DATASET_KEY,
    CHECKOUT_KEY,
    CLONE_MOUNTS_KEY,
    PARENT_KEY,
    PROBE_CHECKOUT_KEY,
    PROBE_DATASET_KEY,
    START_PROBE_KEY,
};

/// Every key is the spelling the design and the setup's file use.
#[test]
fn every_key_is_the_spelling_the_file_holds() {
    assert_eq!(CLONE_MOUNTS_KEY, "cloneMounts");
    assert_eq!(PARENT_KEY, "parent");
    assert_eq!(CHECKOUT_KEY, "checkout");
    assert_eq!(CHECKOUT_DATASET_KEY, "checkoutDataset");
    assert_eq!(START_PROBE_KEY, "startProbe");
    assert_eq!(PROBE_DATASET_KEY, "dataset");
    assert_eq!(PROBE_CHECKOUT_KEY, "checkout");
}

/// The wrapper's and the root modes' own constants are these,
/// so neither can spell a key differently.
#[test]
fn the_wrapper_and_the_root_modes_name_these_constants() {
    assert_eq!(
        crate::start_probe::probe_configuration::START_PROBE_KEY,
        START_PROBE_KEY
    );
    assert_eq!(
        crate::start_probe::probe_configuration::DATASET_KEY,
        PROBE_DATASET_KEY
    );
    assert_eq!(
        crate::start_probe::probe_configuration::CHECKOUT_KEY,
        PROBE_CHECKOUT_KEY
    );
    assert_eq!(
        crate::start_probe::probe_configuration::CLONE_MOUNTS_KEY,
        CLONE_MOUNTS_KEY
    );
    assert_eq!(
        crate::start_probe::probe_configuration::PARENT_KEY,
        PARENT_KEY
    );
    assert_eq!(
        crate::worktree_clone_mount_handler::sweep_configuration::CLONE_MOUNTS_KEY,
        CLONE_MOUNTS_KEY
    );
    assert_eq!(
        crate::worktree_clone_mount_handler::sweep_configuration::PARENT_KEY,
        PARENT_KEY
    );
    assert_eq!(
        crate::worktree_clone_mount_handler::sweep_configuration::CHECKOUT_KEY,
        CHECKOUT_KEY
    );
    assert_eq!(
        crate::worktree_clone_mount_handler::sweep_configuration::CHECKOUT_DATASET_KEY,
        CHECKOUT_DATASET_KEY
    );
    assert_eq!(
        crate::worktree_clone_mount_handler::sweep_configuration::PROBE_OBJECT_KEY,
        START_PROBE_KEY
    );
    assert_eq!(
        crate::worktree_clone_mount_handler::sweep_configuration::PROBE_CHECKOUT_KEY,
        PROBE_CHECKOUT_KEY
    );
}
