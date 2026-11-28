//! What:
//!  The controls of the places a machine fact is read from:
//!  the release's own constants,
//!  a fixture root that changes no component of a place,
//!  and the account's cache or the sentence that names no home.
//! Why:
//!  Every other control of this folder reads a fixture's places,
//!  so a place that escaped its fixture root would point at this machine and
//!  the control would read the wrong file,
//!  and a cache nobody can name must answer a sentence rather than a path at
//!  the file system's root.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(TheRoutePlaces.under('/tmp/x').configuration).toBe('/tmp/x/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc');
//! ```

/// The places one control reads,
/// and the two ways a machine's own places are made.
use super::TheRoutePlaces;
/// A fixture folder of this call's own.
use crate::testing::test_support::fixture;
/// The constants the live places are built from.
use crate::worktree_clone_mount::SLOT_IDENTITY_STORE_PATH;
/// The configuration's own place.
use crate::worktree_clone_mount_handler::handler_install_paths::CONFIGURATION_PATH;
/// The owned paths these controls compare.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  A fixture root takes nothing away from a place:
///  the configuration,
///  the store,
///  the readiness folder and the boot identifier keep every component they
///  carry on this machine,
///  under the root.
/// Why:
///  A place built with `Path::join` and an absolute path would replace the root
///  and read this machine's file,
///  which is the one failure a fixture's places exist to prevent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(places.configuration).toBe(`${root}${CONFIGURATION_PATH}`);
/// ```
#[test]
fn a_fixture_root_keeps_every_component_of_a_place() {
    let root: PathBuf = fixture("places-under-a-root");
    let places: TheRoutePlaces = TheRoutePlaces::under(root.as_path());
    let expected_configuration: PathBuf = root.join(
        Path::new(CONFIGURATION_PATH)
            .strip_prefix("/")
            .expect("an absolute place"),
    );
    assert_eq!(places.configuration, expected_configuration);
    let expected_store: PathBuf = root.join(
        Path::new(SLOT_IDENTITY_STORE_PATH)
            .strip_prefix("/")
            .expect("an absolute place"),
    );
    assert_eq!(places.store, expected_store);
    assert_eq!(
        places.readiness,
        root.join("run/git-worktree-clone-mount-readiness")
    );
    assert_eq!(
        places.boot_identity,
        root.join("proc/sys/kernel/random/boot_id")
    );
    assert_eq!(places.mount_table, root.join("proc/self/mountinfo"));
}

/// What:
///  A fixture's home is the fixture's own,
///  so the cache hangs under the root and never under the person's home.
/// Why:
///  The staging folder is read under the account's cache,
///  and a control that read the person's own cache would report a machine
///  nobody set up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(places.theCacheOrWhy()).toBe(`${root}/home/.cache`);
/// ```
#[test]
fn a_fixtures_cache_hangs_under_the_root() {
    let root: PathBuf = fixture("places-cache");
    let places: TheRoutePlaces = TheRoutePlaces::under(root.as_path());
    assert_eq!(
        places.the_cache_or_why().expect("a fixture names a home"),
        root.join("home").join(".cache")
    );
}

/// What:
///  A machine whose environment names no home answers the sentence,
///  not a cache at the file system's root.
/// Why:
///  `.cache` alone is any account's,
///  so a cache path built without a home would read a folder the route never
///  made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(places.theCacheOrWhy()).toThrow('the environment names no home');
/// ```
#[test]
fn no_home_answers_the_sentence_rather_than_a_root_cache() {
    let mut places: TheRoutePlaces = TheRoutePlaces::under(Path::new("/nonexistent"));
    places.home = None;
    let why: String = places
        .the_cache_or_why()
        .expect_err("no home names no cache");
    assert!(why.contains("names no home"), "{why}");
    assert!(why.contains(".cache"), "{why}");
}

/// What:
///  The live places name the release's own configuration and store,
///  and the device the binding opens.
/// Why:
///  A gather over the live places reads the files a person sees in a report,
///  so the report's path and the install's path are one constant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(TheRoutePlaces.live().configuration).toBe('/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc');
/// ```
#[test]
fn the_live_places_name_the_release_constants() {
    let places: TheRoutePlaces = TheRoutePlaces::live();
    assert_eq!(places.configuration, PathBuf::from(CONFIGURATION_PATH));
    assert_eq!(places.store, PathBuf::from(SLOT_IDENTITY_STORE_PATH));
    assert_eq!(
        places.device,
        PathBuf::from(monochromatic_zfs_request_to_openzfs_kernel_device::DEVICE_PATH)
    );
    assert_eq!(places.zedlets_folder, PathBuf::from("/etc/zfs/zed.d"));
    assert_eq!(places.zed_units.len(), 3);
}
