//! What:
//!  The names the clone-mount sweep acts on:
//!  the checkout dataset whose snapshots its clones are made from,
//!  the pinned clone parent dataset whose immediate children it sweeps,
//!  and the checkout it walks from.
//! Why:
//!  The sweep attaches nothing without a parent to enumerate and a checkout to
//!  walk,
//!  and a clone's origin is a snapshot of the checkout dataset rather than of
//!  the container,
//!  so it reads three names
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety",
//!  which names
//!  `/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc`,
//!  and "Dataset and path layout",
//!  where the checkout dataset and the container are siblings).
//!  The project answered on 2026-11-22 that everything works by default with no
//!  configuration file
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "With no configuration file"):
//!  an entry the file holds wins,
//!  and every entry it leaves out is found from this process's own mount table,
//!  where the checkout is the one mounted ZFS dataset whose last name component
//!  is `monochromatic-git-checkout` and the container is its sibling.
//!  A name that matches none,
//!  or several,
//!  is a refusal that lists what was found rather than a guess.
//!  The project also answered on 2026-11-21 that the start probe's two names sit
//!  inside one `startProbe` object
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  "Answers of 2026-11-21,
//! after the probe was built",
//!  answer "C:
//! nest under one key");
//!  the `cloneMounts` object beside it is this route's own extension,
//!  and the two checkouts it names are required to be one folder when a file
//!  names both,
//!  because a probe that passed for one checkout says nothing about a walk that
//!  starts at another.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const configuration = readConfiguration('/etc/…/git-worktree-clone-mounts.jsonc', liveMountedDatasets);
//! ```

use crate::jsonc_member_text::{MemberReading, text_of_member};
use super::sweep_configuration_refusal::ConfigurationRefusal;
use crate::start_probe::probe_source_name::check_dataset_name;
use crate::zfs_mounted_dataset::mounted_dataset_instance::{MountedDatasetInstance, MountedDatasets};
use crate::zfs_mounted_dataset::mounted_dataset_refusal::MountedDatasetRefusal;
use crate::zfs_mounted_dataset::the_designs_datasets_in_the_mount_table::{the_checkout_dataset_at, the_designs_container_beside, the_one_mount_of, the_one_mounted_checkout};
use std::path::{Path, PathBuf};

/// The object this step's names sit in.
pub const CLONE_MOUNTS_KEY: &str = crate::handler_configuration_keys::CLONE_MOUNTS_KEY;

/// The key naming the pinned clone parent dataset,
/// which is the container every managed clone is a child of.
pub const PARENT_KEY: &str = crate::handler_configuration_keys::PARENT_KEY;

/// The key naming the checkout the walk starts from.
pub const CHECKOUT_KEY: &str = crate::handler_configuration_keys::CHECKOUT_KEY;

/// The key naming the checkout dataset,
/// whose snapshots the design's add route clones.
pub const CHECKOUT_DATASET_KEY: &str = crate::handler_configuration_keys::CHECKOUT_DATASET_KEY;

/// The object the start probe's own names sit in,
/// which the sweep reads only to refuse two different checkouts.
pub const PROBE_OBJECT_KEY: &str = crate::handler_configuration_keys::START_PROBE_KEY;

/// The key inside `startProbe` naming the checkout the probe opens.
pub const PROBE_CHECKOUT_KEY: &str = crate::handler_configuration_keys::PROBE_CHECKOUT_KEY;

/// The document a file that is not there stands for,
/// so every entry is one the sweep finds rather than reads.
pub const NO_FILE_DOCUMENT: &str = "{}";

/// What:
///  Where the sweep's names are,
///  once the configuration and the mount table give them.
/// Why:
///  The sweep reads the checkout dataset to judge an origin,
///  enumerates the parent's children and walks from the checkout,
///  so they travel together and a caller cannot pass the parent of one
///  checkout with another checkout's path or the other's dataset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SweepConfiguration = { parentDataset: string; checkout: string; checkoutDataset: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SweepConfiguration {
    /// The pinned clone parent dataset.
    pub parent_dataset: String,
    /// The checkout whose `.claude/worktrees` folder holds the slots.
    pub checkout: PathBuf,
    /// The checkout dataset a clone's origin must be a snapshot of.
    pub checkout_dataset: String,
}

/// What:
///  One text member the configuration may carry.
/// Why:
///  Every entry is optional now,
///  so the same reading serves all of them and a value of the wrong type is
///  still a refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = optionalText(document, 'parent', path);
/// ```
///
/// # Errors
/// The document is not JSONC,
/// or the member is there and its value is not text.
fn optional_text(document: &str, object: &str, key: &str, path: &Path) -> Result<Option<String>, ConfigurationRefusal> {
    return match text_of_member(document, object, key) {
        MemberReading::Text(text) => Ok(Some(text)),
        MemberReading::Absent => Ok(None),
        MemberReading::DocumentNotJsonc => Err(ConfigurationRefusal::NotJsonc { path: path.to_path_buf() }),
        MemberReading::NotText => Err(ConfigurationRefusal::NotText { path: path.to_path_buf(), key: format!("{object}.{key}") }),
    };
}

/// What:
///  One dataset name the file wrote,
///  checked as a name the kernel takes.
/// Why:
///  A name a file holds is used as written,
///  and a name the kernel would refuse would fail much later in an ioctl.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dataset = namedDataset(text, 'pair', path);
/// ```
///
/// # Errors
/// The name is not one the kernel takes.
fn named_dataset(name: String, key: &str, path: &Path) -> Result<String, ConfigurationRefusal> {
    check_dataset_name(name.as_str()).map_err(|why| return ConfigurationRefusal::NameRefused { path: path.to_path_buf(), key: String::from(key), name: name.clone(), why })?;
    return Ok(name);
}

/// What:
///  The checkout path and dataset the file names,
///  in words,
///  where a discovery that refused is turned into a configuration refusal.
/// Why:
///  A refusal has to name the file a person edits,
///  and the lookup that refused names only what the table held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const found = discovered(theOneMountedCheckout(mounts), path);
/// ```
fn discovered<T>(found: Result<T, MountedDatasetRefusal>, path: &Path) -> Result<T, ConfigurationRefusal> {
    return found.map_err(|refusal| return ConfigurationRefusal::Discovery { path: path.to_path_buf(), refusal });
}

/// What:
///  The checkout the sweep walks and the dataset it is on.
/// Why:
///  A file entry wins,
///  and each entry it leaves out is found from the mount table rather than from
///  the other entry's guess:
///  a path alone takes its dataset from the mount the path is on,
///  a dataset alone takes its path from the one mount of it,
///  and neither takes both from the one mounted instance of the design's leaf.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const (checkout, dataset) = checkoutOf(pathEntry, datasetEntry, mounts, configurationPath);
/// ```
///
/// # Errors
/// The table could not be read,
/// or the name matched none or several.
fn checkout_of(
    path_entry: Option<PathBuf>,
    dataset_entry: Option<String>,
    container_entry: Option<&str>,
    mounts: &dyn MountedDatasets,
    path: &Path,
) -> Result<(PathBuf, String), ConfigurationRefusal> {
    return match (path_entry, dataset_entry) {
        (Some(checkout), Some(dataset)) => Ok((checkout, dataset)),
        (Some(checkout), None) => {
            let dataset: String = discovered(the_checkout_dataset_at(checkout.as_path(), mounts), path)?;
            Ok((checkout, dataset))
        }
        (None, Some(dataset)) => {
            let checkout: PathBuf = discovered(the_one_mount_of(dataset.as_str(), mounts), path)?;
            Ok((checkout, dataset))
        }
        (None, None) => {
            let one: MountedDatasetInstance = discovered(the_one_mounted_checkout(mounts, container_entry), path)?;
            Ok((one.mount_point, one.dataset))
        }
    };
}

/// What:
///  Read the sweep's three names out of one configuration document and one
///  mount table.
/// Why:
///  The document is parsed once and every name comes from the same bytes,
///  so a file changed between two reads cannot make them disagree,
///  and a file that is not there is read as one that names nothing,
/// which is
///  the project's answer of 2026-11-22.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const configuration = parseConfiguration(text, '/etc/…', liveMountedDatasets);
/// ```
///
/// # Errors
/// The document is not JSONC,
/// an entry is not text,
/// a name is not a dataset name,
/// a checkout path is not absolute,
/// a design name matched no dataset or several,
/// or the two checkout keys name different folders.
pub fn parse_configuration(document: &str, path: &Path, mounts: &dyn MountedDatasets) -> Result<SweepConfiguration, ConfigurationRefusal> {
    let dataset_entry: Option<String> = match optional_text(document, CLONE_MOUNTS_KEY, CHECKOUT_DATASET_KEY, path)? {
        Some(name) => Some(named_dataset(name, &format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_DATASET_KEY}"), path)?),
        None => None,
    };
    let path_entry: Option<PathBuf> = match optional_text(document, CLONE_MOUNTS_KEY, CHECKOUT_KEY, path)? {
        Some(text) => {
            let checkout: PathBuf = PathBuf::from(text.as_str());
            if !checkout.is_absolute() {
                return Err(ConfigurationRefusal::CheckoutNotAbsolute {
                    path: path.to_path_buf(),
                    key: format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_KEY}"),
                    text,
                });
            }
            Some(checkout)
        }
        None => None,
    };
    // The clone container this run uses is read before the table is, because a row inside it is no instance
    // of the checkout (the project's first answer of 2026-11-22).
    let parent_entry: Option<String> = match optional_text(document, CLONE_MOUNTS_KEY, PARENT_KEY, path)? {
        Some(name) => Some(named_dataset(name, &format!("{CLONE_MOUNTS_KEY}.{PARENT_KEY}"), path)?),
        None => None,
    };
    let probe_checkout_entry: Option<PathBuf> = match optional_text(document, PROBE_OBJECT_KEY, PROBE_CHECKOUT_KEY, path)? {
        Some(text) => {
            let guessed: PathBuf = PathBuf::from(text.as_str());
            if !guessed.is_absolute() {
                return Err(ConfigurationRefusal::CheckoutNotAbsolute {
                    path: path.to_path_buf(),
                    key: format!("{PROBE_OBJECT_KEY}.{PROBE_CHECKOUT_KEY}"),
                    text,
                });
            }
            Some(guessed)
        }
        None => None,
    };
    let (checkout, checkout_dataset): (PathBuf, String) = checkout_of(path_entry, dataset_entry, parent_entry.as_deref(), mounts, path)?;
    if let Some(probe) = probe_checkout_entry
        && probe != checkout
    {
        return Err(ConfigurationRefusal::CheckoutsDiffer { path: path.to_path_buf(), probe, sweep: checkout });
    }
    let parent_dataset: String = match parent_entry {
        Some(container) => container,
        None => discovered(the_designs_container_beside(checkout_dataset.as_str()), path)?,
    };
    return Ok(SweepConfiguration { parent_dataset, checkout, checkout_dataset });
}

/// What:
///  Read the configuration file,
/// if it is there,
///  and answer the names it holds and the ones the mount table fills in.
/// Why:
///  The release path takes the file the install put in place,
///  and a control hands in a fixture file,
///  so nothing here names the installed path itself;
///  a missing file is the answer "names nothing" rather than a refusal,
/// which is
///  what the project's answer of 2026-11-22 changed.
///  The file names the datasets root acts on,
///  so it is opened without following a link and refused unless this program's
///  own account owns it and no group or other write bit is set
///  (`crate::file_only_root_may_write`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const configuration = readConfiguration(path, liveMountedDatasets);
/// ```
///
/// # Errors
/// Everything `parse_configuration` refuses,
/// a file that is there and unreadable,
/// and a file that is not one this program alone may write.
pub fn read_configuration(path: &Path, mounts: &dyn MountedDatasets) -> Result<SweepConfiguration, ConfigurationRefusal> {
    let document: Option<String> = match crate::file_only_root_may_write::read_file_only_root_may_write(path) {
        // A file that is there and holds nothing is refused rather than read as a file that names nothing:
        // the project's answer of 2026-11-22 says an empty file is not the same as no file.
        Ok(Some(text)) if text.trim().is_empty() => return Err(ConfigurationRefusal::Empty { path: path.to_path_buf() }),
        Ok(text) => text,
        Err(refusal) => return Err(ConfigurationRefusal::NotUnderRootControl { path: path.to_path_buf(), reason: refusal.reason() }),
    };
    return parse_configuration(document.as_deref().unwrap_or(NO_FILE_DOCUMENT), path, mounts);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_configuration_tests.rs"]
mod tests;
