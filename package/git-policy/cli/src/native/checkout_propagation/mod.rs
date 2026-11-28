//! What:
//!  The ZFS route's private-propagation step:
//!  the check that the checkout's own mount passes no mounts on,
//!  the set that makes it private through the descriptor the step verified,
//!  the read that proves the set,
//!  the report of the shared mounts that stand beneath the checkout,
//!  and the root-owned record the answer is filed in.
//! Why:
//!  A mount that replaces the checkout's mount comes up shared again,
//!  and a clone mounted under a shared checkout is delivered to every other
//!  mount namespace,
//!  so the route repairs the setting before it attaches anything
//!  (`doc/decision/cli-git-zfs-checkout-propagation-repair.md`,
//!  which accepts the design whole with its eight choices).
//!  The step is plan step 19's piece,
//!  built ahead of the plan's order and unwired:
//!  its call at the head of `boot-reconcile` and of the default sweep is that
//!  step's wiring,
//!  so no command and no root mode reaches it in this release,
//!  and the guest arm beside these files calls it itself
//!  (`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`).
//!  It is Linux only,
//!  like `statmount(2)`, `mount_setattr(2)` and `/proc/self/mountinfo` it
//!  reads.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `mount_propagation_set.rs`:
//!   one mount's propagation as three flags,
//!   the five words the record carries for the sets the kernel can produce,
//!   and the reading of a table row's own tags.
//! - `propagation_words.rs`:
//!   the record's places,
//!   its version,
//!   kind and keys,
//!   the bound of every field and the derived file bound,
//!   the four outcome words and the words the `step` field carries.
//! - `mount_descriptor_read.rs`:
//!   one mount's facts read from a descriptor by `statx(2)` and
//!   `statmount(2)`,
//!   with both mount numbers,
//!   the set,
//!   the peer group,
//!   the master and the three names.
//! - `propagation_calls.rs`:
//!   the kernel's acts behind one seam a control answers,
//!   the set being a `mount_setattr(2)` on the descriptor and never a path.
//! - `checkout_mount_checks.rs`:
//!   the five checks that prove which instance was read,
//!   and the table row that must describe the same mount.
//! - `shared_mounts_beneath.rs`:
//!   the exact count and the first four names of the shared mounts beneath
//!   the checkout,
//!   each in the kernel's own spelling.
//! - `propagation_record.rs`:
//!   the record's fields,
//!   its one text,
//!   and the parser that reads that text back.
//! - `propagation_record_file.rs`:
//!   the root-owned `0755` directory under `/run`,
//!   the `0644` record,
//!   the staging sibling and the one rename,
//!   and the read that proves what it reads.
//! - `propagation_step.rs`:
//!   the step itself,
//!   its order,
//!   the answer it files and the line it says.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = runTheStep(places, resolution, theKernelsCalls); // private, or left, or refused
//! ```

pub mod checkout_mount_checks;
pub mod mount_descriptor_read;
pub mod mount_propagation_set;
pub mod propagation_calls;
pub mod propagation_record;
pub mod propagation_record_file;
pub mod propagation_step;
pub mod propagation_words;
pub mod shared_mounts_beneath;

pub use checkout_mount_checks::{
    MountRefusal,
    check_the_facts,
    check_the_row,
};
pub use mount_descriptor_read::{
    DescriptorMount,
    MountReadFailure,
    read_the_mount,
};
pub use mount_propagation_set::{
    LONGEST_WORD,
    MountPropagation,
    PRIVATE_WORD,
    SHARED_AND_SLAVE_WORD,
    SHARED_WORD,
    SLAVE_WORD,
    UNBINDABLE_WORD,
};
pub use propagation_calls::{
    CallFailure,
    PropagationCalls,
    TheKernelsCalls,
};
pub use propagation_record::{
    PropagationParseRefusal,
    PropagationRecord,
    parse_record,
};
pub use propagation_record_file::{
    PropagationFile,
    PropagationFileRefusal,
};
pub use propagation_step::{
    CheckoutResolution,
    PropagationDone,
    PropagationPlaces,
    PropagationRefusal,
    run_the_step,
};
pub use propagation_words::{
    BENEATH_PATH_KEY,
    BENEATH_UNREAD,
    MAXIMUM_BENEATH_PATH_BYTES,
    MAXIMUM_BOOT_BYTES,
    MAXIMUM_CHECKOUT_PATH_BYTES,
    MAXIMUM_DATASET_BYTES,
    MAXIMUM_ERRNO_BYTES,
    MAXIMUM_NUMBER_BYTES,
    MAXIMUM_OUTCOME_BYTES,
    MAXIMUM_PROPAGATION_BYTES,
    MAXIMUM_SET_WORD_BYTES,
    MAXIMUM_STEP_BYTES,
    MAXIMUM_TIME_BYTES,
    NONE_WORD,
    PROPAGATION_DIRECTORY,
    PROPAGATION_DIRECTORY_MODE,
    PROPAGATION_KIND,
    PROPAGATION_MODE,
    PROPAGATION_NAME,
    PROPAGATION_VERSION,
    PropagationOutcome,
    PropagationStep,
    SHARED_MOUNTS_NAMED,
};
pub use shared_mounts_beneath::{
    SharedMountsBeneath,
    shared_mounts_beneath,
};

/// The step's arm in a throwaway guest:
/// a real mount made shared,
/// shared and slave,
/// private,
/// slave and unbindable,
/// and a mount that is not the checkout's,
/// each driven through the step as root,
/// and then the accepted design's trigger measurement,
/// whose two rules decide whether the systemd trigger stays.
/// Never part of the release executable,
/// and it prints one `SKIPPED` line anywhere but the step's own guest.
#[cfg(all(test, target_os = "linux"))]
#[path = "propagation_guest_arms_tests.rs"]
mod guest_tests;

/// The fixture and the small helpers the step's guest arm shares across its
/// phases:
/// the guest's pool,
/// its checkout dataset,
/// the table rows a phase reads,
/// and the lines a phase says.
/// Never part of the release executable.
#[cfg(all(test, target_os = "linux"))]
#[path = "propagation_guest_fixture_tests.rs"]
mod guest_fixture;
