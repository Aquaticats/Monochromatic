//! What:
//!  The head destroy of one removal:
//!  its `requested` line,
//!  its one request,
//!  and the absence proof behind its word.
//! Why:
//!  The head is refused by the kernel while the clone holds snapshots of its own,
//!  so it is sent only once every name is settled,
//!  and a snapshot created after the final enumeration is never retried on an unexplained
//!  `EBUSY`
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Removal and recovery",
//!  step 3).
//!  Once the head is gone no live inventory exists,
//!  so its absence is proved by its name and by no child of the pinned parent carrying its GUID,
//!  and that proof covers every snapshot it held
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The absence rule,
//! object by object").
//!  The names the plan recorded are settled by their GUIDs in `removal_snapshot_settlement.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const head = destroyTheHead({ plan, probe, device, ... });
//! ```

use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    DatasetReader,
    ListedChild,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::{
    HeadWord,
    SnapshotOutcome,
};
use crate::worktree_clone_removal_proof_and_plan::removal_snapshot_settlement::push_a_line;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DatasetIdentity,
    DeviceError,
    NvList,
    ZfsDevice,
};

/// What:
///  The one request a head destroy sends.
/// Why:
///  The head is a dataset,
///  so its destroy is `ZFS_IOC_DESTROY` and not the snapshot set's request,
///  and a control answers it without opening this machine's device.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface DatasetDestroyDevice { destroyTheDataset(name): void }
/// ```
pub trait DatasetDestroyDevice {
    /// `ZFS_IOC_DESTROY` of one dataset,
    /// with no defer mark.
    ///
    /// # Throws
    ///
    /// What the binding's own `destroy` throws.
    fn destroy_the_dataset(
        &self,
        name: &str,
    ) -> Result<Option<NvList>, DeviceError>;
}

/// The open device is the production answer.
impl DatasetDestroyDevice for ZfsDevice {
    /// Sends the binding's own dataset destroy,
    /// which never carries a defer mark.
    fn destroy_the_dataset(
        &self,
        name: &str,
    ) -> Result<Option<NvList>, DeviceError> {
        return ZfsDevice::destroy(self, name, false);
    }
}

/// The parent dataset a clone hangs under:
/// its recorded name without the last `/`-separated component.
///
/// # Throws
/// A name with no parent,
/// which the add's layout never makes.
pub fn the_pinned_parent(clone_dataset: &str) -> Result<String, RemovalStop> {
    let Some((parent, _)) = clone_dataset.rsplit_once('/') else {
        return Err(RemovalStop::DoesNotAnswer {
            object: String::from(clone_dataset),
            reason: String::from("the clone's name has no parent dataset, so nothing pins it"),
        });
    };
    return Ok(String::from(parent));
}

/// What:
///  What the pool answers about one clone's own existence.
/// Why:
///  The four answers lead four ways:
///  the clone stands,
///  its name holds another object,
///  it is proved gone,
///  or the pool could not prove either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneAbsence = { kind: 'there'; name; guid } | { kind: 'reused'; guid } | { kind: 'gone' } | { kind: 'unprovable'; said };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloneAbsence {
    /// The clone answers under its recorded name with its recorded GUID.
    There {
        /// The name that answered.
        name: String,
        /// The GUID it answered.
        guid: DatasetGuid,
    },
    /// The recorded name holds another object.
    Reused {
        /// The GUID the name answered.
        guid: DatasetGuid,
    },
    /// The name is absent and no child of the pinned parent carries the GUID.
    Gone,
}

/// What:
///  Whether one clone is gone,
///  proved by its name and by no child of the pinned parent carrying its GUID.
/// Why:
///  The design's rule is both facts,
///  because a renamed clone is the same clone and a reused name is another object.
///
/// # Throws
/// A read the pool refused,
/// which proves nothing either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const absence = theCloneAbsence(probe, clone, guid);
/// ```
pub fn the_clone_absence(
    probe: &dyn DatasetReader,
    clone_dataset: &str,
    clone_guid: DatasetGuid,
) -> Result<CloneAbsence, RemovalStop> {
    let mut reused: Option<DatasetGuid> = None;
    match probe.identity(clone_dataset) {
        Ok(identity) => {
            let live: DatasetGuid = guid_of_reply(&identity, clone_dataset)?;
            if live == clone_guid {
                return Ok(CloneAbsence::There {
                    name: String::from(clone_dataset),
                    guid: live,
                });
            }
            reused = Some(live);
        }
        Err(failure) if failure.errno == Some(libc::ENOENT) => {}
        Err(failure) => {
            return Err(RemovalStop::DoesNotAnswer {
                object: String::from(clone_dataset),
                reason: failure.message(),
            });
        }
    }
    let parent: String = the_pinned_parent(clone_dataset)?;
    let mut cursor: u64 = 0;
    loop {
        let next: Option<ListedChild> =
            probe.next_child(parent.as_str(), cursor).map_err(|said| {
                return RemovalStop::DoesNotAnswer {
                    object: parent.clone(),
                    reason: said,
                };
            })?;
        let Some(child) = next else {
            return match reused {
                Some(guid) => Ok(CloneAbsence::Reused {
                    guid,
                }),
                None => Ok(CloneAbsence::Gone),
            };
        };
        cursor = child.cursor;
        match probe.identity(child.name.as_str()) {
            Ok(identity) => {
                if guid_of_reply(&identity, child.name.as_str())? == clone_guid {
                    return Ok(CloneAbsence::There {
                        name: child.name,
                        guid: clone_guid,
                    });
                }
            }
            Err(failure) if failure.errno == Some(libc::ENOENT) => {}
            Err(failure) => {
                return Err(RemovalStop::DoesNotAnswer {
                    object: child.name,
                    reason: failure.message(),
                });
            }
        }
    }
}

/// The GUID one reply carries,
/// refused when the pool reports none.
fn guid_of_reply(
    identity: &DatasetIdentity,
    name: &str,
) -> Result<DatasetGuid, RemovalStop> {
    return DatasetGuid::parse(identity.stats.dds_guid.to_string().as_str()).map_err(|_| {
        return RemovalStop::DoesNotAnswer {
            object: String::from(name),
            reason: String::from("the reply carries no GUID"),
        };
    });
}

/// What:
///  The head destroy's inputs,
///  which are the plan it writes,
///  the pool it proves through and the one device it sends to.
/// Why:
///  The head's word travels in the plan's header and nowhere else,
///  so the plan and its file are the step's own inputs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeadRequest = { plan; folder; probe; device };
/// ```
pub struct HeadDestroyRequest<'a> {
    /// The plan,
    /// which this step rewrites three times.
    pub plan: &'a mut DestructionPlan,
    /// The folder each write publishes into.
    pub folder: &'a PlanFolder,
    /// The pool reads the absence proof makes.
    pub probe: &'a dyn DatasetReader,
    /// The device the one destroy goes to.
    pub device: &'a dyn DatasetDestroyDevice,
}

/// What:
///  What the head destroy did,
///  with the snapshots its proof covered.
/// Why:
///  The outcome names whether the clone is gone,
///  refused or unknown,
///  and the names the head's proof settles,
///  so the report can print both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeadDestroyed = { head; settled: string[]; errno? };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeadDestroyed {
    /// The word the plan's header ends on.
    pub head: HeadWord,
    /// The snapshots the absence proof settled.
    pub settled: Vec<String>,
    /// The errno of a refused request,
    /// when the kernel kept one.
    pub errno: Option<u32>,
}

/// What:
///  One head destroy:
///  the `requested` write,
///  the one request,
///  and the proof that decides its word.
/// Why:
///  The head is the one request whose in-flight state is stored,
///  and it is stored before the request because a kill inside it leaves a clone that may be gone,
///  which the next run proves rather than guesses.
///  A refusal does not end the run's authority to read the pool:
///  an error may still have taken effect,
///  so the absence proof decides between `gone` and `refused`.
///
/// # Throws
/// A plan write that refused,
/// or a pool that cannot prove the absence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const head = destroyTheHead({ plan, folder, probe, device });
/// ```
pub fn destroy_the_head(
    request: &mut HeadDestroyRequest<'_>
) -> Result<HeadDestroyed, RemovalStop> {
    let clone: String = request.plan.facts.clone_dataset.clone();
    let clone_guid: DatasetGuid = request.plan.facts.clone_guid;
    request.plan.head = HeadWord::Requested;
    publish(request)?;
    let answer: Result<Option<NvList>, DeviceError> =
        request.device.destroy_the_dataset(clone.as_str());
    let errno: Option<u32> = match &answer {
        Ok(_) => None,
        Err(error) => error
            .errno()
            .and_then(|held| return u32::try_from(held).ok()),
    };
    return match the_clone_absence(request.probe, clone.as_str(), clone_guid) {
        Ok(CloneAbsence::Gone) => settle_the_head(request, HeadWord::Gone, None),
        Ok(CloneAbsence::There {
            ..
        }) => settle_the_head(request, HeadWord::Refused, errno),
        Ok(CloneAbsence::Reused {
            guid,
        }) => {
            request.plan.head = HeadWord::Unknown;
            publish(request)?;
            Err(RemovalStop::TheCloneNameWasReused {
                name: clone,
                recorded_guid: clone_guid.file_name(),
                live_guid: guid.file_name(),
            })
        }
        Err(stop) => {
            request.plan.head = HeadWord::Unknown;
            publish(request)?;
            Err(stop)
        }
    };
}

/// The head's word written after its proof,
/// with every remaining name settled by that proof when the head is gone.
fn settle_the_head(
    request: &mut HeadDestroyRequest<'_>,
    head: HeadWord,
    errno: Option<u32>,
) -> Result<HeadDestroyed, RemovalStop> {
    request.plan.head = head;
    let mut settled: Vec<String> = Vec::new();
    if head == HeadWord::Gone {
        for snapshot in request.plan.named_snapshots() {
            if request
                .plan
                .current_state(snapshot.name.as_str())
                .map(|held| return held.outcome == SnapshotOutcome::Destroyed)
                .unwrap_or(false)
            {
                continue;
            }
            push_a_line(request.plan, &snapshot, SnapshotOutcome::Destroyed, None)?;
            settled.push(snapshot.name);
        }
    }
    publish(request)?;
    return Ok(HeadDestroyed {
        head,
        settled,
        errno,
    });
}

/// One plan write through the folder that publishes it whole.
fn publish(request: &HeadDestroyRequest<'_>) -> Result<(), RemovalStop> {
    return match request.folder.publish(request.plan) {
        Ok(_path) => Ok(()),
        Err(refusal) => Err(RemovalStop::ThePlanCouldNotHoldTheAnswer {
            path: request
                .folder
                .path()
                .join(request.plan.facts.clone_guid.file_name()),
            said: refusal.message(request.folder.path()),
        }),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_head_destroy_tests.rs"]
mod tests;
