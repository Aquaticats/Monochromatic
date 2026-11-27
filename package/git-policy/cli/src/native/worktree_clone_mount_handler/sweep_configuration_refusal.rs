//! What:
//!  Why the sweep has no names to act on,
//!  with the words a person reads for each cause in
//!  `sweep_configuration_refusal_words.rs`.
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

use crate::start_probe::probe_source_name::NameRefusal;
use crate::zfs_mounted_dataset::mounted_dataset_refusal::MountedDatasetRefusal;
use std::path::PathBuf;

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
