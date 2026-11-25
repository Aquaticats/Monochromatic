//! What:
//!  The throwaway repository,
//!  slot and record the managed lock's controls are built in.
//! Why:
//!  Every state of the lock is a set of files,
//!  and a control that built them by hand in each test would spell the record's grammar again,
//!  so one fixture writes the shapes Git and the add leave:
//!  a common directory,
//!  a slot folder holding a `.git`,
//!  a `worktrees/<name>` registration with its `gitdir`,
//!  `commondir` and `HEAD`,
//!  a `locked` file,
//!  and a binding record of the grammar the step 10 writer produces.
//!  The fixture is a plain directory tree,
//!  which is what the host controls of this piece run against;
//!  the guest arm is the one whose slot is a mounted clone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixture = Fixture.new('a-verified-lock'); fixture.writeRecord({ nonce }); fixture.writeRegistration('slot');
//! ```

use crate::testing::test_support::fixture;
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_binding_record::binding_words::BINDING_HEADER;
use crate::worktree_clone_binding_record::lock_words::lock_file_bytes;
use crate::worktree_clone_binding_record::registration_home::{binding_name, home_folder};
use crate::worktree_clone_managed_lock::dataset_guid_reads::DatasetGuidReads;
use crate::worktree_clone_mount_protocol::{DatasetGuid, RequestNonce};
use crate::zfs_mounted_dataset::mounted_dataset_instance::{MountHoldingAPath, MountedDatasetInstance, MountedDatasets};
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};

/// What:
///  A mount table a control writes,
///  instead of this machine's.
/// Why:
///  The mounted arm of the two-fact proof differs from the unmounted one,
///  so a control must show both without mounting anything:
///  `holding` is the answer for the path asked about,
///  or a refusal when it is `Err`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mounts = ScriptedMounts.nothing(); // or .coveredBy('pool/clones/locked-slot', slotPath)
/// ```
pub struct ScriptedMounts {
    /// What the path asked about holds,
    /// or why the table could not be read.
    pub holding: Result<Option<MountHoldingAPath>, String>,
    /// The rows a caller asking for the whole table reads.
    pub rows: Vec<MountedDatasetInstance>,
}

impl ScriptedMounts {
    /// No mount covers any path.
    pub fn nothing() -> ScriptedMounts {
        return ScriptedMounts { holding: Ok(None), rows: Vec::new() };
    }

    /// One dataset mounted at one path.
    pub fn a_mount_of(dataset: &str, at: &Path) -> ScriptedMounts {
        return ScriptedMounts {
            holding: Ok(Some(MountHoldingAPath { dataset: Some(String::from(dataset)), filesystem_type: String::from("zfs"), mount_point: at.to_path_buf() })),
            rows: vec![MountedDatasetInstance { dataset: String::from(dataset), mount_point: at.to_path_buf(), root: PathBuf::from("/") }],
        };
    }

    /// A table that cannot be read.
    pub fn unreadable(reason: &str) -> ScriptedMounts {
        return ScriptedMounts { holding: Err(String::from(reason)), rows: Vec::new() };
    }
}

impl MountedDatasets for ScriptedMounts {
    fn mounted_datasets(&self) -> Result<Vec<MountedDatasetInstance>, String> {
        return match &self.holding {
            Ok(_) => Ok(self.rows.clone()),
            Err(reason) => Err(reason.clone()),
        };
    }

    fn mount_holding(&self, path: &Path) -> Result<Option<MountHoldingAPath>, String> {
        // The trait answers the mount the path is *on*, so an unmounted slot under a
        // mounted parent answers its parent; a scripted table says only what it was given.
        return match &self.holding {
            Ok(Some(found)) if found.mount_point == path => Ok(Some(found.clone())),
            Ok(_covered_by_a_parent) => Ok(Some(MountHoldingAPath { dataset: Some(String::from("pool/monochromatic-git-checkout")), filesystem_type: String::from("zfs"), mount_point: path.parent().unwrap_or(path).to_path_buf() })),
            Err(reason) => Err(reason.clone()),
        };
    }
}

/// What:
///  The pool's answers a control writes,
///  instead of asking a pool.
/// Why:
///  The mounted arm compares a GUID,
///  so a control must answer one without a device,
///  and a control whose arm never reads the pool gets an answer that panics if it is asked,
///  which is how the unmounted arms prove they do not read it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const guids = ScriptedGuids.answering('pool/clones/locked-slot', '18446744073709551615'); // or .refusing(reason), or .neverAsked()
/// ```
pub struct ScriptedGuids {
    /// The datasets this control answers for,
    /// each with the answer it gives.
    pub answers: Vec<(String, Result<String, String>)>,
}

impl ScriptedGuids {
    /// One dataset answering the GUID the record holds.
    pub fn answering(dataset: &str, guid: &str) -> ScriptedGuids {
        return ScriptedGuids { answers: vec![(String::from(dataset), Ok(String::from(guid)))] };
    }

    /// One dataset whose read the pool refused.
    pub fn refusing(dataset: &str, reason: &str) -> ScriptedGuids {
        return ScriptedGuids { answers: vec![(String::from(dataset), Err(String::from(reason)))] };
    }

    /// A pool this arm must not read at all.
    pub fn never_asked() -> ScriptedGuids {
        return ScriptedGuids { answers: Vec::new() };
    }
}

impl DatasetGuidReads for ScriptedGuids {
    fn guid_of(&self, dataset: &str) -> Result<DatasetGuid, String> {
        for (named, answer) in &self.answers {
            if named == dataset {
                return match answer {
                    Ok(guid) => Ok(DatasetGuid::parse(guid.as_str()).expect("the control's GUID reads")),
                    Err(reason) => Err(reason.clone()),
                };
            }
        }
        panic!("the pool was asked for `{dataset}`, and this control answers no mount's read");
    }
}

/// The slot's name,
/// which is the record's own key.
pub const SLOT: &str = "locked-slot";

/// What:
///  One record's fields,
///  as a control plants them.
/// Why:
///  A control must be able to write a record whose nonce is known,
///  whose registration is absent or named,
///  and one with a planted damage,
///  so each field is the caller's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordSpec = { nonce, slotIdentity?, registration?, cloneDataset, cloneGuid?, extraLines: string[] };
/// ```
#[derive(Clone, Debug)]
pub struct RecordSpec {
    /// The add's nonce.
    pub nonce: String,
    /// The underlay folder's identity,
    /// as the record writes it.
    pub slot_identity: Option<String>,
    /// The registration's recorded path.
    pub registration: Option<PathBuf>,
    /// The registration directory's identity.
    pub registration_identity: Option<String>,
    /// The clone's dataset name.
    pub clone_dataset: String,
    /// The clone's GUID.
    pub clone_guid: Option<String>,
    /// Lines a control adds after the record's own,
    /// for a repeated key or one the grammar does not know.
    pub extra_lines: Vec<String>,
}

impl RecordSpec {
    /// A record for one nonce whose registration the caller names next.
    pub fn for_nonce(nonce: &str) -> RecordSpec {
        return RecordSpec {
            nonce: String::from(nonce),
            slot_identity: None,
            registration: None,
            registration_identity: None,
            clone_dataset: String::from("pool/clones/locked-slot"),
            clone_guid: Some(String::from("18446744073709551615")),
            extra_lines: Vec::new(),
        };
    }
}

/// What:
///  The fixture's paths,
///  all inside one throwaway directory of the test process.
/// Why:
///  A control names the common directory,
///  the slot and the registration,
///  and the fixture keeps them one object so no test rebuilds the layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Fixture = { root, commonDir, slot, slotPath };
/// ```
pub struct Fixture {
    /// The throwaway directory.
    pub root: PathBuf,
    /// The repository's Git common directory.
    pub common_dir: PathBuf,
    /// The slot's name.
    pub slot: String,
    /// The slot's whole path.
    pub slot_path: PathBuf,
}

impl Fixture {
    /// A fresh fixture with an empty common directory and an empty slot.
    pub fn new(label: &str) -> Fixture {
        let root: PathBuf = fixture(label);
        let common_dir: PathBuf = root.join("repository").join(".git");
        let slot_path: PathBuf = root.join("repository").join(".claude").join("worktrees").join(SLOT);
        std::fs::create_dir_all(common_dir.as_path()).expect("the common directory is made");
        std::fs::create_dir_all(slot_path.as_path()).expect("the slot is made");
        return Fixture { root, common_dir, slot: String::from(SLOT), slot_path };
    }

    /// The wrapper's record folder.
    pub fn home(&self) -> PathBuf {
        return home_folder(self.common_dir.as_path());
    }

    /// The folder Git keeps one directory per registration in.
    pub fn worktrees(&self) -> PathBuf {
        return self.common_dir.join("worktrees");
    }

    /// The slot folder's identity as a record writes it.
    pub fn slot_identity(&self) -> String {
        let identity: FolderIdentity = FolderIdentity::of_folder(self.slot_path.as_path()).expect("the slot's identity reads");
        return identity.encode();
    }

    /// Write this slot's binding record,
    /// private to the account as the add writes it.
    pub fn write_record(&self, spec: &RecordSpec) -> PathBuf {
        let home: PathBuf = self.home();
        std::fs::create_dir_all(home.as_path()).expect("the record folder is made");
        std::fs::set_permissions(home.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the record folder holds its mode");
        let mut text: String = String::from(BINDING_HEADER);
        text.push('\n');
        text.push_str(format!("claim=1\nnonce={}\nrequest=origin\nphase=registered\n", spec.nonce).as_str());
        text.push_str(format!("slot={}\n", crate::worktree_clone_binding_record::binding_encoding::path_as_hexadecimal(self.slot_path.as_path())).as_str());
        if let Some(identity) = spec.slot_identity.as_ref() {
            text.push_str(format!("slotIdentity={identity}\n").as_str());
        }
        if let Some(registration) = spec.registration.as_ref() {
            text.push_str(format!("registration={}\n", crate::worktree_clone_binding_record::binding_encoding::path_as_hexadecimal(registration.as_path())).as_str());
        }
        if let Some(identity) = spec.registration_identity.as_ref() {
            text.push_str(format!("registrationIdentity={identity}\n").as_str());
        }
        text.push_str(format!("cloneDataset={}\n", spec.clone_dataset).as_str());
        if let Some(guid) = spec.clone_guid.as_ref() {
            text.push_str(format!("cloneGuid={guid}\n").as_str());
        }
        for line in &spec.extra_lines {
            text.push_str(line.as_str());
            text.push('\n');
        }
        let path: PathBuf = home.join(binding_name(self.slot.as_str()));
        std::fs::write(path.as_path(), text.as_bytes()).expect("the record is written");
        std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o600)).expect("the record holds its mode");
        return path;
    }

    /// Make one `worktrees/<entry>` registration whose `gitdir` names this slot's `.git`,
    /// with `commondir` and `HEAD` as Git leaves them.
    pub fn write_registration(&self, entry: &str) -> PathBuf {
        let registration: PathBuf = self.worktrees().join(entry);
        std::fs::create_dir_all(registration.as_path()).expect("the registration is made");
        std::fs::write(registration.join("gitdir"), format!("{}\n", self.slot_path.join(".git").display()).as_bytes()).expect("the back-link is written");
        std::fs::write(registration.join("commondir"), b"../..\n").expect("the common directory's record is written");
        std::fs::write(registration.join("HEAD"), b"ref: refs/heads/main\n").expect("the head is written");
        return registration;
    }

    /// Write the slot's `.git` as Git writes it for one registration.
    pub fn write_pointer(&self, registration: &Path) {
        std::fs::write(self.slot_path.join(".git"), format!("gitdir: {}\n", registration.display()).as_bytes()).expect("the pointer is written");
    }

    /// Write one registration's `locked` file,
    /// with the mode Git's own write leaves.
    pub fn write_lock(&self, registration: &Path, bytes: &[u8]) -> PathBuf {
        let path: PathBuf = registration.join("locked");
        std::fs::write(path.as_path(), bytes).expect("the lock is written");
        std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o644)).expect("the lock holds its mode");
        return path;
    }

    /// The bytes Git's file holds for one nonce,
    /// which is the sentence and the newline.
    pub fn the_sentence(&self, nonce: &str) -> Vec<u8> {
        let parsed: RequestNonce = RequestNonce::parse(nonce).expect("the fixture nonce reads");
        return lock_file_bytes(&parsed);
    }
}
