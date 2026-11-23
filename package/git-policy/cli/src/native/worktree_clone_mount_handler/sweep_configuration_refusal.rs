//! What:
//!  Why the sweep has no names to act on,
//!  and the words a person reads for each cause.
//! Why:
//!  A file that cannot be read,
//!  a file that is not JSONC,
//!  a value of the wrong type,
//!  a name the kernel would refuse,
//!  a checkout path that is not absolute,
//!  a design name that matches no dataset or several,
//!  and two keys naming different checkouts send a person to different places,
//!  so each is its own answer
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "With no configuration file").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ConfigurationRefusal = { kind: 'unreadable' | 'not-jsonc' | 'not-text' | 'name-refused' | 'not-absolute' | 'discovery' | 'checks-differ'; ... };
//! ```

use super::sweep_configuration::{CHECKOUT_DATASET_KEY, CHECKOUT_KEY, CLONE_MOUNTS_KEY, PARENT_KEY, PROBE_OBJECT_KEY};
use crate::start_probe::probe_source_name::NameRefusal;
use crate::zfs_mounted_dataset::mounted_dataset_refusal::MountedDatasetRefusal;
use crate::zfs_mounted_dataset::the_designs_datasets_in_the_mount_table::{DESIGN_CHECKOUT_DATASET_NAME, DESIGN_CONTAINER_DATASET_NAME};
use std::path::{Path, PathBuf};

/// What:
///  Why the sweep has no names to act on.
/// Why:
///  Each cause is its own answer,
///  and each names the key or the name it is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigurationRefusal = { kind: 'unreadable' | 'not-jsonc' | 'not-text' | 'name-refused' | 'not-absolute' | 'discovery' | 'checks-differ'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigurationRefusal {
    /// The configuration file is there and could not be read.
    Unreadable {
        /// The path that was read.
        path: PathBuf,
        /// What the read answered.
        reason: String,
    },
    /// The configuration file is there and holds nothing,
    /// which is not the same as no file at all.
    Empty {
        /// The path that was read.
        path: PathBuf,
    },
    /// The configuration is not one JSONC document.
    NotJsonc {
        /// The path that was read.
        path: PathBuf,
    },
    /// The value is not text.
    NotText {
        /// The path that was read.
        path: PathBuf,
        /// The key whose value is not text.
        key: String,
    },
    /// A dataset name's text is not a name the kernel takes.
    NameRefused {
        /// The path that was read.
        path: PathBuf,
        /// The key the name is written at.
        key: String,
        /// The name as written.
        name: String,
        /// Why the name rule refused it.
        why: NameRefusal,
    },
    /// The checkout's text is not an absolute path.
    CheckoutNotAbsolute {
        /// The path that was read.
        path: PathBuf,
        /// The key the value is written at.
        key: String,
        /// The value as written.
        text: String,
    },
    /// A design name matched no dataset or several,
    /// or the value the file left out could not be found.
    Discovery {
        /// The path that was read.
        path: PathBuf,
        /// Why the finding refused.
        refusal: MountedDatasetRefusal,
    },
    /// The probe's checkout and the sweep's checkout are different folders.
    CheckoutsDiffer {
        /// The path that was read.
        path: PathBuf,
        /// The probe's configured checkout.
        probe: PathBuf,
        /// The sweep's configured checkout.
        sweep: PathBuf,
    },
}

impl ConfigurationRefusal {
    /// The path that was read,
    /// which every refusal about this file names.
    pub fn path(&self) -> &Path {
        return match self {
            ConfigurationRefusal::Unreadable { path, .. }
            | ConfigurationRefusal::Empty { path }
            | ConfigurationRefusal::NotJsonc { path }
            | ConfigurationRefusal::NotText { path, .. }
            | ConfigurationRefusal::NameRefused { path, .. }
            | ConfigurationRefusal::CheckoutNotAbsolute { path, .. }
            | ConfigurationRefusal::Discovery { path, .. }
            | ConfigurationRefusal::CheckoutsDiffer { path, .. } => path,
        };
    }

    /// The key one refusal is about,
    /// which is what a person has to write,
    /// or nothing when no single key settles it.
    pub fn key(&self) -> Option<String> {
        return match self {
            ConfigurationRefusal::Unreadable { .. } | ConfigurationRefusal::Empty { .. } | ConfigurationRefusal::NotJsonc { .. } => Some(String::from(CLONE_MOUNTS_KEY)),
            ConfigurationRefusal::NotText { key, .. }
            | ConfigurationRefusal::NameRefused { key, .. }
            | ConfigurationRefusal::CheckoutNotAbsolute { key, .. } => Some(key.clone()),
            ConfigurationRefusal::CheckoutsDiffer { .. } => Some(format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_KEY}")),
            ConfigurationRefusal::Discovery { refusal, .. } => match refusal {
                MountedDatasetRefusal::NoPoolForTheProbeDataset { .. } => Some(String::from(PROBE_OBJECT_KEY)),
                MountedDatasetRefusal::NoParentForTheContainer { .. } => Some(format!("{CLONE_MOUNTS_KEY}.{PARENT_KEY}")),
                MountedDatasetRefusal::TableUnreadable { .. }
                | MountedDatasetRefusal::NoneMounted { .. }
                | MountedDatasetRefusal::SeveralMounted { .. }
                | MountedDatasetRefusal::DatasetNotMounted { .. }
                | MountedDatasetRefusal::DatasetMountedSeveralTimes { .. }
                | MountedDatasetRefusal::NoDatasetAtThePath { .. }
                | MountedDatasetRefusal::LeafNameDiffers { .. }
                | MountedDatasetRefusal::NotADataset { .. } => Some(format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_DATASET_KEY}")),
            },
        };
    }

    /// The refusal in words,
    /// naming the file,
    /// the key and what to write there.
    pub fn message(&self) -> String {
        let cause: String = match self {
            ConfigurationRefusal::Unreadable { path, reason } => format!("the clone-mount configuration {} could not be read ({reason})", path.display()),
            ConfigurationRefusal::Empty { path } => format!(
                "the clone-mount configuration {} is empty, and an empty file is not the same as no file: a file that is present wins, so this sweep acts on nothing until the file holds the entries or is removed",
                path.display()
            ),
            ConfigurationRefusal::NotJsonc { path } => format!("the clone-mount configuration {} is not one JSONC document", path.display()),
            ConfigurationRefusal::NotText { path, key } => format!("`{key}` in {} is not text", path.display()),
            ConfigurationRefusal::NameRefused { name, why, .. } => format!("`{name}` is not a dataset name the kernel takes: {}", why.message()),
            ConfigurationRefusal::CheckoutNotAbsolute { key, text, .. } => {
                format!("`{key}` names `{text}`, which is not an absolute checkout path, and a relative path would be resolved against the handler's own directory")
            }
            ConfigurationRefusal::Discovery { path, refusal } => return refusal.message(path),
            ConfigurationRefusal::CheckoutsDiffer { probe, sweep, .. } => format!(
                "the start probe is configured for the checkout {} and the clone-mount sweep for {}, and one probe's pass says nothing about a walk that starts at another checkout",
                probe.display(),
                sweep.display()
            ),
        };
        return format!(
            "{cause}, so this sweep enumerated nothing, attached nothing and recorded nothing, and every clone's mount intent stands untouched. Ways forward: write any of `{CLONE_MOUNTS_KEY}.{CHECKOUT_KEY}` (the checkout path), `{CLONE_MOUNTS_KEY}.{CHECKOUT_DATASET_KEY}` (the checkout dataset, such as `zpcachyos/ROOT/cos/home/{DESIGN_CHECKOUT_DATASET_NAME}`) and `{CLONE_MOUNTS_KEY}.{PARENT_KEY}` (the clone container, such as `zpcachyos/ROOT/cos/home/{DESIGN_CONTAINER_DATASET_NAME}`) as text in {}; every entry a file leaves out is found from this process's own mount table, so a file with no such object works as long as the checkout dataset `{leaf}` is mounted once. `git monochromatic-git-policy-cli doctor` reports the entries once plan step 18 adds that.",
            self.path().display(),
            leaf = DESIGN_CHECKOUT_DATASET_NAME
        );
    }
}
