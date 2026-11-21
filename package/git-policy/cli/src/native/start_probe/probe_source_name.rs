//! What:
//!  The dataset-name rule the probe's source passes:
//!  the bytes one component may hold,
//!  the name's bound,
//!  and the empty `ready` snapshot the design fixes.
//! Why:
//!  The probe makes no ioctl,
//!  so the name is the only thing it can check before the kernel sees it
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The source:
//! the probe dataset").
//!  A name that is no dataset name would make every boot's attach arm fail
//!  against a configuration the setup wrote,
//!  so the rule is here and the record carries it as a step that did not run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! checkDatasetName('zpcachyos/nosnap/git-worktree-probe'); const source = new ProbeSource(dataset).snapshotName();
//! ```

/// The snapshot the design fixes,
/// empty and never written.
pub const READY_SNAPSHOT: &str = "ready";

/// The bound one dataset name has in OpenZFS,
/// its terminating NUL included,
/// so a name one byte shorter is the longest this build accepts.
pub const DATASET_NAME_BOUND: usize = 255;

/// What:
///  Why one dataset name is not one.
/// Why:
///   The rule is the kernel's,
///  and each cause names the byte that broke it,
///  because "invalid name" alone sends a person guessing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NameRefusal = 'empty' | 'too-long' | 'empty-component' | 'bad-character' | 'snapshot-or-bookmark';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameRefusal {
    /// The name is empty.
    Empty,
    /// The name is longer than the bound.
    TooLong,
    /// One `/`-separated component is empty.
    EmptyComponent,
    /// A byte is not one a component may hold.
    BadCharacter {
        /// The byte as a person reads it.
        byte: char,
    },
    /// The name carries `@` or `#`,
    /// which would name a snapshot or a bookmark.
    SnapshotOrBookmark,
}

impl NameRefusal {
    /// The cause in words.
    pub fn message(self) -> String {
        return match self {
            NameRefusal::Empty => String::from("the name is empty"),
            NameRefusal::TooLong => format!("the name is longer than {DATASET_NAME_BOUND} bytes"),
            NameRefusal::EmptyComponent => String::from("one `/`-separated component is empty"),
            NameRefusal::BadCharacter { byte } => format!("`{byte}` is not a byte a dataset component holds"),
            NameRefusal::SnapshotOrBookmark => String::from("the name carries `@` or `#`, which names a snapshot or a bookmark"),
        };
    }
}

/// What:
///  One dataset name that passed the rule,
///  and the source assembled from it and the fixed snapshot.
/// Why:
///   The record holds the source it attached,
///  and the dataset is kept apart because the name rule judges that alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProbeSource = { dataset: string; snapshot: string }; source.snapshotName === `${dataset}@ready`;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbeSource {
    /// The dataset the configuration names.
    pub dataset: String,
}

impl ProbeSource {
    /// The source the probe attaches:
    /// the configured dataset and the empty `ready` snapshot.
    pub fn snapshot_name(&self) -> String {
        return format!("{}@{}", self.dataset, READY_SNAPSHOT);
    }
}

/// Whether one configured dataset name is a name the kernel takes.
pub fn check_dataset_name(name: &str) -> Result<(), NameRefusal> {
    if name.is_empty() {
        return Err(NameRefusal::Empty);
    }
    if name.len() > DATASET_NAME_BOUND {
        return Err(NameRefusal::TooLong);
    }
    for component in name.split('/') {
        if component.is_empty() {
            return Err(NameRefusal::EmptyComponent);
        }
        for byte in component.chars() {
            if byte == '@' || byte == '#' {
                return Err(NameRefusal::SnapshotOrBookmark);
            }
            if !(byte.is_ascii_alphanumeric() || matches!(byte, '_' | '-' | '.' | ':')) {
                return Err(NameRefusal::BadCharacter { byte });
            }
        }
    }
    return Ok(());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_source_name_tests.rs"]
mod tests;
