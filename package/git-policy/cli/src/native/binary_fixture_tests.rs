//! Binary-level controls for the fixtures themselves: where they live and that they go away however their test ends
//! (issue 1809).

use super::support::{FIXTURE_ROOT_VARIABLE, Fixture, fixture, settle_fixture};
use std::path::{Path, PathBuf};

/// A fixture whose test panics after creating it leaves nothing behind.
#[test]
fn a_fixture_is_removed_when_its_test_panics() {
    let (sender, receiver) = std::sync::mpsc::channel::<PathBuf>();
    let handle = std::thread::spawn(move || {
        let created: Fixture = fixture("drop-on-panic");
        std::fs::write(created.root.join("file"), "content").expect("file");
        sender.send(created.root.clone()).expect("send the root");
        panic!("the test fails before it removes its fixture");
    });
    assert!(handle.join().is_err(), "the thread panics on purpose");
    let root: PathBuf = receiver.recv().expect("the root");
    assert!(!root.exists(), "{} was left behind", root.display());
}

/// A fixture removed explicitly and then dropped does not fail the drop.
#[test]
fn a_fixture_removed_explicitly_is_dropped_quietly() {
    let created: Fixture = fixture("drop-after-remove");
    let root: PathBuf = created.root.clone();
    super::support::remove(&created);
    assert!(!root.exists());
    drop(created);
}

/// Settling keeps a tree on request, removes it otherwise, and accepts a tree that is already gone.
#[test]
fn settling_keeps_removes_and_accepts_a_tree_that_is_gone() {
    let created: Fixture = fixture("settle");
    let root: &Path = created.root.as_path();
    std::fs::write(root.join("file"), "content").expect("file");
    settle_fixture(root, true);
    assert!(root.join("file").exists(), "a kept fixture stays");
    settle_fixture(root, false);
    assert!(!root.exists(), "a settled fixture goes");
    settle_fixture(root, false);
}

/// Fixtures are created in Cargo's per-target scratch directory, not in the system temporary directory, unless the
/// person moved the root on purpose.
#[test]
fn fixtures_are_created_outside_the_system_temporary_directory() {
    if std::env::var_os(FIXTURE_ROOT_VARIABLE).is_some() {
        return;
    }
    let created: Fixture = fixture("location");
    assert!(
        created.root.starts_with(std::fs::canonicalize(env!("CARGO_TARGET_TMPDIR")).expect("target scratch")),
        "{} is not under the target scratch directory",
        created.root.display()
    );
}

/// Two paths on different filesystems are told apart, and a path with itself is not.
/// `/proc` is a filesystem of its own on Linux, whatever the host's disks are.
#[test]
fn the_filesystem_comparison_tells_a_separate_filesystem_apart() {
    let built: &Path = Path::new(super::support::WRAPPER);
    assert!(super::support::same_filesystem(built, built));
    assert!(!super::support::same_filesystem(Path::new("/proc"), built));
    assert!(!super::support::same_filesystem(built, Path::new("/nonexistent-path-for-the-comparison")));
}

/// What:
///  However many callers ask at once, every one ends with a link to the same final file holding the build's bytes,
///  an older build's copy is removed while they link, and nothing but the final file stays.
/// Why:
///  The store publishes without a lock: each caller writes its own partial file and links it to the final name, so
///  the final name is complete whenever it exists and the first publisher wins. Each caller links the result into
///  its own directory straight away, which is what a fixture does while another caller removes older copies.
#[test]
fn concurrent_callers_end_with_links_to_one_final_file_of_the_build() {
    use std::os::unix::fs::MetadataExt;
    let created: Fixture = fixture("store-once");
    let built: &Path = Path::new(super::support::WRAPPER);
    let store: PathBuf = created.root.join(super::wrapper_store::STORE_DIRECTORY);
    std::fs::create_dir_all(&store).expect("store");
    let older: PathBuf = store.join("1-0000000000000000000-cli-git-native");
    std::fs::write(&older, b"older build").expect("older copy");
    let callers: usize = std::thread::available_parallelism().map_or(4, std::num::NonZeroUsize::get).max(4);
    let links: Vec<PathBuf> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..callers)
            .map(|caller| {
                let (root, link): (&Path, PathBuf) = (created.root.as_path(), created.root.join(format!("link-{caller}")));
                return scope.spawn(move || {
                    let stored: PathBuf = super::wrapper_store::shared_copy(built, root).expect("stored copy");
                    std::fs::hard_link(&stored, &link).expect("a fixture links the stored copy");
                    return link;
                });
            })
            .collect();
        return handles.into_iter().map(|handle| { return handle.join().expect("caller finished"); }).collect();
    });
    let identity = |path: &Path| { let metadata = std::fs::metadata(path).expect("metadata"); return (metadata.dev(), metadata.ino()); };
    assert!(links.iter().all(|link| { return identity(link) == identity(&links[0]); }), "the links are not one file");
    let names: Vec<String> = std::fs::read_dir(&store)
        .expect("store")
        .map(|entry| { return entry.expect("entry").file_name().to_string_lossy().into_owned(); })
        .collect();
    assert_eq!(names.len(), 1, "one final file of the one build, no partial and no older copy: {names:?}");
    assert!(!older.exists(), "the older build's copy stayed");
    assert_eq!(std::fs::read(&links[0]).expect("stored bytes"), std::fs::read(built).expect("built bytes"));
    let again: PathBuf = super::wrapper_store::shared_copy(built, created.root.as_path()).expect("later ask");
    assert_eq!(identity(&again), identity(&links[0]), "a later ask replaced the final file");
}

/// A caller that publishes after another has won leaves the winner's file as the stored copy, removes its own partial
/// file, and succeeds; a caller that publishes first becomes the stored copy.
#[test]
fn a_late_publisher_leaves_the_winners_file_in_place() {
    use std::os::unix::fs::MetadataExt;
    let created: Fixture = fixture("store-publish");
    let stored: PathBuf = created.root.join("stored");
    let (winner, loser): (PathBuf, PathBuf) = (created.root.join("winner"), created.root.join("loser"));
    std::fs::write(&winner, b"winner bytes").expect("winner");
    std::fs::write(&loser, b"loser bytes").expect("loser");
    super::wrapper_store::publish(winner.as_path(), stored.as_path()).expect("the first publisher");
    let winner_inode: u64 = std::fs::metadata(&stored).expect("stored").ino();
    assert!(!winner.exists(), "the first publisher removes its partial file");
    super::wrapper_store::publish(loser.as_path(), stored.as_path()).expect("a late publisher succeeds");
    assert!(!loser.exists(), "the late publisher removes its partial file");
    assert_eq!(std::fs::read(&stored).expect("stored bytes"), b"winner bytes");
    assert_eq!(std::fs::metadata(&stored).expect("stored").ino(), winner_inode, "the stored file was replaced");
}

/// A copy of an older build leaves the store when the current build is stored.
#[test]
fn an_older_builds_copy_leaves_the_store() {
    let created: Fixture = fixture("store-older");
    let store: PathBuf = created.root.join(super::wrapper_store::STORE_DIRECTORY);
    std::fs::create_dir_all(&store).expect("store");
    let older: PathBuf = store.join("1-0000000000000000000-cli-git-native");
    std::fs::write(&older, b"older build").expect("older copy");
    let stored: PathBuf = super::wrapper_store::shared_copy(Path::new(super::support::WRAPPER), created.root.as_path())
        .expect("stored copy");
    assert!(!older.exists(), "{} stayed", older.display());
    assert!(stored.exists());
}

/// What:
///  How many of the first extents of a file the filesystem reports as shared with another file,
///  absent when the filesystem cannot report extents.
/// Why:
///  A reflink clone shares every extent with its source (`FIEMAP_EXTENT_SHARED`), and a clone has no pages in the
///  page cache, so starting it reads the executable from disk.
fn shared_extent_count(path: &Path) -> Option<usize> {
    use std::os::fd::AsRawFd;
    /// `_IOWR('f', 11, struct fiemap)`.
    const FS_IOC_FIEMAP: u64 = 0xC020_660B;
    /// Extent flag: the extent is shared with another file.
    const FIEMAP_EXTENT_SHARED: u32 = 0x2000;
    /// Size of `struct fiemap` before its extents.
    const HEADER_BYTES: usize = 32;
    /// Size of one `struct fiemap_extent`.
    const EXTENT_BYTES: usize = 56;
    /// Extents asked for; a clone shares all of them, so the first ones are enough.
    const EXTENT_SLOTS: usize = 256;
    let file: std::fs::File = std::fs::File::open(path).expect("open the copy");
    let mut buffer: Vec<u8> = vec![0; HEADER_BYTES + EXTENT_BYTES * EXTENT_SLOTS];
    // fm_start stays 0; fm_length covers the whole file; fm_flags stays 0 so no write-back is forced.
    buffer[8..16].copy_from_slice(&u64::MAX.to_ne_bytes());
    buffer[24..28].copy_from_slice(&u32::try_from(EXTENT_SLOTS).expect("slot count").to_ne_bytes());
    // SAFETY: `buffer` is a live allocation of the size the request describes and the kernel writes only inside it.
    let result: i32 = unsafe { libc::ioctl(file.as_raw_fd(), FS_IOC_FIEMAP as _, buffer.as_mut_ptr()) };
    if result != 0 {
        return None;
    }
    let mapped: usize = usize::try_from(u32::from_ne_bytes(buffer[20..24].try_into().expect("count bytes")))
        .expect("mapped count");
    let mut shared: usize = 0;
    for index in 0..mapped.min(EXTENT_SLOTS) {
        let flags_at: usize = HEADER_BYTES + EXTENT_BYTES * index + 40;
        let flags: u32 = u32::from_ne_bytes(buffer[flags_at..flags_at + 4].try_into().expect("flag bytes"));
        if flags & FIEMAP_EXTENT_SHARED != 0 {
            shared += 1;
        }
    }
    return Some(shared);
}

/// A byte copy a control makes is a data copy: a clone starts by reading most of the executable from disk,
/// which stalled the first wrapped command of a test for seconds on a busy disk and hit the 5-second bound.
#[test]
fn a_byte_copy_of_the_wrapper_shares_no_storage_with_the_built_executable() {
    let created: Fixture = fixture("copy-storage");
    let copy: PathBuf = created.root.join("byte-copy");
    super::support::copy_executable(Path::new(super::support::WRAPPER), copy.as_path());
    match shared_extent_count(copy.as_path()) {
        Some(shared) => assert_eq!(
            shared,
            0,
            "{} shares {shared} extents with {}",
            copy.display(),
            super::support::WRAPPER
        ),
        None => eprintln!("the filesystem under {} reports no extents; nothing to check", created.root.display()),
    }
}

/// What:
///  Every fixture starts one shared file with the built executable's bytes, through a hard link: the build itself where
///  the fixture is on its filesystem, else the one stored copy of it. One set of cached pages then serves all fixtures
///  of all test processes, and the cold read of a slow disk is paid once, not once per test.
/// Why:
///  Cargo's scratch directory can be on another volume than the final artifacts (a relocated `build.build-dir`), where
///  every fixture used to copy 22 MB instead of linking: 169 fixtures wrote about 3.7 GB (169 times 21.9 MB) in one
///  wave, and a start of a freshly written copy is a start of a new inode. Only a root the person moved with
///  `CLI_GIT_TEST_ROOT` gets a per-fixture copy.
#[test]
fn fixtures_share_one_wrapper_file_with_the_built_executables_bytes() {
    use std::os::unix::fs::MetadataExt;
    if std::env::var_os(FIXTURE_ROOT_VARIABLE).is_some() {
        return;
    }
    let first: Fixture = fixture("share-first");
    let second: Fixture = fixture("share-second");
    let identity = |fixture: &Fixture| {
        let metadata = std::fs::metadata(&fixture.wrapper).expect("fixture wrapper");
        return (metadata.dev(), metadata.ino());
    };
    assert_eq!(
        identity(&first),
        identity(&second),
        "{} and {} are separate files",
        first.wrapper.display(),
        second.wrapper.display()
    );
    assert_eq!(
        std::fs::read(&first.wrapper).expect("fixture bytes"),
        std::fs::read(super::support::WRAPPER).expect("built bytes"),
    );
}
