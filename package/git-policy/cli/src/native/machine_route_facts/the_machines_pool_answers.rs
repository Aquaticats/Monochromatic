//! What:
//!  Every question this machine's route facts ask the kernel and the pool,
//!  behind one seam:
//!  the state of `/dev/zfs`,
//!  the start-time layout gate's verdict,
//!  one dataset's properties with its own GUID,
//!  the clone container's immediate children,
//!  and the delegation the kernel answers for a dataset.
//! Why:
//!  These reads are the only ones that open `/dev/zfs`,
//!  which no control on a person's machine may do:
//!  this machine's home is on a ZFS pool,
//!  and a read that reaches the pool acts on it
//!  (`AGENTS.md`'s rule about a test that opens the device;
//!  `package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`,
//!  "No test on the host opens `/dev/zfs`").
//!  So the gatherer takes a value answering these five questions,
//!  the release hands in the live one,
//!  and a control hands in answers it scripted,
//!  which is the same seam the sweep's own reader is
//!  (`crate::worktree_clone_mount_handler::sweep_dataset_reader`).
//!  Every read keeps the kernel's error number,
//!  so a dataset that is gone and one this account may not read stay two
//!  answers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pool: TheMachinesPool = TheLivePool.at(places);
//! const facts = pool.oneDatasetsFacts('zpcachyos/home/monochromatic-git-checkout'); // { kind: 'read', value } | { kind: 'missing', why } | { kind: 'unreadable', why }
//! ```

/// The places the device is named by.
use super::the_route_places::TheRoutePlaces;
/// The delegation reply the route's grant model is built from.
use crate::doctor_zfs_route::delegation_reply::{
    Grant,
    grants_of,
};
/// The device's state,
/// the layout gate's three answers,
/// and what a dataset's properties become.
use crate::doctor_zfs_route::zfs_checks::{
    AbiFacts,
    DeviceState,
};
/// What one dataset's read answered,
/// and the rule that reads a failed read.
use crate::doctor_zfs_route::zfs_checks_reading::{
    Reading,
    reading_of,
};
/// The facts one kernel read of a dataset answered.
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{
    CloneObjectFacts,
    facts_of,
};
/// The kernel's error number and the message of a failed read.
use crate::worktree_clone_mount_handler::sweep_dataset_reader::ReadFailure;
/// The one walk of a parent dataset's children,
/// with the bound it stops at.
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    Listing,
    ZfsDatasetReader,
    each_child,
};
/// The gate the wrapper and its root modes run before their first request.
use crate::zfs_compatibility_gate::{
    BeforeRequests,
    LiveGate,
    Verdict,
};
/// The binding's device handle,
/// the reply a dataset's statistics carry,
/// the error a request answered,
/// and the step a refusal names.
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    CompatibilityStep,
    DatasetIdentity,
    DeviceError,
    ZfsDevice,
};
/// A borrowed path,
/// and the owned one the device's place keeps.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The five questions the machine's route facts ask of the kernel,
///  asked of one value so a control answers them without a device.
/// Why:
///  A control must prove each answer the gatherer turns into a fact,
///  and none of these can be shown on a host whose home is on the pool,
///  so the answers are a seam and the release's is one implementation of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface TheMachinesPool { devicesState(); layoutsVerdict(); oneDatasetsFacts(name); containersChildren(container); grantsAt(name); }
/// ```
pub(crate) trait TheMachinesPool {
    /// What the look at the device and this account's open of it answered.
    fn the_devices_state(&self) -> DeviceState;
    /// What the start-time layout gate answered.
    fn the_layouts_verdict(&self) -> AbiFacts;
    /// One dataset's properties,
    /// its own GUID,
    /// or why the pool answered none.
    fn one_datasets_facts(
        &self,
        dataset: &str,
    ) -> Reading<CloneObjectFacts>;
    /// The clone container's immediate children,
    /// in the order the kernel lists them.
    fn the_containers_children(
        &self,
        container: &str,
    ) -> Result<Vec<String>, String>;
    /// Every grant the kernel answers for one dataset,
    /// its ancestors' included.
    fn the_grants_at(
        &self,
        dataset: &str,
    ) -> Result<Vec<Grant>, String>;
}

/// What:
///  The release's answers:
///  this machine's `/dev/zfs`,
///  the process's own layout gate,
///  and the pool that answers beneath them.
/// Why:
///  `doctor` is read-only,
///  so every question here is a read of the kernel's own reply,
///  and the gate is the one the wrapper runs before its first request
///  (`doc/decision/cli-git-zfs-runtime-layout-checker.md`).
///  The device's path travels with the value so a report names the same file
///  the look judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const live: TheMachinesPool = TheLivePool.at(TheRoutePlaces.live());
/// ```
#[derive(Clone, Debug)]
pub(crate) struct TheLivePool {
    /// The device path the look reads,
    /// which the release's places name.
    device: PathBuf,
}

impl TheLivePool {
    /// What:
    ///  The live answers for the places this machine runs the route from.
    /// Why:
    ///  The device's path comes from the places rather than a second copy of
    ///  the binding's constant,
    ///  so the look judges the file the gatherer names.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const pool = TheLivePool.at(places);
    /// ```
    pub(crate) fn at(places: &TheRoutePlaces) -> Self {
        return TheLivePool {
            device: places.device.clone(),
        };
    }
}

impl TheMachinesPool for TheLivePool {
    fn the_devices_state(&self) -> DeviceState {
        let path: &Path = self.device.as_path();
        match std::fs::metadata(path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return DeviceState::Absent;
            }
            Err(error) => {
                return DeviceState::Unreadable {
                    why: error.to_string(),
                };
            }
            Ok(_) => {}
        }
        return match ZfsDevice::open_at(path) {
            Ok(_opened) => DeviceState::Opens,
            Err(error) => DeviceState::Denied {
                why: error.to_string(),
            },
        };
    }

    fn the_layouts_verdict(&self) -> AbiFacts {
        return abi_facts_of(LiveGate.verdict());
    }

    fn one_datasets_facts(
        &self,
        dataset: &str,
    ) -> Reading<CloneObjectFacts> {
        let identity: Result<DatasetIdentity, ReadFailure> = match ZfsDevice::open() {
            Ok(device) => device.objset_stats(dataset).map_err(|error| {
                return failed_read(&error);
            }),
            Err(error) => Err(failed_read(&error)),
        };
        return reading_of(identity.map(|held| return facts_of(dataset, &held)));
    }

    fn the_containers_children(
        &self,
        container: &str,
    ) -> Result<Vec<String>, String> {
        let reader: ZfsDatasetReader = ZfsDatasetReader::open().map_err(|error| {
            return format!(
                "the ZFS device could not be opened to list {container}'s children ({error})"
            );
        })?;
        let mut listed: Vec<String> = Vec::new();
        let listing: Listing = each_child(&reader, container, &mut |child| {
            listed.push(String::from(child));
        });
        if let Some(refusal) = listing.refusal {
            return Err(refusal.message());
        }
        let clones: Vec<String> = listed
            .into_iter()
            .filter(|name| return !name.contains('@'))
            .collect();
        return Ok(clones);
    }

    fn the_grants_at(
        &self,
        dataset: &str,
    ) -> Result<Vec<Grant>, String> {
        let device: ZfsDevice = ZfsDevice::open().map_err(|error| {
            return format!(
                "the ZFS device could not be opened to read {dataset}'s delegation ({error})"
            );
        })?;
        let reply = device.get_fsacl(dataset).map_err(|error| {
            return format!("{dataset}'s delegation could not be read ({error})");
        })?;
        return grants_of(&reply);
    }
}

/// What:
///  The layout gate's verdict in the three states a report keeps apart.
/// Why:
///  A refusal at the device step proves nothing about the layout,
///  so it is unknown rather than a second finding about a device the device
///  check already names,
///  and the release text is reported beside a verdict that was reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = abiFactsOf(verdict); // { kind: 'compatible', module, details } | { kind: 'refused', words } | { kind: 'unreadable', why }
/// ```
fn abi_facts_of(verdict: Verdict) -> AbiFacts {
    return match verdict {
        Verdict::Compatible(facts) => AbiFacts::Compatible {
            module: facts.version.clone(),
            details: vec![
                format!("base type information {} bytes", facts.base_btf_bytes),
                format!("module type information {} bytes", facts.module_btf_bytes),
            ],
        },
        Verdict::Refused(refusal) => {
            if refusal.step == CompatibilityStep::Device {
                AbiFacts::Unreadable {
                    why: refusal.to_string(),
                }
            } else {
                AbiFacts::Refused {
                    words: refusal.to_string(),
                }
            }
        }
    };
}

/// What:
///  One failed pool read in the shape the reading rule takes:
///  what the kernel said,
///  and its own error number where it kept one.
/// Why:
///  A dataset the kernel says is gone is a finding and one this account may
///  not read is unknown,
///  and only the error number tells them apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failure = failedRead(error); // { said: String(error), errno: error.errno }
/// ```
fn failed_read(error: &DeviceError) -> ReadFailure {
    return ReadFailure {
        said: error.to_string(),
        errno: error.errno(),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_machines_pool_answers_tests.rs"]
mod tests;
