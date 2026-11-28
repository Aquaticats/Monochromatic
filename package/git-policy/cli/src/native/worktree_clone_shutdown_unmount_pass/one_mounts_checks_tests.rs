//! What:
//!  Controls of the comparisons one mount's turn makes before it unmounts:
//!  the agreeing arm passes,
//!  and the folder the slot hangs in,
//!  the descriptor's mount number,
//!  its device and inode,
//!  the path it shows,
//!  a second walk,
//!  the row's source,
//!  filesystem,
//!  root,
//!  mount point and parent each refuse on their own.
//! Why:
//!  Unmounting acts on whatever stands at a path when the call runs,
//!  so each comparison is one way that mount can be another than the one the
//!  pass read,
//!  and a control that only proved the agreeing arm would prove nothing about
//!  the refusals
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount").
//!  The slot is a real folder of the test's fixture and every call is a stated
//!  answer,
//!  so no control mounts or unmounts anything.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const checked = verifyTheMount(candidate, checkout, recordedParent, lookups, walk, calls);
//! expect(checked.mountId).toBe(41);
//! ```

use super::verify_the_mount;
use crate::fs_mount_options::MountFacts;
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount::FreshSlotLookup;
use crate::worktree_clone_mount::clone_folder_identity::CloneFolderIdentity;
use crate::worktree_clone_shutdown_unmount_pass::shutdown_unmount_pass_test_doubles::{
    StatedCalls,
    StatedLookups,
    StatedWalk,
};
use crate::worktree_clone_shutdown_unmount_pass::shutdown_unmount_pass_test_fixture_values::{
    CLONE,
    MOUNT_ID,
    PARENT_MOUNT_ID,
    fresh_lookup_of,
    identity_of,
    mount_row,
    refusal,
};
use crate::worktree_clone_shutdown_unmount_pass::unmountable_mounts::{
    CandidateMount,
    slot_path_of,
};
use crate::worktree_clone_unmount::staging_call_words::StagingCall;
use std::collections::BTreeMap;
use std::os::unix::fs::MetadataExt;
use std::path::{
    Path,
    PathBuf,
};

/// The slot's own name every control reads.
const SLOT: &str = "one";

/// One control's fixture:
/// the checkout folder,
/// the slot folder inside it,
/// and the candidate the pass reads.
struct Fixture {
    /// The checkout the walk starts at.
    checkout: PathBuf,
    /// The slot's folder,
    /// held open as the walk hands it back.
    slot: std::fs::File,
    /// The candidate the pass reads.
    candidate: CandidateMount,
}

/// One fixture folder with a checkout and one slot inside it.
fn fixture_slot(name: &str) -> Fixture {
    let checkout: PathBuf = fixture(name);
    let slot_path: PathBuf = slot_path_of(checkout.as_path(), SLOT);
    std::fs::create_dir_all(slot_path.as_path()).expect("the slot folder is made");
    let slot: std::fs::File =
        std::fs::File::open(slot_path.as_path()).expect("the slot folder opens");
    return Fixture {
        checkout,
        slot,
        candidate: CandidateMount {
            clone: String::from(CLONE),
            slot: String::from(SLOT),
            slot_path,
        },
    };
}

/// The row the agreeing arm reads.
fn agreeing_row(candidate: &CandidateMount) -> MountFacts {
    return mount_row(
        candidate.clone.as_str(),
        candidate.slot_path.as_path(),
        Path::new("/"),
        "zfs",
        PARENT_MOUNT_ID,
    );
}

/// The call double the agreeing arm reads.
fn agreeing_calls(candidate: &CandidateMount) -> StatedCalls {
    let mut rows: BTreeMap<u64, MountFacts> = BTreeMap::new();
    rows.insert(MOUNT_ID, agreeing_row(candidate));
    return StatedCalls {
        unmounted_by: std::cell::RefCell::new(None),
        mount_id: MOUNT_ID,
        observed_path: candidate.slot_path.clone(),
        rows,
        unmount: Ok(()),
    };
}

/// The fresh lookup the agreeing arm reads,
/// whose parent is the folder the route recorded.
fn agreeing_lookup(
    fixture: &Fixture,
    recorded: &CloneFolderIdentity,
) -> StatedLookups {
    return StatedLookups {
        answer: Ok(Box::new(fresh_lookup_of(&fixture.slot, recorded))),
    };
}

/// One refusal's sentence,
/// from a turn that refused where the control planted the difference.
fn refusal_of(
    fixture: &Fixture,
    recorded: &CloneFolderIdentity,
    lookups: &StatedLookups,
    walk: &StatedWalk,
    calls: &StatedCalls,
) -> String {
    return verify_the_mount(
        &fixture.candidate,
        fixture.checkout.as_path(),
        recorded,
        lookups,
        walk,
        calls,
    )
    .expect_err("this turn refuses");
}

/// The agreeing arm proves the mount and hands back its own descriptor.
#[test]
fn a_clones_own_mount_at_its_slot_passes_every_comparison() {
    let fixture: Fixture = fixture_slot("shutdown-checks-agreeing");
    let recorded: CloneFolderIdentity = identity_of(fixture.checkout.as_path());
    let checked = verify_the_mount(
        &fixture.candidate,
        fixture.checkout.as_path(),
        &recorded,
        &agreeing_lookup(&fixture, &recorded),
        &StatedWalk {
            underlay: None,
            slot: fixture
                .slot
                .try_clone()
                .expect("the fixture descriptor is copied"),
            look_again: MOUNT_ID,
        },
        &agreeing_calls(&fixture.candidate),
    )
    .expect("the clone's own mount at its slot passes");
    assert_eq!(checked.mount_id, MOUNT_ID);
    let facts: std::fs::Metadata = checked
        .target
        .metadata()
        .expect("the checked descriptor is readable");
    assert_eq!(
        facts.ino(),
        fixture.slot.metadata().expect("the slot is readable").ino()
    );
}

/// A slot whose folder hangs in another `.claude/worktrees` refuses.
#[test]
fn a_slot_in_another_worktrees_folder_is_refused() {
    let fixture: Fixture = fixture_slot("shutdown-checks-parent");
    let recorded: CloneFolderIdentity = identity_of(fixture.checkout.as_path());
    let mut other: CloneFolderIdentity = identity_of(fixture.checkout.as_path());
    other.inode += 1;
    let lookup: StatedLookups = StatedLookups {
        answer: Ok(Box::new(fresh_lookup_of(&fixture.slot, &other))),
    };
    let said: String = refusal_of(
        &fixture,
        &recorded,
        &lookup,
        &StatedWalk {
            underlay: None,
            slot: fixture
                .slot
                .try_clone()
                .expect("the fixture descriptor is copied"),
            look_again: MOUNT_ID,
        },
        &agreeing_calls(&fixture.candidate),
    );
    assert!(
        said.contains("is not the `.claude/worktrees` folder this route recorded"),
        "the refusal names the folder, and this one says {said}"
    );
    assert!(
        said.contains("inode"),
        "the refusal names which field differs, and this one says {said}"
    );
}

/// A descriptor on another mount than the fresh walk reached refuses.
#[test]
fn a_descriptor_on_another_mount_is_refused() {
    let fixture: Fixture = fixture_slot("shutdown-checks-mount-id");
    let recorded: CloneFolderIdentity = identity_of(fixture.checkout.as_path());
    let mut calls: StatedCalls = agreeing_calls(&fixture.candidate);
    calls.mount_id = MOUNT_ID + 1;
    let said: String = refusal_of(
        &fixture,
        &recorded,
        &agreeing_lookup(&fixture, &recorded),
        &StatedWalk {
            underlay: None,
            slot: fixture
                .slot
                .try_clone()
                .expect("the fixture descriptor is copied"),
            look_again: MOUNT_ID,
        },
        &calls,
    );
    assert!(
        said.contains("the mount moved while this pass read it"),
        "the refusal says the mount moved, and this one says {said}"
    );
}

/// A slot whose folder was replaced refuses on its inode.
#[test]
fn a_replaced_slot_folder_is_refused() {
    let fixture: Fixture = fixture_slot("shutdown-checks-inode");
    let recorded: CloneFolderIdentity = identity_of(fixture.checkout.as_path());
    let mut capture: FreshSlotLookup = fresh_lookup_of(&fixture.slot, &recorded);
    capture.slot_identity.inode += 1;
    let said: String = refusal_of(
        &fixture,
        &recorded,
        &StatedLookups {
            answer: Ok(Box::new(capture)),
        },
        &StatedWalk {
            underlay: None,
            slot: fixture
                .slot
                .try_clone()
                .expect("the fixture descriptor is copied"),
            look_again: MOUNT_ID,
        },
        &agreeing_calls(&fixture.candidate),
    );
    assert!(
        said.contains("the folder was replaced"),
        "the refusal says the folder was replaced, and this one says {said}"
    );
}

/// A descriptor that shows another path refuses.
#[test]
fn a_descriptor_that_shows_another_path_is_refused() {
    let fixture: Fixture = fixture_slot("shutdown-checks-path");
    let recorded: CloneFolderIdentity = identity_of(fixture.checkout.as_path());
    let mut calls: StatedCalls = agreeing_calls(&fixture.candidate);
    calls.observed_path = fixture.checkout.join("somewhere-else");
    let said: String = refusal_of(
        &fixture,
        &recorded,
        &agreeing_lookup(&fixture, &recorded),
        &StatedWalk {
            underlay: None,
            slot: fixture
                .slot
                .try_clone()
                .expect("the fixture descriptor is copied"),
            look_again: MOUNT_ID,
        },
        &calls,
    );
    assert!(
        said.contains("so the walk reached another place"),
        "the refusal says the walk reached another place, and this one says {said}"
    );
}

/// A second walk that reaches another mount refuses.
#[test]
fn a_second_walk_that_reaches_another_mount_is_refused() {
    let fixture: Fixture = fixture_slot("shutdown-checks-look-again");
    let recorded: CloneFolderIdentity = identity_of(fixture.checkout.as_path());
    let said: String = refusal_of(
        &fixture,
        &recorded,
        &agreeing_lookup(&fixture, &recorded),
        &StatedWalk {
            underlay: None,
            slot: fixture
                .slot
                .try_clone()
                .expect("the fixture descriptor is copied"),
            look_again: MOUNT_ID + 2,
        },
        &agreeing_calls(&fixture.candidate),
    );
    assert!(
        said.contains("a second walk of slot `one` reaches mount"),
        "the refusal names the second walk, and this one says {said}"
    );
}

/// Every field of the row that differs refuses on its own.
#[test]
fn each_field_of_the_row_refuses_on_its_own() {
    let fixture: Fixture = fixture_slot("shutdown-checks-row");
    let recorded: CloneFolderIdentity = identity_of(fixture.checkout.as_path());
    let elsewhere: PathBuf = fixture.checkout.join("a").join("b");
    let cases: Vec<(MountFacts, &str)> = vec![
        (
            mount_row(
                "zpcachyos/nosnap/git-worktree-clones/two",
                fixture.candidate.slot_path.as_path(),
                Path::new("/"),
                "zfs",
                PARENT_MOUNT_ID,
            ),
            "so it is another dataset's mount",
        ),
        (
            mount_row(
                CLONE,
                fixture.candidate.slot_path.as_path(),
                Path::new("/"),
                "tmpfs",
                PARENT_MOUNT_ID,
            ),
            "is a `tmpfs` mount and a clone's own mount is a `zfs` mount",
        ),
        (
            mount_row(
                CLONE,
                fixture.candidate.slot_path.as_path(),
                Path::new("/src"),
                "zfs",
                PARENT_MOUNT_ID,
            ),
            "so this is a bind of a folder inside the clone",
        ),
        (
            mount_row(
                CLONE,
                elsewhere.as_path(),
                Path::new("/"),
                "zfs",
                PARENT_MOUNT_ID,
            ),
            "so the mount moved",
        ),
        (
            mount_row(
                CLONE,
                fixture.candidate.slot_path.as_path(),
                Path::new("/"),
                "zfs",
                PARENT_MOUNT_ID + 1,
            ),
            "so a mount stands between the slot and its folder",
        ),
    ];
    for (row, wanted) in cases {
        let mut rows: BTreeMap<u64, MountFacts> = BTreeMap::new();
        rows.insert(MOUNT_ID, row);
        let calls: StatedCalls = StatedCalls {
            unmounted_by: std::cell::RefCell::new(None),
            mount_id: MOUNT_ID,
            observed_path: fixture.candidate.slot_path.clone(),
            rows,
            unmount: Ok(()),
        };
        let said: String = refusal_of(
            &fixture,
            &recorded,
            &agreeing_lookup(&fixture, &recorded),
            &StatedWalk {
                underlay: None,
                slot: fixture
                    .slot
                    .try_clone()
                    .expect("the fixture descriptor is copied"),
                look_again: MOUNT_ID,
            },
            &calls,
        );
        assert!(
            said.contains(wanted),
            "the refusal should say {wanted:?}, and this one says {said}"
        );
    }
}

/// A walk that refused is a sentence and never a panic.
#[test]
fn a_walk_that_refused_is_a_sentence() {
    let fixture: Fixture = fixture_slot("shutdown-checks-walk-refused");
    let recorded: CloneFolderIdentity = identity_of(fixture.checkout.as_path());
    let said: String = refusal_of(
        &fixture,
        &recorded,
        &StatedLookups {
            answer: Err(format!(
                "the slot `{SLOT}` could not be walked: {}",
                refusal(StagingCall::OpenSlot, libc::ENOENT, "no such folder").detail
            )),
        },
        &StatedWalk {
            underlay: None,
            slot: fixture
                .slot
                .try_clone()
                .expect("the fixture descriptor is copied"),
            look_again: MOUNT_ID,
        },
        &agreeing_calls(&fixture.candidate),
    );
    assert!(
        said.contains("could not be walked"),
        "the walk's own sentence travels back, and this one says {said}"
    );
}
