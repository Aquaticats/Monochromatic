//! What:
//!  The destruction plan as a value:
//!  its header's fields,
//!  its attempt lines,
//!  the first write a removal makes,
//!  the merge rule every reader applies,
//!  and the two field spellings a person may need to read.
//! Why:
//!  The plan is the only record of what a removal verified and submitted,
//!  so its first write holds the whole verified inventory and nothing else:
//!  one line per snapshot,
//!  at attempt `0`,
//!  as `planned`
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "1.
//! The home",
//! "2.
//! The fields" and section 3's step 4).
//!  A snapshot's current state is the outcome on its highest attempt number,
//!  which is the merge rule every reader applies,
//!  and `destroyed` closes a name's history.
//!  This file holds the value and the rules a reader applies to it;
//!  the bytes a plan publishes are `plan_record_bytes.rs`,
//!  the two-stage read is `plan_reading.rs`,
//!  and the parse it calls is `plan_parse.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = DestructionPlan.firstWrite({ ...identities, unmount, snapshots });
//! ```

use crate::worktree_clone_binding_record::binding_encoding::path_as_hexadecimal;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_proof_and_plan::plan_refusal::{
    PlanRefusal,
    PlanRefusalCause,
};
use crate::worktree_clone_removal_proof_and_plan::plan_words::{
    ATTEMPT_NAME_KEY,
    HeadWord,
    SnapshotOutcome,
};
use std::path::Path;

/// What:
///  One snapshot of the clone's own history,
///  by name,
///  GUID and creation transaction group.
/// Why:
///  Each line repeats the tuple so it stands alone,
///  and the removal's proof compares exactly these three,
///  which is what the decided bookmark rule reads too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Snapshot = { name: string; guid: DatasetGuid; createtxg: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanSnapshot {
    /// The snapshot's full name,
    /// `<dataset>@<snapshot>`.
    pub name: String,
    /// The GUID the kernel reports.
    pub guid: DatasetGuid,
    /// The transaction group the snapshot was created in.
    pub createtxg: u64,
}

/// What:
///  One line of one snapshot's history.
/// Why:
///  The project chose one line per attempt,
///  so a later success keeps every earlier refusal readable
///  (`doc/decision/cli-git-zfs-removal-side-records-and-the-adds-record-accepted.md`,
///  "Choice 32").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Attempt = { snapshot: Snapshot; attempt: number; outcome: Outcome; errno?: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanAttempt {
    /// The snapshot this line is about.
    pub snapshot: PlanSnapshot,
    /// This snapshot's own attempt number,
    /// counting this record's writes for it.
    pub attempt: u64,
    /// What the attempt reports.
    pub outcome: SnapshotOutcome,
    /// The kernel's error number on a `refused` line,
    /// and nothing on any other.
    pub errno: Option<u32>,
}

/// What:
///  The identities and the verified inventory one removal records.
/// Why:
///  The first write needs the clone's and the origin's identities,
///  the registration's name bytes and identity,
///  the slot path,
///  and the unmount receipt it verified,
///  together and once,
///  so no caller can write a plan with the GUID of one clone and the dataset name of another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlanFacts = { run; owner; started; cloneDataset; cloneGuid; cloneCreatetxg; originDataset; originDatasetGuid; originSnapshot; originSnapshotGuid; originSnapshotCreatetxg; registrationId; admin; target; unmountNonce; unmountMountId };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanFacts {
    /// The removal run's own nonce,
    /// which has the intent nonce's 32-digit shape and is not a mount request.
    pub run: RequestNonce,
    /// The process that wrote the record,
    /// spelled `<pid>:linux:<start time>`.
    pub owner: String,
    /// The moment the removal started,
    /// as `started_moment` writes it.
    pub started: String,
    /// The clone's full dataset name.
    pub clone_dataset: String,
    /// The clone's GUID.
    pub clone_guid: DatasetGuid,
    /// The clone's own creation transaction group.
    pub clone_createtxg: u64,
    /// The origin dataset's full name.
    pub origin_dataset: String,
    /// The origin dataset's GUID.
    pub origin_dataset_guid: DatasetGuid,
    /// The origin snapshot's full name.
    pub origin_snapshot: String,
    /// The origin snapshot's GUID.
    pub origin_snapshot_guid: DatasetGuid,
    /// The origin snapshot's creation transaction group.
    pub origin_snapshot_createtxg: u64,
    /// The bytes of the registration's directory name under Git's `worktrees` folder.
    pub registration_id: Vec<u8>,
    /// The registration directory's identity,
    /// as `ObjectId::encode` spells it.
    pub admin: String,
    /// The slot's path.
    pub target: std::path::PathBuf,
    /// The nonce of the unmount request the removal verified.
    pub unmount_nonce: RequestNonce,
    /// The mount id that answer named absent,
    /// and nothing when it named no single mount.
    pub unmount_mount_id: Option<u64>,
}

/// What:
///  One whole destruction plan.
/// Why:
///  The writer,
///  the batch,
///  the deferred-origin check,
///  the cancel command and `doctor` all read this one shape,
///  so a field that one of them needs is necessarily in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Plan = PlanFacts & { head: HeadWord; attempts: Attempt[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DestructionPlan {
    /// The identities and the verified inventory the removal recorded.
    pub facts: PlanFacts,
    /// The head destroy's state.
    pub head: HeadWord,
    /// Every line of every snapshot's history,
    /// in the order the file writes them.
    pub attempts: Vec<PlanAttempt>,
}

impl DestructionPlan {
    /// What:
    ///  The plan a verified removal writes before its first delete:
    ///  one `planned` line for each snapshot of its verified inventory,
    ///  at attempt `0`,
    ///  with the head present.
    /// Why:
    ///  A plan written earlier would name an unmount that never happened,
    ///  and a plan that named anything the proof did not verify would be a wish list.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const plan = DestructionPlan.firstWrite(facts, verifiedSnapshots);
    /// ```
    pub fn first_write(
        facts: PlanFacts,
        snapshots: &[PlanSnapshot],
    ) -> DestructionPlan {
        let mut attempts: Vec<PlanAttempt> = snapshots
            .iter()
            .map(|snapshot| {
                return PlanAttempt {
                    snapshot: snapshot.clone(),
                    attempt: 0,
                    outcome: SnapshotOutcome::Planned,
                    errno: None,
                };
            })
            .collect();
        attempts.sort_by(|left, right| return left.snapshot.name.cmp(&right.snapshot.name));
        return DestructionPlan {
            facts,
            head: HeadWord::Present,
            attempts,
        };
    }

    /// What:
    ///  The state a snapshot's history ends on:
    ///  the outcome of its highest attempt number.
    /// Why:
    ///  This is the merge rule,
    ///  and a name whose numbers have a gap reads as a write that was lost before it landed,
    ///  never as damage.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const current = (name) => attempts.filter(a => a.snapshot.name === name).sort((l, r) => r.attempt - l.attempt)[0];
    /// ```
    pub fn current_state(
        &self,
        name: &str,
    ) -> Option<&PlanAttempt> {
        let mut highest: Option<&PlanAttempt> = None;
        for line in &self.attempts {
            if line.snapshot.name != name {
                continue;
            }
            if highest.is_none_or(|held| return line.attempt > held.attempt) {
                highest = Some(line);
            }
        }
        return highest;
    }

    /// What:
    ///  Add one line to a snapshot's history,
    ///  keeping the record in the one order its file writes.
    /// Why:
    ///  The file holds one line per attempt in ascending byte order of the snapshot names and,
    ///  within one name,
    ///  in ascending attempt number,
    ///  so a plan whose lines were appended in the order attempts happened would be a record its own
    ///  reader refuses.
    ///  A number already used for one name,
    ///  and a line for a name whose history a `destroyed` line closed,
    ///  are refused here rather than at the writer.
    ///
    /// # Throws
    /// A repeated attempt number,
    /// or a line after the one that ended the name's history.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// plan.recordLine({ snapshot, attempt: 1, outcome: 'intact' });
    /// ```
    pub fn record_line(
        &mut self,
        line: PlanAttempt,
    ) -> Result<(), PlanRefusal> {
        let gone: bool = self.attempts.iter().any(|held| {
            return held.snapshot.name == line.snapshot.name
                && (held.attempt == line.attempt || held.outcome == SnapshotOutcome::Destroyed);
        });
        if gone {
            return Err(PlanRefusal::at(
                PlanRefusalCause::DuplicateAttempt,
                ATTEMPT_NAME_KEY,
                line.snapshot.name,
            ));
        }
        self.attempts.push(line);
        self.attempts.sort_by(|left, right| {
            return left
                .snapshot
                .name
                .cmp(&right.snapshot.name)
                .then(left.attempt.cmp(&right.attempt));
        });
        return Ok(());
    }

    /// What:
    ///  Every snapshot the plan names,
    ///  each once,
    ///  in ascending byte order of its name.
    /// Why:
    ///  The bookmark rule explains a bookmark by any line for a snapshot,
    ///  so a snapshot named on several lines is one tuple and not several.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const snapshots = dedupeBy(attempts.map(a => a.snapshot), s => s.name);
    /// ```
    pub fn named_snapshots(&self) -> Vec<PlanSnapshot> {
        let mut found: Vec<PlanSnapshot> = Vec::new();
        for line in &self.attempts {
            if found
                .iter()
                .any(|held| return held.name == line.snapshot.name)
            {
                continue;
            }
            found.push(line.snapshot.clone());
        }
        found.sort_by(|left, right| return left.name.cmp(&right.name));
        return found;
    }
}

/// What:
///  The `owner` value one process writes.
/// Why:
///  The field pairs the process id with its birth identity,
///  which is how a later command tells a running removal from one a kill abandoned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owner = `${pid}:${birthIdentity}`;
/// ```
pub fn owner_text(
    pid: u32,
    birth_identity: &str,
) -> String {
    return format!("{pid}:{birth_identity}");
}

/// What:
///  The target field's value for one slot path.
/// Why:
///  A path may hold a newline,
///  so the field is hexadecimal and the plan's `targetHex` is the same spelling the add's
///  binding uses for its `slot=`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const targetHex = hexOf(slotPath);
/// ```
pub fn target_hexadecimal(target: &Path) -> String {
    return path_as_hexadecimal(target);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "plan_record_tests.rs"]
mod tests;
