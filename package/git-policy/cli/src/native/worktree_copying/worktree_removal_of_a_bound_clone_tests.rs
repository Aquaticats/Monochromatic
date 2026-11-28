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
    git,
    remove,
};
use crate::testing::fixture_zfs_binding_record::{
    RegisteredSlot,
    a_registered_slot,
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
use crate::worktree_clone_removal_proof_and_plan::plan_words::{
    HeadWord,
    SnapshotOutcome,
};
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{
    CLONE,
    CLONE_CREATETXG,
    CLONE_GUID,
    ORIGIN_CREATETXG,
    ORIGIN_DATASET,
    ORIGIN_SNAPSHOT,
    ORIGIN_SNAPSHOT_GUID,
    OWN_CREATETXG,
    OWN_GUID,
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
use crate::worktree_command_facts::{
    CopyFacts,
    RemovePlan,
};
use crate::worktree_identity::WorktreeIdentity;
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

/// One real registration of one slot,
/// which Git writes,
/// with the facts a removal carries built from it.
fn bound_slot(name: &str) -> RegisteredSlot {
    return a_registered_slot(name);
}

/// The facts a `git worktree remove --force` of one slot carries.
fn facts_of(bound: &RegisteredSlot) -> CopyFacts {
    return CopyFacts {
        identity: WorktreeIdentity::MainWorktree {
            common_dir: bound.common.clone(),
            git_dir: bound.common.clone(),
            worktree_root: bound.repository.clone(),
        },
        common_dir: Some(bound.common.clone()),
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
        common_dir: Some(fixture.common_dir.clone()),
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

/// The slot and the common directory a removal's facts name,
/// which are the two things the refusal reads,
/// or nothing when the facts name no removal at all.
fn the_slot_and_its_common(facts: &CopyFacts) -> Option<(PathBuf, PathBuf)> {
    let target: PathBuf = facts.removal_target.clone()?;
    let common: PathBuf = facts.common_dir.clone()?;
    return Some((target, common));
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

/// A mount table no control may read:
/// asking it for one fails the test.
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
fn refusal_of_or_none(facts: &CopyFacts) -> Option<BoundSlotRemovalAnswer> {
    let mounts: ScriptedMountTable = nothing_is_mounted();
    let intents: ScriptedIntent = no_intent_asked();
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &mounts,
        intents: &intents,
    };
    return refusal_of_or_none_with(facts, &sources);
}

/// The refusal's answer for one slot over the slot sources a control answers,
/// which the states that stay Git's ask for.
fn refusal_of_or_none_with(
    facts: &CopyFacts,
    sources: &SlotReadingSources<'_>,
) -> Option<BoundSlotRemovalAnswer> {
    let (target, common): (PathBuf, PathBuf) = the_slot_and_its_common(facts)?;
    return the_removal_with(
        target.as_path(),
        common.as_path(),
        RemovalReads::Unreadable(NO_POOL_WAS_ASKED),
        sources,
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
    let (target, common): (PathBuf, PathBuf) =
        the_slot_and_its_common(facts).expect("the facts name one removal");
    return match the_removal_with(target.as_path(), common.as_path(), reads, sources) {
        Some(BoundSlotRemovalAnswer {
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
    let bound: RegisteredSlot = bound_slot("removal-of-a-bound-clone");
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
        stderr.contains(crate::worktree_clone_binding_record::THE_REMOVALS_WORDS),
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
    let unrecorded: RegisteredSlot = bound_slot("removal-of-an-unrecorded-slot");
    assert!(refusal_of_or_none(&facts_of(&unrecorded)).is_none());
    let adding: RegisteredSlot = bound_slot("removal-of-a-pending-add");
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
///  A slot a destruction plan names is the removal's own,
///  and the lines it prints say what that run resumes.
/// Why:
///  The design's rule is that a removal reads the plan and then the live pinned
///  parent and that no other fact decides its path
///  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
///  "Which resume path a removal takes"),
///  and the slot state calls such a slot a pending removal whatever its binding's
///  phase says,
///  so the entry took only a mounted or an unmounted clone and left this one to
///  Git.
///  Git's own removal there drops the registration and the slot folder while the
///  plan and the clone stand,
///  which is the state the slot report warns about.
#[test]
fn a_slot_a_destruction_plan_names_is_the_removals_own() {
    let fixture: RemovalFixture = RemovalFixture::healthy("removal-of-a-pending-removal");
    let pool: ScriptedPool = fixture.healthy_pool();
    let plan: PathBuf = fixture.plant_a_plan(
        HeadWord::Present,
        vec![fixture.a_line_of_the_plan(
            OWN_SNAPSHOT,
            OWN_GUID,
            OWN_CREATETXG,
            0,
            SnapshotOutcome::Planned,
        )],
    );
    let before: Vec<(String, Option<Vec<u8>>)> = the_tree(fixture.common());
    let mounts: ScriptedMountTable = nothing_is_mounted();
    let intents: ScriptedIntent = no_intent_asked();
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &mounts,
        intents: &intents,
    };
    let left_to_git: Option<BoundSlotRemovalAnswer> = the_removal_with(
        fixture.slot_path.as_path(),
        fixture.common_dir.as_path(),
        RemovalReads::Scripted(&pool),
        &sources,
    );
    let answer: BoundSlotRemovalAnswer = left_to_git.expect(
        "a slot a destruction plan names is the removal's own, and Git's own removal would drop the registration while the clone stands",
    );
    assert_eq!(
        answer.code, REFUSED_EXIT_CODE,
        "a control's own pool answer reports what a removal would do rather than removing"
    );
    assert!(
        answer.stderr.contains("state: pending removal"),
        "{}",
        answer.stderr
    );
    assert!(
        answer
            .stderr
            .contains("stands, so this run resumes it: it reads the plan, proves every name the plan records again, sends no request for a name the plan records as destroyed, carries the loss forward and ends the plan with the removal's own records"),
        "{}",
        answer.stderr
    );
    assert!(
        answer
            .stderr
            .contains(format!("A removal's destruction plan at {}", plan.display()).as_str()),
        "the sentence names the plan's own file: {}",
        answer.stderr
    );
    assert_eq!(
        the_tree(fixture.common()),
        before,
        "the answer wrote nothing, the plan it read included"
    );
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
        stderr.contains(crate::worktree_clone_binding_record::THE_REMOVALS_WORDS),
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
///  A record filed under the slot's name that cannot be read refuses the removal,
///  and no mount table is read to decide it.
/// Why:
///  The alpha.47 pass's finding 12:
///  with the record overwritten by garbage the move refused while the removal
///  went to Git,
///  whose `-f -f` deleted the registration and the slot folder;
///  the record is the whole question,
///  so the answer comes before any mount or pool read
///  (`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`).
#[test]
fn a_record_that_cannot_be_read_refuses_the_removal() {
    let bound: RegisteredSlot = bound_slot("removal-of-an-unreadable-record");
    let home: PathBuf = crate::worktree_clone_binding_record::registration_home::home_folder(
        bound.common.as_path(),
    );
    std::fs::create_dir_all(home.as_path()).expect("the fixture home");
    std::fs::write(home.join("one.binding"), b"not a record\n").expect("the foreign record writes");
    let intents: ScriptedIntent = no_intent_asked();
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &ATableNoControlReads,
        intents: &intents,
    };
    let (target, common): (PathBuf, PathBuf) =
        the_slot_and_its_common(&facts_of(&bound)).expect("the facts name one removal");
    let refusal: BoundSlotRemovalAnswer = the_removal_with(
        target.as_path(),
        common.as_path(),
        RemovalReads::Unreadable("this control reads no pool"),
        &sources,
    )
    .expect("a record that cannot be read refuses the removal");
    assert_eq!(refusal.code, REFUSED_EXIT_CODE);
    assert!(
        refusal.stderr.contains("one.binding"),
        "the refusal names the record it could not read: {}",
        refusal.stderr
    );
    assert!(
        refusal.stderr.contains("cannot be read"),
        "{}",
        refusal.stderr
    );
    assert!(
        refusal.stderr.contains("restore it from a copy"),
        "the refusal names the repair: {}",
        refusal.stderr
    );
    remove(bound.root.as_path());
}

/// The pool sentence a control hands in where a refusal must read no pool:
/// its text in the report would prove the pool was read.
const NO_POOL_WAS_ASKED: &str = "this control reads no pool";

/// What:
///  A removal of a slot with no binding record and a mount standing at its path
///  is refused,
///  naming the mount,
///  and no pool is read.
/// Why:
///  The owner's answer of 2026-11-26:
///  a slot nothing proves is refused rather than left to Git,
///  because Git's own removal would delete the registration and the underlay
///  folder while the mount at the slot may hold a managed clone's files
///  (`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`).
#[test]
fn a_slot_with_no_record_and_a_mount_at_it_refuses_the_removal() {
    let bound: RegisteredSlot = bound_slot("removal-of-no-record-with-a-mount");
    // A repository the route has served holds its record folder, and this slot has no record in it.
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.root.join("two").as_path(),
            bound.registration.as_path(),
            Some(bound.pointer.clone()),
        ),
        "two",
    );
    let slot: String = bound
        .slot
        .to_str()
        .expect("the fixture slot path is utf-8")
        .to_string();
    let mounts: ScriptedMountTable = ScriptedMountTable::holding(&[
        row(1, 0, "/", "/", "zfs", "tank/ROOT"),
        row(50, 1, "/", slot.as_str(), "zfs", "tank/some-other-dataset"),
    ]);
    let intents: ScriptedIntent = no_intent_asked();
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &mounts,
        intents: &intents,
    };
    let (target, common): (PathBuf, PathBuf) =
        the_slot_and_its_common(&facts_of(&bound)).expect("the facts name one removal");
    let refusal: BoundSlotRemovalAnswer = the_removal_with(
        target.as_path(),
        common.as_path(),
        RemovalReads::Unreadable(NO_POOL_WAS_ASKED),
        &sources,
    )
    .expect("a mount no record names refuses the removal");
    assert_eq!(refusal.code, REFUSED_EXIT_CODE);
    assert!(
        refusal.stderr.contains("no record is filed under")
            && refusal.stderr.contains("tank/some-other-dataset"),
        "the refusal names the mount it could not prove: {}",
        refusal.stderr
    );
    assert!(
        !refusal.stderr.contains(NO_POOL_WAS_ASKED),
        "the refusal reads no pool: {}",
        refusal.stderr
    );
    assert!(bound.slot.is_dir(), "nothing was removed");
    remove(bound.root.as_path());
}

/// What:
///  A removal of a path whose slot name holds a record that names another path
///  is refused,
///  naming both paths,
///  without asking the clone's intent or the pool.
/// Why:
///  The owner's answer of 2026-11-26 refuses the removal of a slot no record
///  proves,
///  and a record keyed by this name for another path proves nothing about the
///  worktree the command names.
///  The intent and the pool are raw ZFS requests,
///  so the refusal decides before them,
///  which the readers this control hands in fail the test over.
#[test]
fn a_record_naming_another_path_refuses_the_removal() {
    let bound: RegisteredSlot = bound_slot("removal-of-a-record-of-another-path");
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
    let elsewhere: PathBuf = bound.root.join("elsewhere").join("one");
    std::fs::create_dir_all(elsewhere.as_path()).expect("the plain folder");
    let plain: PathBuf =
        std::fs::canonicalize(elsewhere.as_path()).expect("canonical plain folder");
    let mounts: ScriptedMountTable = nothing_is_mounted();
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &mounts,
        intents: &NoIntentMayBeAsked,
    };
    let refusal: BoundSlotRemovalAnswer = the_removal_with(
        plain.as_path(),
        bound.common.as_path(),
        RemovalReads::Unreadable(NO_POOL_WAS_ASKED),
        &sources,
    )
    .expect("a record of another path refuses the removal");
    assert_eq!(refusal.code, REFUSED_EXIT_CODE);
    assert!(
        refusal.stderr.contains("not this slot's record")
            && refusal
                .stderr
                .contains(bound.slot.to_str().expect("the fixture slot path is utf-8")),
        "the refusal names the path the record holds: {}",
        refusal.stderr
    );
    assert!(
        !refusal.stderr.contains(NO_POOL_WAS_ASKED),
        "the refusal reads no pool: {}",
        refusal.stderr
    );
    assert!(plain.is_dir(), "nothing was removed");
    remove(bound.root.as_path());
}

/// What:
///  A removal of a slot whose binding record says a whole clone was made,
///  at a moment the mount table cannot be read,
///  is refused and names the reader's reason,
///  before any pool read.
/// Why:
///  The owner's answer of 2026-11-26 refuses the removal where the mount table
///  cannot be read:
///  a clone the record calls filled may be mounted at the slot,
///  and Git's own `-f -f` removal would delete inside it
///  (the alpha.47 pass's finding 12).
#[test]
fn an_unreadable_mount_table_refuses_the_removal() {
    let bound: RegisteredSlot = bound_slot("removal-of-an-unreadable-mount-table");
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
    let mounts: ScriptedMountTable =
        ScriptedMountTable::unreadable("the table is too large to read");
    let intents: ScriptedIntent = no_intent_asked();
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &mounts,
        intents: &intents,
    };
    let (target, common): (PathBuf, PathBuf) =
        the_slot_and_its_common(&facts_of(&bound)).expect("the facts name one removal");
    let refusal: BoundSlotRemovalAnswer = the_removal_with(
        target.as_path(),
        common.as_path(),
        RemovalReads::Unreadable(NO_POOL_WAS_ASKED),
        &sources,
    )
    .expect("an unreadable mount table refuses the removal");
    assert_eq!(refusal.code, REFUSED_EXIT_CODE);
    assert!(
        refusal.stderr.contains("the mount table could not be read")
            && refusal.stderr.contains("the table is too large to read"),
        "the refusal names the reader's reason: {}",
        refusal.stderr
    );
    assert!(
        !refusal.stderr.contains(NO_POOL_WAS_ASKED),
        "the refusal reads no pool: {}",
        refusal.stderr
    );
    assert!(bound.slot.is_dir(), "nothing was removed");
    remove(bound.root.as_path());
}

/// What:
///  An unreadable mount table refuses the removal of a record that has reached the
///  phase that makes a clone,
///  and leaves the removal of an earlier add's phase to Git.
/// Why:
///  `SPEC.md` does not refuse a pending add's removal,
///  because the add's own refusal names `git worktree remove <slot>` as its way
///  forward,
///  but from `cloned` on a clone exists and only the table says whether it is
///  mounted at the slot,
///  which is the case the owner's answer of 2026-11-26 refuses.
#[test]
fn an_unreadable_mount_table_refuses_from_the_phase_that_makes_a_clone() {
    for (phase, refused) in [(Phase::Snapshotted, false), (Phase::Cloned, true)] {
        let bound: RegisteredSlot =
            bound_slot(&format!("removal-of-{}-unreadable-table", phase.word()));
        publish_under(
            bound.common.as_path(),
            &record_naming(
                phase,
                bound.slot.as_path(),
                bound.registration.as_path(),
                Some(bound.pointer.clone()),
            ),
            "one",
        );
        let mounts: ScriptedMountTable =
            ScriptedMountTable::unreadable("the table is too large to read");
        let intents: ScriptedIntent = no_intent_asked();
        let sources: SlotReadingSources<'_> = SlotReadingSources {
            mounts: &mounts,
            intents: &intents,
        };
        let answer: Option<BoundSlotRemovalAnswer> =
            refusal_of_or_none_with(&facts_of(&bound), &sources);
        assert_eq!(
            answer.is_some(),
            refused,
            "phase `{}` and an unreadable table: {answer:?}",
            phase.word()
        );
        remove(bound.root.as_path());
    }
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
    let bound: RegisteredSlot = bound_slot("removal-of-no-records");
    let intents: ScriptedIntent = no_intent_asked();
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &ATableNoControlReads,
        intents: &intents,
    };
    let (target, common): (PathBuf, PathBuf) =
        the_slot_and_its_common(&facts_of(&bound)).expect("the facts name one removal");
    assert!(
        the_removal_with(
            target.as_path(),
            common.as_path(),
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
    let bound: RegisteredSlot = bound_slot("removal-of-no-target");
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
///  This control answers the pool itself,
///  so the printed proof and loss are what a person gets from the release's
///  path without the device,
///  and the unchanged tree is the promise that nothing was unmounted or
///  recorded for a step the control cannot finish.
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
        stderr.contains(crate::worktree_clone_binding_record::THE_REMOVALS_WORDS),
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
        stderr.contains("the removal itself did not run here"),
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

/// An intent source no control may ask:
/// a question fails the test instead of answering.
/// Why:
///  The clone's intent is read through this machine's ZFS device,
///  and the control below proves the refusal never asks it for a worktree the
///  record does not name.
struct NoIntentMayBeAsked;

impl CloneIntentReads for NoIntentMayBeAsked {
    fn intent_of(
        &self,
        _clone: &str,
    ) -> Result<IntentReading, String> {
        panic!("the refusal asked the clone's intent before it compared the record's slot path");
    }
}

/// What:
///  A removal of a plain worktree that shares a bound slot's last name is
///  refused,
///  and neither the clone's intent nor the pool is asked.
/// Why:
///  The record is keyed by the slot's last name,
///  so a removal of another worktree with that last name reads a record that
///  names a different path,
///  and the owner's answer of 2026-11-26 refuses a removal no record proves.
///  The read must not ask the clone's intent or the plan folder first:
///  that was one guarded open of the kernel's ZFS device for a command that
///  endangers nothing (the alpha.47 pass's Low finding 5),
///  and the readers this control hands in fail the test if they are asked.
#[test]
fn a_plain_worktree_that_shares_a_slots_last_name_opens_no_device() {
    let bound: RegisteredSlot = bound_slot("removal-of-a-same-leaf-worktree");
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
    let plain_made: PathBuf = bound.root.join("elsewhere").join("one");
    std::fs::create_dir_all(bound.root.join("elsewhere")).expect("elsewhere");
    git(
        bound.repository.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            plain_made.to_str().expect("utf-8"),
            "HEAD",
        ],
    );
    let plain: PathBuf =
        std::fs::canonicalize(plain_made.as_path()).expect("canonical plain worktree");
    let mounts: ScriptedMountTable = nothing_is_mounted();
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &mounts,
        intents: &NoIntentMayBeAsked,
    };
    let refusal: BoundSlotRemovalAnswer = the_removal_with(
        plain.as_path(),
        bound.common.as_path(),
        RemovalReads::Unreadable(NO_POOL_WAS_ASKED),
        &sources,
    )
    .expect("a record that names another path refuses the removal without asking the pool");
    assert!(
        refusal.stderr.contains("not this slot's record"),
        "the refusal names the record's own path: {}",
        refusal.stderr
    );
    assert!(
        !refusal.stderr.contains(NO_POOL_WAS_ASKED),
        "the refusal reads no pool: {}",
        refusal.stderr
    );
    assert!(plain.is_dir(), "the plain worktree stands");
    assert!(bound.slot.is_dir(), "the bound slot stands");
    remove(bound.root.as_path());
}

/// What:
///  The unreadable record's sentence names no command this release lacks,
///  and each piece of advice beside it does what the sentence says:
///  restoring the record from a copy brings the proof back,
///  and deleting it makes the removal Git's own,
///  which removes the registration and the slot folder for a slot nothing then
///  proves.
/// Why:
///  The sentence used to say "settle it",
/// and no management grammar of this
///  release admits the settling command
///  (`git monochromatic-git-policy-cli settle-worktree-clone-record` answers the
///  usage text with exit 2,
/// as the sibling sentences changed by `5defef5371`
///  say);
///  the record's own reason offers "delete it and run the command again",
/// and
///  the sentence now says what that costs.
#[test]
fn restoring_or_deleting_an_unreadable_record_does_what_the_sentence_says() {
    let bound: RegisteredSlot = bound_slot("removal-advice-after-an-unreadable-record");
    let home: PathBuf = crate::worktree_clone_binding_record::registration_home::home_folder(
        bound.common.as_path(),
    );
    std::fs::create_dir_all(home.as_path()).expect("the fixture home");
    let record: PathBuf = home.join("one.binding");
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
    let copy: PathBuf = bound.root.join("one.binding.copy");
    std::fs::copy(record.as_path(), copy.as_path()).expect("the record is copied");
    std::fs::write(record.as_path(), b"not a record\n").expect("the record is overwritten");
    // The ladder's own fast removal stays out, so the run below is Git's own removal once the refusal answers
    //  nothing.
    let mut facts: CopyFacts = facts_of(&bound);
    facts.remove_plan = None;
    let stderr: String = refusal_of(&facts, RemovalReads::Unreadable(NO_POOL_WAS_ASKED));
    assert!(stderr.contains("cannot be read"), "{stderr}");
    assert!(
        !stderr.contains("settle it"),
        "the sentence sends nobody to a settling command: {stderr}"
    );
    assert!(
        !stderr.contains("settle-worktree-clone-record"),
        "and names none: {stderr}"
    );
    assert!(
        stderr.contains("Deleting it makes the removal Git's own"),
        "the sentence says what deleting the record does: {stderr}"
    );
    assert!(
        stderr.contains("reachable by nobody"),
        "and what a clone bound to the slot then loses: {stderr}"
    );
    // "restore it from a copy": the record reads again, and the refusal is the one a whole clone gets.
    std::fs::copy(copy.as_path(), record.as_path()).expect("the record is restored");
    let restored: String = refusal_of(&facts, RemovalReads::Unreadable(NO_POOL_WAS_ASKED));
    assert!(
        restored.contains("a whole managed clone is bound to this slot"),
        "the restored record proves the slot again: {restored}"
    );
    // "delete it and run the command again": nothing files under the slot name and nothing is mounted at it,
    //  so the refusal answers nothing and Git's own removal ends well.
    std::fs::remove_file(record.as_path()).expect("the record goes");
    let words: Vec<OsString> = [
        "-C",
        bound.repository.to_str().expect("utf-8"),
        "worktree",
        "remove",
        "-f",
        "-f",
        "one",
    ]
    .iter()
    .map(|word| return OsString::from(word))
    .collect();
    let run: crate::worktree_copy_forward::ForwardedRun =
        crate::worktree_copy_forward::run_forwarded_git(
            Path::new(crate::test_support::REAL_GIT),
            words.as_slice(),
            crate::test_support::isolated_overlay().as_slice(),
            Some(&facts),
        );
    assert_eq!(
        run,
        crate::worktree_copy_forward::ForwardedRun::Ran(0),
        "Git's own removal ends well once no record proves the slot"
    );
    assert!(!bound.slot.exists(), "Git removed the slot folder");
    assert!(!bound.registration.exists(), "and its registration");
    remove(bound.root.as_path());
}
