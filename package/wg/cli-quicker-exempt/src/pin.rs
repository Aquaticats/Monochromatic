//! Owns bpffs validation,
//!  collision-free cgroup pin paths,
//!  and atomic link replacement.
//! Each part is a file of its own:
//! the bpffs check and the roots this tool creates in `pin_root.rs`,
//! a cgroup's identity and the bpffs path its pins live at in `pin_key.rs`,
//! one complete set of new pins replacing the old one in `pin_replacement.rs`,
//! and one lifecycle command with the lock it holds in `pin_lifecycle.rs`.
//! Everything a caller names is re-exported here,
//! so a caller keeps naming `crate::pin`.

/// A cgroup's identity,
/// and the bpffs path its pins live at.
#[path = "pin_key.rs"]
pub mod key;
/// One lifecycle command:
/// the lock held for it,
/// and the attach or detach it runs.
#[path = "pin_lifecycle.rs"]
pub mod lifecycle;
/// One complete set of new pins replacing the old one,
/// and the removal a detach asks for.
#[path = "pin_replacement.rs"]
pub mod replacement;
/// The bpffs check and the roots this tool creates there.
#[path = "pin_root.rs"]
pub mod root;

/// The bpffs path of a cgroup's pins,
/// which the tests name.
#[cfg(test)]
pub use key::pin_dir_for;
/// A cgroup's identity,
/// and the bpffs path its pins live at.
pub use key::{
    cgroup_identity_path,
    cgroup_key,
};
/// The lock of one lifecycle command,
/// and the attach or detach it runs.
pub use lifecycle::{
    acquire_lifecycle_lock,
    attach_cgroup,
    detach_cgroup,
};
/// The removal and the commit of a pin set,
/// exposed to the sibling tests without widening the production path.
#[cfg(test)]
pub use replacement::{
    commit_staging_for_test,
    remove_exact_pin_dir_for_test,
};
/// The roots this tool creates in bpffs,
/// which the tests name.
#[cfg(test)]
pub use root::{
    BPF_FS,
    PIN_ROOT,
};
