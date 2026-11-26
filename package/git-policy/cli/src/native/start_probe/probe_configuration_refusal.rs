//! What:
//!  Why the probe could not tell its source or its checkout,
//!  and the closed word the record carries for each cause.
//! Why:
//!  The record holds a word rather than prose
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The result record and its fingerprint"),
//!  and a person has to tell a file that cannot be read from a name the kernel
//!  refuses and from a name that matched no dataset or several
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  "Answer of 2026-11-22:
//! the probe needs no configuration file").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type SourceRefusal = { kind: 'configuration-unreadable' | 'name-refused' | 'no-mounted-checkout' | ... };
//! ```

use super::probe_configuration::{
    CHECKOUT_KEY,
    DATASET_KEY,
    START_PROBE_KEY,
};
use super::probe_source_name::NameRefusal;
use crate::zfs_mounted_dataset::mounted_dataset_refusal::MountedDatasetRefusal;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Why the source name could not be told,
///  or why the name the file holds is not one.
/// Why:
///   Every cause is a different thing for a person to look at,
///  and the record carries a closed word for each so a reader needs no prose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SourceRefusal = { kind: 'configuration-unreadable' | 'configuration-unparsable' | 'name-not-text' | 'name-refused' | 'no-mounted-checkout' | 'several-mounted-checkouts' | 'mount-table-unreadable' | 'no-source-name' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceRefusal {
    /// The configuration file could not be read.
    ConfigurationUnreadable {
        /// The path that was read.
        path: PathBuf,
        /// What the read answered.
        reason: String,
    },
    /// The configuration file is not one this program alone may write:
    /// a link at the path,
    /// another account's file,
    /// or a mode that lets another account write it.
    ConfigurationNotUnderRootControl {
        /// The path that was read.
        path: PathBuf,
        /// Why the file is not one this program alone may write.
        reason: String,
    },
    /// The configuration file is there and holds nothing,
    /// which is not the same as no file at all.
    ConfigurationEmpty {
        /// The path that was read.
        path: PathBuf,
    },
    /// The configuration file is not one JSONC document.
    ConfigurationUnparsable {
        /// The path that was read.
        path: PathBuf,
        /// What the parse answered.
        reason: String,
    },
    /// The key's value is not text.
    NameNotText {
        /// The path that was read.
        path: PathBuf,
    },
    /// The value is text the name rule refuses.
    NameRefused {
        /// The name as written.
        name: String,
        /// Why it is refused.
        why: NameRefusal,
    },
    /// The checkout the probe dataset's pool would come from could not be told,
    /// or the pool could not be named.
    Checkout {
        /// What the lookup refused.
        refusal: MountedDatasetRefusal,
    },
    /// The file names no probe dataset and this run told no checkout,
    /// so the pool the design's name needs is not known.
    CheckoutNotFound {
        /// The path that was read.
        path: PathBuf,
    },
}

impl SourceRefusal {
    /// The word the record carries for this cause.
    pub fn word(&self) -> &'static str {
        return match self {
            SourceRefusal::ConfigurationUnreadable {
                ..
            } => "configuration-unreadable",
            SourceRefusal::ConfigurationNotUnderRootControl {
                ..
            } => "configuration-not-under-root-control",
            SourceRefusal::ConfigurationEmpty {
                ..
            } => "configuration-empty",
            SourceRefusal::ConfigurationUnparsable {
                ..
            } => "configuration-unparsable",
            SourceRefusal::NameNotText {
                ..
            } => "name-not-text",
            SourceRefusal::NameRefused {
                ..
            } => "name-refused",
            SourceRefusal::Checkout {
                refusal,
            } => refusal.word(),
            SourceRefusal::CheckoutNotFound {
                ..
            } => "no-mounted-checkout",
        };
    }

    /// The refusal in words,
    /// naming the path,
    /// the key and what to change.
    pub fn message(
        &self,
        configuration: &Path,
    ) -> String {
        let key: String = format!("{START_PROBE_KEY}.{DATASET_KEY}");
        return match self {
            SourceRefusal::ConfigurationUnreadable {
                path,
                reason,
            } => format!(
                "the probe's source is not configured: {} could not be read ({reason}). Nothing was attached. Ways forward: make the file readable by root and run the probe again.",
                path.display()
            ),
            SourceRefusal::ConfigurationNotUnderRootControl {
                path,
                reason,
            } => format!(
                "the probe's source is not configured: {} is not a file this program alone may write ({reason}). Nothing was attached. Ways forward: make it root's own file with `chown root:root {}` and `chmod go-w {}`, and run the probe again.",
                path.display(),
                path.display(),
                path.display()
            ),
            SourceRefusal::ConfigurationEmpty {
                path,
            } => format!(
                "the probe's source is not configured: {} is empty, and an empty file is not the same as no file. Nothing was attached. Ways forward: remove the file so the probe finds its dataset by the design's own name, or write `{key}` in it.",
                path.display()
            ),
            SourceRefusal::ConfigurationUnparsable {
                path,
                reason,
            } => format!(
                "the probe's source is not configured: {} is not one JSONC document ({reason}). Nothing was attached. Ways forward: fix the file so it parses, then run the probe again.",
                path.display()
            ),
            SourceRefusal::NameNotText {
                path,
            } => format!(
                "the probe's source is not configured: `{key}` in {} is not one text value. Nothing was attached. Ways forward: give it one quoted dataset name.",
                path.display()
            ),
            SourceRefusal::NameRefused {
                name,
                why,
            } => format!(
                "the probe's source is not configured: `{key}` names `{name}`, which is not a dataset name the kernel takes ({}). Nothing was attached. Ways forward: give `{key}` the dataset name, with no snapshot, bookmark or path separator a name may not hold.",
                why.message()
            ),
            SourceRefusal::Checkout {
                refusal,
            } => format!(
                "the probe's dataset is not configured and its design's own name needs the checkout: {cause}. Nothing was attached. {remedy}",
                cause = refusal.cause(),
                remedy = probe_checkout_remedy(
                    refusal,
                    configuration,
                    format!("{START_PROBE_KEY}.{CHECKOUT_KEY}").as_str()
                )
            ),
            SourceRefusal::CheckoutNotFound {
                path,
            } => format!(
                "the probe's dataset is not configured and no checkout was found to take the pool from, so `nosnap/git-worktree-probe` has no pool to sit in ({} was read). Nothing was attached. Ways forward: write `{key}` in {} naming the probe dataset, or mount the checkout dataset so the route can find the pool.",
                path.display(),
                configuration.display()
            ),
        };
    }
}

/// What:
///  What settles a look-up the probe could not answer,
///  in the probe's own words.
/// Why:
///  The sweep's remedy names `cloneMounts.checkout` and
///  `cloneMounts.checkoutDataset`,
///  which the probe does not read,
///  so a person who follows it finds the probe still blocked
///  (the second pass's finding B2).
///  The probe's key is `startProbe.checkout`,
///  and the mount of the checkout dataset is the other way,
///  so this names those two and nothing the sweep owns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remedy = probeCheckoutRemedy(refusal, path, 'startProbe.checkout');
/// ```
fn probe_checkout_remedy(
    refusal: &MountedDatasetRefusal,
    path: &Path,
    key: &str,
) -> String {
    let file = path.display();
    return match refusal {
        MountedDatasetRefusal::TableUnreadable {
            ..
        } => format!(
            "Ways forward: run the probe from a process that can read `/proc/self/mountinfo`, or write `{key}` in {file} naming the checkout path."
        ),
        MountedDatasetRefusal::SeveralMounted {
            ..
        }
        | MountedDatasetRefusal::DatasetMountedSeveralTimes {
            ..
        } => format!(
            "Ways forward: leave one mounted instance of the checkout dataset and run the probe again, or write `{key}` in {file} naming the checkout path."
        ),
        MountedDatasetRefusal::NoPoolForTheProbeDataset {
            ..
        } => format!(
            "Ways forward: write `{START_PROBE_KEY}.{DATASET_KEY}` in {file} naming the probe dataset in full."
        ),
        MountedDatasetRefusal::NoParentForTheContainer {
            ..
        }
        | MountedDatasetRefusal::NoneMounted {
            ..
        }
        | MountedDatasetRefusal::DatasetNotMounted {
            ..
        }
        | MountedDatasetRefusal::NoDatasetAtThePath {
            ..
        }
        | MountedDatasetRefusal::LeafNameDiffers {
            ..
        }
        | MountedDatasetRefusal::NotADataset {
            ..
        } => format!(
            "Ways forward: mount the checkout dataset `monochromatic-git-checkout`, which this release carries no setup for, or write `{key}` in {file} naming the checkout path."
        ),
    };
}

/// What:
///  Why the checkout path the probe opens could not be told.
/// Why:
///   The checkout-root step opens one folder read-only,
///  so a checkout that cannot be told is a step that did not run rather than a
///  probe failure,
///  and the record's word says which kind of finding stopped it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CheckoutRefusal = 'configuration-unparsable' | 'name-not-text' | 'name-refused' | 'no-mounted-checkout' | 'several-mounted-checkouts' | 'mount-table-unreadable';
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckoutRefusal {
    /// The file is there and holds nothing,
    /// which is not the same as no file at all.
    Empty,
    /// The file is not one JSONC document.
    Unparsable,
    /// The key is there and its value is not text.
    NotText,
    /// The value is text that is not an absolute path.
    NotAbsolute,
    /// No mounted instance of the design's checkout dataset was found,
    /// or the path it was given is not on one.
    NotFound {
        /// What the lookup refused.
        refusal: MountedDatasetRefusal,
    },
}

impl CheckoutRefusal {
    /// The word the record carries for this cause.
    pub fn word(&self) -> &'static str {
        return match self {
            CheckoutRefusal::Empty => "configuration-empty",
            CheckoutRefusal::Unparsable => "configuration-unparsable",
            CheckoutRefusal::NotText => "name-not-text",
            CheckoutRefusal::NotAbsolute => "name-refused",
            CheckoutRefusal::NotFound {
                refusal,
            } => refusal.word(),
        };
    }

    /// The refusal in words,
    /// naming the key and what to change.
    pub fn message(
        &self,
        path: &Path,
    ) -> String {
        let key: String = format!("{START_PROBE_KEY}.{CHECKOUT_KEY}");
        return match self {
            CheckoutRefusal::Empty => format!(
                "the probe's checkout could not be told: {} is empty, and an empty file is not the same as no file. Ways forward: remove the file so the probe finds the mounted checkout dataset, or write `{key}` in it.",
                path.display()
            ),
            CheckoutRefusal::Unparsable => format!(
                "the probe's checkout could not be told: {} is not one JSONC document. Ways forward: fix the file so it parses, or remove it so the probe finds the mounted checkout dataset.",
                path.display()
            ),
            CheckoutRefusal::NotText => format!(
                "the probe's checkout could not be told: `{key}` in {} is not one text value. Ways forward: give it one absolute path, or remove the key so the probe finds the mounted checkout dataset.",
                path.display()
            ),
            CheckoutRefusal::NotAbsolute => format!(
                "the probe's checkout could not be told: `{key}` in {} is not an absolute path, and a relative one would be resolved against the worker's own directory. Ways forward: write the checkout's absolute path there, or remove the key.",
                path.display()
            ),
            CheckoutRefusal::NotFound {
                refusal,
            } => format!(
                "the probe's checkout could not be told: {cause}. The probe added nothing to the mount table and enables no managed attach. {remedy}",
                cause = refusal.cause(),
                remedy = probe_checkout_remedy(refusal, path, key.as_str())
            ),
        };
    }
}
