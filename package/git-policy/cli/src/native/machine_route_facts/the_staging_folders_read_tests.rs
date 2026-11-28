//! What:
//!  The controls of the staging folders under the account's cache:
//!  one folder a removal derives,
//!  one that holds an entry,
//!  one whose name no removal derives,
//!  a cache that is not there,
//!  and a machine whose environment names no home.
//! Why:
//!  A staging folder another account could write,
//!  one holding content and one whose name nothing derived are three states a
//!  person must see,
//!  and a cache nobody can name must answer unknown rather than the pass an
//!  empty listing gets.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theStagingFacts({ places, account }).folders).toEqual([{ name, state, derived: true, empty: true }]);
//! ```

/// The places a fixture names.
use super::super::the_route_places::TheRoutePlaces;
/// The read a control judges.
use super::the_staging_facts;
/// What one look at a path answered.
use crate::doctor_zfs_route::handler_checks::PathState;
/// One folder under the cache and the facts the check reads.
use crate::doctor_zfs_route::staging_folder_checks::{
    StagedFolder,
    StagingFolderFacts,
};
/// One fixture folder of this call's own.
use crate::testing::test_support::fixture;
/// The staging prefix every folder's name begins with.
use crate::worktree_clone_unmount::staging_folder_name::STAGING_FOLDER_PREFIX;
/// The owner and mode bits a fixture's folders are given.
use std::os::unix::fs::PermissionsExt;
/// The owned and borrowed paths these controls use.
use std::path::{
    Path,
    PathBuf,
};

/// The account a control states,
/// which owns every folder the control makes.
const AN_ACCOUNT: u32 = 1000;

/// What:
///  A folder a removal derives answers its name,
///  its owner and mode,
///  its derivation and that it holds no entry,
///  and a folder holding an entry answers that it holds one.
/// Why:
///  The staging check reports each of the four facts beside the folder's name,
///  so a control must see them apart to know which sentence a state prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(folders[0]).toMatchObject({ derived: true, empty: true });
/// expect(folders[1]).toMatchObject({ derived: true, empty: false });
/// ```
#[test]
fn a_derived_folder_answers_its_mode_its_derivation_and_its_emptiness() {
    let root: PathBuf = fixture("staging-folders");
    let places: TheRoutePlaces = TheRoutePlaces::under(root.as_path());
    let cache: PathBuf = places.the_cache_or_why().expect("a fixture names a home");
    make_folder(cache.as_path(), 0o700);
    let empty: PathBuf = cache.join(format!("{STAGING_FOLDER_PREFIX}1837462836473829"));
    make_folder(empty.as_path(), 0o700);
    let holding: PathBuf = cache.join(format!("{STAGING_FOLDER_PREFIX}1837462836473830"));
    make_folder(holding.as_path(), 0o700);
    std::fs::write(holding.join("a-file"), b"x").expect("the entry");
    let facts: StagingFolderFacts = the_staging_facts(&places, AN_ACCOUNT);
    assert_eq!(
        facts.cache.state,
        PathState::Directory {
            owner: crate::worktree_clone_add_route::add_grants::this_account(),
            mode: 0o700,
        }
    );
    let derived: &StagedFolder = facts
        .folders
        .iter()
        .find(|folder| return folder.name.ends_with("1837462836473829"))
        .expect("the empty folder");
    assert!(derived.derived);
    assert!(derived.empty);
    let filled: &StagedFolder = facts
        .folders
        .iter()
        .find(|folder| return folder.name.ends_with("1837462836473830"))
        .expect("the folder holding an entry");
    assert!(filled.derived);
    assert!(!filled.empty);
}

/// What:
///  A name the staging prefix begins but no removal derives answers that its
///  name is not derived.
/// Why:
///  Removing a name nothing derived would destroy a stranger's folder,
///  so the check reports the name and leaves it in place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(folder).toMatchObject({ derived: false });
/// ```
#[test]
fn a_name_nothing_derived_answers_that_its_name_is_not_derived() {
    let root: PathBuf = fixture("staging-stranger");
    let places: TheRoutePlaces = TheRoutePlaces::under(root.as_path());
    let cache: PathBuf = places.the_cache_or_why().expect("a fixture names a home");
    make_folder(cache.as_path(), 0o700);
    let stranger: PathBuf = cache.join(format!("{STAGING_FOLDER_PREFIX}not-a-guid"));
    make_folder(stranger.as_path(), 0o700);
    let facts: StagingFolderFacts = the_staging_facts(&places, AN_ACCOUNT);
    assert_eq!(facts.folders.len(), 1);
    assert!(!facts.folders[0].derived);
}

/// What:
///  A cache that is not there answers absent,
///  and a machine whose environment names no home answers unknown with the
///  sentence.
/// Why:
///  A cache nobody set up holds no staging folder,
///  while a cache nobody can name proves nothing about one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theStagingFacts({ places, account }).cache.state).toEqual({ kind: 'absent' });
/// ```
#[test]
fn no_cache_is_absent_and_no_home_is_unknown() {
    let root: PathBuf = fixture("staging-no-cache");
    let absent: TheRoutePlaces = TheRoutePlaces::under(root.as_path());
    let facts: StagingFolderFacts = the_staging_facts(&absent, AN_ACCOUNT);
    assert_eq!(facts.cache.state, PathState::Absent);
    let mut homeless: TheRoutePlaces = TheRoutePlaces::under(root.as_path());
    homeless.home = None;
    let homeless_facts: StagingFolderFacts = the_staging_facts(&homeless, AN_ACCOUNT);
    let PathState::Unreadable {
        why,
    } = &homeless_facts.cache.state
    else {
        panic!("no home is unknown: {:?}", homeless_facts.cache.state);
    };
    assert!(why.contains("names no home"), "{why}");
    assert_eq!(homeless_facts.account, AN_ACCOUNT);
}

/// One folder with the mode a staging folder is made with,
/// and its parents.
fn make_folder(
    folder: &Path,
    mode: u32,
) {
    std::fs::create_dir_all(folder).expect("a folder");
    std::fs::set_permissions(folder, std::fs::Permissions::from_mode(mode))
        .expect("a folder's mode");
}
