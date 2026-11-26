//! What:
//!  Controls of the attach step's seam:
//!  the request a sweep hands in,
//!  and the four answers a caller must tell apart.
//! Why:
//!  The real step mounts a real clone and only the throwaway guest may run it,
//!  so what this host proves is that a control's step is driven with the
//!  request the sweep built,
//!  and that a retained mount and a rollback are different answers rather than
//!  one "failure".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(step.attach(request)).toEqual({ kind: 'attached', mount });
//! ```

use super::{
    AttachRequest,
    AttachStepOutcome,
    CloneAttachStep,
    CloneMountCall,
    RealAttachStep,
    RealRollbackCalls,
    RollbackCalls,
    roll_back_a_mount,
};
use crate::testing::standing_slot_mounts::the_slots_mount_reads;
use crate::worktree_clone_mount::{
    CloneFolderIdentity,
    HeldCloneSlot,
    HoldCloneSlotRequest,
    MountIdentity,
    observe_the_clone_slot,
    publish_parent_identity_under_root_control,
    publish_slot_identity_under_root_control,
};
use crate::worktree_clone_mount_handler::sweep_standing_mounts::LiveStandingMounts;
use std::cell::RefCell;
use std::os::unix::fs::PermissionsExt;
use std::path::{
    Path,
    PathBuf,
};

/// A step that answers from a script and remembers what it was asked,
/// which is how a control sees the sweep's own request.
struct ScriptedStep {
    /// The answer it gives.
    answer: AttachStepOutcome,
    /// The requests it was given.
    asked: RefCell<Vec<(PathBuf, String, String, PathBuf)>>,
}

impl CloneAttachStep for ScriptedStep {
    fn attach(
        &self,
        request: &AttachRequest<'_>,
    ) -> AttachStepOutcome {
        self.asked.borrow_mut().push((
            request.checkout.to_path_buf(),
            String::from(request.slot),
            String::from(request.clone),
            request.store.to_path_buf(),
        ));
        return self.answer.clone();
    }
}

/// The mount a control's step says it made.
fn mounted() -> MountIdentity {
    return MountIdentity {
        mount_id: 41,
        parent_mount_id: 40,
        mount_point: PathBuf::from("/home/user/Monochromatic/.claude/worktrees/one"),
        source: String::from("zpcachyos/nosnap/clones/one"),
        file_system_type: String::from("zfs"),
        device: 57,
        inode: 1_337,
        observed_path: PathBuf::from("/home/user/Monochromatic/.claude/worktrees/one"),
    };
}

/// What:
///  The answers one step can give stay separate answers,
///  and each reads as what happened rather than as a summary.
/// Why:
///  A receipt word is one of the protocol's own and `done` covers both a mount
///  this step made and this clone's own mount it found,
///  so the step's own answer must still tell the two apart:
///  an attach claims the post-attach proofs ran and a standing mount claims
///  none of them did.
#[test]
fn the_steps_answers_stay_separate() {
    let answers: [AttachStepOutcome; 4] = [
        AttachStepOutcome::AlreadyStanding {
            mount: mounted(),
        },
        AttachStepOutcome::RolledBack {
            said: String::from(
                "the mount 41 left the kernel through the descriptor the attach acted on",
            ),
        },
        AttachStepOutcome::Retained {
            said: String::from("the mount 41 is still attached at /tmp/elsewhere"),
        },
        AttachStepOutcome::Refused {
            step: String::from("walk"),
            errno: None,
            said: String::from("the walk refused the slot"),
        },
    ];
    for answer in answers {
        assert_ne!(
            answer,
            AttachStepOutcome::Attached {
                mount: mounted()
            },
            "only a proved place is an attach: {answer:?}"
        );
    }
    assert_ne!(
        AttachStepOutcome::AlreadyStanding {
            mount: mounted()
        },
        AttachStepOutcome::Attached {
            mount: mounted()
        },
        "a mount this step made and a mount it found standing are different answers even for one mount"
    );
    assert_ne!(
        AttachStepOutcome::RolledBack {
            said: String::from("left")
        },
        AttachStepOutcome::Retained {
            said: String::from("left")
        },
        "a rolled-back mount and a retained one are different answers even when their sentences start alike"
    );
}

/// The step is handed the checkout and the slot and the clone and the store the sweep
/// built,
/// and nothing else.
#[test]
fn the_step_is_handed_the_four_names_of_one_clone() {
    let step: ScriptedStep = ScriptedStep {
        answer: AttachStepOutcome::Attached {
            mount: mounted(),
        },
        asked: RefCell::new(Vec::new()),
    };
    let request: AttachRequest<'_> = AttachRequest {
        checkout: Path::new("/home/user/Monochromatic"),
        slot: "one",
        clone: "zpcachyos/nosnap/clones/one",
        store: Path::new(
            "/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities",
        ),
    };
    let outcome: AttachStepOutcome = step.attach(&request);
    assert_eq!(
        outcome,
        AttachStepOutcome::Attached {
            mount: mounted()
        }
    );
    let asked = step.asked.borrow();
    assert_eq!(asked.len(), 1);
    let (checkout, slot, clone, store) = &asked[0];
    assert_eq!(checkout, Path::new("/home/user/Monochromatic"));
    assert_eq!(slot, "one");
    assert_eq!(clone, "zpcachyos/nosnap/clones/one");
    assert_eq!(
        store,
        Path::new("/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities")
    );
}

/// What:
///  A rollback whose unmount answered success but whose slot still shows a mount
///  is retained,
///  not reported as rolled back.
/// Why:
///  The success text this step prints says the slot holds no mount,
///  and an unmount that answered zero is not evidence of absence:
///  the project's rollback rule is that a mount which cannot be recorded is
///  taken down and only a proved clear slot ends the step,
///  so the read-back is what makes the sentence true
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Misplaced attach").
#[test]
fn a_rollback_whose_slot_still_holds_a_mount_is_retained() {
    if !crate::testing::root_namespace_test_support::in_a_root_namespace(
        "CLI_GIT_SWEEP_ATTACH_ROLLBACK_ROOT",
        "worktree_clone_mount_handler::sweep_attach_step::tests::a_rollback_whose_slot_still_holds_a_mount_is_retained",
    ) {
        return;
    }
    /// A rollback whose unmount answers success and whose slot always shows the
    /// mount the attach made.
    struct StillMounted {
        /// What the slot is on after the unmount answered.
        mount_id: u64,
    }

    impl RollbackCalls for StillMounted {
        fn unmount(
            &self,
            _target: &std::fs::File,
        ) -> Result<(), crate::worktree_clone_mount::AttachRefusal> {
            return Ok(());
        }

        fn slot_mount(
            &self,
            _request: &AttachRequest<'_>,
        ) -> Result<u64, String> {
            return Ok(self.mount_id);
        }
    }

    let checkout: PathBuf = crate::test_support::fixture("sweep-attach-rollback").join("checkout");
    let parent: PathBuf = checkout.join(".claude").join("worktrees");
    std::fs::create_dir_all(parent.join("one").as_path()).expect("the fixture chain is made");
    let store: PathBuf = checkout
        .parent()
        .expect("the fixture has a root")
        .join("store");
    std::fs::create_dir(store.as_path()).expect("the store folder is made");
    std::fs::set_permissions(store.as_path(), std::fs::Permissions::from_mode(0o755))
        .expect("the store is root's and readable");
    let identity: CloneFolderIdentity = CloneFolderIdentity::capture(
        &std::fs::File::open(parent.as_path()).expect("the parent opens"),
    )
    .expect("the identity reads");
    publish_parent_identity_under_root_control(store.as_path(), &identity, &identity)
        .expect("the parent record stands");
    let request: AttachRequest<'_> = AttachRequest {
        checkout: checkout.as_path(),
        slot: "one",
        clone: "zpcachyos/nosnap/clones/one",
        store: store.as_path(),
    };
    let held: HeldCloneSlot = observe_the_clone_slot(&HoldCloneSlotRequest {
        checkout: checkout.as_path(),
        slot: "one",
        store: store.as_path(),
    })
    .expect("the fixture slot is held");

    // The unmount answered success and the slot is still on the mount the attach made, so the rollback is not done.
    let still: StillMounted = StillMounted {
        mount_id: 41,
    };
    let outcome: AttachStepOutcome = roll_back_a_mount(
        &request,
        &held,
        41,
        40,
        String::from("the mount landed beside the slot"),
        &still,
    );
    assert!(
        matches!(outcome, AttachStepOutcome::Retained { .. }),
        "a slot still on a mount is retained: {outcome:?}"
    );
    assert!(
        outcome_about(&outcome).contains("still on mount 41"),
        "the report names the mount the slot shows: {}",
        outcome_about(&outcome)
    );

    // The positive control: the same call whose slot is back on the parent folder's own mount is a rollback.
    struct Clear {
        /// What the slot is on after the unmount answered.
        mount_id: u64,
    }

    impl RollbackCalls for Clear {
        fn unmount(
            &self,
            _target: &std::fs::File,
        ) -> Result<(), crate::worktree_clone_mount::AttachRefusal> {
            return Ok(());
        }

        fn slot_mount(
            &self,
            _request: &AttachRequest<'_>,
        ) -> Result<u64, String> {
            return Ok(self.mount_id);
        }
    }

    let clear: Clear = Clear {
        mount_id: 40,
    };
    let rolled: AttachStepOutcome = roll_back_a_mount(
        &request,
        &held,
        41,
        40,
        String::from("the mount landed beside the slot"),
        &clear,
    );
    assert!(
        matches!(rolled, AttachStepOutcome::RolledBack { .. }),
        "a slot on the parent's own mount is clear: {rolled:?}"
    );
    assert!(
        outcome_about(&rolled).contains("holds no mount"),
        "the report says the slot holds no mount: {}",
        outcome_about(&rolled)
    );

    // A read-back that cannot be made is no proof either, so the mount is retained.
    struct Unreadable;

    impl RollbackCalls for Unreadable {
        fn unmount(
            &self,
            _target: &std::fs::File,
        ) -> Result<(), crate::worktree_clone_mount::AttachRefusal> {
            return Ok(());
        }

        fn slot_mount(
            &self,
            _request: &AttachRequest<'_>,
        ) -> Result<u64, String> {
            return Err(String::from("the slot could not be read back"));
        }
    }

    let unreadable: AttachStepOutcome = roll_back_a_mount(
        &request,
        &held,
        41,
        40,
        String::from("the mount landed beside the slot"),
        &Unreadable,
    );
    assert!(
        matches!(unreadable, AttachStepOutcome::Retained { .. }),
        "an unreadable slot proves nothing: {unreadable:?}"
    );
    assert!(
        outcome_about(&unreadable).contains("could not be read back"),
        "the report keeps the read's words: {}",
        outcome_about(&unreadable)
    );
}

/// What:
///  The real attach step answers a mount call that placed a mount and could not
///  take it down as retained,
///  not as a refusal that left nothing behind.
/// Why:
///  The mapping is the step's own call,
///  and a retained mount is the answer that files a failure receipt and makes
///  `boot-reconcile` publish `failed`;
///  a step that reduced it to "refused" would lose the only durable notice of
///  the mount.
#[test]
fn the_real_attach_step_answers_a_retained_mount_as_retained() {
    if !crate::testing::root_namespace_test_support::in_a_root_namespace(
        "CLI_GIT_SWEEP_ATTACH_RETAINED_ROOT",
        "worktree_clone_mount_handler::sweep_attach_step::tests::the_real_attach_step_answers_a_retained_mount_as_retained",
    ) {
        return;
    }
    use crate::worktree_clone_mount::{
        AttachCall,
        AttachRefusal,
        AttachedMount,
        Placement,
    };

    struct RetainingMount {
        /// The refusal the mount call answers.
        refusal: AttachRefusal,
    }

    impl CloneMountCall for RetainingMount {
        fn attach_the_mount(
            &self,
            _source: &str,
            _target: &std::fs::File,
        ) -> Result<AttachedMount, AttachRefusal> {
            return Err(self.refusal.clone());
        }
    }

    let checkout: PathBuf = crate::test_support::fixture("sweep-attach-retained").join("checkout");
    let parent: PathBuf = checkout.join(".claude").join("worktrees");
    std::fs::create_dir_all(parent.join("one").as_path()).expect("the fixture chain is made");
    let store: PathBuf = checkout
        .parent()
        .expect("the fixture has a root")
        .join("store");
    std::fs::create_dir(store.as_path()).expect("the store folder is made");
    std::fs::set_permissions(store.as_path(), std::fs::Permissions::from_mode(0o755))
        .expect("the store is root's and readable");
    let identity: CloneFolderIdentity = CloneFolderIdentity::capture(
        &std::fs::File::open(parent.as_path()).expect("the parent opens"),
    )
    .expect("the identity reads");
    publish_parent_identity_under_root_control(store.as_path(), &identity, &identity)
        .expect("the parent record stands");
    let slot_identity: CloneFolderIdentity = CloneFolderIdentity::capture(
        &std::fs::File::open(parent.join("one").as_path()).expect("the slot opens"),
    )
    .expect("the slot identity reads");
    publish_slot_identity_under_root_control(store.as_path(), "one", &slot_identity, &identity)
        .expect("the slot record stands");
    let request: AttachRequest<'_> = AttachRequest {
        checkout: checkout.as_path(),
        slot: "one",
        clone: "zpcachyos/nosnap/clones/one",
        store: store.as_path(),
    };
    let refusal: AttachRefusal = AttachRefusal {
        call: AttachCall::MountNumber,
        source: String::from("zpcachyos/nosnap/clones/one"),
        errno: libc::EIO,
        detail: String::from("the mount number could not be read"),
        placement: Placement::PlacedAndRetained,
    };
    let standing: LiveStandingMounts = the_slots_mount_reads("zpcachyos/nosnap/clones/one", "/");
    let step: RealAttachStep<'_, RealRollbackCalls, RetainingMount> = RealAttachStep {
        calls: RealRollbackCalls,
        mount_call: RetainingMount {
            refusal,
        },
        standing: &standing,
    };
    let outcome: AttachStepOutcome = step.attach(&request);
    assert!(
        matches!(outcome, AttachStepOutcome::Retained { .. }),
        "a mount that stands is retained: {outcome:?}"
    );
    assert!(
        outcome_about(&outcome).contains("could not be taken down"),
        "the answer says what became of the mount: {}",
        outcome_about(&outcome)
    );
}

/// The sentence one outcome carries,
/// for the rollback controls that compare it.
fn outcome_about(outcome: &AttachStepOutcome) -> String {
    return match outcome {
        AttachStepOutcome::Attached {
            mount,
        }
        | AttachStepOutcome::AlreadyStanding {
            mount,
        } => mount.describe(),
        AttachStepOutcome::RolledBack {
            said,
        }
        | AttachStepOutcome::Retained {
            said,
        } => said.clone(),
        AttachStepOutcome::Refused {
            said,
            ..
        } => said.clone(),
    };
}

/// What:
///  A refused attach call that placed a mount and could not take it down is
///  answered as retained,
///  and one that took it down again is answered as refused.
/// Why:
///  Only the retained answer files a failure receipt,
///  and a mount left standing must never read as "nothing was mounted".
#[test]
fn a_placed_mount_that_could_not_be_taken_down_is_retained() {
    use crate::worktree_clone_mount::{
        AttachCall,
        AttachRefusal,
        Placement,
    };

    let retained: AttachRefusal = AttachRefusal {
        call: AttachCall::MountNumber,
        source: String::from("zpcachyos/nosnap/clones/one"),
        errno: libc::EIO,
        detail: String::from("no such file or directory"),
        placement: Placement::PlacedAndRetained,
    };
    let outcome: AttachStepOutcome = super::the_outcome_of_a_refused_attach(&retained);
    assert!(
        matches!(outcome, AttachStepOutcome::Retained { .. }),
        "a mount that stands is retained: {outcome:?}"
    );
    assert!(
        outcome_about(&outcome).contains("could not be taken down"),
        "the report says what became of the mount: {}",
        outcome_about(&outcome)
    );

    let taken_down: AttachRefusal = AttachRefusal {
        placement: Placement::PlacedThenTakenDown,
        ..retained.clone()
    };
    let refused: AttachStepOutcome = super::the_outcome_of_a_refused_attach(&taken_down);
    assert!(
        matches!(refused, AttachStepOutcome::Refused { ref step, .. } if step == "mount-id"),
        "a mount taken down again is a refusal: {refused:?}"
    );

    let nothing: AttachRefusal = AttachRefusal {
        call: AttachCall::MountDescriptor,
        placement: Placement::NothingPlaced,
        ..retained
    };
    let before_the_move: AttachStepOutcome = super::the_outcome_of_a_refused_attach(&nothing);
    assert!(
        matches!(before_the_move, AttachStepOutcome::Refused { ref step, .. } if step == "fsmount"),
        "{before_the_move:?}"
    );
}
