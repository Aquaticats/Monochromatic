//! What:
//!  The account companion of the ZFS add route:
//!  the small user-namespace check the add's preflight runs before its first
//!  write,
//!  inside the wrapper's own executable,
//!  with its deadline,
//! its printed skip and its printed refusals.
//! Why:
//!  The project chose "Root plus account companion" for Q6:
//!  the account runs the companion before each managed add,
//!  the root probe stays the only evidence about the dataset mount path,
//!  and the companion never touches ZFS,
//! the key path or `/dev/zfs`
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//! Q6 and Q-a).
//!  A host that refuses it a user namespace,
//!  the capabilities inside one,
//!  or a mount the setup makes prints one skip line and the add goes ahead;
//!  every other failure refuses that add's managed attach and leaves the slot
//!  to the reflink or ordinary Git route.
//!  It runs in the wrapper's one executable rather than as a second program,
//!  because the project's one-program rule bars another executable,
//! and a
//!  re-exec of the wrapper is barred by its no-self-invocation rule.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `companion_calls.rs`:
//!   the calls a record can name and one failed call's value.
//! - `companion_steps.rs`:
//!   the steps in the child's order and the skip's boundary.
//! - `companion_buffers.rs`:
//!   every name and line the child writes,
//!   made before the fork.
//! - `companion_records.rs`:
//!   one step's eight bytes and their decode.
//! - `companion_settings.rs`:
//!   the three settings the skip line prints.
//! - `companion_file_calls.rs`:
//!   the file and identity calls.
//! - `companion_mount_calls.rs`:
//!   the namespace,
//!   tmpfs,
//!   bind and unmount calls.
//! - `companion_descriptor_calls.rs`:
//!   the walk and the descriptor attach.
//! - `companion_mount_table.rs`:
//!   the mount ID and the table's facts.
//! - `companion_process_calls.rs`:
//!   the pipe,
//!   the fork,
//!   the pid descriptor,
//!   the kill and the ending a wait answers.
//! - `companion_child_tree.rs`:
//!   the scratch tree and the walk to the slot.
//! - `companion_child_controls.rs`:
//!   the four refusal controls.
//! - `companion_child_attach.rs`:
//!   the attach,
//!   its read-back and the second child that moves and unmounts.
//! - `companion_child.rs`:
//!   the child's sequence and its records.
//! - `companion_scratch.rs`:
//!   the run's directory under `$XDG_RUNTIME_DIR` and its sweep.
//! - `companion_deadline.rs`:
//!   the bound the parent enforces,
//!   beside its measurement.
//! - `companion_test_switch.rs`:
//!   the test-only switch and its fixtures.
//! - `companion_lines.rs`:
//!   the verdict and the lines a person reads.
//! - `companion_parent.rs`:
//!   one run,
//!   its bounded wait and its verdict.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!beforeTheAdd().theManagedAttachGoesAhead()) return null; // the slot takes the reflink or Git route
//! ```

/// The calls a record can name.
pub mod companion_calls;
/// The steps in the child's order.
pub mod companion_steps;
/// Every name and line the child writes.
pub mod companion_buffers;
/// One step's record.
pub mod companion_records;
/// The three settings the skip line prints.
pub mod companion_settings;
/// The file and identity calls.
pub mod companion_file_calls;
/// The namespace,
/// tmpfs,
/// bind and unmount calls.
pub mod companion_mount_calls;
/// The walk,
/// the attach and the two moves.
pub mod companion_descriptor_calls;
/// The mount ID and the mount table's facts.
pub mod companion_mount_table;
/// The pipe,
/// the fork,
/// the wait,
/// the kill and the ending a wait answers.
pub mod companion_process_calls;
/// The scratch tree and the walk to the slot.
pub mod companion_child_tree;
/// The four refusal controls.
pub mod companion_child_controls;
/// The attach and the second child.
pub mod companion_child_attach;
/// The child's sequence.
pub mod companion_child;
/// The run's directory and its sweep.
pub mod companion_scratch;
/// The bound the parent enforces.
pub mod companion_deadline;
/// The test-only switch.
pub mod companion_test_switch;
/// The verdict and its lines.
pub mod companion_lines;
/// One run and its verdict.
pub mod companion_parent;

/// The check the add's preflight runs before its first write,
/// printing every line it carries.
pub use companion_parent::the_managed_attach_may_go_ahead;
/// What one run means for the add.
pub use companion_lines::{CompanionVerdict, Refusal, Skip, THE_FALLBACK_ROUTE};
/// The bound the parent enforces.
pub use companion_deadline::{COMPANION_DEADLINE, COMPANION_DEADLINE_MARGIN};
/// The test-only switch the controls set.
pub use companion_test_switch::{Switch, TEST_SWITCH_VARIABLE};
