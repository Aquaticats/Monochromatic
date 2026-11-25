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
    /// The file is there and is not one this program alone may write:
    /// a link at the path,
    /// another account's file,
    /// or a mode that lets another account write it.
    NotUnderRootControl {
        /// The path that was read.
        path: PathBuf,
        /// Why the file is not one this program alone may write.
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
            | ConfigurationRefusal::NotUnderRootControl { path, .. }
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
            ConfigurationRefusal::Unreadable { .. }
            | ConfigurationRefusal::NotUnderRootControl { .. }
            | ConfigurationRefusal::Empty { .. }
            | ConfigurationRefusal::NotJsonc { .. } => Some(String::from(CLONE_MOUNTS_KEY)),
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

    /// What was found,
    /// in one clause,
    /// with no remedy and no sentence about what the sweep did.
    /// Why:
    ///  A refusal that repeats its finding twice,
    ///  or chains two clauses about the sweep acting on nothing,
    ///  makes a reader work out which half carries the fact.
    fn cause(&self) -> String {
        return match self {
            ConfigurationRefusal::Unreadable { path, reason } => format!("the clone-mount configuration {} could not be read ({reason})", path.display()),
            ConfigurationRefusal::NotUnderRootControl { path, reason } => {
                format!("the clone-mount configuration {} is not a file this program alone may write ({reason})", path.display())
            }
            ConfigurationRefusal::Empty { path } => format!(
                "the clone-mount configuration {} is empty, and an empty file is not the same as no file: a file that is present wins",
                path.display()
            ),
            ConfigurationRefusal::NotJsonc { path } => format!("the clone-mount configuration {} is not one JSONC document", path.display()),
            ConfigurationRefusal::NotText { path, key } => format!("`{key}` in {} is not text", path.display()),
            ConfigurationRefusal::NameRefused { name, why, .. } => format!("`{name}` is not a dataset name the kernel takes: {}", why.message()),
            ConfigurationRefusal::CheckoutNotAbsolute { key, text, .. } => {
                format!("`{key}` names `{text}`, which is not an absolute checkout path, and a relative path would be resolved against the handler's own directory")
            }
            ConfigurationRefusal::Discovery { refusal, .. } => refusal.cause(),
            ConfigurationRefusal::CheckoutsDiffer { probe, sweep, .. } => format!(
                "the start probe is configured for the checkout {} and the clone-mount sweep for {}, and one probe's pass says nothing about a walk that starts at another checkout",
                probe.display(),
                sweep.display()
            ),
        };
    }

    /// The first thing a person does about this cause,
    /// naming the fault rather than a command no release carries.
    fn adjustment(&self) -> String {
        return match self {
            ConfigurationRefusal::Unreadable { path, .. } => format!("fix what stopped the read of {}, then run the sweep again.", path.display()),
            ConfigurationRefusal::NotUnderRootControl { path, .. } => format!(
                "make {} one this program alone may write, with `chown root:root {}` and `chmod go-w {}`, then run the sweep again.",
                path.display(),
                path.display(),
                path.display()
            ),
            ConfigurationRefusal::Empty { .. } => String::from("write the entries the file should hold, or remove the file so the route finds the checkout from this process's own mount table."),
            ConfigurationRefusal::NotJsonc { path, .. } => format!("fix {} so it parses as one JSONC document, then run the sweep again.", path.display()),
            ConfigurationRefusal::NotText { key, .. } => format!("write one text value at `{key}`, or remove that key."),
            ConfigurationRefusal::NameRefused { key, .. } => {
                format!("correct the name at `{key}` to a dataset name the kernel takes, or remove that key.")
            }
            ConfigurationRefusal::CheckoutNotAbsolute { key, .. } => format!("write the checkout's absolute path at `{key}`, or remove that key."),
            ConfigurationRefusal::Discovery { .. } => String::new(),
            ConfigurationRefusal::CheckoutsDiffer { .. } => String::from("make `cloneMounts.checkout` and `startProbe.checkout` name one folder, or remove one of them."),
        };
    }

    /// The entries a configuration file can carry,
    /// which settle the look-up for every person who reads a refusal about this
    /// file,
    /// and the answer a file that leaves them out gets.
    /// Why:
    ///  The wrapper and the root modes both find every entry a file leaves out from this process's own
    ///  mount table,
    ///  so a reader who has no file needs no setup a release does not carry,
    ///  and the names are written once rather than in every refusal.
    fn entries(&self) -> String {
        return format!(
            "The entries a file can carry are `{CLONE_MOUNTS_KEY}.{CHECKOUT_KEY}` (the checkout path), `{CLONE_MOUNTS_KEY}.{CHECKOUT_DATASET_KEY}` (the checkout dataset, such as `zpcachyos/ROOT/cos/home/{DESIGN_CHECKOUT_DATASET_NAME}`) and `{CLONE_MOUNTS_KEY}.{PARENT_KEY}` (the clone container, such as `zpcachyos/ROOT/cos/home/{DESIGN_CONTAINER_DATASET_NAME}`), as text in {file}; every entry a file leaves out is found from this process's own mount table, so a file with no such object works as long as the checkout dataset `{leaf}` is mounted once.",
            file = self.path().display(),
            leaf = DESIGN_CHECKOUT_DATASET_NAME
        );
    }

    /// The refusal in words,
    /// naming what was found,
    /// what settles it and the entries a file can carry.
    pub fn message(&self) -> String {
        if let ConfigurationRefusal::Discovery { path, refusal } = self {
            return refusal.message(path);
        }
        return format!(
            "{cause}, so this sweep enumerated nothing, attached nothing and recorded nothing, and every clone's mount intent stands untouched. Ways forward: {adjustment} {entries}",
            cause = self.cause(),
            adjustment = self.adjustment(),
            entries = self.entries()
        );
    }
}
