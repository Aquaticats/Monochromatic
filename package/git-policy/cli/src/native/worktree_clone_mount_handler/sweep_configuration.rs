//! What:
//!  The names the clone-mount sweep takes from the root-owned handler
//!  configuration:
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
//!  The project decided on 2026-11-22 that step 19's one-time setup writes the
//!  checkout dataset's and the container's names into that file,
//!  and that a file holding no such names is read through the design's own
//!  names:
//!  the checkout dataset is the mount the configured checkout path is on and
//!  must carry the design's leaf name,
//!  and the container is that dataset's sibling
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "The datasets the sweep acts on").
//!  Nobody edits the file by hand.
//!  The project also answered on 2026-11-21 that the start probe's two names sit
//!  inside one `startProbe` object
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  "Answers of 2026-11-21,
//! after the probe was built",
//!  answer "C:
//! nest under one key");
//!  the `cloneMounts` object beside it is this route's own extension,
//!  and the two checkouts it names are required to be one folder,
//!  because a probe that passed for one checkout says nothing about a walk that
//!  starts at another.
//!  A configuration that is absent,
//!  unreadable or names nothing the parse accepts is a sweep that attaches
//!  nothing and says which key to write,
//!  never a sweep that guesses a dataset or a checkout.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const configuration = readConfiguration('/etc/…/git-worktree-clone-mounts.jsonc', liveCheckoutMounts);
//! ```

use super::sweep_checkout_mount::{CheckoutMounts, DESIGN_CHECKOUT_DATASET_NAME, DESIGN_CONTAINER_DATASET_NAME};
use crate::jsonc_member_text::{MemberReading, text_of_member};
use crate::start_probe::probe_source_name::{NameRefusal, check_dataset_name};
use std::path::{Path, PathBuf};

/// The object this step's names sit in.
pub const CLONE_MOUNTS_KEY: &str = "cloneMounts";

/// The key naming the pinned clone parent dataset,
/// which is the container every managed clone is a child of.
pub const PARENT_KEY: &str = "parent";

/// The key naming the checkout the walk starts from.
pub const CHECKOUT_KEY: &str = "checkout";

/// The key naming the checkout dataset,
/// whose snapshots the design's add route clones.
pub const CHECKOUT_DATASET_KEY: &str = "checkoutDataset";

/// The object the start probe's own names sit in,
/// which the sweep reads only to refuse two different checkouts.
pub const PROBE_OBJECT_KEY: &str = "startProbe";

/// The key inside `startProbe` naming the checkout the probe opens.
pub const PROBE_CHECKOUT_KEY: &str = "checkout";

/// What:
///  Where the sweep's names are,
///  once the configuration gives them.
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
///  Why the configuration gave the sweep no names.
/// Why:
///  A missing file,
///  a file that does not parse,
///  a key another part has not written,
///  a value of the wrong type,
///  a name the kernel would refuse,
///  a checkout path that is not absolute,
///  a checkout that is not on a dataset,
///  a checkout dataset that is not the design's,
///  and two keys naming different checkouts are distinct things for a person
///  to look at,
///  and each names the key they hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigurationRefusal = { kind: 'missing' | 'unreadable' | 'not-jsonc' | 'absent' | 'not-text' | 'name-refused' | 'not-absolute' | 'checkout-mount' | 'checks-differ'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigurationRefusal {
    /// The configuration file is not there.
    Missing {
        /// The path that was read.
        path: PathBuf,
    },
    /// The configuration file could not be read.
    Unreadable {
        /// The path that was read.
        path: PathBuf,
        /// What the read answered.
        reason: String,
    },
    /// The configuration is not one JSONC document.
    NotJsonc {
        /// The path that was read.
        path: PathBuf,
    },
    /// The object or the key is not there.
    Absent {
        /// The path that was read.
        path: PathBuf,
        /// The key a person must write.
        key: String,
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
        /// The key the name is written at.
        key: String,
        /// The name as written.
        name: String,
        /// Why the name rule refused it.
        why: NameRefusal,
    },
    /// The checkout's text is not an absolute path.
    CheckoutNotAbsolute {
        /// The value as written.
        text: String,
    },
    /// The checkout's mount could not be read,
    /// so the design's own name decides nothing here.
    CheckoutMount {
        /// The path that was read.
        path: PathBuf,
        /// The key a person writes the dataset name at.
        key: String,
        /// What the mount's read answered.
        said: String,
    },
    /// The checkout is not on a dataset,
    /// so no design name can find the dataset it is on.
    CheckoutNotADataset {
        /// The checkout's path.
        checkout: PathBuf,
        /// The filesystem the checkout is on.
        found: String,
        /// The key a person writes the dataset name at.
        key: String,
    },
    /// The dataset the checkout is on does not carry the design's name.
    CheckoutIsNotTheDesignDataset {
        /// The dataset the checkout is on.
        found: String,
        /// The design's leaf name.
        wanted: &'static str,
        /// The key a person writes the dataset name at.
        key: String,
    },
    /// The dataset the design's sibling name would sit beside has no parent,
    /// so there is no name to build.
    NoParentForTheDesignName {
        /// The dataset whose parent was needed.
        dataset: String,
        /// The key a person writes the container's name at.
        key: String,
    },
    /// The probe's checkout and the sweep's checkout are different folders.
    CheckoutsDiffer {
        /// The probe's configured checkout.
        probe: PathBuf,
        /// The sweep's configured checkout.
        sweep: PathBuf,
    },
}

impl ConfigurationRefusal {
    /// The key one refusal is about,
    /// which is what a person has to write.
    pub fn key(&self) -> String {
        return match self {
            ConfigurationRefusal::Missing { .. } | ConfigurationRefusal::Unreadable { .. } | ConfigurationRefusal::NotJsonc { .. } => String::from(CLONE_MOUNTS_KEY),
            ConfigurationRefusal::Absent { key, .. }
            | ConfigurationRefusal::NotText { key, .. }
            | ConfigurationRefusal::NameRefused { key, .. }
            | ConfigurationRefusal::CheckoutMount { key, .. }
            | ConfigurationRefusal::CheckoutNotADataset { key, .. }
            | ConfigurationRefusal::CheckoutIsNotTheDesignDataset { key, .. }
            | ConfigurationRefusal::NoParentForTheDesignName { key, .. } => key.clone(),
            ConfigurationRefusal::CheckoutNotAbsolute { .. } => format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_KEY}"),
            ConfigurationRefusal::CheckoutsDiffer { .. } => format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_KEY}"),
        };
    }

    /// The refusal in words,
    /// naming the file,
    /// the key and what to write there.
    pub fn message(&self) -> String {
        let cause: String = match self {
            ConfigurationRefusal::Missing { path } => format!("the clone-mount configuration {} is not there", path.display()),
            ConfigurationRefusal::Unreadable { path, reason } => format!("the clone-mount configuration {} could not be read ({reason})", path.display()),
            ConfigurationRefusal::NotJsonc { path } => format!("the clone-mount configuration {} is not one JSONC document", path.display()),
            ConfigurationRefusal::Absent { path, key } => format!("the clone-mount configuration {} names no `{key}`", path.display()),
            ConfigurationRefusal::NotText { path, key } => format!("`{key}` in {} is not text", path.display()),
            ConfigurationRefusal::NameRefused { name, why, .. } => format!("`{name}` is not a dataset name the kernel takes: {}", why.message()),
            ConfigurationRefusal::CheckoutNotAbsolute { text } => {
                format!("`{text}` is not an absolute checkout path, and a relative path would be resolved against the handler's own directory")
            }
            ConfigurationRefusal::CheckoutMount { path, said, .. } => format!("the mount the checkout named in {} is on could not be read: {said}", path.display()),
            ConfigurationRefusal::CheckoutNotADataset { checkout, found, .. } => format!(
                "the checkout {} is on a `{found}` mount rather than on a dataset, and the design's own name only finds a dataset",
                checkout.display()
            ),
            ConfigurationRefusal::CheckoutIsNotTheDesignDataset { found, wanted, .. } => format!(
                "the checkout is on the dataset `{found}`, and the dataset this route finds by its own name is the one whose last component is `{wanted}`"
            ),
            ConfigurationRefusal::NoParentForTheDesignName { dataset, .. } => format!("`{dataset}` has no parent dataset, so `{DESIGN_CONTAINER_DATASET_NAME}` cannot be named beside it"),
            ConfigurationRefusal::CheckoutsDiffer { probe, sweep } => format!(
                "the start probe is configured for the checkout {} and the clone-mount sweep for {}, and one probe's pass says nothing about a walk that starts at another checkout",
                probe.display(),
                sweep.display()
            ),
        };
        return format!(
            "{cause}, so this sweep enumerated nothing, attached nothing and recorded nothing, and every clone's mount intent stands untouched. Ways forward: write the key as text in the root-owned file, with `{CLONE_MOUNTS_KEY}.{PARENT_KEY}` naming the clone parent dataset such as `zpcachyos/ROOT/cos/home/{DESIGN_CONTAINER_DATASET_NAME}`, `{CLONE_MOUNTS_KEY}.{CHECKOUT_KEY}` naming the checkout such as `/home/user/Monochromatic`, and `{CLONE_MOUNTS_KEY}.{CHECKOUT_DATASET_KEY}` naming the checkout dataset such as `zpcachyos/ROOT/cos/home/{DESIGN_CHECKOUT_DATASET_NAME}`; a file holding no dataset names is read through those two design names, and `doctor` reports `{key}` once plan step 18 adds it.",
            key = self.key()
        );
    }
}

/// What:
///  One text member the configuration must carry.
/// Why:
///  The keys are read the same way,
///  so a missing one and a value of the wrong type are worded once rather than
///  twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = requiredText(document, 'checkout', path);
/// ```
fn required_text(document: &str, key: &str, path: &Path) -> Result<String, ConfigurationRefusal> {
    return match text_of_member(document, CLONE_MOUNTS_KEY, key) {
        MemberReading::Text(text) => Ok(text),
        MemberReading::DocumentNotJsonc => Err(ConfigurationRefusal::NotJsonc { path: path.to_path_buf() }),
        MemberReading::Absent => Err(ConfigurationRefusal::Absent { path: path.to_path_buf(), key: format!("{CLONE_MOUNTS_KEY}.{key}") }),
        MemberReading::NotText => Err(ConfigurationRefusal::NotText { path: path.to_path_buf(), key: format!("{CLONE_MOUNTS_KEY}.{key}") }),
    };
}

/// What:
///  One text member the configuration may carry,
///  answering nothing when the object or the member is absent.
/// Why:
///  The two dataset names are the ones step 19's setup writes and this build
///  also finds by the design's names,
///  so their absence is a fallback rather than a refusal,
///  while a value of the wrong type is still a refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = optionalText(document, 'parent', path);
/// ```
///
/// # Errors
/// The document is not JSONC,
/// or the member is there and its value is not text.
fn optional_text(document: &str, key: &str, path: &Path) -> Result<Option<String>, ConfigurationRefusal> {
    return match text_of_member(document, CLONE_MOUNTS_KEY, key) {
        MemberReading::Text(text) => Ok(Some(text)),
        MemberReading::Absent => Ok(None),
        MemberReading::DocumentNotJsonc => Err(ConfigurationRefusal::NotJsonc { path: path.to_path_buf() }),
        MemberReading::NotText => Err(ConfigurationRefusal::NotText { path: path.to_path_buf(), key: format!("{CLONE_MOUNTS_KEY}.{key}") }),
    };
}

/// The key one dataset name is written at,
/// as a refusal names it.
fn dataset_key(key: &str) -> String {
    return format!("{CLONE_MOUNTS_KEY}.{key}");
}

/// The last `/`-separated component of a dataset name.
fn leaf_of(dataset: &str) -> &str {
    return dataset.rsplit_once('/').map_or(dataset, |(_parent, leaf)| return leaf);
}

/// What:
///  The dataset the configured checkout is on,
///  found by the design's own name.
/// Why:
///  A file that holds no dataset names is read through the layout the design
///  fixes,
///  and the checkout path is the one live fact that says which dataset that
///  layout put there;
///  a checkout on another dataset,
/// or on no dataset,
/// is refused with the key a
///  person writes rather than guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dataset = findTheCheckoutDataset(checkout, mounts, path);
/// ```
///
/// # Errors
/// The mount could not be read,
/// it is no dataset mount,
/// or the dataset does not carry the design's leaf name.
fn find_the_checkout_dataset(checkout: &Path, mounts: &dyn CheckoutMounts, path: &Path) -> Result<String, ConfigurationRefusal> {
    let key: String = dataset_key(CHECKOUT_DATASET_KEY);
    let mount = mounts
        .mount_holding(checkout)
        .map_err(|refusal| return ConfigurationRefusal::CheckoutMount { path: path.to_path_buf(), key: key.clone(), said: refusal.message() })?;
    let Some(dataset) = mount.dataset() else {
        return Err(ConfigurationRefusal::CheckoutNotADataset { checkout: checkout.to_path_buf(), found: mount.filesystem_type, key });
    };
    if leaf_of(dataset) != DESIGN_CHECKOUT_DATASET_NAME {
        return Err(ConfigurationRefusal::CheckoutIsNotTheDesignDataset { found: String::from(dataset), wanted: DESIGN_CHECKOUT_DATASET_NAME, key });
    }
    return Ok(String::from(dataset));
}

/// What:
///  The container the design places beside the checkout dataset.
/// Why:
///  The two are siblings in the design's layout,
///  so a file holding no container name gets the one the layout fixes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const container = `${parentOf(checkoutDataset)}/monochromatic-git-worktree-clone-datasets`;
/// ```
///
/// # Errors
/// The checkout dataset has no parent component.
fn the_designs_container_beside(checkout_dataset: &str) -> Result<String, ConfigurationRefusal> {
    let Some((parent, _leaf)) = checkout_dataset.rsplit_once('/') else {
        return Err(ConfigurationRefusal::NoParentForTheDesignName { dataset: String::from(checkout_dataset), key: dataset_key(PARENT_KEY) });
    };
    return Ok(format!("{parent}/{DESIGN_CONTAINER_DATASET_NAME}"));
}

/// What:
///  Read the sweep's three names out of one configuration document.
/// Why:
///  The document is parsed once and every name comes from the same bytes,
///  so a file changed between two reads cannot make them disagree,
///  and the two dataset names fall back to the design's layout only when the
///  file holds neither.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const configuration = parseConfiguration(text, '/etc/…', liveCheckoutMounts);
/// ```
///
/// # Errors
/// The document is not JSONC,
/// a required key is absent or is not text,
/// a name is not a dataset name,
/// the checkout is not an absolute path,
/// the checkout's mount cannot be read or is not the design's dataset,
/// or the two checkout keys name different folders.
pub fn parse_configuration(document: &str, path: &Path, mounts: &dyn CheckoutMounts) -> Result<SweepConfiguration, ConfigurationRefusal> {
    let checkout_text: String = required_text(document, CHECKOUT_KEY, path)?;
    let checkout: PathBuf = PathBuf::from(checkout_text.as_str());
    if !checkout.is_absolute() {
        return Err(ConfigurationRefusal::CheckoutNotAbsolute { text: checkout_text });
    }
    let probe_checkout: PathBuf = match text_of_member(document, PROBE_OBJECT_KEY, PROBE_CHECKOUT_KEY) {
        MemberReading::Text(text) => PathBuf::from(text),
        MemberReading::Absent | MemberReading::NotText => checkout.clone(),
        MemberReading::DocumentNotJsonc => return Err(ConfigurationRefusal::NotJsonc { path: path.to_path_buf() }),
    };
    if probe_checkout != checkout {
        return Err(ConfigurationRefusal::CheckoutsDiffer { probe: probe_checkout, sweep: checkout });
    }
    let checkout_dataset: String = match optional_text(document, CHECKOUT_DATASET_KEY, path)? {
        Some(name) => {
            check_dataset_name(name.as_str()).map_err(|why| return ConfigurationRefusal::NameRefused { key: dataset_key(CHECKOUT_DATASET_KEY), name: name.clone(), why })?;
            name
        }
        None => find_the_checkout_dataset(checkout.as_path(), mounts, path)?,
    };
    let parent_dataset: String = match optional_text(document, PARENT_KEY, path)? {
        Some(name) => {
            check_dataset_name(name.as_str()).map_err(|why| return ConfigurationRefusal::NameRefused { key: dataset_key(PARENT_KEY), name: name.clone(), why })?;
            name
        }
        None => the_designs_container_beside(checkout_dataset.as_str())?,
    };
    return Ok(SweepConfiguration { parent_dataset, checkout, checkout_dataset });
}

/// What:
///  Read the configuration file and answer the names it holds.
/// Why:
///  The release path takes the file the install put in place,
///  and a control hands in a fixture file,
///  so nothing here names the installed path itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const configuration = readConfiguration(path, liveCheckoutMounts);
/// ```
///
/// # Errors
/// Everything `parse_configuration` refuses,
/// and a file that is absent or unreadable.
pub fn read_configuration(path: &Path, mounts: &dyn CheckoutMounts) -> Result<SweepConfiguration, ConfigurationRefusal> {
    let document: String = std::fs::read_to_string(path).map_err(|error| {
        return match error.kind() {
            std::io::ErrorKind::NotFound => ConfigurationRefusal::Missing { path: path.to_path_buf() },
            _ => ConfigurationRefusal::Unreadable { path: path.to_path_buf(), reason: error.to_string() },
        };
    })?;
    return parse_configuration(document.as_str(), path, mounts);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_configuration_tests.rs"]
mod tests;
