//! What:
//!  The two user properties the clone-mount route reads and writes,
//!  the version word their values begin with,
//!  the two operation words,
//!  the nonce's length and
//!  the byte bound an intent is measured against before it is parsed.
//! Why:
//!  The design fixes the durable intent as the clone's local
//!  `org.monochromatic` property named below,
//!  accepts exactly `v1:<nonce>:mount` or `v1:<nonce>:unmount`,
//!  and bounds the byte length before parsing
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering";
//!  the build order is `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 7).
//!  The nudge is the second property,
//!  and it is not an instruction:
//!  it carries the same 32-hex-digit nonce and nothing else.
//!  Every name and word lives here so a typo is one edit rather than a
//!  protocol that silently reads nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const DESIRED_STATE = 'org.monochromatic:git-worktree-clone-mount-desired-state';
//! const MAXIMUM_INTENT_BYTES = 43;
//! ```

/// The clone's local property carrying its durable mount intent.
pub const DESIRED_STATE_PROPERTY: &str = "org.monochromatic:git-worktree-clone-mount-desired-state";

/// The clone's local property a waiter changes to nudge a sweep.
/// It is not a mount instruction.
pub const SWEEP_NONCE_PROPERTY: &str = "org.monochromatic:git-worktree-clone-mount-sweep-nonce";

/// The version word every intent value begins with;
/// a nudge carries the nonce alone and no version word.
pub const VALUE_VERSION: &str = "v1";

/// The word in an intent value that asks for an attach.
pub const MOUNT_WORD: &str = "mount";

/// The word in an intent value that asks for an unmount.
pub const UNMOUNT_WORD: &str = "unmount";

/// The nonce's digits:
/// 32 lowercase ASCII hexadecimal characters,
/// which is 16 bytes.
pub const NONCE_HEX_DIGITS: usize = 32;

/// The separator between an intent's three fields.
pub const FIELD_SEPARATOR: char = ':';

/// The longest intent value the grammar admits,
/// `v1:<32 hex digits>:unmount`,
/// in bytes.
/// A longer value is refused before any field is split.
pub const MAXIMUM_INTENT_BYTES: usize = VALUE_VERSION.len() + 1 + NONCE_HEX_DIGITS + 1 + UNMOUNT_WORD.len();

/// The property the reply's `source` key carries,
/// whose value is the name of the dataset the property is set on.
pub const PROPERTY_SOURCE_KEY: &str = "source";

/// The property the reply's `value` key carries.
pub const PROPERTY_VALUE_KEY: &str = "value";

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_mount_property_names_tests.rs"]
mod tests;
