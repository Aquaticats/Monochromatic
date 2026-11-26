//! Controls for the refusal of a removal of a slot a whole managed clone is
//! bound to:
//! the two states that refuse,
//! the states that stay Git's,
//! the read-only proof a whole clone's refusal prints,
//! every way that proof stops,
//! and the one promise they share,
//! that the command writes no byte and creates no plan.

use super::*;
use crate::test_support::{
    fixture,
    git,
    remove,
};
use crate::testing::fixture_zfs_binding_record::{
    publish_under,
    record_naming,
};
use crate::testing::mounted_dataset_table::{
    ScriptedMountTable,
    row,
};
use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_mount_protocol::clone_mount_intent::IntentReading;
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{
    CLONE,
    CLONE_CREATETXG,
    CLONE_GUID,
    ORIGIN_CREATETXG,
    ORIGIN_DATASET,
    ORIGIN_SNAPSHOT,
    ORIGIN_SNAPSHOT_GUID,
    OWN_SNAPSHOT,
    POOL,
    POOL_GUID,
    RemovalFixture,
    SANOID_SNAPSHOT,
    SLOT,
    ScriptedPool,
    an_object,
    the_stats,
};
use crate::worktree_clone_slot_state::slot_state_live_read::{
    CloneIntentReads,
    SlotReadingSources,
};
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_remove_plan::RemovePlan;
use crate::zfs_mounted_dataset::mounted_dataset_instance::{
    MountHoldingAPath,
    MountedDatasetInstance,
    MountedDatasets,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DatasetIdentity,
    NvList,
};
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// One fixture slot with a real registration,
/// its record,
/// and the facts a removal carries.
struct Bound {
    /// The fixture root.
    root: PathBuf,
    /// The Git common directory.
    common: PathBuf,
    /// The slot's path.
    slot: PathBuf,
    /// The bytes Git wrote into the slot.
    pointer: Vec<u8>,
    /// The registration's path.
    registration: PathBuf,
}

/// One real registration of one slot.
fn bound_slot(name: &str) -> Bound {
    let root: PathBuf = fixture(name);
    let repository: PathBuf = root.join("repository");
    std::fs::create_dir(repository.as_path()).expect("repository");
    git(repository.as_path(), &["init", "--quiet"]);
    git(
        repository.as_path(),
        &["commit", "--quiet", "--allow-empty", "--message", "base"],
    );
    let slot: PathBuf = root.join("one");
    git(
        repository.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            slot.to_str().expect("utf-8"),
            "HEAD",
        ],
    );
    let common: PathBuf = repository.join(".git");
    let pointer: Vec<u8> = std::fs::read(slot.join(".git")).expect("pointer reads");
    let registration: PathBuf = common.join("worktrees").join("one");
    return Bound {
        root,
        common,
        slot,
        pointer,
        registration,
    };
}

/// The facts a `git worktree remove --force` of one slot carries.
fn facts_of(bound: &Bound) -> CopyFacts {
    return CopyFacts {
        identity: WorktreeIdentity::MainWorktree {
            common_dir: bound.common.clone(),
            git_dir: bound.common.clone(),
            worktree_root: bound.root.join("repository"),
        },
        creates: false,
        destination: None,
        add_plan: None,
        remove_plan: Some(RemovePlan {
            target: bound.slot.clone(),
            operand: OsString::from("one"),
            force: true,
        }),
        removal_target: Some(bound.slot.clone()),
        quiet: false,
    };
}

/// The facts a `git worktree remove --force` of the removal fixture's slot carries.
fn facts_of_the_fixture(fixture: &RemovalFixture) -> CopyFacts {
    return CopyFacts {
        identity: WorktreeIdentity::MainWorktree {
            common_dir: fixture.common_dir.clone(),
            git_dir: fixture.common_dir.clone(),
            worktree_root: fixture.common_dir.clone(),
        },
        creates: false,
        destination: None,
        add_plan: None,
        remove_plan: Some(RemovePlan {
            target: fixture.slot_path.clone(),
            operand: OsString::from(SLOT),
            force: true,
        }),
        removal_target: Some(fixture.slot_path.clone()),
        quiet: false,
    };
}

/// What:
///  Every name under one folder and the bytes each file holds,
///  sorted by name.
/// Why:
///  The refusal promises that it wrote nothing,
///  and a tree of the same names and bytes is what proves it:
///  no plan,
///  no intent,
///  and no record rewritten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tree = (root) => walk(root).map(({ name, bytes }) => [name, bytes]);
/// ```
fn the_tree(root: &Path) -> Vec<(String, Option<Vec<u8>>)> {
    let mut found: Vec<(String, Option<Vec<u8>>)> = Vec::new();
    walk(root, root, &mut found);
    found.sort();
    return found;
}

/// One folder's entries,
/// each with its own contents.
fn walk(
    root: &Path,
    folder: &Path,
    found: &mut Vec<(String, Option<Vec<u8>>)>,
) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for listed in entries {
        let listed_entry = listed.expect("the folder lists");
        let path: PathBuf = listed_entry.path();
        let name: String = path
            .strip_prefix(root)
            .expect("the entry is under the root")
            .display()
            .to_string();
        if path.is_dir() {
            found.push((name, None));
            walk(root, path.as_path(), found);
        } else {
            found.push((
                name,
                Some(std::fs::read(path.as_path()).expect("the entry reads")),
            ));
        }
    }
}

/// The refusal's printed lines,
/// or the panic that says the command was left to Git.
fn refusal_of(
    facts: &CopyFacts,
    reads: RemovalReads<'_>,
) -> String {
    let mounts: ScriptedMountTable = nothing_is_mounted();
    let intents: ScriptedIntent = no_intent_asked();
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &mounts,
        intents: &intents,
    };
    return refusal_of_with(facts, reads, &sources);
}

/// One mount table in which nothing stands at any slot,
/// so a control decides the slot's state without reading this machine's mounts.
fn nothing_is_mounted() -> ScriptedMountTable {
    return ScriptedMountTable::holding(&[row(1, 0, "/", "/", "zfs", "tank/ROOT")]);
}

/// A mount table no control may read: asking it for one fails the test.
/// Why:
///  The refusal answers nothing for a repository with no record folder,
///  and a table that fails when read proves it never looked.
struct ATableNoControlReads;

impl MountedDatasets for ATableNoControlReads {
    fn mounted_datasets(&self) -> Result<Vec<MountedDatasetInstance>, String> {
        panic!("the refusal read the mount table before it looked for a record");
    }

    fn mount_holding(
        &self,
        _path: &Path,
    ) -> Result<Option<MountHoldingAPath>, String> {
        panic!("the refusal read the mount at the slot before it looked for a record");
    }
}

/// The refusal's answer for one slot with nothing mounted and no intent set,
/// which the states that stay Git's ask for.
fn refusal_of_or_none(facts: &CopyFacts) -> Option<ForwardedRun> {
    let mounts: ScriptedMountTable = nothing_is_mounted();
    let intents: ScriptedIntent = no_intent_asked();
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &mounts,
        intents: &intents,
    };
    return refuse_the_removal_with(
        facts,
        RemovalReads::Unreadable("this control reads no pool"),
        &sources,
    );
}

/// One scripted mount-intent source.
/// Why:
///  The clone's intent is a raw ZFS request,
///  so a host control answers it instead of opening this machine's device.
struct ScriptedIntent {
    /// The reading every question is answered with,
    /// or the refusal the device would have given.
    answer: Result<IntentReading, String>,
}

impl CloneIntentReads for ScriptedIntent {
    fn intent_of(
        &self,
        _clone: &str,
    ) -> Result<IntentReading, String> {
        return self.answer.clone();
    }
}

/// The intent source a control that says nothing about the intent answers with:
/// a clone whose property was never set,
/// which is what the add leaves behind.
fn no_intent_asked() -> ScriptedIntent {
    return ScriptedIntent {
        answer: Ok(IntentReading::Absent),
    };
}

/// The refusal's printed lines over the reads and the slot sources a control answers.
fn refusal_of_with(
    facts: &CopyFacts,
    reads: RemovalReads<'_>,
    sources: &SlotReadingSources<'_>,
) -> String {
    return match refuse_the_removal_with(facts, reads, sources) {
        Some(ForwardedRun::Failed {
            stderr,
            code,
        }) => {
            assert_eq!(
                code, REFUSED_EXIT_CODE,
                "a refusal ends with the wrapper's own status"
            );
            stderr
        }
        other => panic!("a bound slot's removal refuses: {other:?}"),
    };
}

/// What:
///  A removal of a slot whose record proves a whole worktree is refused,
///  and the report names the slot and the clone a Git removal would orphan.
/// Why:
///  Git's own removal deletes the registration and the underlay pointer while
///  the clone dataset keeps the files,
///  which is the loss this refusal prevents in a release with no clone
///  removal.
#[test]
fn a_whole_worktree_refuses_its_removal() {
    let bound: Bound = bound_slot("removal-of-a-bound-clone");
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.slot.as_path(),
            bound.registration.as_path(),
            Some(bound.pointer.clone()),
        ),
        "one",
    );
    let stderr: String = refusal_of(
        &facts_of(&bound),
        RemovalReads::Unreadable("this control reads no pool"),
    );
    assert!(
        stderr.contains("this release has no removal for a clone"),
        "{stderr}"
    );
    assert!(stderr.contains("state: known unmounted clone"), "{stderr}");
    assert!(stderr.contains("next safe action:"), "{stderr}");
    assert!(
        stderr.contains("the removal's lineage proof could not run"),
        "a record that never reached the clone is reported rather than guessed at: {stderr}"
    );
    assert!(bound.slot.is_dir(), "nothing was removed");
    assert!(bound.registration.is_dir(), "the registration stands");
    remove(bound.root.as_path());
}

/// What:
///  A slot with no record,
///  and a slot whose record says the add is unfinished,
///  are left to Git.
/// Why:
///  The add's own refusal names `git worktree remove <slot>` as its way
///  forward,
///  and a slot no record names is not this route's to refuse.
#[test]
fn every_other_slot_stays_gits() {
    let unrecorded: Bound = bound_slot("removal-of-an-unrecorded-slot");
    assert!(refusal_of_or_none(&facts_of(&unrecorded)).is_none());
    let adding: Bound = bound_slot("removal-of-a-pending-add");
    publish_under(
        adding.common.as_path(),
        &record_naming(
            Phase::Snapshotted,
            adding.slot.as_path(),
            adding.registration.as_path(),
            Some(adding.pointer.clone()),
        ),
        "one",
    );
    assert!(refusal_of_or_none(&facts_of(&adding)).is_none());
    remove(unrecorded.root.as_path());
    remove(adding.root.as_path());
}

/// What:
///  A `git worktree remove` that names a bound slot is refused in every
///  spelling Git accepts,
///  the double force included,
///  because `SPEC.md` exempts none and Git's own lock hint names `-f -f`.
/// Why:
///  The double force is the spelling Git itself suggests for a locked
///  registration,
///  so it is the one a person reaches for,
///  and letting it through deletes the registration and the underlay pointer
///  while the clone stands.
#[test]
fn every_spelling_of_a_bound_slots_removal_is_refused() {
    let fixture: RemovalFixture = RemovalFixture::healthy("command-path-every-spelling");
    let pool: ScriptedPool = fixture.healthy_pool();
    let before: Vec<(String, Option<Vec<u8>>)> = the_tree(fixture.common());
    let mut facts: CopyFacts = facts_of_the_fixture(&fixture);
    // The double force is no plain form, so the served plan is absent and only the refusal's own reading names the slot.
    facts.remove_plan = None;
    let stderr: String = refusal_of(&facts, RemovalReads::Scripted(&pool));
    assert!(
        stderr.contains("this release has no removal for a clone"),
        "{stderr}"
    );
    assert!(stderr.contains("state: known unmounted clone"), "{stderr}");
    assert!(
        stderr.contains(&format!("the clone `{CLONE}` (GUID `{CLONE_GUID}`")),
        "{stderr}"
    );
    assert_eq!(
        the_tree(fixture.common()),
        before,
        "the refusal wrote nothing"
    );
    assert_eq!(fixture.the_plan_folder_holds(), 0);
}

/// What:
///  A repository with no record folder answers nothing without reading the
///  mount table.
/// Why:
///  Every removal in an ordinary repository is this case,
///  and the mount table is a file read per command that no record can need:
///  the table this control hands in fails the test if it is asked anything.
#[test]
fn a_repository_without_records_reads_no_mount_table() {
    let bound: Bound = bound_slot("removal-of-no-records");
    let intents: ScriptedIntent = no_intent_asked();
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &ATableNoControlReads,
        intents: &intents,
    };
    assert!(
        refuse_the_removal_with(
            &facts_of(&bound),
            RemovalReads::Unreadable("this control reads no pool"),
            &sources
        )
        .is_none()
    );
    remove(bound.root.as_path());
}

/// What:
///  A run that names no worktree to remove,
///  which is every other command,
///  answers nothing.
/// Why:
///  The refusal exists for one removal,
///  and a command that names no directory must not read a slot at all.
#[test]
fn a_command_without_a_removal_target_answers_nothing() {
    let bound: Bound = bound_slot("removal-of-no-target");
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.slot.as_path(),
            bound.registration.as_path(),
            Some(bound.pointer.clone()),
        ),
        "one",
    );
    let mut facts: CopyFacts = facts_of(&bound);
    facts.remove_plan = None;
    facts.removal_target = None;
    assert!(refusal_of_or_none(&facts).is_none());
    remove(bound.root.as_path());
}

/// What:
///  The refusal reads the clone's mount intent through the reader it was given,
///  not through this machine's ZFS device.
/// Why:
///  The slot report prints what that reader answered,
///  so a control that names its own answer proves the command path asks the
///  seam and a host control opens no device.
#[test]
fn the_refusal_asks_the_intent_reader_it_was_given() {
    let fixture: RemovalFixture = RemovalFixture::healthy("command-path-intent-seam");
    let pool: ScriptedPool = fixture.healthy_pool();
    let before: Vec<(String, Option<Vec<u8>>)> = the_tree(fixture.common());
    let mounts: ScriptedMountTable = nothing_is_mounted();
    let intents: ScriptedIntent = ScriptedIntent {
        answer: Err(String::from("this control reads no intent")),
    };
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &mounts,
        intents: &intents,
    };
    let stderr: String = refusal_of_with(
        &facts_of_the_fixture(&fixture),
        RemovalReads::Scripted(&pool),
        &sources,
    );
    assert!(
        stderr.contains("this control reads no intent"),
        "the report names the reader's own answer: {stderr}"
    );
    assert_eq!(
        the_tree(fixture.common()),
        before,
        "the refusal wrote nothing"
    );
    assert_eq!(fixture.the_plan_folder_holds(), 0);
}

/// What:
///  A bound clone's removal prints what its proof matched and every name a
///  removal would lose,
///  and writes no record,
///  no intent and no plan.
/// Why:
///  This release removes no clone,
///  so the printed proof and loss are what a person gets,
///  and the unchanged tree is the promise that nothing was unmounted or
///  recorded for a step this release cannot finish.
#[test]
fn a_bound_clone_prints_the_proof_and_the_loss_and_writes_nothing() {
    let fixture: RemovalFixture = RemovalFixture::healthy("command-path-healthy");
    let pool: ScriptedPool = fixture.healthy_pool();
    let before: Vec<(String, Option<Vec<u8>>)> = the_tree(fixture.common());
    let stderr: String = refusal_of(
        &facts_of_the_fixture(&fixture),
        RemovalReads::Scripted(&pool),
    );
    // The refusal a person reads is this whole block, so a log of one run keeps its shape beside the assertions.
    println!("{stderr}");
    assert!(
        stderr.contains("this release has no removal for a clone"),
        "{stderr}"
    );
    assert!(stderr.contains("state: known unmounted clone"), "{stderr}");
    assert!(
        stderr.contains(&format!("the clone `{CLONE}` (GUID `{CLONE_GUID}`")),
        "{stderr}"
    );
    assert!(
        stderr.contains(&format!(
            "descends from the origin snapshot `{ORIGIN_SNAPSHOT}`"
        )),
        "{stderr}"
    );
    assert!(
        stderr.contains(&format!("snapshot {CLONE}@{OWN_SNAPSHOT}")),
        "{stderr}"
    );
    assert!(
        stderr.contains(&format!("snapshot {CLONE}@{SANOID_SNAPSHOT}")),
        "{stderr}"
    );
    assert!(
        stderr.contains("this release does not remove a clone yet"),
        "{stderr}"
    );
    assert_eq!(
        the_tree(fixture.common()),
        before,
        "the command wrote no record, no intent and no plan"
    );
    assert_eq!(
        fixture.the_plan_folder_holds(),
        0,
        "no destruction plan was written"
    );
}

/// What:
///  A promoted clone's removal is refused with the proof's own sentence,
///  before anything is written.
/// Why:
///  Item 11 measured that a promoted clone's transferred history is the
///  checkout's own,
///  so the proof refuses and names the origin the pool reports beside the
///  recorded one.
#[test]
fn a_promoted_clone_is_refused_by_the_proof_and_nothing_is_written() {
    let fixture: RemovalFixture = RemovalFixture::healthy("command-path-promoted");
    let promoted: ScriptedPool = fixture.healthy_pool().with(
        CLONE,
        DatasetIdentity {
            stats: the_stats(CLONE_GUID, false, CLONE_CREATETXG, 0, ORIGIN_DATASET),
            properties: NvList::new(),
        },
    );
    let before: Vec<(String, Option<Vec<u8>>)> = the_tree(fixture.common());
    let stderr: String = refusal_of(
        &facts_of_the_fixture(&fixture),
        RemovalReads::Scripted(&promoted),
    );
    assert!(
        stderr.contains("the removal's own checks stopped it before anything was written"),
        "{stderr}"
    );
    assert!(
        stderr.contains(&format!(
            "the clone `{CLONE}` answers with origin `{ORIGIN_DATASET}`"
        )),
        "{stderr}"
    );
    assert!(
        stderr.contains(&format!("the add recorded `{ORIGIN_SNAPSHOT}`")),
        "{stderr}"
    );
    assert!(
        !stderr.contains("the lineage proof a removal makes passed"),
        "a promoted clone's proof does not pass: {stderr}"
    );
    assert_eq!(
        the_tree(fixture.common()),
        before,
        "the refusal wrote nothing"
    );
    assert_eq!(fixture.the_plan_folder_holds(), 0);
}

/// What:
///  A snapshot the clone inherited refuses the removal by name,
///  and every protected object the control's pool lists is still listed.
/// Why:
///  The plan's refusal arm asks for the promotion control and the survival of
///  every protected snapshot and bookmark,
///  and the command's own answer is where a person reads it.
#[test]
fn a_transferred_snapshot_is_refused_and_every_protected_object_survives() {
    let fixture: RemovalFixture = RemovalFixture::healthy("command-path-transferred");
    let mut pool: ScriptedPool = fixture.healthy_pool();
    pool.snapshots.push(an_object(
        EntryKind::Snapshot,
        "earliest",
        ORIGIN_SNAPSHOT_GUID,
        ORIGIN_CREATETXG,
    ));
    let listed: Vec<String> = pool
        .snapshots
        .iter()
        .map(|object| return object.full_name())
        .collect();
    let stderr: String = refusal_of(
        &facts_of_the_fixture(&fixture),
        RemovalReads::Scripted(&pool),
    );
    assert!(
        stderr.contains(&format!("the live snapshot `{CLONE}@earliest`")),
        "{stderr}"
    );
    assert!(stderr.contains("history this clone inherited"), "{stderr}");
    assert_eq!(
        pool.snapshots
            .iter()
            .map(|object| return object.full_name())
            .collect::<Vec<String>>(),
        listed,
        "every protected snapshot is still there"
    );
    assert_eq!(fixture.the_plan_folder_holds(), 0);
}

/// What:
///  A bookmark neither a live snapshot nor a plan line explains is refused by
///  name through the command,
///  and no snapshot or bookmark is destroyed.
/// Why:
///  The rule refuses the whole removal and names each bookmark,
///  which is the command's answer a person reads before their own
///  `zfs destroy`.
#[test]
fn an_unexplained_bookmark_is_refused_by_name_through_the_command() {
    let fixture: RemovalFixture = RemovalFixture::healthy("command-path-bookmark");
    let mut pool: ScriptedPool = fixture.healthy_pool();
    pool.bookmarks.push(an_object(
        EntryKind::Bookmark,
        "from-a-gone-snapshot",
        4242,
        19,
    ));
    let stderr: String = refusal_of(
        &facts_of_the_fixture(&fixture),
        RemovalReads::Scripted(&pool),
    );
    assert!(
        stderr.contains(&format!("`{CLONE}#from-a-gone-snapshot`")),
        "{stderr}"
    );
    assert!(
        stderr.contains(
            "the removal stops before its first delete with every snapshot and bookmark still there"
        ),
        "{stderr}"
    );
    assert_eq!(fixture.the_plan_folder_holds(), 0);
}

/// What:
///  A pool this build cannot read still ends in the same refusal,
///  with the reader's reason printed and nothing changed.
/// Why:
///  The gate may refuse a module this build does not know,
///  and a person then learns why the proof did not run instead of reading a
///  refusal that claims a proof it never made.
#[test]
fn a_pool_that_cannot_be_read_still_refuses_and_names_the_reason() {
    let fixture: RemovalFixture = RemovalFixture::healthy("command-path-unreadable-pool");
    let before: Vec<(String, Option<Vec<u8>>)> = the_tree(fixture.common());
    let stderr: String = refusal_of(
        &facts_of_the_fixture(&fixture),
        RemovalReads::Unreadable("the ZFS compatibility gate refused before any pool request"),
    );
    assert!(stderr.contains("the removal's lineage proof could not run, because the ZFS compatibility gate refused before any pool request"), "{stderr}");
    assert!(
        stderr.contains("every snapshot, bookmark, record and registration still stands"),
        "{stderr}"
    );
    assert_eq!(the_tree(fixture.common()), before, "nothing was written");
}

/// What:
///  The pool a live command reads is the release's own device:
///  the proof names the pool the add recorded when the device answers,
///  and the first check is the pool's own GUID.
/// Why:
///  The three checks each refuse on one recorded identity,
///  and a control that could only reach the scripted arm would leave the
///  release answer unexercised.
#[test]
fn the_scripted_pool_reaches_every_identity_the_proof_compares() {
    let fixture: RemovalFixture = RemovalFixture::healthy("command-path-identities");
    let another_pool: ScriptedPool = fixture.healthy_pool().with(
        POOL,
        DatasetIdentity {
            stats: the_stats(POOL_GUID + 1, false, 1, 0, ""),
            properties: NvList::new(),
        },
    );
    let stderr: String = refusal_of(
        &facts_of_the_fixture(&fixture),
        RemovalReads::Scripted(&another_pool),
    );
    assert!(
        stderr.contains(&format!("`{POOL}` answers with poolGuid")),
        "{stderr}"
    );
    assert!(stderr.contains("not the one the add made"), "{stderr}");
    assert!(
        stderr.contains(&format!("the add recorded `{POOL_GUID}`")),
        "{stderr}"
    );
    assert_eq!(fixture.the_plan_folder_holds(), 0);
}
