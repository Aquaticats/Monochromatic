//! What:
//!  Controls of one mount's turn:
//!  the idle mount is unmounted from the held descriptor,
//!  a slot another handler holds is left to it,
//!  a busy mount is kept and names its holder,
//!  a busy mount whose scan could not be made says so,
//!  any other kernel refusal leaves the mount,
//!  a mount whose checks refuse makes no kernel call at all,
//!  and the claim is released however the turn ends.
//! Why:
//!  The design forbids conflicting actions on one mount and says a busy clone
//!  is named and never forced,
//!  so each arm is a mount a person reads a line about and a kernel that was
//!  called or was not
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit";
//!  "Requests,
//! results and ordering").
//!  The unmount is the double's answer and the slot is a fixture folder,
//!  so no control mounts,
//!  unmounts or reads `/run`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reached = unmountOne(request, checkout, recordedParent, candidate);
//! reached.ending; // { kind: 'unmounted', mountId } | { kind: 'held' } | { kind: 'left' }
//! ```

use crate::fs_mount_options::MountFacts;
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount::clone_folder_identity::CloneFolderIdentity;
use crate::worktree_clone_mount::slot_identity_store::SLOT_IDENTITY_STORE_PATH;
use crate::worktree_clone_mount_handler::per_slot_root_claim::SlotClaimDirectory;
use crate::worktree_clone_mount_protocol::clone_mount_readiness_file::ReadinessFile;
use crate::worktree_clone_shutdown_unmount_pass::one_mounts_unmount::unmount_one;
use crate::worktree_clone_shutdown_unmount_pass::shutdown_unmount_answer::{
    Holder,
    MountEnding,
    ReachedMount,
};
use crate::worktree_clone_shutdown_unmount_pass::shutdown_unmount_pass_test_doubles::{
    StatedCalls,
    StatedGuids,
    StatedHolders,
    StatedLookups,
    StatedMounts,
    StatedUnmounts,
    StatedWalk,
    StatedWorktrees,
};
use crate::worktree_clone_shutdown_unmount_pass::shutdown_unmount_pass_test_fixture_values::{
    CLONE,
    MOUNT_ID,
    PARENT_MOUNT_ID,
    busy,
    clone_guid,
    fresh_lookup_of,
    identity_of,
    mount_row,
    refusal,
    scan_finding_one_holder,
    scan_with_one_incomplete_read,
};
use crate::worktree_clone_shutdown_unmount_pass::shutdown_unmount_places::ShutdownUnmountPlaces;
use crate::worktree_clone_shutdown_unmount_pass::shutdown_unmount_request::ShutdownUnmountRequest;
use crate::worktree_clone_shutdown_unmount_pass::unmountable_mounts::{
    CandidateMount,
    slot_path_of,
};
use crate::worktree_clone_unmount::staging_call_words::StagingCall;
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasetInstance;
use std::collections::BTreeMap;
use std::path::{
    Path,
    PathBuf,
};

/// The slot every control reads.
const SLOT: &str = "one";

/// One control's world:
/// the places,
/// the recorded folder identity,
/// the slot's folder,
/// the candidate,
/// and every call the pass makes.
struct World {
    /// The places the pass reads and writes.
    places: ShutdownUnmountPlaces,
    /// The identity the route recorded for the folder the slots hang in.
    recorded: CloneFolderIdentity,
    /// The slot's folder,
    /// held open as the walk hands it back.
    slot: std::fs::File,
    /// The mount the pass reads.
    candidate: CandidateMount,
    /// The staging route's answers.
    calls: StatedCalls,
    /// The holder scan's answer.
    holders: StatedHolders,
    /// The clone's GUID as the pool answers it.
    guids: StatedGuids,
}

impl World {
    /// The mount table the pass reads,
    /// which one turn never asks.
    fn rows(&self) -> StatedMounts {
        return StatedMounts {
            rows: vec![MountedDatasetInstance {
                dataset: String::from(CLONE),
                mount_point: self.candidate.slot_path.clone(),
                root: PathBuf::from("/"),
            }],
        };
    }

    /// The record of the folder the slots hang in.
    fn recorded_folder(&self) -> StatedWorktrees {
        return StatedWorktrees {
            answer: Ok(self.recorded.clone()),
        };
    }

    /// The agreeing lookup of this world's slot.
    fn lookup(&self) -> StatedLookups {
        return StatedLookups {
            answer: Ok(Box::new(fresh_lookup_of(&self.slot, &self.recorded))),
        };
    }

    /// The walk that hands this world's slot back.
    fn walk(&self) -> StatedWalk {
        return StatedWalk {
            underlay: None,
            slot: self
                .slot
                .try_clone()
                .expect("the fixture descriptor is copied"),
            look_again: MOUNT_ID,
        };
    }
}

/// One control's world at one fixture,
/// with the row and the calls of the agreeing arm.
fn world(name: &str) -> World {
    let root: PathBuf = fixture(name);
    let checkout: PathBuf = root.join("checkout");
    let slot_path: PathBuf = slot_path_of(checkout.as_path(), SLOT);
    std::fs::create_dir_all(slot_path.as_path()).expect("the slot folder is made");
    let slot: std::fs::File =
        std::fs::File::open(slot_path.as_path()).expect("the slot folder opens");
    let recorded: CloneFolderIdentity = identity_of(checkout.as_path());
    let candidate: CandidateMount = CandidateMount {
        clone: String::from(CLONE),
        slot: String::from(SLOT),
        slot_path,
    };
    let mut rows: BTreeMap<u64, MountFacts> = BTreeMap::new();
    rows.insert(
        MOUNT_ID,
        mount_row(
            CLONE,
            candidate.slot_path.as_path(),
            Path::new("/"),
            "zfs",
            PARENT_MOUNT_ID,
        ),
    );
    return World {
        places: ShutdownUnmountPlaces {
            readiness: ReadinessFile::at(root.join("readiness"), this_uid()),
            configuration: root.join("configuration.jsonc"),
            claims: SlotClaimDirectory::at(root.join("claims"), this_uid()),
            store: PathBuf::from(SLOT_IDENTITY_STORE_PATH),
            boot_identity: root.join("boot_id"),
            proc_root: root.join("proc"),
            self_task: root.join("proc").join("self"),
        },
        recorded,
        slot,
        candidate,
        calls: StatedCalls {
            unmounted_by: std::cell::RefCell::new(None),
            mount_id: MOUNT_ID,
            observed_path: PathBuf::new(),
            rows,
            unmount: Ok(()),
        },
        holders: StatedHolders {
            answer: Ok(Box::new(scan_finding_one_holder())),
        },
        guids: StatedGuids {
            answer: Ok(clone_guid()),
        },
    };
}

/// This account's own number.
fn this_uid() -> u32 {
    // SAFETY: `getuid` takes no argument and changes nothing.
    return unsafe { libc::getuid() };
}

/// One turn of one mount,
/// with the claim released however it ended.
fn turn(world: &World) -> ReachedMount {
    let checkout: PathBuf = world.candidate.slot_path.clone();
    let walk: StatedWalk = world.walk();
    let lookup: StatedLookups = world.lookup();
    let unmounts: StatedUnmounts = StatedUnmounts {
        answer: world.calls.unmount.clone().map_err(|refusal| {
            return (refusal.errno, refusal.detail);
        }),
        asked: std::cell::RefCell::new(Vec::new()),
    };
    let mut calls: StatedCalls = StatedCalls {
        unmounted_by: std::cell::RefCell::new(None),
        mount_id: world.calls.mount_id,
        observed_path: world.candidate.slot_path.clone(),
        rows: world.calls.rows.clone(),
        unmount: world.calls.unmount.clone(),
    };
    calls.observed_path = world.candidate.slot_path.clone();
    let rows: StatedMounts = world.rows();
    let recorded: StatedWorktrees = world.recorded_folder();
    let request: ShutdownUnmountRequest<'_> = ShutdownUnmountRequest {
        places: &world.places,
        mounts: &rows,
        calls: &calls,
        walk: &walk,
        unmounts: &unmounts,
        lookups: &lookup,
        worktrees: &recorded,
        guids: &world.guids,
        holders: &world.holders,
    };
    return unmount_one(
        &request,
        checkout
            .parent()
            .expect("the slot has a parent")
            .parent()
            .expect("the checkout"),
        &world.recorded,
        &world.candidate,
    );
}

/// The pass makes the one unmount call, names the slot, and never unmounts through
/// the clone's own descriptor.
#[test]
fn the_pass_makes_one_unmount_call_naming_the_slot() {
    let world: World = world("unmount-call-named");
    let walk: StatedWalk = world.walk();
    let lookup: StatedLookups = world.lookup();
    let unmounts: StatedUnmounts = StatedUnmounts {
        answer: Ok(()),
        asked: std::cell::RefCell::new(Vec::new()),
    };
    let mut calls: StatedCalls = StatedCalls {
        unmounted_by: std::cell::RefCell::new(None),
        mount_id: MOUNT_ID,
        observed_path: world.candidate.slot_path.clone(),
        rows: world.calls.rows.clone(),
        unmount: Ok(()),
    };
    calls.observed_path = world.candidate.slot_path.clone();
    let rows: StatedMounts = world.rows();
    let recorded: StatedWorktrees = world.recorded_folder();
    let request: ShutdownUnmountRequest<'_> = ShutdownUnmountRequest {
        places: &world.places,
        mounts: &rows,
        calls: &calls,
        walk: &walk,
        unmounts: &unmounts,
        lookups: &lookup,
        worktrees: &recorded,
        guids: &world.guids,
        holders: &world.holders,
    };
    let reached: ReachedMount = unmount_one(
        &request,
        world
            .candidate
            .slot_path
            .parent()
            .expect("the slot has a parent")
            .parent()
            .expect("the checkout"),
        &world.recorded,
        &world.candidate,
    );
    assert_eq!(
        reached.ending,
        MountEnding::Unmounted {
            mount_id: MOUNT_ID
        }
    );
    assert_eq!(
        *unmounts.asked.borrow(),
        vec![String::from(SLOT)],
        "the call is made once and names the slot it unmounts"
    );
    assert_eq!(
        *calls.unmounted_by.borrow(),
        None,
        "no unmount goes through the staging route's own call, which would have to be named by the clone's descriptor"
    );
}

/// An idle mount is unmounted and the answer names its mount number.
#[test]
fn an_idle_mount_is_unmounted() {
    let world: World = world("unmount-idle");
    let reached: ReachedMount = turn(&world);
    assert_eq!(reached.slot, SLOT);
    assert_eq!(reached.clone, CLONE);
    assert_eq!(
        reached.ending,
        MountEnding::Unmounted {
            mount_id: MOUNT_ID
        }
    );
    assert!(
        world.places.claims.claim(SLOT).is_ok(),
        "the claim is released, so the next pass may take the slot"
    );
}

/// A slot another handler holds is left to that handler,
/// and no kernel call is made.
#[test]
fn a_held_slot_is_left_to_the_holder() {
    let world: World = world("unmount-held-slot");
    let held = world
        .places
        .claims
        .claim(SLOT)
        .expect("the control takes the slot first");
    let reached: ReachedMount = turn(&world);
    match &reached.ending {
        MountEnding::Left {
            said,
        } => assert!(
            said.contains("another handler holds this slot's claim"),
            "the line names the holder, and this one says {said}"
        ),
        other => panic!("a held slot is left, and this one is {other:?}"),
    }
    held.release();
    assert!(
        world.places.claims.claim(SLOT).is_ok(),
        "the pass released nothing it did not take: the slot is free once the holder lets it go"
    );
}

/// A busy mount is kept,
///  names its holders,
///  and says what was not done.
#[test]
fn a_busy_mount_is_kept_and_names_its_holders() {
    let mut world: World = world("unmount-busy");
    world.calls.unmount = Err(busy());
    let reached: ReachedMount = turn(&world);
    match &reached.ending {
        MountEnding::Held {
            mount_id,
            said,
            holdings,
        } => {
            assert_eq!(*mount_id, MOUNT_ID);
            assert_eq!(said, "a process holds the mount");
            let found = holdings
                .as_ref()
                .expect("the scan was made for the busy mount");
            assert_eq!(
                found.holders,
                vec![Holder {
                    namespace: String::from("mnt:[4026532001]"),
                    task: PathBuf::from("/proc/1234"),
                    mounts: 1,
                }]
            );
        }
        other => panic!("a busy mount is held and named, and this one is {other:?}"),
    }
}

/// A busy mount whose scan could not be made carries the scan's own refusal.
#[test]
fn a_busy_mount_whose_scan_failed_says_why() {
    let mut world: World = world("unmount-busy-no-scan");
    world.calls.unmount = Err(busy());
    world.guids.answer = Err(String::from("the device could not be opened"));
    let reached: ReachedMount = turn(&world);
    match &reached.ending {
        MountEnding::Held {
            holdings,
            ..
        } => {
            let said: String = holdings
                .as_ref()
                .expect_err("no scan could be bound to a GUID")
                .clone();
            assert!(
                said.contains("no holder scan could be made"),
                "the sentence says the scan was not made, and this one says {said}"
            );
        }
        other => panic!("a busy mount is held, and this one is {other:?}"),
    }
}

/// A scan that read part of the tree carries its incomplete count.
#[test]
fn a_scan_that_read_part_of_the_tree_carries_its_count() {
    let mut world: World = world("unmount-busy-incomplete");
    world.calls.unmount = Err(busy());
    world.holders.answer = Ok(Box::new(scan_with_one_incomplete_read()));
    let reached: ReachedMount = turn(&world);
    match &reached.ending {
        MountEnding::Held {
            holdings,
            ..
        } => assert_eq!(
            holdings
                .as_ref()
                .expect("the scan was made")
                .incomplete_reads,
            1
        ),
        other => panic!("a busy mount is held, and this one is {other:?}"),
    }
}

/// Any other kernel refusal leaves the mount where it was.
#[test]
fn another_kernel_refusal_leaves_the_mount() {
    let mut world: World = world("unmount-other-refusal");
    world.calls.unmount = Err(refusal(
        StagingCall::Unmount,
        libc::EINVAL,
        "Invalid argument",
    ));
    let reached: ReachedMount = turn(&world);
    match &reached.ending {
        MountEnding::Left {
            said,
        } => assert!(
            said.contains("the kernel would not unmount the clone's own mount: Invalid argument"),
            "the line carries the kernel's own words, and this one says {said}"
        ),
        other => panic!("another refusal leaves the mount, and this one is {other:?}"),
    }
}

/// A mount whose checks refuse makes no unmount call at all.
#[test]
fn a_mount_whose_checks_refuse_is_never_unmounted() {
    let world: World = world("unmount-checks-refuse");
    let mut calls: StatedCalls = StatedCalls {
        unmounted_by: std::cell::RefCell::new(None),
        mount_id: MOUNT_ID + 9,
        observed_path: world.candidate.slot_path.clone(),
        rows: world.calls.rows.clone(),
        unmount: Ok(()),
    };
    calls.observed_path = world.candidate.slot_path.clone();
    let walk: StatedWalk = world.walk();
    let lookup: StatedLookups = world.lookup();
    let unmounts: StatedUnmounts = StatedUnmounts {
        answer: Ok(()),
        asked: std::cell::RefCell::new(Vec::new()),
    };
    let rows: StatedMounts = world.rows();
    let recorded: StatedWorktrees = world.recorded_folder();
    let request: ShutdownUnmountRequest<'_> = ShutdownUnmountRequest {
        places: &world.places,
        mounts: &rows,
        calls: &calls,
        walk: &walk,
        unmounts: &unmounts,
        lookups: &lookup,
        worktrees: &recorded,
        guids: &world.guids,
        holders: &world.holders,
    };
    let reached: ReachedMount = unmount_one(
        &request,
        world
            .candidate
            .slot_path
            .parent()
            .expect("the slot has a parent")
            .parent()
            .expect("the checkout"),
        &world.recorded,
        &world.candidate,
    );
    match &reached.ending {
        MountEnding::Left {
            said,
        } => assert!(
            said.contains("so the mount moved while this pass read it"),
            "the check refused before any call, and this one says {said}"
        ),
        other => panic!("a mount that fails its checks is left, and this one is {other:?}"),
    }
}
