//! What:
//!  The repository,
//! the plan,
//! the gate,
//! the pool reader and the settle a finish-pass control starts from,
//!  and the small helpers that read what a pass left.
//! Why:
//!  Every arm changes exactly one thing from a state the design names,
//!  so the plan's head,
//!  the clone's absence,
//!  one binding field,
//!  the settle's answer and the gate's verdict are values a control sets
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The finish pass's live read and its tail").
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { fixture, plan } = aFinishedRemoval('one-arm');
//! const lines = passLines(fixture, liveFolder, gate, probe, settle);
//! ```

use crate::worktree_clone_binding_record::registration_home::{
    clone_inventory_name,
    home_folder,
    plans_folder,
};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    DatasetReader,
    ListedChild,
    ReadFailure,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_finish_pass::the_pass_seams::FinishPass;
use crate::worktree_clone_removal_finish_pass::the_plans_folder::FolderLook;
use crate::worktree_clone_removal_finish_pass::the_plans_folder::ThePlansFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_record::{
    DestructionPlan,
    PlanFacts,
    PlanSnapshot,
};
use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{
    CLONE,
    CLONE_CREATETXG,
    CLONE_GUID,
    ORIGIN_CREATETXG,
    ORIGIN_DATASET,
    ORIGIN_DATASET_GUID,
    ORIGIN_SNAPSHOT,
    ORIGIN_SNAPSHOT_GUID,
    OWN_GUID,
    RemovalFixture,
    SANOID_GUID,
    SANOID_SNAPSHOT,
    the_stats,
};
use crate::worktree_clone_removal_proof_and_plan::removal_settle::{
    RegistrationSettle,
    SettleAnswer,
    SettleRequest,
};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::zfs_compatibility_gate::{
    BeforeRequests,
    Verdict,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    CompatibilityRefusal,
    CompatibilityStep,
    DatasetIdentity,
    NvList,
};
use std::cell::{
    Cell,
    RefCell,
};
use std::path::{
    Path,
    PathBuf,
};

/// The slot every control's repository holds.
pub const SLOT: &str = "fix-flaky-test";

/// The GUID a dataset that took the clone's name answers,
/// which is no GUID this plan recorded.
pub const ANOTHER_CLONE_GUID: u64 = 1;

/// The plan's run nonce,
/// which no control reads.
pub const RUN_NONCE: &str = "00112233445566778899aabbccddeeff";

/// The plan's own first snapshot,
/// as the sample records it.
const FIRST_OWN: &str = "tank/wt/fix-flaky-test@worktree-own";

/// One repository an add finished at,
/// with its binding,
/// its origin inventory,
/// its registration directory
/// and a plan whose head is `gone`,
/// the state a kill between the head destroy and the settle leaves.
pub fn a_finished_removal(name: &str) -> (RemovalFixture, DestructionPlan) {
    let fixture: RemovalFixture = RemovalFixture::healthy(name);
    let plan: DestructionPlan = a_plan_for(&fixture, HeadWord::Gone);
    write_the_clone_inventory(&fixture);
    let _path: PathBuf = write_the_plan(&fixture, &plan);
    return (fixture, plan);
}

/// The clone-side inventory an add writes,
/// which the pass removes with the binding.
pub fn write_the_clone_inventory(fixture: &RemovalFixture) {
    let home: PathBuf = home_folder(fixture.common());
    std::fs::create_dir_all(home.as_path()).expect("the record folder");
    std::fs::write(
        home.join(clone_inventory_name(SLOT)).as_path(),
        b"# a clone inventory\n",
    )
    .expect("the clone inventory");
}

/// One plan whose facts are the fixture's own,
/// with the head a control names.
pub fn a_plan_for(
    fixture: &RemovalFixture,
    head: HeadWord,
) -> DestructionPlan {
    let facts: PlanFacts = PlanFacts {
        run: RequestNonce::parse(RUN_NONCE).expect("the control's nonce is canonical"),
        owner: String::from("48213:linux:9481733"),
        started: String::from("2026-11-25T09:12:44-05:00"),
        clone_dataset: String::from(CLONE),
        clone_guid: DatasetGuid::parse(CLONE_GUID.to_string().as_str())
            .expect("the fixture's GUID is canonical"),
        clone_createtxg: CLONE_CREATETXG,
        origin_dataset: String::from(ORIGIN_DATASET),
        origin_dataset_guid: DatasetGuid::parse(ORIGIN_DATASET_GUID.to_string().as_str())
            .expect("the fixture's GUID is canonical"),
        origin_snapshot: String::from(ORIGIN_SNAPSHOT),
        origin_snapshot_guid: DatasetGuid::parse(ORIGIN_SNAPSHOT_GUID.to_string().as_str())
            .expect("the fixture's GUID is canonical"),
        origin_snapshot_createtxg: ORIGIN_CREATETXG,
        registration_id: Vec::from(SLOT.as_bytes()),
        admin: the_registrations_identity(fixture),
        target: fixture.slot_path.clone(),
        unmount_nonce: RequestNonce::parse("0a1b2c3d4e5f60718293a4b5c6d7e8f9")
            .expect("the digits are a nonce"),
        unmount_mount_id: None,
    };
    let snapshots: Vec<PlanSnapshot> = vec![
        PlanSnapshot {
            name: String::from(FIRST_OWN),
            guid: DatasetGuid::parse(OWN_GUID.to_string().as_str())
                .expect("the fixture's GUID is canonical"),
            createtxg: ORIGIN_CREATETXG,
        },
        PlanSnapshot {
            name: format!("{CLONE}@{SANOID_SNAPSHOT}"),
            guid: DatasetGuid::parse(SANOID_GUID.to_string().as_str())
                .expect("the fixture's GUID is canonical"),
            createtxg: ORIGIN_CREATETXG,
        },
    ];
    let mut plan: DestructionPlan = DestructionPlan::first_write(facts, snapshots.as_slice());
    plan.head = head;
    return plan;
}

/// The identity the plan records for the registration directory,
/// read from the folder the fixture made.
pub fn the_registrations_identity(fixture: &RemovalFixture) -> String {
    let opened: std::fs::File = std::fs::File::open(fixture.registration.as_path())
        .expect("the fixture's registration opens");
    return crate::fs_beneath::identity_of(&opened)
        .expect("the registration's identity reads")
        .encode();
}

/// The plan published under its clone's GUID,
/// as a removal's writer leaves it.
pub fn write_the_plan(
    fixture: &RemovalFixture,
    plan: &DestructionPlan,
) -> PathBuf {
    let folder: PlanFolder =
        PlanFolder::open_or_create(fixture.common()).expect("the plan folder opens");
    return folder.publish(plan).expect("the plan publishes");
}

/// Whether anything stands at one path.
pub fn stands(path: &Path) -> bool {
    return std::fs::symlink_metadata(path).is_ok();
}

/// The plan folder of one fixture.
pub fn the_plan_folder(fixture: &RemovalFixture) -> PathBuf {
    return plans_folder(fixture.common());
}

/// The wrapper's record folder of one fixture.
pub fn the_home(fixture: &RemovalFixture) -> PathBuf {
    return home_folder(fixture.common());
}

/// What:
///  The plan folder a control answers with,
///  counting the looks the pass takes.
/// Why:
///  The healthy state is one look and no listing,
///  so an arm that counts is what shows the pass asks once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folder = { answer: { kind: 'no-folder' }, looks: 0 };
/// ```
pub struct AFolder {
    /// What every look answers.
    pub answer: FolderLook,
    /// How many looks the pass took.
    pub looks: Cell<usize>,
}

impl AFolder {
    /// The folder a control answers with.
    pub fn answering(answer: FolderLook) -> AFolder {
        return AFolder {
            answer,
            looks: Cell::new(0),
        };
    }
}

impl ThePlansFolder for AFolder {
    fn look(
        &self,
        _folder: &Path,
    ) -> FolderLook {
        self.looks.set(self.looks.get() + 1);
        return self.answer.clone();
    }
}

/// What:
///  The compatibility gate a control answers with.
/// Why:
///  A refusing gate makes no request at all,
///  and this arm needs no kernel file to be refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gate = { verdict: () => refused('the device cannot be opened') };
/// ```
pub struct AGate {
    /// Whether the gate refuses,
    /// and what it says.
    pub refused: Option<String>,
}

impl AGate {
    /// A gate that passed.
    pub fn passing() -> AGate {
        return AGate {
            refused: None,
        };
    }

    /// A gate that refuses with one detail.
    pub fn refusing(detail: &str) -> AGate {
        return AGate {
            refused: Some(String::from(detail)),
        };
    }
}

impl BeforeRequests for AGate {
    fn verdict(&self) -> Verdict {
        let Some(detail) = &self.refused else {
            return Verdict::Compatible(crate::zfs_compatibility_gate::Facts {
                target: "x86_64-unknown-linux-gnu",
                version: None,
                absent: Vec::new(),
                base_btf_bytes: 0,
                module_btf_bytes: 0,
                header_bytes: (0, 0),
            });
        };
        return Verdict::Refused(CompatibilityRefusal {
            step: CompatibilityStep::Device,
            file: None,
            detail: detail.clone(),
        });
    }
}

/// What:
///  The pool a control answers:
///  what the clone's name holds,
///  the children of the pinned parent,
///  and a read that fails.
/// Why:
///  The absence proof reads exactly those three,
///  and each arm of the design's absence rule is one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const probe = { clone: undefined, children: [] };
/// ```
pub struct AProbe {
    /// The GUID the clone's own name answers,
    /// or nothing when the name is absent.
    pub clone: Option<DatasetGuid>,
    /// The pinned parent's children,
    /// each name and the GUID it answers.
    pub children: Vec<(String, DatasetGuid)>,
    /// A read every call refuses with,
    /// which proves nothing either way.
    pub refuses: Option<String>,
}

impl AProbe {
    /// The clone gone:
    /// its name answers nothing and no child carries its GUID.
    pub fn absent() -> AProbe {
        return AProbe {
            clone: None,
            children: Vec::new(),
            refuses: None,
        };
    }

    /// The clone still answering under its own name and GUID.
    pub fn standing() -> AProbe {
        return AProbe {
            clone: Some(
                DatasetGuid::parse(CLONE_GUID.to_string().as_str())
                    .expect("the fixture's GUID is canonical"),
            ),
            children: Vec::new(),
            refuses: None,
        };
    }

    /// The name absent and a child carrying the recorded GUID,
    /// which is the same clone under another name.
    pub fn renamed_to(name: &str) -> AProbe {
        return AProbe {
            clone: None,
            children: vec![(
                String::from(name),
                DatasetGuid::parse(CLONE_GUID.to_string().as_str())
                    .expect("the fixture's GUID is canonical"),
            )],
            refuses: None,
        };
    }

    /// The name holding another object.
    pub fn reused(guid: u64) -> AProbe {
        return AProbe {
            clone: Some(
                DatasetGuid::parse(guid.to_string().as_str())
                    .expect("the control's GUID is canonical"),
            ),
            children: Vec::new(),
            refuses: None,
        };
    }

    /// The same probe with every read refused.
    pub fn refusing(
        mut self,
        said: &str,
    ) -> AProbe {
        self.refuses = Some(String::from(said));
        return self;
    }
}

impl DatasetReader for AProbe {
    fn identity(
        &self,
        dataset: &str,
    ) -> Result<DatasetIdentity, ReadFailure> {
        if let Some(said) = &self.refuses {
            return Err(ReadFailure {
                said: said.clone(),
                errno: Some(libc::EIO),
            });
        }
        for (name, guid) in &self.children {
            if name == dataset {
                return Ok(an_identity(*guid));
            }
        }
        if dataset != CLONE {
            return Err(ReadFailure {
                said: String::from("no such dataset"),
                errno: Some(libc::ENOENT),
            });
        }
        return match self.clone {
            Some(guid) => Ok(an_identity(guid)),
            None => Err(ReadFailure {
                said: String::from("no such dataset"),
                errno: Some(libc::ENOENT),
            }),
        };
    }

    fn next_child(
        &self,
        _parent: &str,
        cursor: u64,
    ) -> Result<Option<ListedChild>, String> {
        if let Some(said) = &self.refuses {
            return Err(said.clone());
        }
        let at: usize = cursor as usize;
        return match self.children.get(at) {
            None => Ok(None),
            Some((name, _guid)) => Ok(Some(ListedChild {
                name: name.clone(),
                cursor: cursor + 1,
            })),
        };
    }
}

/// One dataset's own reply,
/// carrying `guid`.
fn an_identity(guid: DatasetGuid) -> DatasetIdentity {
    return DatasetIdentity {
        stats: the_stats(
            guid.file_name().as_str().parse::<u64>().unwrap_or(1),
            false,
            1,
            0,
            "",
        ),
        properties: NvList::new(),
    };
}

/// What:
///  The settle a control answers,
///  keeping every request and refusing when an arm says so.
/// Why:
///  The pass settles only after its own proof,
///  so a control asserts the request's proof names the plan's slot and clone,
///  and an arm that refuses shows what the pass leaves standing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settle = { calls: 0, refuses: false };
/// ```
pub struct ASettle {
    /// Whether the settle refuses,
    /// and what it says.
    pub refuses: Option<String>,
    /// How many calls the pass made.
    pub calls: Cell<usize>,
    /// Every slot the pass asked to settle.
    pub slots: RefCell<Vec<String>>,
}

impl ASettle {
    /// A settle that answers what it did.
    pub fn succeeding() -> ASettle {
        return ASettle {
            refuses: None,
            calls: Cell::new(0),
            slots: RefCell::new(Vec::new()),
        };
    }

    /// A settle that refuses with one sentence.
    pub fn refusing(said: &str) -> ASettle {
        return ASettle {
            refuses: Some(String::from(said)),
            calls: Cell::new(0),
            slots: RefCell::new(Vec::new()),
        };
    }
}

impl RegistrationSettle for ASettle {
    fn settle_the_registration(
        &self,
        request: &SettleRequest<'_>,
    ) -> Result<SettleAnswer, RemovalStop> {
        self.calls.set(self.calls.get() + 1);
        self.slots.borrow_mut().push(String::from(request.slot));
        assert_eq!(
            request.proof.slot(),
            request.slot,
            "the destruction proof names the slot the settle is asked about"
        );
        if let Some(said) = &self.refuses {
            return Err(RemovalStop::TheSettleRefused {
                step: String::from("the registration"),
                said: said.clone(),
            });
        }
        return Ok(SettleAnswer {
            wrote_the_lost_lock: false,
            released_the_lock: true,
            removed_the_registration: true,
            removed_the_underlay: false,
            lines: Vec::new(),
        });
    }
}

/// What:
///  The pass run over one repository with the seams a control names,
///  which is one call and no file left out.
/// Why:
///  Every arm differs in one seam,
///  so one call keeps the rest of the arrangement out of each arm.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = thePass(fixture, liveFolder, gate, probe, settle);
/// ```
pub fn the_pass(
    fixture: &RemovalFixture,
    folder: &dyn ThePlansFolder,
    gate: &dyn BeforeRequests,
    probe: &dyn DatasetReader,
    settle: &dyn RegistrationSettle,
) -> Vec<String> {
    let pass: FinishPass<'_> = FinishPass {
        common_dir: fixture.common(),
        folder,
        gate,
        probe,
        settle,
    };
    return crate::worktree_clone_removal_finish_pass::pass_over_the_plans::finish_the_removal_plans(&pass);
}
