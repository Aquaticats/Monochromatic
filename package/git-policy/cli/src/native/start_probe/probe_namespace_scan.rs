//! What:
//!  Arm 7's scan:
//!  every mount namespace of this PID namespace that lists the probe's exact
//!  ZFS source,
//!  grouped by the task's own mount-namespace identity,
//!  and the two comparisons the design makes with it.
//! Why:
//!  The exclusive create proved no superblock existed when the probe attached,
//!  so a further namespace that mentions the source is either a mount the probe
//!  leaked or another root process's mount made during the run,
//!  and both are that arm's failure
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The arms").
//!  A process-visible scan is not proof of absence,
//!  which the record and the design keep saying.
//!  The scan reads a `/proc` root the caller names,
//!  so a control reads a fixture tree and never this machine's tasks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mentioned = scanNamespaces('/proc', source); // ['mnt:[4026531840]', 'mnt:[4026532001]']
//! expectExactly(['mnt:[4026531840]', 'mnt:[4026532001]'], mentioned);
//! ```

use crate::fs_mount_options::{MountFacts, parse_mount_table};
use std::path::{Path, PathBuf};

/// The bound one representative mount table is read within,
/// the wrapper's own bound for a table another namespace holds.
pub const MOUNT_TABLE_LIMIT: u64 = crate::plain_file::MOUNT_TABLE_LIMIT;

/// What:
///  Why the scan could not say which namespaces mention the source.
/// Why:
///  A scan that read no table has proved nothing,
///  so the arm fails rather than reporting a clear result,
///  and each cause names what could not be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScanRefusal = { kind: 'proc' | 'table'; path: string; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScanRefusal {
    /// The `/proc` root could not be listed.
    Proc {
        /// The directory.
        path: PathBuf,
        /// What the call answered.
        reason: String,
    },
    /// A representative mount table could not be read.
    Table {
        /// The file.
        path: PathBuf,
        /// What the read answered.
        reason: String,
    },
}

impl ScanRefusal {
    /// The refusal in words,
    /// naming the file and what to change.
    pub fn message(&self) -> String {
        return match self {
            ScanRefusal::Proc { path, reason } => format!(
                "the probe could not list {} ({reason}), so it cannot say which mount namespaces hold the source it attached. The namespace arm failed rather than reporting a clear result.",
                path.display()
            ),
            ScanRefusal::Table { path, reason } => format!(
                "the probe could not read {} ({reason}), so the namespace it belongs to cannot be compared. The namespace arm failed rather than reporting a clear result.",
                path.display()
            ),
        };
    }
}

/// What:
///  One mount namespace the scan found,
///  with the representative table it read.
/// Why:
///   The identity is the kernel's own `mnt:[...]` text from the task's
///  `ns/mnt` link,
///  and the table is one live task's,
///  so the comparison names namespaces rather than pids.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NamespaceTable = { namespace: string; mounts: MountFacts[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamespaceTable {
    /// The mount-namespace identity,
    /// such as `mnt:[4026531840]`.
    pub namespace: String,
    /// One live task's mount table in that namespace.
    pub mounts: Vec<MountFacts>,
}

impl NamespaceTable {
    /// Whether this namespace lists the exact ZFS source.
    pub fn mentions(&self, source: &str) -> bool {
        return self.mounts.iter().any(|facts| return facts.filesystem_type == "zfs" && facts.source == source);
    }
}

/// What:
///  Why the namespaces that mention the source are not the ones the pass
///  expects.
/// Why:
///   The design's first pass expects exactly the worker's and the holder's,
///  and its later passes expect the worker's alone or none,
///  so a namespace left over or missing is a named difference rather than a
///  count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScanDifference = { kind: 'unexpected' | 'missing'; namespace: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScanDifference {
    /// A namespace that mentions the source is not one the pass expects.
    Unexpected {
        /// The namespace.
        namespace: String,
    },
    /// A namespace the pass expects does not mention the source.
    Missing {
        /// The namespace.
        namespace: String,
    },
}

impl ScanDifference {
    /// The difference in words.
    pub fn message(&self) -> String {
        return match self {
            ScanDifference::Unexpected { namespace } => format!(
                "mount namespace {namespace} lists the probe's source, and it is not one the probe made: either the probe leaked its mount or another root process mounted the same source during the run."
            ),
            ScanDifference::Missing { namespace } => {
                format!("mount namespace {namespace} was expected to list the probe's source, and the scan found it nowhere, so the mount the probe made is not visible where it must be.")
            }
        };
    }
}

/// What:
///  Read every live task's mount-namespace identity under one `/proc` root,
///  read one live task's mount table per identity,
///  and answer the identities that list the source.
/// Why:
///   Grouping by the kernel's own namespace link before reading a table keeps
///  the scan to one table per namespace however many tasks it holds,
///  and the caller compares the answer with the namespaces the probe made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mentioned = namespaces_mentioning(source, scanNamespaces('/proc'));
/// ```
///
/// # Errors
/// The `/proc` root could not be listed or a representative table could not
///  be read.
pub fn scan_namespaces(proc_root: &Path, source: &str) -> Result<Vec<String>, ScanRefusal> {
    let namespaces: Vec<NamespaceTable> = namespaces_outside(proc_root)?;
    let mut mentioned: Vec<String> = Vec::new();
    for namespace in namespaces {
        if namespace.mentions(source) {
            mentioned.push(namespace.namespace);
        }
    }
    return Ok(mentioned);
}

/// What:
///  One representative table per live mount namespace.
/// Why:
///   A task that exits between the listing and the read is skipped rather than
///  failing the scan,
///  and a namespace with no live task is invisible,
///  which the design states as the scan's own limit.
///
/// # Errors
/// The listing or a representative table could not be read.
pub fn namespaces_outside(proc_root: &Path) -> Result<Vec<NamespaceTable>, ScanRefusal> {
    let entries: std::collections::BTreeMap<String, PathBuf> = match std::fs::read_dir(proc_root) {
        Ok(iter) => iter,
        Err(error) => {
            return Err(ScanRefusal::Proc { path: proc_root.to_path_buf(), reason: error.to_string() });
        }
    }
    .filter_map(|entry| return entry.ok())
    .filter(|entry| return entry.file_name().to_string_lossy().chars().all(|character| return character.is_ascii_digit()))
    .filter_map(|entry| {
        let namespace: String = std::fs::read_link(entry.path().join("ns/mnt")).ok()?.to_string_lossy().into_owned();
        return Some((namespace, entry.path()));
    })
    .collect();
    let mut tables: Vec<NamespaceTable> = Vec::new();
    for (namespace, task) in entries {
        let table_path: PathBuf = task.join("mountinfo");
        let text: Vec<u8> = match crate::plain_file::read_plain_file(table_path.as_path(), MOUNT_TABLE_LIMIT) {
            Ok(bytes) => bytes,
            Err(error) => return Err(ScanRefusal::Table { path: table_path, reason: error.to_string() }),
        };
        tables.push(NamespaceTable { namespace, mounts: parse_mount_table(text.as_slice()) });
    }
    return Ok(tables);
}

/// What:
///  Whether the namespaces that mention the source are exactly the ones a pass
///  expects,
///  and the first difference when they are not.
/// Why:
///   The expected set is a list the caller owns,
///  and comparing both directions is what makes a leaked mount and a missing
///  one separate findings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const difference = compare(mentioned, expected);
/// ```
pub fn compare_namespaces(mentioned: &[String], expected: &[&str]) -> Option<ScanDifference> {
    if let Some(namespace) = mentioned.iter().find(|namespace| return !expected.contains(&namespace.as_str())) {
        return Some(ScanDifference::Unexpected { namespace: namespace.clone() });
    }
    if let Some(namespace) = expected.iter().find(|namespace| return !mentioned.iter().any(|found| return found == *namespace)) {
        return Some(ScanDifference::Missing { namespace: String::from(*namespace) });
    }
    return None;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_namespace_scan_tests.rs"]
mod tests;
