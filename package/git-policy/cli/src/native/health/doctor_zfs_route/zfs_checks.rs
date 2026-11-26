//! What:
//!  The checks plan step 18 runs before anything else:
//!  `/dev/zfs` and the account's open of it,
//!  the loaded module's layout against the request contract,
//!  the pool the checkout dataset names,
//!  the checkout dataset and the clone container with their decided
//!  properties,
//!  each dataset's encryption key,
//!  and every clone's attach properties.
//! Why:
//!  The design's preflight reads these facts before a claim, slot or clone is
//!  written,
//!  and `doctor` reports the same facts without acting on them
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints" and
//!  "Add and Git registration",
//!  step 1;
//!  `doc/planning/cli-git-zfs-runtime-layout-checker-design.md`,
//!  whose verdict the ABI check reads).
//!  Every value here is a reading,
//!  so a dataset that is not there,
//!  a dataset this account could not read and a dataset whose property moved
//!  are three different answers:
//!  the first two decide between a finding and unknown,
//!  and the third is always a finding naming the property.
//!  Nothing here opens the device:
//!  the caller hands in what it read,
//!  so a control answers without this machine's kernel files.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = { role: DatasetRole.Container, name, reading: Reading.Read(cloneObjectFacts) };
//! const outcome = checkDataset(facts); // { kind: 'pass' | 'finding' | 'unknown' }
//! ```

use super::check_outcome::{
    Check,
    CheckOutcome,
    CheckReport,
    finding,
    pass,
    unknown,
};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{
    CloneObjectFacts,
    MOUNTPOINT_PROPERTY,
    PropertyValue,
    value_of,
};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::ReadFailure;
use crate::worktree_clone_mount_protocol::local_user_property::{
    PropertyReading,
    read_local_property,
};
use std::path::PathBuf;

//region Limits the kernel and the design fix

/// The error number the kernel answers for an object that is not there.
///
/// Why:
///  A dataset whose name is gone is a finding a person can act on,
///  while a read that failed for any other reason proves nothing and stays
///  unknown,
///  so the one number that separates them is named here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NOT_FOUND = 2; // ENOENT
/// ```
pub const NOT_FOUND_ERRNO: i32 = 2;

/// The object-set type a filesystem has,
/// from the sweep's own reading of the same field.
pub const FILESYSTEM_OBJECT_SET: u32 =
    crate::worktree_clone_mount_handler::sweep_dataset_facts::DMU_OBJECT_SET_FILESYSTEM;

/// The `mountpoint` of a dataset nothing mounts automatically.
pub const LEGACY_MOUNTPOINT: &str = "legacy";

/// The `canmount` value that keeps a dataset from an automatic mount.
pub const CANMOUNT_OFF: u64 = 0;

/// The `setuid` and `devices` value that removes the capability from a mount.
pub const FLAG_OFF: u64 = 0;

/// The `keystatus` value that means the key is not loaded.
pub const KEY_UNAVAILABLE: u64 =
    crate::worktree_clone_mount_handler::sweep_dataset_facts::KEY_STATUS_UNAVAILABLE;

/// The `encryption` value of a dataset that is not encrypted.
pub const ENCRYPTION_OFF: &str = "off";

/// The property the clone container's automatic mounting is read from.
pub const CANMOUNT_PROPERTY: &str = "canmount";

//endregion

//region The reading of one dataset

/// What:
///  What a read of one dataset answered:
///  the facts,
///  that the dataset is not there,
///  or that this caller could not read it.
/// Why:
///  The three answers decide different checks:
///  a dataset that is gone is a finding,
///  a dataset this account may not read is unknown,
///  and only the facts let a property be judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reading<T> = { kind: 'read'; value: T } | { kind: 'missing'; why: string } | { kind: 'unreadable'; why: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Reading<T> {
    /// The dataset answered with its facts.
    Read(T),
    /// The dataset is not in the pool.
    Missing {
        /// What the read answered.
        why: String,
    },
    /// The caller could not read the dataset.
    Unreadable {
        /// What the read answered.
        why: String,
    },
}

/// What:
///  Whether a failed dataset read means the dataset is gone or that it could
///  not be read.
/// Why:
///  The kernel's `ENOENT` is the one answer that proves absence,
///  and a read with no error number proves nothing about the dataset either
///  way,
///  so the rule lives here rather than at each call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = read.errno === 2 ? { kind: 'missing', why: read.said } : { kind: 'unreadable', why: read.said };
/// ```
pub fn reading_of<T>(read: Result<T, ReadFailure>) -> Reading<T> {
    return match read {
        Ok(value) => Reading::Read(value),
        Err(failure) if failure.errno == Some(NOT_FOUND_ERRNO) => Reading::Missing {
            why: failure.said,
        },
        Err(failure) => Reading::Unreadable {
            why: failure.said,
        },
    };
}

//endregion

//region `/dev/zfs`

/// What:
///  What the device path said when the caller looked at it.
/// Why:
///  A device that is not there,
///  a device this account may not open and a device whose state could not be
///  read send a person to three different places,
///  so the three states are kept apart before any judgement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DeviceState = { kind: 'opens' } | { kind: 'denied'; why } | { kind: 'absent' } | { kind: 'unreadable'; why };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeviceState {
    /// The caller opened the device.
    Opens,
    /// The path is there and the open refused.
    Denied {
        /// What the open answered.
        why: String,
    },
    /// The path is not there.
    Absent,
    /// The path's own state could not be read.
    Unreadable {
        /// What the look answered.
        why: String,
    },
}

/// What:
///  The device the route reads the pool through,
///  and what the caller's look at it answered.
/// Why:
///  One value carries the path so a finding names the device a person must
///  check,
///  rather than a word like "the device".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DeviceFacts = { path: string; state: DeviceState };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceFacts {
    /// The device path,
    /// which is `/dev/zfs` for this route.
    pub path: PathBuf,
    /// What the look and the open answered.
    pub state: DeviceState,
}

/// What:
///  Whether the route's device is there and this account may open it.
/// Why:
///  Every other pool read stands on this one,
///  and the design's preflight asks for the account's own access rather than
///  root's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkDevice({ path: '/dev/zfs', state }); // pass | finding | unknown
/// ```
pub fn check_device(facts: &DeviceFacts) -> CheckOutcome {
    let path: String = facts.path.display().to_string();
    return match &facts.state {
        DeviceState::Opens => pass(format!("{path} is there and this account opened it")),
        DeviceState::Absent => finding(
            "device",
            format!("{path} is not there, so no pool read can run on this machine"),
        ),
        DeviceState::Denied {
            why,
        } => finding(
            "open",
            format!(
                "{path} is there and this account's open refused ({why}); the route needs the account to open it, not only root"
            ),
        ),
        DeviceState::Unreadable {
            why,
        } => unknown(format!(
            "{path} could not be looked at ({why}), so its presence and the account's open are unproved"
        )),
    };
}

//endregion

//region The ABI verdict

/// What:
///  What the start-time layout gate answered,
///  in the three states a report keeps apart.
/// Why:
///  The gate's own two outcomes are a compatible layout and a refusal,
///  and a refusal at the device or identity step proves nothing about the
///  layout,
///  so that case is unknown here rather than a second finding about a device
///  the device check already names
///  (`doc/decision/cli-git-zfs-runtime-layout-checker.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AbiFacts = { kind: 'compatible'; module?: string; details: string[] } | { kind: 'refused'; words } | { kind: 'unreadable'; why };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbiFacts {
    /// The loaded module's declaration matches the request contract.
    Compatible {
        /// The module's release text,
        /// which is reported and never compared with a rule.
        module: Option<String>,
        /// The files and sizes the gate compared.
        details: Vec<String>,
    },
    /// The layout comparison refused.
    Refused {
        /// The refusal in the words the gate prints.
        words: String,
    },
    /// The gate could not compare the layout.
    Unreadable {
        /// Why no comparison happened.
        why: String,
    },
}

/// What:
///  Whether the loaded module's layout matches the contract this build writes
///  requests from.
/// Why:
///  A mismatch disables every raw request,
///  reads included,
///  so a person must see it before anything else about the pool,
///  and the release text is reported beside the verdict because nothing
///  compares it with a rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkAbi({ kind: 'refused', words }); // finding: module-layout
/// ```
pub fn check_abi(facts: &AbiFacts) -> CheckOutcome {
    return match facts {
        AbiFacts::Compatible {
            module,
            details,
        } => {
            let release: String = module
                .as_ref()
                .map_or(String::from("no release text"), |text| {
                    return format!("release {text}");
                });
            pass(format!(
                "the loaded module's layout matches this build's request contract ({release}; {})",
                details.join(", ")
            ))
        }
        AbiFacts::Refused {
            words,
        } => finding(
            "module-layout",
            format!(
                "{words}; every raw request stays disabled, reads included, and no flag or environment turns this into a pass"
            ),
        ),
        AbiFacts::Unreadable {
            why,
        } => unknown(format!(
            "the layout gate ran no comparison ({why}), so the module's declaration is unproved"
        )),
    };
}

//endregion

//region The pool

/// What:
///  The pool the checkout dataset names,
///  and what a read of the pool itself answered.
/// Why:
///  A pool name in a configuration can outlive the pool,
///  and the design's preflight asks whether the pool is there before any
///  dataset below it is judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PoolFacts = { pool: string; reading: Reading<number> };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolFacts {
    /// The pool's name.
    pub pool: String,
    /// The pool's own identity,
    /// or why it was not read.
    pub reading: Reading<u64>,
}

/// What:
///  Whether the pool named by the checkout dataset answers.
/// Why:
///  A missing pool and an unreadable one are different for a person:
///  the first says an import or a name is wrong,
///  the second that this run could not prove the pool either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkPool({ pool, reading }); // pass | finding('pool') | unknown
/// ```
pub fn check_pool(facts: &PoolFacts) -> CheckOutcome {
    let pool: &str = facts.pool.as_str();
    return match &facts.reading {
        Reading::Read(guid) => pass(format!("pool {pool} answers with GUID {guid}")),
        Reading::Missing {
            why,
        } => finding(
            "pool",
            format!(
                "pool {pool} is not there ({why}); import it or correct the checkout dataset's name"
            ),
        ),
        Reading::Unreadable {
            why,
        } => unknown(format!(
            "pool {pool} could not be read ({why}), so its presence is unproved"
        )),
    };
}

//endregion

//region The datasets and their properties

/// What:
///  Which dataset of the route a fact belongs to,
///  which decides the properties it must carry.
/// Why:
///  The checkout is mounted at its path,
///  the clone container is a legacy dataset nothing mounts automatically,
///  and each clone is attached by descriptor under the container's
///  properties,
///  so one property rule per role is what the checks compare against
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Dataset and path layout" and
///  "Scoped delegation").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DatasetRole = { kind: 'checkout'; mountPoint: string } | { kind: 'container' } | { kind: 'clone' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DatasetRole {
    /// The dataset the checkout sits on,
    /// mounted at the checkout path.
    Checkout {
        /// The path the checkout is mounted at.
        mount_point: PathBuf,
    },
    /// The unmounted container every managed clone is an immediate child of.
    Container,
    /// One managed clone below the container.
    Clone,
}

/// What:
///  One dataset of the route,
///  its role and what a read of it answered.
/// Why:
///  Every property check reads the same facts,
///  so the role and the reading travel together and a finding names both the
///  dataset and the property it failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DatasetFacts = { role: DatasetRole; name: string; reading: Reading<CloneObjectFacts> };
/// ```
#[derive(Clone, Debug)]
pub struct DatasetFacts {
    /// Which dataset of the route this is.
    pub role: DatasetRole,
    /// The dataset's full name.
    pub name: String,
    /// The facts the kernel answered,
    /// or why it answered none.
    pub reading: Reading<CloneObjectFacts>,
}

impl DatasetFacts {
    /// The check this dataset's facts answer.
    pub fn check(&self) -> Check {
        return match self.role {
            DatasetRole::Checkout {
                ..
            } => Check::CheckoutDataset,
            DatasetRole::Container => Check::ContainerDataset,
            DatasetRole::Clone => Check::CloneProperties,
        };
    }
}

/// What:
///  One dataset's properties against the state the route attaches it in,
///  as a finding or nothing.
/// Why:
///  The checkout, the container and each clone hold the same facts in a
///  different shape,
///  so one comparison answers all three and a new property is one arm rather
///  than three.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failure = property_failure(role, facts); // { field, detail } | undefined
/// ```
fn property_failure(
    role: &DatasetRole,
    facts: &CloneObjectFacts,
) -> Option<(String, String)> {
    if facts.is_snapshot || facts.object_type != FILESYSTEM_OBJECT_SET {
        return Some((
            String::from("object-type"),
            format!(
                "`{}` is not a filesystem (type {}; snapshot: {}), and this route acts on filesystems only",
                facts.name, facts.object_type, facts.is_snapshot
            ),
        ));
    }
    match role {
        DatasetRole::Checkout {
            mount_point,
        } => {
            let expected: String = mount_point.display().to_string();
            let found: String = facts.mountpoint.describe("mountpoint");
            if facts.mountpoint.text() != Some(expected.as_str()) {
                return Some((
                    String::from("mountpoint"),
                    format!(
                        "`{}`: {found}, and the checkout is mounted at {expected}",
                        facts.name
                    ),
                ));
            }
            return None;
        }
        DatasetRole::Container => return container_failure(facts),
        DatasetRole::Clone => return clone_failure(facts),
    }
}

/// What:
///  The container's properties:
///  `legacy` set on the container itself,
///  unmountable automatically,
///  and no set-user-ID or device nodes in a mount of it.
/// Why:
///  A clone inherits `mountpoint` from its own parent,
///  and OpenZFS names the nearest dataset that set the value locally as the source,
///  so a container that inherits `legacy` from an ancestor hands each clone
///  `inherited from <the ancestor>` rather than `inherited from <the container>`,
///  which the read after the clone request refuses
///  (`worktree_clone_add_route::add_zfs::the_clones_mountpoint_failure`,
///  `doc/decision/cli-git-zfs-clone-inherits-legacy-from-the-container.md`).
///  The route's two checks are therefore the same judgement of the same container,
///  and this one stands before any write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = readLocalProperty(facts.properties, facts.name, 'mountpoint');
/// if (source.kind !== 'local') return { field: 'mountpoint', detail: `... must set 'legacy' itself` };
/// ```
fn container_failure(facts: &CloneObjectFacts) -> Option<(String, String)> {
    if facts.mountpoint.text() != Some(LEGACY_MOUNTPOINT) {
        return Some((
            String::from("mountpoint"),
            format!(
                "`{}`: {}, and the clone container must be `{LEGACY_MOUNTPOINT}`",
                facts.name,
                facts.mountpoint.describe("mountpoint")
            ),
        ));
    }
    let source: PropertyReading<'_> = read_local_property(
        Some(&facts.properties),
        facts.name.as_str(),
        MOUNTPOINT_PROPERTY,
    );
    match &source {
        PropertyReading::Local {
            ..
        } => {}
        PropertyReading::Inherited {
            from,
        } => {
            return Some((
                String::from("mountpoint"),
                format!(
                    "`{}`: {}, inherited from `{from}`, and the clone container must set `{LEGACY_MOUNTPOINT}` on itself, because each clone inherits the value from its own parent and not from an ancestor above it",
                    facts.name,
                    facts.mountpoint.describe(MOUNTPOINT_PROPERTY)
                ),
            ));
        }
        PropertyReading::Absent => {
            return Some((
                String::from("mountpoint"),
                format!(
                    "`{}`: the reply carries no source for `{MOUNTPOINT_PROPERTY}`, so nothing proves the container set `{LEGACY_MOUNTPOINT}` on itself",
                    facts.name
                ),
            ));
        }
        PropertyReading::Malformed(refusal) => {
            return Some((
                String::from("mountpoint"),
                refusal.message(MOUNTPOINT_PROPERTY),
            ));
        }
    }
    let canmount: PropertyValue = value_of(&facts.properties, CANMOUNT_PROPERTY);
    if canmount.number() != Some(CANMOUNT_OFF) {
        return Some((
            String::from(CANMOUNT_PROPERTY),
            format!(
                "`{}`: {}, and the container must be off so no clone mounts automatically",
                facts.name,
                canmount.describe(CANMOUNT_PROPERTY)
            ),
        ));
    }
    return flag_failure(facts);
}

/// A clone's properties: legacy inherited from the container, and no
/// set-user-ID or device nodes in its mount.
fn clone_failure(facts: &CloneObjectFacts) -> Option<(String, String)> {
    if facts.mountpoint.text() != Some(LEGACY_MOUNTPOINT) {
        return Some((
            String::from("mountpoint"),
            format!(
                "`{}`: {}, and a clone must inherit `{LEGACY_MOUNTPOINT}`",
                facts.name,
                facts.mountpoint.describe("mountpoint")
            ),
        ));
    }
    return flag_failure(facts);
}

/// The two flags a mount of any dataset in this route must be without.
fn flag_failure(facts: &CloneObjectFacts) -> Option<(String, String)> {
    for (property, found) in [("setuid", &facts.setuid), ("devices", &facts.devices)] {
        if found.number() != Some(FLAG_OFF) {
            return Some((
                String::from(property),
                format!(
                    "`{}`: {}, and `{property}` must be off for a mount of this dataset",
                    facts.name,
                    found.describe(property)
                ),
            ));
        }
    }
    return None;
}

/// What:
///  One dataset against the properties its role requires.
/// Why:
///  The checkout check names the checkout path,
///  the container check names the container,
///  and a failure names the property,
///  so a person reading one line learns both the dataset and the field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkDataset(facts); // pass | finding(field) | unknown
/// ```
pub fn check_dataset(facts: &DatasetFacts) -> CheckOutcome {
    return match &facts.reading {
        Reading::Unreadable {
            why,
        } => unknown(format!(
            "`{}` could not be read ({why}), so its properties are unproved",
            facts.name
        )),
        Reading::Missing {
            why,
        } => finding(
            "dataset",
            format!("`{}` is not in its pool ({why})", facts.name),
        ),
        Reading::Read(read) => {
            let failure: Option<(String, String)> = property_failure(&facts.role, read);
            return match failure {
                Some((field, detail)) => finding(field, detail),
                None => pass(format!(
                    "`{}` carries the properties its role requires",
                    facts.name
                )),
            };
        }
    };
}

/// What:
///  The dataset's encryption key,
///  where the dataset has one.
/// Why:
///  A clone made while the pool's key was unloaded can exist with its data
///  unreadable,
///  and the design's preflight asks for key availability before an attach
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Add and Git registration",
///  step 1).
///  A dataset that is not encrypted has no key to load and passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkKey(datasets); // pass | finding('keystatus') | unknown
/// ```
pub fn check_key(datasets: &[DatasetFacts]) -> CheckOutcome {
    let mut unreadable: Vec<String> = Vec::new();
    let mut encrypted: usize = 0;
    for dataset in datasets {
        let Reading::Read(facts) = &dataset.reading else {
            unreadable.push(dataset.name.clone());
            continue;
        };
        if facts.encryption.text() == Some(ENCRYPTION_OFF) {
            continue;
        }
        encrypted += 1;
        if facts.key_status.number() == Some(KEY_UNAVAILABLE) {
            let root: String = facts
                .encryption_root
                .text()
                .unwrap_or("an unread encryption root")
                .to_string();
            return finding(
                "keystatus",
                format!(
                    "`{}`: `keystatus` is unavailable, so the key of {root} is not loaded and the dataset's data cannot be read",
                    facts.name
                ),
            );
        }
    }
    if !unreadable.is_empty() {
        return unknown(format!(
            "the key state of {} could not be read, so no key answer holds for the route",
            unreadable.join(", ")
        ));
    }
    return pass(format!(
        "every key is loaded ({encrypted} encrypted dataset(s) read)"
    ));
}

/// What:
///  Every clone against the properties this route attaches it with.
/// Why:
///  A clone that lost `mountpoint=legacy` could be mounted by the system's own
///  generator,
///  and one that gained `setuid` or `devices` widens what its mount exposes,
///  so a person must see the clone and the property rather than a count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkCloneProperties(clones); // pass | finding(property) | unknown
/// ```
pub fn check_clone_properties(clones: &[DatasetFacts]) -> CheckOutcome {
    let mut failures: Vec<(String, String)> = Vec::new();
    let mut unreadable: Vec<String> = Vec::new();
    let mut read: usize = 0;
    for clone in clones {
        match &clone.reading {
            Reading::Read(facts) => {
                read += 1;
                if let Some(failure) = clone_failure(facts) {
                    failures.push(failure);
                }
            }
            Reading::Missing {
                ..
            }
            | Reading::Unreadable {
                ..
            } => unreadable.push(clone.name.clone()),
        }
    }
    if let Some((field, first)) = failures.first() {
        let mut detail: String = first.clone();
        for (_, other) in failures.iter().skip(1) {
            detail.push_str("; ");
            detail.push_str(other);
        }
        return finding(field.clone(), detail);
    }
    if !unreadable.is_empty() {
        return unknown(format!(
            "the properties of {} could not be read, so no property answer holds for every clone",
            unreadable.join(", ")
        ));
    }
    return pass(format!(
        "{read} clone(s) carry `mountpoint={LEGACY_MOUNTPOINT}`, `setuid=off` and `devices=off`"
    ));
}

//endregion

/// What:
///  One report per dataset of the route,
///  each named by its role.
/// Why:
///  The wiring run prints one line per dataset,
///  so a person reads the checkout,
///  the container and each clone separately,
///  and a control asserts one of them without building the others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reports = datasetReports(datasets); // [{ check: 'checkout-dataset', outcome }]
/// ```
pub fn dataset_reports(datasets: &[DatasetFacts]) -> Vec<CheckReport> {
    let mut reports: Vec<CheckReport> = Vec::new();
    for dataset in datasets {
        reports.push(CheckReport::new(dataset.check(), check_dataset(dataset)));
    }
    return reports;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "zfs_checks_tests.rs"]
mod tests;
