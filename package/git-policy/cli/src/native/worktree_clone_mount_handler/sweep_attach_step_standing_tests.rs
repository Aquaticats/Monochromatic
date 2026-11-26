//! What:
//!  The four answers the attach step gives a slot that already stands on a mount
//!  or holds none:
//!  the recorded clone's own mount,
//!  another dataset's mount,
//!  a mount that carries this clone's name but is not its own mount,
//!  and an empty slot.
//! Why:
//!  A request for a clone already attached at its slot must be answered `done`
//!  without a second attach
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  A5),
//!  and the walk never crosses a mount,
//!  so the step asks the sweep's own standing proof and answers that mount where
//!  the proof names this clone's own.
//!  Everything else at the slot keeps the walk's refusal,
//!  and the three refusals are what keep the accept from being "any mount will
//!  do":
//!  another dataset's mount,
//!  a bind of this clone (the same source,
//!  another root) and a mount whose two enforced attributes are gone are each
//!  not this clone's own mount.
//!  A ZFS dataset cannot be mounted on a person's machine,
//!  so the slot's tmpfs stands in for the mount and the shared fixture shapes
//!  its table row into a dataset's own mount;
//!  every other field,
//!  including the attributes the third case removes,
//!  is the kernel's own answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(step.attach(one)).toEqual({ kind: 'already-standing', mount }); // its own mount, no mount call
//! expect(step.attach(one)).toEqual({ kind: 'refused', step: 'walk' });   // anything else at the slot
//! ```

use super::{
    AttachRequest,
    AttachStepOutcome,
    CloneAttachStep,
    CloneMountCall,
    RealAttachStep,
    RollbackCalls,
};
use crate::fs_beneath_mount_facts::mount_id_of;
use crate::testing::root_namespace_test_support::in_a_root_namespace;
use crate::testing::standing_slot_mounts::{
    MountedSlot,
    the_slots_mount_reads,
};
use crate::worktree_clone_mount::{
    AttachCall,
    AttachRefusal,
    AttachedMount,
    CloneFolderIdentity,
    Placement,
    publish_parent_identity_under_root_control,
    publish_slot_identity_under_root_control,
};
use crate::worktree_clone_mount_handler::sweep_standing_mounts::{
    LiveStandingMounts,
    StandingMounts,
};
use std::cell::Cell;
use std::os::unix::fs::PermissionsExt;
use std::path::{
    Path,
    PathBuf,
};

/// The variable the namespace helper sets in the half that really runs,
/// named for this module's controls.
const INSIDE: &str = "CLI_GIT_SWEEP_ATTACH_STANDING_ROOT";

/// The clone every control asks about.
const CLONE: &str = "zpcachyos/nosnap/clones/one";

/// A dataset that is not the clone,
/// which a slot may hold instead.
const ANOTHER_DATASET: &str = "zpcachyos/nosnap/clones/two";

/// The root a bind of a dataset shows instead of `/`.
const A_BIND_ROOT: &str = "/.claude/worktrees/one";

/// One fixture:
/// a checkout whose slot's identity the store records,
/// so the walk passes every component except the mount itself.
struct Fixture {
    /// The checkout the walk starts at.
    checkout: PathBuf,
    /// The slot's folder.
    slot: PathBuf,
    /// The store of recorded identities.
    store: PathBuf,
}

impl Fixture {
    /// Build the fixture under this run's own temporary root.
    fn build(name: &str) -> Self {
        let root: PathBuf =
            crate::test_support::fixture(format!("sweep-attach-standing-{name}").as_str())
                .join("checkout");
        let parent: PathBuf = root.join(".claude").join("worktrees");
        let slot: PathBuf = parent.join("one");
        std::fs::create_dir_all(slot.as_path()).expect("the fixture chain is made");
        let store: PathBuf = root.parent().expect("the fixture has a root").join("store");
        std::fs::create_dir(store.as_path()).expect("the store folder is made");
        std::fs::set_permissions(store.as_path(), std::fs::Permissions::from_mode(0o755))
            .expect("the store is root's and readable");
        let identity: CloneFolderIdentity = identity_of(parent.as_path());
        publish_parent_identity_under_root_control(store.as_path(), &identity, &identity)
            .expect("the parent record stands");
        let slot_identity: CloneFolderIdentity = identity_of(slot.as_path());
        publish_slot_identity_under_root_control(store.as_path(), "one", &slot_identity, &identity)
            .expect("the slot's record stands");
        return Self {
            checkout: root,
            slot,
            store,
        };
    }

    /// The request the sweep hands the step.
    fn request(&self) -> AttachRequest<'_> {
        return AttachRequest {
            checkout: self.checkout.as_path(),
            slot: "one",
            clone: CLONE,
            store: self.store.as_path(),
        };
    }
}

/// One folder's identity now.
fn identity_of(path: &Path) -> CloneFolderIdentity {
    return CloneFolderIdentity::capture(&std::fs::File::open(path).expect("the folder opens"))
        .expect("the identity reads");
}

/// The mount the kernel says the fixture's slot is on.
fn the_slots_mount(path: &Path) -> u64 {
    return mount_id_of(&std::fs::File::open(path).expect("the slot opens"))
        .expect("the slot's mount reads");
}

/// What:
///  One mount call a control answers:
///  it counts its calls and refuses at the move,
///  so a slot the walk reaches is shown to reach the attach and no mount is
///  made.
/// Why:
///  A fixture cannot make a mount of a ZFS dataset,
///  and a call that answered a mount would make the post-attach proofs judge a
///  mount that is not there and roll a real fixture folder back;
///  the count is what tells "the walk reached the attach" from "the step
///  answered a standing mount".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const call = ACountedRefusingMountCall(asked); // count++, throw EBUSY at move_mount
/// ```
struct ACountedRefusingMountCall<'a> {
    /// How many times the step asked it to mount,
    /// which the control keeps so it can read the count after the step owns the
    /// call.
    asked: &'a Cell<usize>,
}

impl CloneMountCall for ACountedRefusingMountCall<'_> {
    fn attach_the_mount(
        &self,
        source: &str,
        _target: &std::fs::File,
    ) -> Result<AttachedMount, AttachRefusal> {
        self.asked.set(self.asked.get() + 1);
        return Err(AttachRefusal {
            call: AttachCall::Place,
            source: String::from(source),
            errno: libc::EBUSY,
            detail: String::from("a control's call makes no mount"),
            placement: Placement::NothingPlaced,
        });
    }
}

/// What:
///  The rollback's own calls as a control answers them:
///  the module's answers never reach one,
///  so a call here is a control that has gone wrong.
/// Why:
///  The release calls would unmount a real mount a person's machine owns,
///  and a silent stand-in would hide the day an answer stops being what this
///  module proves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const calls = new NothingToRollBack(); // every call throws
/// ```
struct NothingToRollBack;

impl RollbackCalls for NothingToRollBack {
    fn unmount(
        &self,
        _target: &std::fs::File,
    ) -> Result<(), AttachRefusal> {
        panic!("no control of this module reaches a rollback's unmount");
    }

    fn slot_mount(
        &self,
        _request: &AttachRequest<'_>,
    ) -> Result<u64, String> {
        panic!("no control of this module reads the slot back for a rollback");
    }
}

/// What:
///  The step over one standing proof,
///  with a mount call that counts its calls into `asked`.
/// Why:
///  Whether a second attach was attempted is what separates the answer from a
///  refusal,
///  and the count outlives the step because the step owns the call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const step = theStep(standing, asked);
/// ```
fn the_step<'a>(
    standing: &'a dyn StandingMounts,
    asked: &'a Cell<usize>,
) -> RealAttachStep<'a, NothingToRollBack, ACountedRefusingMountCall<'a>> {
    return RealAttachStep {
        calls: NothingToRollBack,
        mount_call: ACountedRefusingMountCall {
            asked,
        },
        standing,
    };
}

/// What:
///  A slot holding the recorded clone's own mount is answered from that mount,
///  and the step makes no mount of its own.
/// Why:
///  A clone whose intent is `mount` keeps that intent after it attached,
///  so the ordinary repeated request is for a clone already at its slot;
///  the answer names the mount the kernel shows there,
///  which is what a waiting command confirms,
///  and no second attach is attempted at a mount the kernel would refuse.
#[test]
fn a_slot_holding_the_recorded_clones_mount_is_answered_from_it() {
    if !in_a_root_namespace(
        INSIDE,
        "worktree_clone_mount_handler::sweep_attach_step::standing_tests::a_slot_holding_the_recorded_clones_mount_is_answered_from_it",
    ) {
        return;
    }
    let fixture: Fixture = Fixture::build("own-mount");
    let slot_mount: MountedSlot = MountedSlot::at(fixture.slot.as_path());
    let standing: LiveStandingMounts = the_slots_mount_reads(CLONE, "/");
    let asked: Cell<usize> = Cell::new(0);
    let step = the_step(&standing, &asked);
    let outcome: AttachStepOutcome = step.attach(&fixture.request());
    let AttachStepOutcome::AlreadyStanding {
        mount,
    } = outcome
    else {
        panic!("the clone's own mount at its slot is answered from that mount: {outcome:?}");
    };
    assert_eq!(
        mount.mount_id,
        the_slots_mount(slot_mount.path()),
        "the answer names the mount the kernel shows at the slot"
    );
    assert_eq!(
        mount.source, CLONE,
        "the answer names this clone as the mount's source"
    );
    assert_eq!(
        mount.observed_path,
        std::fs::canonicalize(fixture.slot.as_path()).expect("the slot's canonical path"),
        "the answer names the slot the mount stands at"
    );
    assert_eq!(
        asked.get(),
        0,
        "no second attach is attempted at the mount the slot holds"
    );
}

/// What:
///  A slot holding another dataset's mount keeps the walk's refusal.
/// Why:
///  The proof asks the kernel for the source of the one mount at the slot,
///  and another dataset's name is not this clone's,
///  so no answer of this step may put the mount at the slot down as this
///  clone's own.
#[test]
fn a_slot_holding_another_datasets_mount_keeps_the_walks_refusal() {
    if !in_a_root_namespace(
        INSIDE,
        "worktree_clone_mount_handler::sweep_attach_step::standing_tests::a_slot_holding_another_datasets_mount_keeps_the_walks_refusal",
    ) {
        return;
    }
    let fixture: Fixture = Fixture::build("another-dataset");
    let _slot_mount: MountedSlot = MountedSlot::at(fixture.slot.as_path());
    let standing: LiveStandingMounts = the_slots_mount_reads(ANOTHER_DATASET, "/");
    let asked: Cell<usize> = Cell::new(0);
    let step = the_step(&standing, &asked);
    let outcome: AttachStepOutcome = step.attach(&fixture.request());
    let AttachStepOutcome::Refused {
        step: refused_at,
        errno,
        said,
    } = outcome
    else {
        panic!("another dataset's mount is refused as the walk refuses it: {outcome:?}");
    };
    assert_eq!(refused_at, "walk", "the refusal is the walk's own step");
    assert_eq!(errno, None, "a boundary refusal names no errno");
    assert!(
        said.contains("never crosses such a boundary"),
        "the sentence is the walk's boundary refusal: {said}"
    );
    assert_eq!(
        asked.get(),
        0,
        "nothing is mounted over another dataset's mount"
    );
}

/// What:
///  A slot holding a mount that carries this clone's name but is not its own
///  mount keeps the walk's refusal.
/// Why:
///  A bind of a dataset shows the dataset's name as its source,
///  and the route's identity rule tells such a mount from a dataset's own by the
///  root it shows;
///  a slot answered as this clone's own on its source alone would accept a bind
///  the walk refuses.
#[test]
fn a_slot_holding_a_bind_of_this_clone_keeps_the_walks_refusal() {
    if !in_a_root_namespace(
        INSIDE,
        "worktree_clone_mount_handler::sweep_attach_step::standing_tests::a_slot_holding_a_bind_of_this_clone_keeps_the_walks_refusal",
    ) {
        return;
    }
    let fixture: Fixture = Fixture::build("same-name-bind");
    let _slot_mount: MountedSlot = MountedSlot::at(fixture.slot.as_path());
    let standing: LiveStandingMounts = the_slots_mount_reads(CLONE, A_BIND_ROOT);
    let asked: Cell<usize> = Cell::new(0);
    let step = the_step(&standing, &asked);
    let outcome: AttachStepOutcome = step.attach(&fixture.request());
    let AttachStepOutcome::Refused {
        step: refused_at,
        said,
        ..
    } = outcome
    else {
        panic!("a bind of this clone is refused as the walk refuses it: {outcome:?}");
    };
    assert_eq!(refused_at, "walk", "the refusal is the walk's own step");
    assert!(
        said.contains("never crosses such a boundary"),
        "the sentence is the walk's boundary refusal: {said}"
    );
    assert_eq!(
        asked.get(),
        0,
        "nothing is mounted over a mount that is not this clone's own"
    );
}

/// What:
///  An empty slot is walked and reaches the step's own mount call,
///  which is the answer the standing proof must not take over.
/// Why:
///  This is the control the three occupied-slot answers are read against:
///  a step that answered a standing mount for every slot,
///  or that stopped asking the mount call at all,
///  would pass all three and never attach a clone.
///  The call refuses,
///  because a fixture cannot mount a ZFS dataset and a mount that is not there
///  would make the post-attach proofs judge it,
///  so the refusal is the call's own step rather than the walk's.
#[test]
fn an_empty_slot_is_walked_and_reaches_the_steps_own_mount_call() {
    if !in_a_root_namespace(
        INSIDE,
        "worktree_clone_mount_handler::sweep_attach_step::standing_tests::an_empty_slot_is_walked_and_reaches_the_steps_own_mount_call",
    ) {
        return;
    }
    let fixture: Fixture = Fixture::build("empty-slot");
    let standing: LiveStandingMounts = the_slots_mount_reads(CLONE, "/");
    let asked: Cell<usize> = Cell::new(0);
    let step = the_step(&standing, &asked);
    let outcome: AttachStepOutcome = step.attach(&fixture.request());
    assert_eq!(
        asked.get(),
        1,
        "an empty slot reaches the step's own mount call"
    );
    let AttachStepOutcome::Refused {
        step: ref refused_at,
        ref errno,
        ..
    } = outcome
    else {
        panic!(
            "an empty slot is walked and attached rather than answered from a mount: {outcome:?}"
        );
    };
    assert_eq!(
        refused_at, "move-mount",
        "the answer is the mount call's own step, so the walk reached the attach"
    );
    assert_eq!(
        errno.as_deref(),
        Some("EBUSY"),
        "the call's own errno is kept: {outcome:?}"
    );
}
