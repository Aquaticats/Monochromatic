//! What:
//!  What one snapshot-destroy request did to one submitted name:
//!  `destroyed`,
//! `refused` with the kernel's error number,
//! `intact`,
//! or `unknown`.
//! Why:
//!  The kernel destroys a whole request or none of it:
//! its Lua program checks every
//!  name with `zfs.check.destroy` before any `zfs.sync.destroy`,
//! and
//!  `dsl_destroy_snapshots_nvl` states that on failure no snapshot is
//!  destroyed and the error list is filled
//! (`module/zfs/dsl_destroy.c:596` to `:600`,
//!  OpenZFS `zfs-2.4.4`).
//!  So a refused name is `refused`,
//! a submitted name the list does not name is
//!  `intact`,
//! and a failure that cannot be attributed to names is `unknown`.
//!  The vocabulary is the one the destruction-plan design settled for a tuple
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The states");
//! the caller keeps it with the removal record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type NameOutcome = { kind: 'destroyed' | 'refused' | 'intact' | 'unknown'; errno?: number; said?: string };
//! ```

/// What one submitted name became.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NameOutcome {
    /// The request returned success,
    /// so this name is destroyed or was already absent.
    Destroyed,
    /// The request named this snapshot as one it refused.
    Refused {
        /// The kernel's error number for this name.
        errno: i32,
    },
    /// The request failed and did not name this snapshot,
    /// so the kernel's
    ///  own no-destruction-on-failure rule leaves it as it was.
    Intact,
    /// The request's effect cannot be attributed to this name.
    Unknown {
        /// Why it cannot be attributed.
        said: String,
    },
}

/// One submitted name with what the request did to it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubmittedName {
    /// The snapshot name exactly as the request submitted it.
    pub name: String,
    /// What the request did to it.
    pub outcome: NameOutcome,
}
