//! What:
//!  The controls of the folder a removal's unmount binds:
//!  the release's own creation over a fixture cache and a pin this machine's own
//!  mount facts make,
//!  and the order that calls it before it asks for an unmount.
//! Why:
//!  The handler's unmount turn opens that folder first and answers `cancelled`
//!  at `staging-folder` when it is absent,
//!  so a removal whose order never makes it can never finish on a real pool.
//!  The account's cache and the root-written pin are the two facts this machine
//!  cannot give a control,
//!  so the control answers them with a fixture cache and a pin built from that
//!  cache's own mount,
//!  and the creation itself is the release's `create_the_staging_folder`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done = completeWith(theCompletion({...}), NOW, theFixtureStagingFolder(), theUnmountThatReadsIt());
//! expect(theUnmountThatReadsIt().seen).toBe(0o700);
//! ```

use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountIntent;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::ReceiptOutcome;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_fixtures::{
    ClearingDevice,
    EpochProbe,
    LivePool,
    ORIGIN_OWN_NAME,
    ScriptedHead,
    ScriptedHolders,
    ScriptedSettle,
    TheCompletion,
    a_clear_scan,
    a_fixture,
    an_intent_property,
    complete_with,
    the_budget,
    the_checks_pool,
    the_checks_pool_of,
};
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{
    CLONE,
    CLONE_CREATETXG,
    CLONE_GUID,
    ORIGIN_SNAPSHOT,
    RemovalFixture,
    the_stats,
};
use crate::worktree_clone_removal_proof_and_plan::removal_staging_folder::RemovalStagingFolder;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::{
    RemovalUnmount,
    UnmountAnswer,
};
use crate::worktree_clone_unmount::staging_cleanup::create_the_staging_folder;
use crate::worktree_clone_unmount::staging_folder_name::staging_folder_name;
use crate::worktree_clone_unmount::user_cache_pin::{
    UserCachePin,
    cache_mount_facts,
};
use crate::worktree_removal_deferred_origin_check::fixtures::Pool;
use crate::worktree_removal_deferred_origin_check::origin_mark::MARK_OFF;
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;
use std::cell::{
    Cell,
    RefCell,
};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::rc::Rc;

/// What:
///  The staging folder a control answers with:
///  the release's own creation over a fixture cache and a pin of this machine's
///  own mount facts.
/// Why:
///  The creation's place and its pin are the account's,
///  so the control states both and keeps the release's function as the thing
///  under test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const staging = { asked: 0, makeTheFolder(guid) { asked += 1; createTheStagingFolder(home, pin, guid); } };
/// ```
struct AStagingFolderOfAFixtureCache {
    /// The account's home the control states,
    /// whose `.cache` is the fixture cache.
    home: PathBuf,
    /// The pin the fixture cache answers.
    pin: UserCachePin,
    /// How many times the order asked for the folder.
    asked: Cell<usize>,
}

impl RemovalStagingFolder for AStagingFolderOfAFixtureCache {
    fn make_the_folder(
        &self,
        clone_guid: DatasetGuid,
    ) -> Result<(), RemovalStop> {
        self.asked.set(self.asked.get() + 1);
        let _made: bool = create_the_staging_folder(self.home.as_path(), &self.pin, clone_guid)
            .map_err(|said| {
                return RemovalStop::TheStagingFolderCouldNotBeMade {
                    step: String::from(super::STAGING_FOLDER_STEP),
                    said,
                };
            })?;
        // What: `Ok(())` is the success variant of `Result`, saying the folder stands.
        // Why: the control answers the step, and the order goes on to ask for the unmount.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return;
        // ```
        return Ok(());
    }
}

/// What:
///  The unmount answer a control gives,
///  which reads the folder the order should have made before it asked.
/// Why:
///  The claim is about the moment of the request:
///  the handler opens that folder first,
///  so a control that reads it here fails exactly where a real handler would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unmount = { seen: undefined, requestTheUnmount(intent) { this.seen = modeOf(folder); return done(intent); } };
/// ```
struct AnUnmountThatReadsTheFolder {
    /// The clone's GUID,
    /// which the folder's name derives from.
    clone_guid: DatasetGuid,
    /// The account's home the control states.
    home: PathBuf,
    /// The folder's mode at the moment the unmount was asked for,
    /// and nothing when no folder stood.
    seen: RefCell<Option<u32>>,
}

impl RemovalUnmount for AnUnmountThatReadsTheFolder {
    fn request_the_unmount(
        &self,
        intent: &MountIntent,
    ) -> Result<UnmountAnswer, RemovalStop> {
        let folder: PathBuf = self
            .home
            .join(".cache")
            .join(staging_folder_name(self.clone_guid));
        let mode: Option<u32> = std::fs::metadata(folder.as_path())
            .ok()
            .map(|facts| return facts.mode() & 0o7777);
        *self.seen.borrow_mut() = mode;
        return Ok(UnmountAnswer {
            dataset: self.clone_guid,
            operation: intent.operation,
            nonce: intent.nonce.clone(),
            outcome: ReceiptOutcome::Done,
            mount_id: None,
            failed_step: None,
            errno: None,
        });
    }
}

/// What:
///  A pin of the fixture cache's own mount,
///  which is what the `user-cache` pin records about the account's cache.
/// Why:
///  Every check compares those two fields,
///  and a control's fixture cache is on this machine's disk,
///  so the pin is read from the mount the fixture really is on rather than made
///  up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pin = { source: mountSource(cache), rootInode: inodeOf(cache), device: 1 };
/// ```
fn the_fixture_pins(cache: &std::fs::File) -> UserCachePin {
    let facts = cache_mount_facts(cache).expect("the fixture cache's mount reads");
    let opened: std::fs::Metadata = cache
        .metadata()
        .expect("the fixture cache's metadata reads");
    return UserCachePin {
        source: facts.source,
        root_inode: opened.ino(),
        device: 1,
    };
}

/// What:
///  A removal that reaches its unmount finds its staging folder made by the
///  release's own creation,
///  at mode `0700`,
///  after the pin check the records name.
/// Why:
///  No release code made that folder,
///  so every removal of a real clone asked the handler to bind a folder nothing
///  had made,
///  and stopped at `staging-folder` (cause B of issue 6669).
///  The cache and the pin are this machine's own facts,
///  so the control states both and keeps the release's function.
#[test]
fn a_removal_reaches_its_unmount_with_the_folder_the_release_makes() {
    let fixture_at: RemovalFixture = a_fixture("staging-folder-the-removal-makes");
    let home: PathBuf = fixture("staging-folder-the-removal-makes");
    let cache_path: PathBuf = home.join(".cache");
    std::fs::create_dir_all(cache_path.as_path()).expect("the fixture cache is made");
    let cache: std::fs::File =
        std::fs::File::open(cache_path.as_path()).expect("the fixture cache opens");
    // What: the clone's GUID, read once, because the removal removes the binding record this read comes from.
    // Why: a later read would find no record at all, and the control still needs the folder's name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const cloneGuid = fixture.recorded().cloneGuid;
    // ```
    let clone_guid: DatasetGuid = fixture_at.recorded().clone_guid;
    let staging: AStagingFolderOfAFixtureCache = AStagingFolderOfAFixtureCache {
        home: home.clone(),
        pin: the_fixture_pins(&cache),
        asked: Cell::new(0),
    };
    let unmount: AnUnmountThatReadsTheFolder = AnUnmountThatReadsTheFolder {
        clone_guid,
        home: home.clone(),
        seen: RefCell::new(None),
    };
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe {
        pool: check,
        hidden: Rc::clone(&hidden),
    };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let head: ScriptedHead =
        ScriptedHead::succeeding(fixture_at.plan_folder(), clone_guid, Rc::clone(&hidden));
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    complete_with(
        &TheCompletion {
            fixture: &fixture_at,
            pool: &pool,
            check: &probe,
            probe: &probe,
            device: &device,
            head: &head,
            holders: &holders,
            settle: &settle,
            cancelling: None,
            budget: the_budget(),
        },
        crate::worktree_clone_removal_proof_and_plan::removal_completion_fixtures::NOW,
        &staging,
        &unmount,
    )
    .expect("a removal whose folder stands at its unmount finishes");
    assert_eq!(
        staging.asked.get(),
        1,
        "the order asked for the folder once, before it asked for an unmount"
    );
    assert_eq!(
        *unmount.seen.borrow(),
        Some(0o700),
        "the folder stood at mode 0700 when the unmount was asked for"
    );
    let folder: PathBuf = cache_path.join(staging_folder_name(clone_guid));
    let facts: std::fs::Metadata = std::fs::metadata(folder.as_path()).expect("the folder stands");
    assert_eq!(
        facts.mode() & 0o7777,
        0o700,
        "the folder keeps the route's mode"
    );
    assert_eq!(
        facts.uid(),
        std::fs::metadata(cache_path.as_path())
            .expect("the fixture cache's metadata reads")
            .uid(),
        "the folder belongs to the account whose cache holds it"
    );
}

/// What:
///  The creation's own answers over a fixture cache:
///  a folder that already stands is no second folder,
///  and a pin that refuses makes nothing at all.
/// Why:
///  A removal that is run again meets its own folder,
///  and a cache whose mount is not the pinned dataset must write nothing on the
///  home underlay,
///  which is the ordering the accepted staging design rests on.
#[test]
fn the_creation_makes_no_second_folder_and_makes_none_when_the_pin_refuses() {
    let home: PathBuf = fixture("staging-folder-the-creation");
    let cache_path: PathBuf = home.join(".cache");
    std::fs::create_dir_all(cache_path.as_path()).expect("the fixture cache is made");
    let cache: std::fs::File =
        std::fs::File::open(cache_path.as_path()).expect("the fixture cache opens");
    let pin: UserCachePin = the_fixture_pins(&cache);
    let clone_guid: DatasetGuid =
        DatasetGuid::parse("7828944690444299989").expect("the control's GUID is canonical");
    let folder: PathBuf = cache_path.join(staging_folder_name(clone_guid));
    let made: bool = create_the_staging_folder(home.as_path(), &pin, clone_guid)
        .expect("the folder is made over the fixture cache");
    assert!(made, "the first call makes the folder");
    let second: bool = create_the_staging_folder(home.as_path(), &pin, clone_guid)
        .expect("a folder that already stands is no refusal");
    assert!(
        !second,
        "the second call makes no second folder for the same clone"
    );
    let refused_pin: UserCachePin = UserCachePin {
        source: String::from("tank/another-dataset"),
        root_inode: pin.root_inode,
        device: pin.device,
    };
    let other: DatasetGuid =
        DatasetGuid::parse("17157080670377245572").expect("the control's GUID is canonical");
    let refusal: String = create_the_staging_folder(home.as_path(), &refused_pin, other)
        .expect_err("a pin of another dataset refuses the creation");
    assert!(
        refusal.contains("tank/another-dataset"),
        "the refusal names the dataset the pin records: {refusal}"
    );
    assert!(
        std::fs::metadata(cache_path.join(staging_folder_name(other)).as_path()).is_err(),
        "a refused pin made no folder"
    );
    assert!(
        std::fs::metadata(folder.as_path()).is_ok(),
        "the folder the first call made still stands"
    );
}

/// What:
///  One whole arrival at the removal's step 2 over the fixture's own facts,
///  which the controls below differ in one seam of.
/// Why:
///  The two controls that read the folder are about the order rather than the
///  plan,
///  so the arrangement is written once and each control states only what it is
///  about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removal = theRemovalOver(fixture, pool, check, device, head, holders, settle);
/// ```
fn the_removal_over<'a>(
    fixture: &'a RemovalFixture,
    pool: &'a LivePool,
    probe: &'a EpochProbe,
    device: &'a ClearingDevice,
    head: &'a ScriptedHead,
    holders: &'a ScriptedHolders,
    settle: &'a ScriptedSettle,
) -> TheCompletion<'a> {
    return TheCompletion {
        fixture,
        pool,
        check: probe,
        probe,
        device,
        head,
        holders,
        settle,
        cancelling: None,
        budget: the_budget(),
    };
}

/// What:
///  A removal resumed over a plan that already stands makes the folder its
///  unmount binds,
///  at mode `0700`, before it asks for the unmount.
/// Why:
///  A second attempt is the case a retry takes,
///  and the whole removal makes the folder on both ways in:
///  a fresh entry and a plan it adopts.
///  A resumed run whose folder is missing would ask the handler to bind nothing,
///  which is the defect this seam exists to prevent.
#[test]
fn a_resumed_removal_makes_the_folder_its_unmount_binds() {
    let fixture_at: RemovalFixture = a_fixture("staging-folder-on-a-resume");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    fixture_at.plant_a_plan(HeadWord::Present, Vec::new());
    let check: Pool = the_checks_pool_of(MARK_OFF, an_intent_property("unmount"));
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe {
        pool: check,
        hidden: Rc::clone(&hidden),
    };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let clone_guid: DatasetGuid = fixture_at.recorded().clone_guid;
    let head: ScriptedHead =
        ScriptedHead::succeeding(fixture_at.plan_folder(), clone_guid, Rc::clone(&hidden));
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    let home: PathBuf = fixture("staging-folder-on-a-resume-home");
    let cache_path: PathBuf = home.join(".cache");
    std::fs::create_dir_all(cache_path.as_path()).expect("the fixture cache is made");
    let cache: std::fs::File =
        std::fs::File::open(cache_path.as_path()).expect("the fixture cache opens");
    let staging: AStagingFolderOfAFixtureCache = AStagingFolderOfAFixtureCache {
        home: home.clone(),
        pin: the_fixture_pins(&cache),
        asked: Cell::new(0),
    };
    let unmount: AnUnmountThatReadsTheFolder = AnUnmountThatReadsTheFolder {
        clone_guid,
        home,
        seen: RefCell::new(None),
    };
    complete_with(
        &the_removal_over(
            &fixture_at,
            &pool,
            &probe,
            &device,
            &head,
            &holders,
            &settle,
        ),
        crate::worktree_clone_removal_proof_and_plan::removal_completion_fixtures::NOW,
        &staging,
        &unmount,
    )
    .expect("a resumed removal whose folder stands at its unmount finishes");
    assert_eq!(
        staging.asked.get(),
        1,
        "the resumed order asked for the folder once, before it asked for an unmount"
    );
    assert_eq!(
        *unmount.seen.borrow(),
        Some(0o700),
        "the folder stood at mode 0700 when the resumed removal asked for its unmount"
    );
}

/// What:
///  A removal whose lineage proof refuses makes no folder at all.
/// Why:
///  The folder is written, and a removal that cannot prove the clone must not
///  write anything before its refusal,
///  so the creation belongs after the proof and not before it.
#[test]
fn a_refused_proof_makes_no_folder() {
    let fixture_at: RemovalFixture = a_fixture("staging-folder-after-a-refusal");
    let mut pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    for listed in pool.identities.iter_mut() {
        if listed.0 == CLONE {
            listed.1 = DatasetIdentity {
                stats: the_stats(CLONE_GUID + 1, false, CLONE_CREATETXG, 0, ORIGIN_SNAPSHOT),
                properties: NvList::new(),
            };
        }
    }
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe {
        pool: check,
        hidden: Rc::clone(&hidden),
    };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let clone_guid: DatasetGuid = fixture_at.recorded().clone_guid;
    let head: ScriptedHead =
        ScriptedHead::succeeding(fixture_at.plan_folder(), clone_guid, Rc::clone(&hidden));
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    let home: PathBuf = fixture("staging-folder-after-a-refusal-home");
    let cache_path: PathBuf = home.join(".cache");
    std::fs::create_dir_all(cache_path.as_path()).expect("the fixture cache is made");
    let cache: std::fs::File =
        std::fs::File::open(cache_path.as_path()).expect("the fixture cache opens");
    let staging: AStagingFolderOfAFixtureCache = AStagingFolderOfAFixtureCache {
        home: home.clone(),
        pin: the_fixture_pins(&cache),
        asked: Cell::new(0),
    };
    let unmount: AnUnmountThatReadsTheFolder = AnUnmountThatReadsTheFolder {
        clone_guid,
        home: home.clone(),
        seen: RefCell::new(None),
    };
    let stop: RemovalStop = complete_with(
        &the_removal_over(
            &fixture_at,
            &pool,
            &probe,
            &device,
            &head,
            &holders,
            &settle,
        ),
        crate::worktree_clone_removal_proof_and_plan::removal_completion_fixtures::NOW,
        &staging,
        &unmount,
    )
    .expect_err("a clone the pool answers with another GUID is refused");
    assert!(
        matches!(stop, RemovalStop::TheCloneCannotBeProved { .. }),
        "the proof's own refusal, not {stop:?}"
    );
    assert_eq!(
        staging.asked.get(),
        0,
        "the folder was never asked for, because the proof refused first"
    );
    assert_eq!(
        *unmount.seen.borrow(),
        None,
        "nothing was unmounted, so no folder was read at a request"
    );
    assert!(
        std::fs::metadata(cache_path.as_path()).is_ok(),
        "the fixture cache stands"
    );
}
