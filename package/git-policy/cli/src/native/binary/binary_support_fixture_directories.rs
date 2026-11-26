//! What:
//!  Making and removing the disposable directory of a fixture,
//! and the wrapper link inside it.
//! Why:
//!  Making a fixture needs the fixture type and the limits,
//! and nothing in the file of the helpers that run commands uses anything but its result,
//! so the directory work lives in a file of its own.
//!  Where a fixture root is,
//!  how one folder is named and removed,
//!  and the core-dump limit
//!  are written once for this target and the library's unit tests in
//!  `../testing/fixture_support_shared_by_both_test_targets.rs`,
//!  which both include by `#[path]`
//!  (`doc/debt-ledger-2026-11-25.md`, item 11).
//!  This file keeps what only the binary controls need:
//!  the wrapper link a fixture's `bin/git` points at,
//!  the fixture type itself,
//!  and the report a vanished fixture root is answered with.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixture = await mkdtemp(...); await symlink(wrapper, join(fixture, "bin/git"));
//! ```

use std::path::{Path, PathBuf};
use std::process::Command;
use crate::support_fixture_service::Fixture;
use crate::support_program_names_and_time_bound::WRAPPER;
use crate::support_resource_limits::ServiceLaunch;

/// What:
///  The shared fixture helpers this file used to define,
///  named here so every `crate::support_fixture_directories::name` and `support::name` path stays the same.
/// Why:
///  Both test targets now read one copy of the helpers
///  (`../testing/fixture_support_shared_by_both_test_targets.rs`),
///  and a caller that named one of them through this module goes on working.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from '../shared/fixture-support.ts';
/// ```
pub(crate) use crate::fixture_support_shared_by_both_test_targets::{
    FIXTURE_ROOT_VARIABLE, REMOVAL_PAUSE, REMOVAL_TRIES, forbid_core_dumps, fixture_root, made_fixture_folder,
    missing_fixture_root_report, remove_patiently, run_fixture_root, settle_fixture, system_temporary_directories,
};

/// What:
///  Read the fixture's wrapper once per test process,
/// so its pages are in the page cache before any bounded command
///  starts it,
/// and the wait for a slow disk happens here and not under the 5-second test hang guard.
/// Why:
///  Every fixture's wrapper is a hard link of the built executable,
/// so all fixtures of all test processes share one
///  set of cached pages.
/// Starting an executable that is not cached reads most of the file in its first fault (a 22 MB
///  debug build read 16 MiB;
/// `doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`).
pub fn warm_wrapper(linked: &Path) {
    /// Set once the executable has been read by this process.
    static WARMED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    WARMED.get_or_init(|| {
        use std::io::Read;
        let mut file: std::fs::File = std::fs::File::open(linked).expect("open the fixture wrapper");
        let mut chunk: Vec<u8> = vec![0; 1 << 20];
        while file.read(chunk.as_mut_slice()).expect("read the built wrapper") > 0 {}
    });
}

/// What:
///  Give a fixture its wrapper path:
/// a hard link of the built executable;
/// when the link fails because the fixture is on
///  another filesystem (`EXDEV`),
/// a hard link of the one stored copy of the build in `store_root`;
/// and a data copy
///  only where neither link works.
/// Why:
///  The wrapper never reads its own path except to recognise itself by file identity and to find hook entries by
///  the name it was started under (`real_git_candidate::same_file`,
/// `main.rs`),
/// and no control writes to it,
/// so a
///  private copy bought nothing but 22 MB of writes per fixture and a cold first start of a new inode per test.
///  The fallback follows the link's own failure and not a comparison of device numbers,
/// which bind mounts defeat.
///  The store of a run stands in the run's shared fixture root, so every test process of one run links the one copy
///  (`fixture_in`).
///  169 fixtures that each copied wrote about 3.7 GB (169 times 21.9 MB) in one wave
///  (`doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`).
pub fn link_built_wrapper(built: &Path, to: &Path, store_root: &Path) {
    if let Err(error) = std::fs::hard_link(built, to) {
        let stored: std::io::Result<()> = if error.raw_os_error() == Some(libc::EXDEV) {
            super::wrapper_store::link_stored(built, store_root, to, &mut |_| {})
        } else {
            Err(error)
        };
        if let Err(reason) = stored {
            eprintln!("fixture wrapper: no hard link of {} at {} ({reason}); copying it instead", built.display(), to.display());
            copy_executable(built, to);
        }
    }
    // After the link, not before: on overlay storage a hard link of an image-layer file copies the file up to a
    // new inode, whose pages are the ones every fixture then shares and the ones a start needs.
    warm_wrapper(to);
}

/// Create a fixture in the fixture root:
///  `<root>/wrapper/monochromatic-git-policy-cli` (hard link of the built executable, of the run's one stored copy of
///  it, or a copy of its bytes, as `fixture_in` says) and `<root>/bin/git` (symbolic link to it).
pub fn fixture(name: &str) -> Fixture {
    return fixture_in(fixture_root().as_path(), run_fixture_root().as_path(), name);
}

/// What:
///  Whether this run may go on to make a fixture whose name is not valid UTF-8,
///  starting the same test inside a namespace of its own where this run's fixture root refuses such a name.
/// Why:
///   A control that writes such a name proves nothing where the file system refuses it,
///  and it must not pass by returning early either,
///  so the decision and the tmpfs both live in one place
///  (`fixture_root_taking_any_name`,
///  whose doc says how the re-execution works);
///  this wrapper only supplies the root the binary controls use,
///  which is the `test-fixtures/process-<pid>` folder beside the build output,
///  or the `process-<pid>` folder under the directory `CLI_GIT_TEST_ROOT` names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!inAFixtureRootTakingAnyName('event_path::events_carry_the_exact_bytes_of_a_name_that_is_not_utf8')) return;
/// ```
pub fn in_a_fixture_root_taking_any_name(test: &str) -> bool {
    return crate::fixture_root_taking_any_name::in_a_root_taking_any_name(fixture_root().as_path(), test);
}

/// What:
///  Create a fixture in `directory` whose wrapper comes from `store_root`,
///  which may be on another filesystem than the build;
///  see `fixture`.
/// Why:
///   The wrapper is a hard link of the built executable where `directory`'s file system is the build's,
///  and otherwise a hard link of the one stored copy of the build in `store_root`
///  (`wrapper_store`, which publishes the copy without a lock and prunes another build's idle ones).
///  The store of a run stands in the run's shared fixture root,
///  so every process of the run links the one copy instead of writing a copy of the whole executable into a folder of
///  its own:
///  the gate's root is a 768 MiB tmpfs, and a copy per process filled it (issue 6488).
///  A control that puts its fixtures on a file system of its own passes that file system as both,
///  because a store on the build's file system cannot be linked there either.
///  Every call names a folder of its own,
///  so two tests that pass the same name never share one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const root = madeFixtureFolder(directory, 'native-binary', name);
/// linkBuiltWrapper(WRAPPER, join(root, 'wrapper', 'monochromatic-git-policy-cli'), storeRoot);
/// ```
pub fn fixture_in(directory: &Path, store_root: &Path, name: &str) -> Fixture {
    forbid_core_dumps();
    let root: PathBuf = made_fixture_folder(directory, "native-binary", name);
    for subdirectory in ["wrapper", "bin", "home"] {
        std::fs::create_dir(root.join(subdirectory)).expect("fixture directory");
    }
    let wrapper: PathBuf = root.join("wrapper/monochromatic-git-policy-cli");
    link_built_wrapper(Path::new(WRAPPER), wrapper.as_path(), store_root);
    std::os::unix::fs::symlink(&wrapper, root.join("bin/git")).expect("git-named link");
    return Fixture {
        root,
        wrapper,
        service: std::sync::Mutex::new(None),
        service_environment: std::sync::Mutex::new(Vec::new()),
        service_launch: std::sync::Mutex::new(ServiceLaunch::default()),
        socket_folders: std::sync::Mutex::new(crate::support_socket_folders::SocketFolders::default()),
    };
}

/// Remove only the fixture directory a control created.
pub fn remove(fixture: &Fixture) {
    remove_patiently(&fixture.root, &|root| { return std::fs::remove_dir_all(root); }, REMOVAL_TRIES, REMOVAL_PAUSE).expect("remove only the fixture");
}

/// The one writer of executable fixture files,
/// shared with the unit tests.
#[path = "../testing/fixture_executable.rs"]
mod fixture_executable;

pub use fixture_executable::executable;

/// What:
///  Copy an executable through a child `cp` process,
///  keeping its mode.
/// Why:
///   A copy made by this process would be open for writing here,
///  with the same
///       "Text file busy" window as `executable` describes.
///  Fixtures reach the wrapper through a hard link (`fixture`),
/// so this is for controls that need a second file with
///  the wrapper's bytes and a different identity.
///  The copy is a real data copy (`--reflink=never`):
/// on btrfs,
/// which also backs overlay storage,
/// the default
///  `cp` clones the extents,
/// and the clone has no pages in the page cache,
/// so the first start of the copy reads
///  most of the executable from disk inside the 5-second test hang guard (a 22 MB debug build read 16 MiB in 4 major faults;
///  `doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`).
/// A data copy leaves the pages
///  it wrote in the cache,
/// and a slow read of the original happens here,
/// outside any bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawnSync('cp', ['--reflink=never', '--', from, to]);
/// ```
pub fn copy_executable(from: &Path, to: &Path) {
    let copied: bool = Command::new("cp")
        .arg("--reflink=never")
        .arg("--")
        .arg(from)
        .arg(to)
        .status()
        .expect("start fixture copy")
        .success();
    assert!(copied, "fixture copy failed: {from:?} to {to:?}");
}
