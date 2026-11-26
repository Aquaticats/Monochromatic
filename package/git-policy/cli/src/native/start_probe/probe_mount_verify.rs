//! What:
//!  What the attach arm's own read-back must show:
//!  the mount's ID,
//!  the `zfs` filesystem,
//!  the exact source it asked for,
//!  and the read-only,
//!  `nosuid` and `nodev` flags it asked for.
//! Why:
//!  The arm reports the flags it asked for and reads them back,
//!  and it does not prove that a later mount is refused them
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "Limits").
//!  A mount found by ID rather than by path is the one the attach made,
//!  because the worker holds its descriptor and the table names the ID that
//!  descriptor created.
//!  The mount table arrives as text,
//!  so the comparison is exercised without a mount.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mount = verifyMount(table, { mountId: 41, source: 'zpcachyos/nosnap/git-worktree-probe@ready' });
//! ```

use crate::fs_mount_options::MountFacts;
use std::path::PathBuf;

/// The filesystem type the probe's source is mounted as.
pub const EXPECTED_FILESYSTEM: &str = "zfs";

/// The flags the attach asks for,
/// read back as mount options.
pub const EXPECTED_FLAGS: &[&str] = &["ro", "nosuid", "nodev"];

/// What:
///  One mount the attach asked for.
/// Why:
///   The mount's ID is the one the descriptor created,
///  and the source is the exact name the configuration produced,
///  so neither another mount of the same dataset nor a mount of another source
///  passes for this one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ExpectedMount = { mountId: number; source: string };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpectedMount<'a> {
    /// The kernel's own ID of the mount the attach made.
    pub mount_id: u64,
    /// The source the attach wrote into the context.
    pub source: &'a str,
}

/// What:
///  A mount the read-back accepted,
///  with the facts a reader of the record wants named.
/// Why:
///   `doctor` prints what the probe mounted,
///  and the record says the flags were read back rather than assumed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type VerifiedMount = { mountId: number; filesystem: string; source: string; options: string[]; mountPoint: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedMount {
    /// The mount's ID.
    pub mount_id: u64,
    /// Its filesystem type.
    pub filesystem: String,
    /// Its source,
    /// as the kernel lists it.
    pub source: String,
    /// The options the table lists for it.
    pub options: Vec<String>,
    /// Its mount point.
    pub mount_point: PathBuf,
}

/// What:
///  Why the mount the attach made is not the mount it asked for.
/// Why:
///   Each cause names the field and both values,
///  so the record says what the kernel did rather than only that it differed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type VerifyFailure = { kind: 'not-listed' | 'other-filesystem' | 'other-source' | 'missing-flag'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerifyFailure {
    /// The mount ID the descriptor created has no line in the table.
    NotListed {
        /// The expected ID.
        expected: u64,
    },
    /// The mount is listed as another filesystem.
    OtherFilesystem {
        /// The filesystem the table lists.
        found: String,
    },
    /// The mount's source is not the one the attach wrote.
    OtherSource {
        /// The source the table lists.
        found: String,
        /// The source the attach wrote.
        expected: String,
    },
    /// One of the flags the attach asked for is not in the table's line.
    MissingFlag {
        /// The flag.
        flag: &'static str,
    },
}

impl VerifyFailure {
    /// The failure in words,
    /// naming the field and what to do about it.
    pub fn message(&self) -> String {
        return match self {
            VerifyFailure::NotListed {
                expected,
            } => format!(
                "the probe's attach made mount {expected}, and the mount table of its own namespace has no line for it. The mount did not land where the probe placed it."
            ),
            VerifyFailure::OtherFilesystem {
                found,
            } => format!(
                "the probe's attach made a mount whose filesystem is `{found}`, not `{EXPECTED_FILESYSTEM}`."
            ),
            VerifyFailure::OtherSource {
                found,
                expected,
            } => {
                format!(
                    "the probe's attach made a mount whose source is `{found}`, not the `{expected}` it asked for."
                )
            }
            VerifyFailure::MissingFlag {
                flag,
            } => format!("the probe's attach made a mount without the `{flag}` flag it asked for."),
        };
    }
}

/// What:
///  Whether the table holds the mount the attach asked for,
///  with the source and flags it asked for.
/// Why:
///   The comparison runs on the namespace's own table,
///  which the worker read after `move_mount`,
///  and every field is checked rather than the first that differs,
///  so the record can name the mount that was made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mount = verifyMount(table, expected); // throws VerifyFailure
/// ```
///
/// # Errors
/// The mount is not listed,
///  is another filesystem,
///  carries another source,
///  or lacks one of the three flags.
pub fn verify_mount(
    table: &[MountFacts],
    expected: ExpectedMount<'_>,
) -> Result<VerifiedMount, VerifyFailure> {
    let Some(facts) = table
        .iter()
        .find(|facts| return facts.mount_id == expected.mount_id)
    else {
        return Err(VerifyFailure::NotListed {
            expected: expected.mount_id,
        });
    };
    if facts.filesystem_type != EXPECTED_FILESYSTEM {
        return Err(VerifyFailure::OtherFilesystem {
            found: facts.filesystem_type.clone(),
        });
    }
    if facts.source != expected.source {
        return Err(VerifyFailure::OtherSource {
            found: facts.source.clone(),
            expected: String::from(expected.source),
        });
    }
    for flag in EXPECTED_FLAGS {
        if !facts.has_option(flag) {
            return Err(VerifyFailure::MissingFlag {
                flag,
            });
        }
    }
    return Ok(VerifiedMount {
        mount_id: facts.mount_id,
        filesystem: facts.filesystem_type.clone(),
        source: facts.source.clone(),
        options: facts.mount_options.clone(),
        mount_point: facts.mount_point.clone(),
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_mount_verify_tests.rs"]
mod tests;
