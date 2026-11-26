//! What:
//!  The fixture every control of this folder starts from:
//!  a repository's Git common directory,
//!  a slot folder,
//!  a registration folder with a real identity,
//!  a published binding record,
//!  a scripted pool and a scripted mount table.
//! Why:
//!  The command removes evidence,
//!  so a control asserts what stands afterwards,
//!  and the record's identities must be the fixture folder's own or the
//!  registration proof refuses every arm.
//!  The pool is scripted because no control on a person's machine may open
//!  `/dev/zfs`,
//!  and the mount table is the shared scripted one.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { common, slotPath, registration } = theFixture('settling-healthy');
//! publishTheRecord(common, slotPath, registration, Phase.Filled);
//! const probe = ScriptedProbe.with([...]);
//! ```

use crate::testing::test_support::fixture;
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_binding_record::record_publication::write_binding;
use crate::worktree_clone_binding_record::registration_home::open_or_create_home;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    DatasetReader,
    ListedChild,
    ReadFailure,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_slot_state::slot_state_test_records::record_at;
use crate::worktree_removal_deferred_origin_check::fixtures::stats;
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DatasetIdentity,
    DmuObjsetStats,
    NvList,
};
use std::cell::RefCell;
use std::path::PathBuf;

/// The slot's name every control uses.
pub(crate) const SLOT: &str = "topic";

/// The clone's GUID the fixture record carries,
/// which the design's example confirms as `3102`.
pub(crate) const CLONE_GUID: u64 = 4_821_193_048_571_903_102;

/// What:
///  One fixture's three folders.
/// Why:
///  The slot,
/// the registration and the common directory are the three the
///  command reads,
///  so a control names them once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Fixture = { root; common; slotPath; registration };
/// ```
pub(crate) struct TheFixture {
    /// The fixture's root.
    pub root: PathBuf,
    /// The repository's Git common directory.
    pub common: PathBuf,
    /// The slot folder.
    pub slot_path: PathBuf,
    /// The registration folder under Git's `worktrees`.
    pub registration: PathBuf,
}

/// What:
///  One fresh fixture with its folders made.
/// Why:
///  Every control starts from the same three folders and differs in what it
///  plants,
///  so one function makes them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const made = theFixture('settling-healthy');
/// ```
pub(crate) fn the_fixture(name: &str) -> TheFixture {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join(".git");
    std::fs::create_dir(common.as_path()).expect("the common directory");
    let slot_path: PathBuf = root.join(".claude").join("worktrees").join(SLOT);
    std::fs::create_dir_all(slot_path.as_path()).expect("the slot");
    let registration: PathBuf = common.join("worktrees").join(SLOT);
    std::fs::create_dir_all(registration.as_path()).expect("the registration");
    std::fs::write(registration.join("HEAD"), b"ref: refs/heads/main\n")
        .expect("the registration's HEAD");
    std::fs::write(registration.join("commondir"), b"../..\n")
        .expect("the registration's commondir");
    std::fs::write(
        registration.join("gitdir"),
        format!("{}/.git\n", slot_path.display()),
    )
    .expect("the registration's gitdir");
    return TheFixture {
        root,
        common,
        slot_path,
        registration,
    };
}

/// What:
///  The record one fixture holds,
///  with the fixture's own identities.
/// Why:
///  The record's `registrationIdentity=` and `slotIdentity=` are folder
///  identities,
///  so a record built from another folder's would refuse every arm here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = theRecord(made, Phase.Filled);
/// ```
pub(crate) fn the_record(
    made: &TheFixture,
    phase: Phase,
) -> BindingRecord {
    let mut record: BindingRecord = record_at(phase, made.slot_path.as_path());
    record.slot = made.slot_path.as_os_str().as_encoded_bytes().to_vec();
    record.slot_identity =
        Some(FolderIdentity::of_folder(made.slot_path.as_path()).expect("the slot's identity"));
    record.registration = Some(made.registration.as_os_str().as_encoded_bytes().to_vec());
    record.registration_identity = Some(
        FolderIdentity::of_folder(made.registration.as_path())
            .expect("the registration's identity"),
    );
    return record;
}

/// What:
///  One record published under the fixture's common directory.
/// Why:
///  The release reader is what the command asks,
///  so a control writes through the release writer rather than by hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// publishTheRecord(made, Phase.Filled);
/// ```
pub(crate) fn publish_the_record(
    made: &TheFixture,
    phase: Phase,
) {
    let home: std::fs::File =
        open_or_create_home(made.common.as_path()).expect("the record home opens");
    write_binding(&home, SLOT, &the_record(made, phase)).expect("the record writes");
}

/// What:
///  The slot's `.git` pointer naming the fixture's registration.
/// Why:
///  The recipe's first step removes only a pointer whose bytes name this
///  registration,
///  so the healthy arm needs one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeThePointer(made);
/// ```
pub(crate) fn write_the_pointer(made: &TheFixture) {
    std::fs::write(
        made.slot_path.join(".git"),
        format!("gitdir: {}\n", made.registration.display()),
    )
    .expect("the pointer writes");
}

/// What:
///  One dataset identity the scripted pool answers with.
/// Why:
///  The absence proof reads the clone's GUID and the parent's children,
///  so a control needs both without a pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = anIdentity(CLONE_GUID, 7);
/// ```
pub(crate) fn an_identity(
    guid: u64,
    createtxg: u64,
) -> DatasetIdentity {
    return DatasetIdentity {
        stats: stats(guid, false, createtxg, 0, ""),
        properties: NvList::new(),
    };
}

/// What:
///  A GUID as the fixture writes it.
/// Why:
///  The records and the probes take the type rather than a number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const guid = aGuid(CLONE_GUID);
/// ```
pub(crate) fn a_guid(value: u64) -> DatasetGuid {
    return DatasetGuid::parse(value.to_string().as_str()).expect("the fixture's number is a GUID");
}

/// What:
///  A pool that answers the names a control scripts,
///  lists the children a control scripts,
///  and records what it was asked.
/// Why:
///  The two answers the proof needs are one dataset's identity and the
///  parent's children,
///  and a listing that refuses is the state the command exists for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const probe = new ScriptedProbe({ answers, children, listing });
/// ```
pub(crate) struct ScriptedProbe {
    /// The identities by name,
    /// each with an error number where a control makes it fail.
    pub answers: Vec<(String, Result<DmuObjsetStats, ReadFailure>)>,
    /// The parent's children,
    /// in listing order.
    pub children: Vec<String>,
    /// What the child listing answers,
    /// which is a refusal when a control makes the absence unprovable.
    pub listing: Result<(), String>,
    /// Every name the proof asked about,
    /// in order.
    pub reads: RefCell<Vec<String>>,
}

impl ScriptedProbe {
    /// A pool that answers `answers`,
    /// lists no children and refuses nothing.
    pub(crate) fn answering(answers: Vec<(&str, DatasetIdentity)>) -> ScriptedProbe {
        return ScriptedProbe {
            answers: answers
                .into_iter()
                .map(|(name, identity)| return (String::from(name), Ok(identity.stats)))
                .collect(),
            children: Vec::new(),
            listing: Ok(()),
            reads: RefCell::new(Vec::new()),
        };
    }

    /// A pool that answers nothing,
    /// so every name it is asked about is absent.
    pub(crate) fn empty() -> ScriptedProbe {
        return ScriptedProbe {
            answers: Vec::new(),
            children: Vec::new(),
            listing: Ok(()),
            reads: RefCell::new(Vec::new()),
        };
    }

    /// The same pool listing these children.
    pub(crate) fn listing_children(
        mut self,
        children: &[&str],
    ) -> ScriptedProbe {
        self.children = children
            .iter()
            .map(|name| return String::from(*name))
            .collect();
        return self;
    }

    /// The same pool answering one name with one identity.
    pub(crate) fn answering_one(
        mut self,
        name: &str,
        identity: DatasetIdentity,
    ) -> ScriptedProbe {
        self.answers.push((String::from(name), Ok(identity.stats)));
        return self;
    }

    /// The same pool refusing the child listing,
    /// which is the absence a person must confirm.
    pub(crate) fn refusing_the_listing(
        mut self,
        said: &str,
    ) -> ScriptedProbe {
        self.listing = Err(String::from(said));
        return self;
    }
}

impl DatasetReader for ScriptedProbe {
    fn identity(
        &self,
        dataset: &str,
    ) -> Result<DatasetIdentity, ReadFailure> {
        self.reads.borrow_mut().push(String::from(dataset));
        return match self.answers.iter().find(|(name, _)| return name == dataset) {
            Some((_name, Ok(found))) => Ok(DatasetIdentity {
                stats: *found,
                properties: NvList::new(),
            }),
            Some((_name, Err(failure))) => Err(failure.clone()),
            None => Err(ReadFailure {
                said: format!("this scripted pool holds no `{dataset}`"),
                errno: Some(libc::ENOENT),
            }),
        };
    }

    fn next_child(
        &self,
        _parent: &str,
        cursor: u64,
    ) -> Result<Option<ListedChild>, String> {
        self.listing.as_ref().map_err(|said| return said.clone())?;
        let Some(name) = self.children.get(cursor as usize) else {
            return Ok(None);
        };
        return Ok(Some(ListedChild {
            name: name.clone(),
            cursor: cursor + 1,
        }));
    }
}

/// What:
///  The path a control's record file lives at.
/// Why:
///  Refusal sentences name the file,
///  so a control asserts the name it expects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = bindingPath(made);
/// ```
pub(crate) fn binding_path(made: &TheFixture) -> PathBuf {
    return made
        .common
        .join("monochromatic-git-policy-cli")
        .join("worktree-clones")
        .join("v1")
        .join(format!("{SLOT}.binding"));
}
