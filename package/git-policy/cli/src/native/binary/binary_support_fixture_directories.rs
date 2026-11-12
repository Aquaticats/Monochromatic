//! What:
//!  Making and removing the disposable directory of a fixture,
//! and the wrapper link inside it.
//! Why:
//!  Making a fixture needs the fixture type and the limits,
//! and nothing in the file of the helpers that run commands uses anything but its result,
//! so the directory work lives in a file of its own.
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

/// Environment variable that moves the fixture root,
/// for example onto a tmpfs for speed.
pub const FIXTURE_ROOT_VARIABLE: &str = "CLI_GIT_TEST_ROOT";

/// The directory fixtures are created in.
///  Cargo's per-target scratch directory,
/// never `/tmp`:
/// a shared tmpfs with a per-user quota fills up
///  when failing tests leave fixtures behind (issue 1809),
/// and then every command of every session fails.
///  A container gate keeps that directory inside the wrapper crate's own folder (`target/tmp`),
/// so a fixture crate written below the fixture root must declare a `[workspace]` of its own
///  (`binary_package_repair_tests.rs`,
///  `shared_target_manifest`),
/// or Cargo finds the wrapper crate above it and refuses to build the fixture
///  (`doc/troubleshooting/cargo-fixture-crate-joins-the-wrapper-workspace.md`).
pub fn fixture_root() -> PathBuf {
    let root: PathBuf = std::env::var_os(FIXTURE_ROOT_VARIABLE)
        .map_or_else(|| { return PathBuf::from(env!("CARGO_TARGET_TMPDIR")); }, PathBuf::from);
    std::fs::create_dir_all(&root).expect("fixture root");
    return root;
}

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
///  `<root>/wrapper/monochromatic-git-policy-cli` (hard link of the built executable) and `<root>/bin/git` (symbolic link to it).
pub fn fixture(name: &str) -> Fixture {
    return fixture_in(fixture_root().as_path(), name);
}

/// What:
///  What is known about a fixture root that could not be used,
/// so the failure names the folder that went:
/// whether the root exists,
/// and what its parent and its grandparent hold,
/// or that they are gone too.
/// Why:
///  A run of the gate once lost its fixture root between `create_dir_all` and the next `create_dir`,
/// and 217 later tests failed with a bare `NotFound`
/// (`doc/handover/repository-move-dangling-link-items.md`,
/// open harness item);
/// nothing in the log said what removed it or how far up the loss went.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = `the fixture root ${root} ${await exists(root) ? 'exists' : 'does not exist'}; ...`;
/// ```
pub fn missing_fixture_root_report(root: &Path) -> String {
    let parent: &Path = root.parent().unwrap_or(root);
    let grandparent: &Path = parent.parent().unwrap_or(parent);
    let state: &str = if root.exists() { "exists" } else { "does not exist" };
    return format!(
        "the fixture root {} {state}; {}; {}",
        root.display(),
        what_a_folder_holds("its parent", parent),
        what_a_folder_holds("its grandparent", grandparent)
    );
}

/// How many entries of a folder the report names before it says there are more.
const REPORTED_ENTRIES: usize = 40;

/// The phrase that says what a folder holds,
/// or that it does not exist,
/// or why it cannot be listed.
fn what_a_folder_holds(label: &str, folder: &Path) -> String {
    if !folder.exists() {
        return format!("{label} {} does not exist", folder.display());
    }
    let listing = match std::fs::read_dir(folder) {
        Ok(listed) => listed,
        Err(error) => return format!("{label} {} cannot be listed: {error}", folder.display()),
    };
    let mut names: Vec<String> = Vec::new();
    for entry in listing.flatten() {
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    names.sort();
    let more: usize = names.len().saturating_sub(REPORTED_ENTRIES);
    names.truncate(REPORTED_ENTRIES);
    let tail: String = if more == 0 { String::new() } else { format!(" and {more} more") };
    return format!("{label} {} holds: {}{tail}", folder.display(), names.join(", "));
}

/// Create a fixture in `directory`,
/// which may be on another filesystem than the build;
/// see `fixture`.
pub fn fixture_in(directory: &Path, name: &str) -> Fixture {
    forbid_core_dumps();
    if let Err(error) = std::fs::create_dir_all(directory) {
        // A root that vanished (a container gate lost `target/tmp` after 5,018 of 5,265 tests on 2026-11-04) leaves nothing
        //  to say what became of it: the report names the root, its parent and its grandparent.
        panic!("fixture directory root: {error}; {}", missing_fixture_root_report(directory));
    }
    let created: PathBuf = directory.join(format!("native-binary-{}-{name}", std::process::id()));
    if created.exists() {
        std::fs::remove_dir_all(&created).expect("remove stale fixture");
    }
    if let Err(error) = std::fs::create_dir(&created) {
        panic!("fresh fixture: {error}; {}", missing_fixture_root_report(directory));
    }
    let root: PathBuf = std::fs::canonicalize(&created).expect("canonical fixture");
    for subdirectory in ["wrapper", "bin", "home"] {
        std::fs::create_dir(root.join(subdirectory)).expect("fixture directory");
    }
    let wrapper: PathBuf = root.join("wrapper/monochromatic-git-policy-cli");
    link_built_wrapper(Path::new(WRAPPER), wrapper.as_path(), directory);
    std::os::unix::fs::symlink(&wrapper, root.join("bin/git")).expect("git-named link");
    return Fixture { root, wrapper, service: std::sync::Mutex::new(None), service_environment: std::sync::Mutex::new(Vec::new()), service_launch: std::sync::Mutex::new(ServiceLaunch::default()) };
}

/// How many times a removal that finds the directory still gaining entries is tried,
/// and the pause between tries.
pub const REMOVAL_TRIES: usize = 50;

/// The pause between two tries of a removal.
pub const REMOVAL_PAUSE: std::time::Duration = std::time::Duration::from_millis(100);

/// What:
///  Remove the tree at `root` with `attempt`,
/// trying again while the directory is not empty.
/// Why:
///   A landed commit or a command that stopped a service leaves a child finishing for a moment,
/// and removing a directory that
///  still gains entries fails with "not empty";
/// that is an event that ends by itself,
/// so the removal waits for it.
/// A tree that
///  is already gone is not a failure,
/// and any other error is returned at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(root, { recursive: true, maxRetries: tries, retryDelay: pause })
/// ```
pub fn remove_patiently(root: &Path, attempt: &dyn Fn(&Path) -> std::io::Result<()>, tries: usize, pause: std::time::Duration) -> std::io::Result<()> {
    let mut last: std::io::Result<()> = Ok(());
    for _ in 0..tries {
        match attempt(root) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => {
                last = Err(error);
                std::thread::sleep(pause);
            }
            Err(error) => return Err(error),
        }
    }
    return last;
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

/// What:
///  Forbid core dumps for this test process and everything it starts,
/// once.
/// Why:
///   The binary tests kill services and abort wrappers on purpose,
/// and each dump costs the system's core-dump handler processor
///  time and disk writes on a disk other sessions share.
/// The limit is inherited across `fork` and `exec`,
/// and the hard limit
///  stops a child from raising it;
/// it is the harness's,
/// and the product sets none.
pub fn forbid_core_dumps() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let none: libc::rlimit = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
        // SAFETY: `setrlimit` reads the one `rlimit` it is given and touches no memory of ours.
        let _set = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &none) };
    });
}
