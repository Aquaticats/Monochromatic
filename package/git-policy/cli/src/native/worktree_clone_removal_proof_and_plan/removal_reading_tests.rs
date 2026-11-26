//! What:
//!  The controls of the removal's read-only reading:
//!  the healthy reading and the lines it prints,
//!  each refusal the proof and the bookmark rule make,
//!  and the one guarantee they all share,
//!  that the reading wrote and unmounted nothing.
//! Why:
//!  This release's removal command runs no writing step,
//!  so the reading is what a person meets,
//!  and a control that asserted only the sentence would not show whether a plan,
//!  an intent or a byte of a record changed.
//!  Each control therefore compares the whole record tree before and after,
//!  which is the test the brief asks for:
//!  no write happens before plan step 16's wiring.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const before = tree(fixture.common());
//! expect(() => readTheRemoval(entry, refusedPool)).toThrow();
//! expect(tree(fixture.common())).toEqual(before);
//! ```

use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_removal_proof_and_plan::removal_entry::RemovalEntry;
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
    guid,
    the_stats,
};
use crate::worktree_clone_removal_proof_and_plan::removal_reading::{
    RemovalReading,
    read_the_removal,
};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::{
    BookmarkRefusalReason,
    RemovalStop,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DatasetIdentity,
    NvList,
};
use std::path::{
    Path,
    PathBuf,
};

/// The entry one fixture makes.
fn entry(fixture: &RemovalFixture) -> RemovalEntry<'_> {
    return RemovalEntry {
        common_dir: fixture.common(),
        slot: SLOT,
    };
}

/// What:
///  Every name under one folder and the bytes each file holds,
///  sorted by name.
/// Why:
///  A reading that wrote a plan,
///  an intent or a record would change this value,
///  and a set of files with identical names and bytes is what proves it did not.
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

/// What:
///  One command's reading of one removed clone,
///  with the tree it left behind asserted against the tree it found.
/// Why:
///  Every control makes the same promise,
///  and one helper keeps them from asserting it differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = readWithoutWriting(fixture, pool);
/// ```
fn read_without_writing(
    fixture: &RemovalFixture,
    pool: &ScriptedPool,
) -> Result<RemovalReading, RemovalStop> {
    let before: Vec<(String, Option<Vec<u8>>)> = the_tree(fixture.common());
    let answer = read_the_removal(&entry(fixture), pool);
    assert_eq!(
        the_tree(fixture.common()),
        before,
        "the reading changed no record, no plan and no intent"
    );
    assert_eq!(
        fixture.the_plan_folder_holds(),
        0,
        "no destruction plan was written"
    );
    return answer;
}

/// What:
///  A healthy removal reads,
///  prints the proof it matched and every name it would lose,
///  and leaves every record byte as it was.
/// Why:
///  This is the command's whole answer in a release with no clone removal,
///  so the lines are the deliverable and the unchanged tree is the promise.
#[test]
fn a_healthy_removal_reads_its_loss_and_writes_nothing() {
    let fixture: RemovalFixture = RemovalFixture::healthy("reading-healthy");
    let pool: ScriptedPool = fixture.healthy_pool();
    let reading: RemovalReading =
        read_without_writing(&fixture, &pool).expect("the healthy removal reads");
    assert_eq!(reading.clone, CLONE);
    assert_eq!(reading.clone_guid, guid(CLONE_GUID));
    assert_eq!(reading.clone_createtxg, CLONE_CREATETXG);
    assert_eq!(reading.origin_snapshot, ORIGIN_SNAPSHOT);
    assert_eq!(reading.origin_snapshot_guid, guid(ORIGIN_SNAPSHOT_GUID));
    assert_eq!(reading.origin_snapshot_createtxg, ORIGIN_CREATETXG);
    let printed: String = reading.lines().join("\n");
    assert!(
        printed.contains(&format!("the clone `{CLONE}` (GUID `{CLONE_GUID}`")),
        "{printed}"
    );
    assert!(
        printed.contains(&format!(
            "descends from the origin snapshot `{ORIGIN_SNAPSHOT}`"
        )),
        "{printed}"
    );
    assert!(
        printed.contains(&format!("snapshot {CLONE}@{OWN_SNAPSHOT}")),
        "{printed}"
    );
    assert!(
        printed.contains(&format!("snapshot {CLONE}@{SANOID_SNAPSHOT}")),
        "{printed}"
    );
    assert!(
        printed.contains("this release does not remove a clone yet"),
        "{printed}"
    );
    assert!(
        printed.contains("no removal intent was filed and no destruction plan was written"),
        "{printed}"
    );
}

/// What:
///  A promoted clone reads as another lineage and the reading refuses,
///  with the origin the pool now reports named beside the recorded one.
/// Why:
///  Item 11 measured why this must precede any delete:
///  a promoted clone's transferred history is the checkout's own.
#[test]
fn a_promoted_clone_is_refused_and_nothing_is_written() {
    let fixture: RemovalFixture = RemovalFixture::healthy("reading-promoted");
    // Promotion makes the checkout a child of the clone, so the clone's live origin is a dataset and not the recorded
    // origin snapshot.
    let promoted: ScriptedPool = fixture.healthy_pool().with(
        CLONE,
        DatasetIdentity {
            stats: the_stats(CLONE_GUID, false, CLONE_CREATETXG, 0, ORIGIN_DATASET),
            properties: NvList::new(),
        },
    );
    match read_without_writing(&fixture, &promoted)
        .expect_err("a promoted clone's lineage is not the recorded one")
    {
        RemovalStop::OriginIsNotTheRecordedOne {
            clone,
            recorded,
            live,
        } => {
            assert_eq!(clone, CLONE);
            assert_eq!(recorded, ORIGIN_SNAPSHOT);
            assert_eq!(live.as_deref(), Some(ORIGIN_DATASET));
        }
        other => panic!("a promoted clone refuses on its origin, not {other:?}"),
    }
}

/// What:
///  A snapshot the clone inherited is refused,
///  and the pool still lists it and every other protected object afterwards.
/// Why:
///  The reading holds the pool's own list and never submits a destroy,
///  so the control asserts the same objects are still there,
///  which is the survival the plan's refusal arm requires.
#[test]
fn a_transferred_snapshot_is_refused_and_every_protected_object_survives() {
    let fixture: RemovalFixture = RemovalFixture::healthy("reading-transferred");
    let mut pool: ScriptedPool = fixture.healthy_pool();
    pool.snapshots.push(an_object(
        EntryKind::Snapshot,
        "earliest",
        ORIGIN_SNAPSHOT_GUID,
        ORIGIN_CREATETXG,
    ));
    let before: Vec<String> = pool
        .snapshots
        .iter()
        .map(|object| return object.full_name())
        .collect();
    match read_without_writing(&fixture, &pool)
        .expect_err("an inherited snapshot refuses the removal")
    {
        RemovalStop::TransferredHistory {
            snapshot,
            createtxg,
            clone_createtxg,
            ..
        } => {
            assert_eq!(snapshot, format!("{CLONE}@earliest"));
            assert_eq!(createtxg, ORIGIN_CREATETXG);
            assert_eq!(clone_createtxg, CLONE_CREATETXG);
        }
        other => panic!("an inherited snapshot refuses as inherited history, not {other:?}"),
    }
    let after: Vec<String> = pool
        .snapshots
        .iter()
        .map(|object| return object.full_name())
        .collect();
    assert_eq!(after, before, "every protected snapshot is still listed");
}

/// What:
///  A replacement clone,
///  one whose GUID is not the add's,
///  is refused before anything is read of its snapshots.
/// Why:
///  The slot's name is used again,
///  so the GUID is the only identity that tells this clone from the next one at the same path.
#[test]
fn a_clone_of_another_guid_is_refused() {
    let fixture: RemovalFixture = RemovalFixture::healthy("reading-replacement");
    let replacement: ScriptedPool = fixture.healthy_pool().with(
        CLONE,
        DatasetIdentity {
            stats: the_stats(CLONE_GUID + 1, false, CLONE_CREATETXG, 0, ORIGIN_SNAPSHOT),
            properties: NvList::new(),
        },
    );
    match read_without_writing(&fixture, &replacement)
        .expect_err("a same-name replacement clone is not the recorded one")
    {
        RemovalStop::IdentityDiffers {
            object,
            field,
            ..
        } => {
            assert_eq!(object, CLONE);
            assert_eq!(field, "cloneGuid");
        }
        other => panic!("a replacement clone refuses on its GUID, not {other:?}"),
    }
}

/// What:
///  A clone recreated under the same name and GUID but another creation transaction group is refused.
/// Why:
///  The group is what a removal compares to tell the clone's own history from the history it inherited,
///  so a clone of another group is another dataset even when a receive kept its GUID.
#[test]
fn a_clone_of_another_creation_group_is_refused() {
    let fixture: RemovalFixture = RemovalFixture::healthy("reading-createtxg");
    let another: ScriptedPool = fixture.healthy_pool().with(
        CLONE,
        DatasetIdentity {
            stats: the_stats(CLONE_GUID, false, CLONE_CREATETXG + 1, 0, ORIGIN_SNAPSHOT),
            properties: NvList::new(),
        },
    );
    match read_without_writing(&fixture, &another)
        .expect_err("a clone of another group is not the recorded one")
    {
        RemovalStop::IdentityDiffers {
            field,
            recorded,
            live,
            ..
        } => {
            assert_eq!(field, "cloneCreatetxg");
            assert_eq!(recorded, CLONE_CREATETXG.to_string());
            assert_eq!(live, (CLONE_CREATETXG + 1).to_string());
        }
        other => panic!("a clone of another group refuses on its group, not {other:?}"),
    }
}

/// What:
///  A bookmark whose tuple the origin side recorded is refused,
///  which is the object a snapshot-only comparison misses.
/// Why:
///  Item 13 measured a bookmark travelling with its snapshot's tuple,
///  so a promoted clone can hold one under a name of its own.
#[test]
fn a_bookmark_of_the_origin_side_is_refused() {
    let fixture: RemovalFixture = RemovalFixture::healthy("reading-origin-bookmark");
    let mut pool: ScriptedPool = fixture.healthy_pool();
    pool.bookmarks.push(an_object(
        EntryKind::Bookmark,
        "from-origin",
        ORIGIN_SNAPSHOT_GUID,
        ORIGIN_CREATETXG,
    ));
    match read_without_writing(&fixture, &pool)
        .expect_err("an origin-side object refuses the removal")
    {
        RemovalStop::OriginSideHit {
            kind,
            name,
            ..
        } => {
            assert_eq!(kind, EntryKind::Bookmark);
            assert_eq!(name, format!("{CLONE}#from-origin"));
        }
        other => panic!("an origin-side bookmark refuses as inherited history, not {other:?}"),
    }
}

/// What:
///  A bookmark neither a live snapshot nor a plan line explains is refused by name.
/// Why:
///  The decided rule `A: Live or planned` refuses the whole removal and names each bookmark,
///  because nothing in this design destroys one.
#[test]
fn a_bookmark_the_rule_cannot_explain_is_refused_by_name() {
    let fixture: RemovalFixture = RemovalFixture::healthy("reading-unexplained-bookmark");
    let mut pool: ScriptedPool = fixture.healthy_pool();
    pool.bookmarks.push(an_object(
        EntryKind::Bookmark,
        "from-a-gone-snapshot",
        4242,
        19,
    ));
    match read_without_writing(&fixture, &pool)
        .expect_err("an unexplained bookmark refuses the removal")
    {
        RemovalStop::BookmarksRefused {
            refused,
            ..
        } => {
            assert_eq!(refused.len(), 1);
            assert_eq!(refused[0].name, format!("{CLONE}#from-a-gone-snapshot"));
            assert_eq!(refused[0].reason, BookmarkRefusalReason::Unexplained);
        }
        other => panic!("an unexplained bookmark refuses as one, not {other:?}"),
    }
}

/// What:
///  A bookmark one of the clone's own live snapshots explains passes,
///  and the reading prints it as a name to be lost.
/// Why:
///  The rule's passing arm is the one that keeps a removal going,
///  and the head destroy takes the bookmark with it,
///  so a person is told before anything is destroyed.
#[test]
fn a_bookmark_an_own_snapshot_explains_passes_and_is_printed() {
    let fixture: RemovalFixture = RemovalFixture::healthy("reading-explained-bookmark");
    let mut pool: ScriptedPool = fixture.healthy_pool();
    pool.bookmarks.push(an_object(
        EntryKind::Bookmark,
        "from-own",
        OWN_GUID,
        OWN_CREATETXG,
    ));
    let reading: RemovalReading = read_without_writing(&fixture, &pool)
        .expect("a bookmark of an own snapshot keeps the lineage proved");
    assert_eq!(reading.lost.bookmarks, vec![format!("{CLONE}#from-own")]);
    assert!(
        reading.lines().join("\n").contains("cannot be restored"),
        "the printed loss says the bookmark cannot be restored"
    );
}

/// What:
///  A plan already standing for this clone is named and changed by nothing.
/// Why:
///  A standing plan means a removal already recorded what it verified,
///  and this release writes none,
///  so the reading stops and leaves the plan exactly as it found it.
#[test]
fn a_plan_already_standing_is_named_and_left_alone() {
    let fixture: RemovalFixture = RemovalFixture::healthy("reading-standing-plan");
    let pool: ScriptedPool = fixture.healthy_pool();
    let plans: PathBuf = fixture.plan_folder();
    std::fs::create_dir_all(plans.as_path()).expect("the plan folder is made");
    let mut permissions: std::fs::Permissions = std::fs::metadata(plans.as_path())
        .expect("the folder's metadata reads")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o700);
    std::fs::set_permissions(plans.as_path(), permissions).expect("the folder is private");
    let stand: PathBuf = plans.join(CLONE_GUID.to_string());
    std::fs::write(stand.as_path(), "a plan this reader cannot parse\n")
        .expect("the standing plan is written");
    let before: Vec<(String, Option<Vec<u8>>)> = the_tree(fixture.common());
    match read_the_removal(&entry(&fixture), &pool).expect_err("a standing plan stops the reading")
    {
        RemovalStop::PlanUnreadable {
            path,
            ..
        } => assert_eq!(path, stand),
        other => {
            panic!("a standing plan that does not read is refused as unreadable, not {other:?}")
        }
    }
    assert_eq!(
        the_tree(fixture.common()),
        before,
        "the reading changed nothing, the standing file included"
    );
}

/// What:
///  A pool whose listing cannot be read completely is a refusal and not a short history.
/// Why:
///  A partial inventory would let a removal destroy a snapshot it never listed.
#[test]
fn an_incomplete_inventory_is_refused() {
    let fixture: RemovalFixture = RemovalFixture::healthy("reading-incomplete");
    let mut pool: ScriptedPool = fixture.healthy_pool();
    pool.listing_refusal = Some(String::from("the bookmark listing stopped"));
    match read_without_writing(&fixture, &pool)
        .expect_err("an unreadable listing refuses the removal")
    {
        RemovalStop::InventoryUnreadable {
            clone,
            reason,
        } => {
            assert_eq!(clone, CLONE);
            assert_eq!(reason, "the bookmark listing stopped");
        }
        other => panic!("an unreadable listing refuses as an unreadable inventory, not {other:?}"),
    }
}

/// What:
///  The pool dataset is proved too,
///  and a pool of another GUID refuses before the clone is read.
/// Why:
///  A clone of the right name in another pool is another clone,
///  and every name a plan records is a name in the recorded pool.
#[test]
fn a_pool_of_another_pool_guid_is_refused() {
    let fixture: RemovalFixture = RemovalFixture::healthy("reading-pool");
    let another: ScriptedPool = fixture.healthy_pool().with(
        POOL,
        DatasetIdentity {
            stats: the_stats(POOL_GUID + 1, false, 1, 0, ""),
            properties: NvList::new(),
        },
    );
    match read_without_writing(&fixture, &another)
        .expect_err("a pool of another GUID is not the recorded one")
    {
        RemovalStop::IdentityDiffers {
            object,
            field,
            ..
        } => {
            assert_eq!(object, POOL);
            assert_eq!(field, "poolGuid");
        }
        other => panic!("another pool refuses on its pool GUID, not {other:?}"),
    }
}
