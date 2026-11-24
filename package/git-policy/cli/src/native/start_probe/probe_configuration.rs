//! What:
//!  The two names the probe acts on:
//!  the dataset whose `ready` snapshot is the source,
//!  and the checkout the walk opens read-only.
//! Why:
//!  The project answered on 2026-11-22 that a missing configuration file no
//!  longer leaves the route blocked
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  "Answer of 2026-11-22:
//! the probe needs no configuration file"):
//!  with no file,
//!  or with a file that holds no `startProbe.dataset` or `startProbe.checkout`,
//!  the probe finds its dataset by the name its design fixes,
//!  `nosnap/git-worktree-probe` in the pool that holds the checkout,
//!  and its checkout as the sweep finds one
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The source:
//! the probe dataset").
//!  A file that is present wins,
//!  and a file that is there and unreadable,
//!  unparsable or of the wrong type is still a refusal,
//!  because it won a precedence it cannot fill.
//!  The project answered on 2026-11-21 that both names sit inside one
//!  `startProbe` object,
//!  as `dataset` and `checkout`,
//!  so the root-owned configuration holds one object rather than two flat
//!  keys.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const checkout = checkoutPath(textOrNone, configurationPath, liveMountedDatasets);
//! const source = sourceName(textOrNone, configurationPath, liveMountedDatasets, checkout);
//! ```

use super::probe_configuration_refusal::{CheckoutRefusal, SourceRefusal};
use super::probe_source_name::{ProbeSource, check_dataset_name};
use crate::jsonc_member_text::{MemberReading, text_of_member};
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasets;
use crate::zfs_mounted_dataset::the_designs_datasets_in_the_mount_table::{the_checkout_dataset_at, the_designs_probe_dataset_under, the_one_mounted_checkout};
use std::path::{Path, PathBuf};

/// The one object the project answered holds both names.
pub const START_PROBE_KEY: &str = crate::handler_configuration_keys::START_PROBE_KEY;

/// The key inside `startProbe` that names the probe dataset,
/// beside the pool and dataset names the configuration pins.
pub const DATASET_KEY: &str = crate::handler_configuration_keys::PROBE_DATASET_KEY;

/// The key inside `startProbe` that names the checkout the walk opens
/// read-only.
pub const CHECKOUT_KEY: &str = crate::handler_configuration_keys::PROBE_CHECKOUT_KEY;

/// The object the sweep's names sit in,
/// whose clone-container key the probe reads too,
/// so that a clone in the container this run uses is no instance of the
/// checkout for either program
/// (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
/// "Four further answers on the look-up with no configuration file").
pub const CLONE_MOUNTS_KEY: &str = crate::handler_configuration_keys::CLONE_MOUNTS_KEY;

/// The key naming the clone container this run uses.
pub const PARENT_KEY: &str = crate::handler_configuration_keys::PARENT_KEY;

/// What:
///  One configuration document's text value at one key inside `startProbe`.
/// Why:
///  Both keys are one JSONC member,
///  and the parse and the key lookup are the repository's own single reader of
///  that file,
///  so a missing file and a missing key cannot be worded twice and the two
///  readers of the configuration cannot drift
///  (`package/git-policy/cli/src/native/foundation/jsonc_member_text.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = textAtKey(document, 'dataset'); // undefined when absent or not text
/// ```
fn text_at_key(configuration: &str, key: &str) -> MemberReading {
    return text_of_member(configuration, START_PROBE_KEY, key);
}

/// What:
///  The clone container this run uses,
///  which a row inside is no instance of the checkout for.
/// Why:
///  The project's first answer of 2026-11-22 has the probe read the sweep's
///  container key as well as its own object:
///  a clone the account makes inside the container this run uses must not count
///  as a second mounted checkout.
///  The probe reads it only to know which container to ignore;
///  a value that is not text names no container,
///  and the sweep refuses that file itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const container = textOfMember(document, 'cloneMounts', 'parent') ?? undefined;
/// ```
fn clone_container_in_use(document: &str) -> Option<String> {
    return match text_of_member(document, CLONE_MOUNTS_KEY, PARENT_KEY) {
        MemberReading::Text(container) => Some(container),
        MemberReading::Absent | MemberReading::NotText | MemberReading::DocumentNotJsonc => None,
    };
}

/// What:
///  Read the configuration file and answer its text,
///  or nothing when it is not there.
/// Why:
///   The file is read once,
///  so both keys come from the same bytes and a file changed between the two
///  reads cannot make them disagree,
///  and a file that is not there is no longer a refusal.
///  The probe runs as root and the file names the datasets the route acts on,
///  so it is opened without following a link and refused unless this program's
///  own account owns it and no group or other write bit is set
///  (`crate::file_only_root_may_write`).
///
/// # Errors
/// The file is there and unreadable,
/// or it is not one this program alone may write.
pub fn read_configuration(path: &Path) -> Result<Option<String>, SourceRefusal> {
    return match crate::file_only_root_may_write::read_file_only_root_may_write(path) {
        // A file that is there and holds nothing is refused rather than read as a file that names nothing:
        // the project's answer of 2026-11-22 says an empty file is not the same as no file.
        Ok(Some(text)) if text.trim().is_empty() => Err(SourceRefusal::ConfigurationEmpty { path: path.to_path_buf() }),
        Ok(text) => Ok(text),
        Err(refusal) => Err(SourceRefusal::ConfigurationNotUnderRootControl { path: path.to_path_buf(), reason: refusal.reason() }),
    };
}

/// What:
///  The checkout the walk opens read-only:
///  the file's `startProbe.checkout` when it holds one,
///  and otherwise the one mounted instance of the design's checkout dataset.
/// Why:
///   The probe opens this folder and nothing else,
///  and a file that names it wins over what the table would be searched for,
///  while a table that matches none or several is refused rather than guessed
///  at.
///
/// # Errors
/// The file is not JSONC,
/// the key is not text,
/// a path is not absolute,
/// or the design's name matched none or several.
pub fn checkout_path(document: Option<&str>, mounts: &dyn MountedDatasets) -> Result<PathBuf, CheckoutRefusal> {
    let Some(held) = document else {
        return the_one_mounted_checkout(mounts, None).map(|one| return one.mount_point).map_err(|refusal| return CheckoutRefusal::NotFound { refusal });
    };
    if held.trim().is_empty() {
        return Err(CheckoutRefusal::Empty);
    }
    return match text_at_key(held, CHECKOUT_KEY) {
        MemberReading::Text(text) => {
            let checkout: PathBuf = PathBuf::from(text);
            if checkout.is_absolute() {
                Ok(checkout)
            } else {
                Err(CheckoutRefusal::NotAbsolute)
            }
        }
        MemberReading::Absent => {
            let container: Option<String> = clone_container_in_use(held);
            the_one_mounted_checkout(mounts, container.as_deref())
                .map(|one| return one.mount_point)
                .map_err(|refusal| return CheckoutRefusal::NotFound { refusal })
        }
        MemberReading::NotText => Err(CheckoutRefusal::NotText),
        MemberReading::DocumentNotJsonc => Err(CheckoutRefusal::Unparsable),
    };
}

/// What:
///  The source the probe attaches:
///  the file's `startProbe.dataset` when it holds one,
///  and otherwise the design's own name in the pool that holds the checkout.
/// Why:
///   The worker reads no property and no directory at start,
///  so this is the whole of the probe's knowledge about its source before the
///  attach,
///  and the pool is the only part of the design's name the checkout supplies.
///
/// # Errors
/// The document is not JSONC,
/// the key is not text,
/// the text is not a dataset name,
/// or the checkout's dataset and pool could not be told.
pub fn source_name(document: Option<&str>, path: &Path, mounts: &dyn MountedDatasets, checkout: Option<&Path>) -> Result<ProbeSource, SourceRefusal> {
    let reading: MemberReading = match document {
        Some(text) => text_at_key(text, DATASET_KEY),
        None => MemberReading::Absent,
    };
    return match reading {
        MemberReading::Text(name) => {
            if let Err(why) = check_dataset_name(name.as_str()) {
                return Err(SourceRefusal::NameRefused { name, why });
            }
            Ok(ProbeSource { dataset: name })
        }
        MemberReading::Absent => found_source(mounts, checkout, path),
        MemberReading::NotText => Err(SourceRefusal::NameNotText { path: path.to_path_buf() }),
        MemberReading::DocumentNotJsonc => Err(SourceRefusal::ConfigurationUnparsable { path: path.to_path_buf(), reason: String::from("the document did not parse") }),
    };
}

/// What:
///  The design's probe dataset for a checkout whose dataset the file did not
///  name.
/// Why:
///   The design's name fixes the path below the pool,
///  so the checkout's dataset is what supplies the pool,
///  and a checkout on no dataset of the design's leaf name is refused with the
///  entry that settles it.
///
/// # Errors
/// The checkout's dataset could not be told,
/// or its name has no pool component.
fn found_source(mounts: &dyn MountedDatasets, checkout: Option<&Path>, path: &Path) -> Result<ProbeSource, SourceRefusal> {
    let Some(told) = checkout else {
        return Err(SourceRefusal::CheckoutNotFound { path: path.to_path_buf() });
    };
    let dataset: String = the_checkout_dataset_at(told, mounts).map_err(|refusal| return SourceRefusal::Checkout { refusal })?;
    let probe: String = the_designs_probe_dataset_under(dataset.as_str()).map_err(|refusal| return SourceRefusal::Checkout { refusal })?;
    return Ok(ProbeSource { dataset: probe });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_configuration_tests.rs"]
mod tests;
