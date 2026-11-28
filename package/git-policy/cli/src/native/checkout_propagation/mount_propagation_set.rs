//! What:
//!  One mount's propagation as the kernel states it:
//!  the set of flags itself,
//!  the word the record carries for the set,
//!  the reading of a mount table row's own tags,
//!  and the flags a set of each word is written as.
//! Why:
//!  The step acts on a set and never on one flag:
//!  a mount can be shared and slave at once,
//!  and an unbindable mount stands alone,
//!  so a check that asked one word whether it were shared would misread a set
//!  the kernel can produce
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "The call that sets it",
//!  and its F30 read from `fs/namespace.c`'s `mnt_to_propagation_flags`).
//!  The five words are the record's,
//!  so a person,
//!  `doctor` and a guest arm read the same combinations.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type MountPropagation = { shared: boolean; slave: boolean; unbindable: boolean };
//! const wordOf = (set: MountPropagation) => set.shared && set.slave ? 'shared+slave' : /* ... */;
//! ```

use crate::subvolume::fs_mount_options::MountFacts;

/// The word the record carries for a mount that passes nothing on and receives
/// nothing.
pub const PRIVATE_WORD: &str = "private";

/// The word the record carries for a mount that passes its mounts on to its
/// peers.
pub const SHARED_WORD: &str = "shared";

/// The word the record carries for a mount that receives another mount's
/// mounts.
pub const SLAVE_WORD: &str = "slave";

/// The word the record carries for a mount that no peer group can hold.
pub const UNBINDABLE_WORD: &str = "unbindable";

/// The word the record carries for a mount that passes mounts on and receives
/// them,
/// which is the set a machine inside a container has.
pub const SHARED_AND_SLAVE_WORD: &str = "shared+slave";

/// The longest word any set is written as,
/// which the record's field bound is derived from.
pub const LONGEST_WORD: &str = SHARED_AND_SLAVE_WORD;

/// The optional field a mount table row prints for a mount in a peer group.
const SHARED_FIELD: &str = "shared:";

/// The optional field a mount table row prints for a mount that receives
/// another's mounts.
const MASTER_FIELD: &str = "master:";

/// The mount option a mount table row prints for a mount no peer group holds.
const UNBINDABLE_OPTION: &str = "unbindable";

/// What:
///  One mount's propagation,
///  as the three flags the kernel keeps for it.
/// Why:
///  The three are independent bits in `mount_setattr` and in the answer
///  `statmount(2)` gives,
///  so they are held as three flags rather than as one word,
///  and the words are derived from them.
///  A `&'static str` sibling does not fit:
/// the word is computed from three
///  flags a caller sets at run time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountPropagation = { shared: boolean; slave: boolean; unbindable: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MountPropagation {
    /// Whether the mount passes mounts on to its peers.
    shared: bool,
    /// Whether the mount receives mounts from a master.
    slave: bool,
    /// Whether no peer group can hold the mount.
    unbindable: bool,
}

impl MountPropagation {
    /// What:
    ///  The set the kernel named in `statmount(2)`'s own word for it,
    ///  or nothing when the word names a combination the kernel cannot make.
    /// Why:
    ///  The kernel answers `MS_SHARED`,
    ///  `MS_SLAVE`,
    ///  `MS_UNBINDABLE` or `MS_PRIVATE` for a mount
    ///  (`fs/namespace.c`,
    ///  `mnt_to_propagation_flags`),
    ///  and a set with two flags other than shared and slave would mean this
    ///  build read another kernel's answer,
    ///  which the step must refuse rather than report.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const ofKernelSet = (bits: number): MountPropagation | undefined => ...; // undefined on an unknown set
    /// ```
    pub fn of_the_kernel_set(bits: u64) -> Option<Self> {
        // The one bit every mount carries: `MS_PRIVATE` when no other flag is set, so it is never read alone.
        let known: u64 = libc::MS_SHARED | libc::MS_SLAVE | libc::MS_UNBINDABLE | libc::MS_PRIVATE;
        if bits & !known != 0 {
            // A bit this build does not know means the answer is another shape, not a private mount.
            return None;
        }
        let shared: bool = bits & libc::MS_SHARED != 0;
        let slave: bool = bits & libc::MS_SLAVE != 0;
        let unbindable: bool = bits & libc::MS_UNBINDABLE != 0;
        if unbindable && (shared || slave) {
            // The kernel clears the peer group and the master when it makes a mount unbindable, so this set is no answer.
            return None;
        }
        return Some(MountPropagation {
            shared,
            slave,
            unbindable,
        });
    }

    /// What:
    ///  The set a mount table row's own tags describe.
    /// Why:
    ///  The table prints a shared mount's peer group as `shared:<n>`,
    ///  a slave's master as `master:<n>` and an unbindable mount as the
    ///  `unbindable` mount option,
    ///  so the row answers the same three flags the kernel's answer does
    ///  (`proc_pid_mountinfo(5)`;
    ///  `src/native/health/doctor_zfs_route/mount_checks.rs`,
    ///  which reads the same two fields).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const ofTableRow = (row: MountFacts) => ({ shared: row.has('shared:'), slave: row.has('master:'), unbindable: row.hasOption('unbindable') });
    /// ```
    pub fn of_a_table_row(row: &MountFacts) -> Self {
        return MountPropagation {
            shared: row.optional_field(SHARED_FIELD).is_some(),
            slave: row.optional_field(MASTER_FIELD).is_some(),
            unbindable: row.has_option(UNBINDABLE_OPTION),
        };
    }

    /// What:
    ///  Whether the mount passes its mounts on to peers,
    ///  which is the state the step sets private.
    /// Why:
    ///  A mount that is shared and slave at once delivers to its peers too,
    ///  and the project chose to treat it as shared
    ///  (`doc/decision/cli-git-zfs-checkout-propagation-repair.md`,
    ///  "A mount that is shared and slave at once").
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const carriesShared = (set: MountPropagation) => set.shared;
    /// ```
    pub fn carries_shared(&self) -> bool {
        return self.shared;
    }

    /// What:
    ///  Whether the mount neither passes mounts on nor receives any.
    /// Why:
    ///  A mount in this set is the step's `already-private` answer,
    ///  and every other set without `MS_SHARED` is its `left` answer,
    ///  so the two must be told apart.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const isPrivate = (set: MountPropagation) => !set.shared && !set.slave && !set.unbindable;
    /// ```
    pub fn is_private(&self) -> bool {
        return !self.shared && !self.slave && !self.unbindable;
    }

    /// What:
    ///  The word the record and the summary line carry for this set.
    /// Why:
    ///  The record's `before` and `after` fields are read by a person and by
    ///  `doctor`,
    ///  so each set is one of the five words those readers know.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const word = (set: MountPropagation): string => /* 'private' | 'shared' | 'slave' | 'unbindable' | 'shared+slave' */;
    /// ```
    pub fn word(&self) -> &'static str {
        if self.unbindable {
            return UNBINDABLE_WORD;
        }
        if self.shared && self.slave {
            return SHARED_AND_SLAVE_WORD;
        }
        if self.shared {
            return SHARED_WORD;
        }
        if self.slave {
            return SLAVE_WORD;
        }
        return PRIVATE_WORD;
    }

    /// What:
    ///  The set one of the five words names,
    ///  or nothing when the word is not one of them.
    /// Why:
    ///  The record's reader and its parser both need the inverse of `word`,
    ///  so one table answers both and a word added here is a word both accept.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const fromWord = (word: string): MountPropagation | undefined => WORDS[word];
    /// ```
    pub fn from_word(word: &str) -> Option<Self> {
        return match word {
            PRIVATE_WORD => Some(MountPropagation {
                shared: false,
                slave: false,
                unbindable: false,
            }),
            SHARED_WORD => Some(MountPropagation {
                shared: true,
                slave: false,
                unbindable: false,
            }),
            SLAVE_WORD => Some(MountPropagation {
                shared: false,
                slave: true,
                unbindable: false,
            }),
            UNBINDABLE_WORD => Some(MountPropagation {
                shared: false,
                slave: false,
                unbindable: true,
            }),
            SHARED_AND_SLAVE_WORD => Some(MountPropagation {
                shared: true,
                slave: true,
                unbindable: false,
            }),
            _ => {
                // A word this build does not know is a record another build wrote, so a caller must refuse it.
                return None;
            }
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "mount_propagation_set_tests.rs"]
mod tests;
