//! Binary-level controls for the stored copy of the wrapper that fixtures on another filesystem than the build link:
//! its name is the build's content, it is published without a lock, a copy that disappears before a fixture links it
//! is made again, and fixtures in any root share one file.

use super::support::{Fixture, WRAPPER, fixture, fixture_in};
use super::wrapper_store::{
    IDLE_BUILD_AGE, PARTIAL_PREFIX, RESERVE_ATTEMPTS, STORE_DIRECTORY, content_name, link_stored, publish, remove_idle_builds,
    remove_idle_builds_at, BuildName, name_once, reserve_partial, shared_copy, shared_copy_named, shared_copy_waiting, wait_for_publisher, wait_for_publisher_on, WaitClock, WaitEnd,
};

use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Barrier, OnceLock};
use std::time::Duration;

/// The device and inode of a file, which name the file whatever path reaches it.
fn identity(path: &Path) -> (u64, u64) {
    let metadata: std::fs::Metadata = std::fs::metadata(path).expect("metadata");
    return (metadata.dev(), metadata.ino());
}

/// The names in a directory, sorted, hidden ones included.
fn names_in(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(directory)
        .expect("directory")
        .map(|entry| { return entry.expect("entry").file_name().to_string_lossy().into_owned(); })
        .collect();
    names.sort();
    return names;
}

/// The name of a build is its bytes: equal length and equal modification time with different bytes are two builds,
/// and equal bytes with different times are one.
#[test]
fn the_name_of_a_build_is_its_content() {
    let created: Fixture = fixture("store-name");
    let (first, second, third): (PathBuf, PathBuf, PathBuf) =
        (created.root.join("first"), created.root.join("second"), created.root.join("third"));
    std::fs::write(&first, b"build one").expect("first");
    std::fs::write(&second, b"build two").expect("second");
    std::fs::write(&third, b"build one").expect("third");
    let stamp: std::time::SystemTime = std::time::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
    for (path, modified) in [(&first, stamp), (&second, stamp), (&third, stamp + Duration::from_secs(5))] {
        std::fs::File::open(path).expect("open").set_modified(modified).expect("set the time");
    }
    assert_eq!(std::fs::metadata(&first).expect("first").len(), std::fs::metadata(&second).expect("second").len());
    let (name_first, name_second, name_third) =
        (content_name(&first).expect("first"), content_name(&second).expect("second"), content_name(&third).expect("third"));
    assert_ne!(name_first, name_second, "equal length and time, different bytes");
    assert_eq!(name_first, name_third, "equal bytes, different time");
}

/// How many callers ask for a build's name at the same moment.
const NAME_CALLERS: usize = 4;

/// How long the first read waits for the other callers to have asked; the wait ends sooner, as soon as they have, and
/// only a caller that never arrives makes it run out.
const ARRIVAL_DEADLINE: Duration = Duration::from_secs(2);

/// However many callers ask for the name of a build at once, the build is read once: the first read is held open
/// until every caller has asked, so a caller that would start a read of its own has started it.
#[test]
fn callers_that_ask_together_for_the_name_of_a_build_read_it_once() {
    let cell: OnceLock<BuildName> = OnceLock::new();
    let (reads, arrived): (AtomicUsize, AtomicUsize) = (AtomicUsize::new(0), AtomicUsize::new(0));
    let start: Barrier = Barrier::new(NAME_CALLERS);
    let names: Vec<String> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..NAME_CALLERS)
            .map(|_| {
                return scope.spawn(|| {
                    start.wait();
                    arrived.fetch_add(1, Ordering::SeqCst);
                    return name_once(&cell, || {
                        reads.fetch_add(1, Ordering::SeqCst);
                        let deadline: std::time::Instant = std::time::Instant::now() + ARRIVAL_DEADLINE;
                        while arrived.load(Ordering::SeqCst) < NAME_CALLERS && std::time::Instant::now() < deadline {
                            std::thread::yield_now();
                        }
                        return Ok(String::from("the name"));
                    })
                    .expect("the name");
                });
            })
            .collect();
        return handles.into_iter().map(|handle| { return handle.join().expect("caller finished"); }).collect();
    });
    assert_eq!(reads.load(Ordering::SeqCst), 1, "callers that asked together each read the build");
    assert!(names.iter().all(|name| { return name == "the name"; }), "{names:?}");
}

/// A read that failed is the outcome for every later caller, with the same kind of error, and no later caller reads
/// again.
#[test]
fn a_failed_read_of_the_name_is_the_outcome_for_later_callers() {
    let cell: OnceLock<BuildName> = OnceLock::new();
    let reads: AtomicUsize = AtomicUsize::new(0);
    let first = name_once(&cell, || {
        reads.fetch_add(1, Ordering::SeqCst);
        return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
    });
    let later = name_once(&cell, || {
        reads.fetch_add(1, Ordering::SeqCst);
        return Ok(String::from("a read that should not happen"));
    });
    assert_eq!(first.expect_err("the first read failed").kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(later.expect_err("the failure stays").kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(reads.load(Ordering::SeqCst), 1, "a later caller read again");
}

/// How many callers publish against each other in one control.
const PUBLISHING_CALLERS: usize = 4;

/// How many bytes the stand-in build has. Each caller writes one partial copy of it, so the control writes
/// `PUBLISHING_CALLERS` times this, whatever the host's core count, and not as many copies of the real 22 MB executable
/// as the host has cores.
const STAND_IN_BYTES: usize = 64 * 1024;

/// Count one more caller at the gate and wait, up to `ARRIVAL_DEADLINE`, until `callers` have; return whether they had
/// when this caller left. A caller that died before the gate makes the others leave late and say so, not hang.
fn meet_at_gate(arrived: &AtomicUsize, callers: usize) -> bool {
    arrived.fetch_add(1, Ordering::SeqCst);
    let deadline: std::time::Instant = std::time::Instant::now() + ARRIVAL_DEADLINE;
    while arrived.load(Ordering::SeqCst) < callers && std::time::Instant::now() < deadline {
        std::thread::yield_now();
    }
    return arrived.load(Ordering::SeqCst) >= callers;
}

/// However many callers ask at once, every one ends with a link to the same final file holding the build's bytes, a
/// recent copy of another build stays while they link, and nothing else stays: no partial file, no second copy.
/// Every caller holds its finished partial copy at the gate before any of them publishes, so all of them publish
/// against each other whatever the schedule; the build is a small stand-in so that no more is written than
/// `PUBLISHING_CALLERS` copies of `STAND_IN_BYTES`.
/// The identity of the final file is taken before a later ask, so a later ask that replaced it is seen.
#[test]
fn concurrent_callers_end_with_links_to_one_final_file_of_the_build() {
    let created: Fixture = fixture("store-once");
    let built: PathBuf = created.root.join("stand-in-build");
    std::fs::write(&built, vec![0xA5_u8; STAND_IN_BYTES]).expect("stand-in build");
    let name: String = content_name(&built).expect("the stand-in's name");
    let store: PathBuf = created.root.join(STORE_DIRECTORY);
    std::fs::create_dir_all(&store).expect("store");
    let other_build: String = String::from("0000000000000000-1-monochromatic-git-policy-cli");
    std::fs::write(store.join(&other_build), b"another build").expect("another build's copy");
    let (arrived, together): (AtomicUsize, AtomicUsize) = (AtomicUsize::new(0), AtomicUsize::new(0));
    let links: Vec<PathBuf> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..PUBLISHING_CALLERS)
            .map(|caller| {
                let link: PathBuf = created.root.join(format!("link-{caller}"));
                let (root, build, stored_name, gate, counted): (&Path, &Path, &str, &AtomicUsize, &AtomicUsize) =
                    (created.root.as_path(), built.as_path(), name.as_str(), &arrived, &together);
                return scope.spawn(move || {
                    let stored: PathBuf = shared_copy_named(build, stored_name, root, &|| {
                        if meet_at_gate(gate, PUBLISHING_CALLERS) {
                            counted.fetch_add(1, Ordering::SeqCst);
                        }
                    })
                    .expect("stored copy");
                    std::fs::hard_link(&stored, &link).expect("a fixture links the stored copy");
                    return link;
                });
            })
            .collect();
        return handles.into_iter().map(|handle| { return handle.join().expect("caller finished"); }).collect();
    });
    assert_eq!(together.load(Ordering::SeqCst), PUBLISHING_CALLERS, "not every caller published against all the others");
    let first_identity: (u64, u64) = identity(&links[0]);
    assert!(links.iter().all(|link| { return identity(link) == first_identity; }), "the links are not one file");
    let mut expected: Vec<String> = vec![name.clone(), other_build];
    expected.sort();
    assert_eq!(names_in(&store), expected, "only the final file and the recent copy of the other build");
    assert_eq!(std::fs::read(&links[0]).expect("stored bytes"), std::fs::read(&built).expect("built bytes"));
    let later: PathBuf = shared_copy_named(&built, name.as_str(), created.root.as_path(), &|| {}).expect("later ask");
    assert_eq!(identity(&later), first_identity, "a later ask replaced the final file");
}


/// A caller that publishes after another has won leaves the winner's file as the stored copy, removes its own partial
/// file, and succeeds; a caller that publishes first becomes the stored copy.
#[test]
fn a_late_publisher_leaves_the_winners_file_in_place() {
    let created: Fixture = fixture("store-publish");
    let stored: PathBuf = created.root.join("stored");
    let (winner, loser): (PathBuf, PathBuf) = (created.root.join("winner"), created.root.join("loser"));
    std::fs::write(&winner, b"winner bytes").expect("winner");
    std::fs::write(&loser, b"loser bytes").expect("loser");
    publish(winner.as_path(), stored.as_path()).expect("the first publisher");
    let winner_inode: u64 = identity(&stored).1;
    assert!(!winner.exists(), "the first publisher removes its partial file");
    publish(loser.as_path(), stored.as_path()).expect("a late publisher succeeds");
    assert!(!loser.exists(), "the late publisher removes its partial file");
    assert_eq!(std::fs::read(&stored).expect("stored bytes"), b"winner bytes");
    assert_eq!(identity(&stored).1, winner_inode, "the stored file was replaced");
}

/// What a caller that died after linking its partial file to a published file leaves: the published file under the
/// partial names this process identifier and the counter values below `count` would produce.
fn plant_aliasing_leftovers(store: &Path, published: &Path, count: usize) -> Vec<PathBuf> {
    std::fs::create_dir_all(store).expect("store");
    std::fs::write(published, PUBLISHED_BYTES).expect("published file");
    return (0..count)
        .map(|index| {
            let leftover: PathBuf = store.join(format!("{PARTIAL_PREFIX}{}-{index}", std::process::id()));
            std::fs::hard_link(published, &leftover).expect("a leftover partial file aliasing the published file");
            return leftover;
        })
        .collect();
}

/// The bytes of the file a leftover partial name aliases.
const PUBLISHED_BYTES: &[u8] = b"a published copy that must never be written through another name";

/// Whether the published file still holds its bytes; a plain answer, so a failure never prints a whole executable.
fn still_published(published: &Path) -> bool {
    return std::fs::read(published).is_ok_and(|bytes| { return bytes == PUBLISHED_BYTES; });
}

/// A partial name that exists is never reused, whatever process left it: the reservation moves on to a name nobody
/// has, the files under the taken names keep their bytes and their one inode, and a store in which every name tried
/// is taken is an error and not an overwrite.
#[test]
fn a_partial_name_that_exists_is_never_reused() {
    let created: Fixture = fixture("store-reserve");
    let store: PathBuf = created.root.join(STORE_DIRECTORY);
    let published: PathBuf = store.join("published");
    let leftovers: Vec<PathBuf> = plant_aliasing_leftovers(store.as_path(), published.as_path(), RESERVE_ATTEMPTS - 1);
    let counter: AtomicUsize = AtomicUsize::new(0);
    let reserved: PathBuf = reserve_partial(store.as_path(), &counter).expect("a free name remains");
    assert!(!leftovers.contains(&reserved), "a taken name was handed out again: {}", reserved.display());
    assert_eq!(std::fs::metadata(&reserved).expect("reserved").len(), 0, "the reserved file is new and empty");
    assert!(still_published(&published), "a leftover name was written through");
    assert_eq!(
        std::fs::metadata(&published).expect("published").nlink(),
        u64::try_from(leftovers.len() + 1).expect("link count"),
        "a leftover was replaced or removed"
    );
    let exhausted: PathBuf = created.root.join("exhausted-store");
    let exhausted_published: PathBuf = exhausted.join("published");
    plant_aliasing_leftovers(exhausted.as_path(), exhausted_published.as_path(), RESERVE_ATTEMPTS);
    assert!(
        reserve_partial(exhausted.as_path(), &AtomicUsize::new(0)).is_err(),
        "every name tried was taken, yet one was handed out"
    );
    assert!(still_published(&exhausted_published), "a name was written through");
}

/// A caller of the stored copy that finds the partial names of its process identifier taken by leftovers aliasing a
/// published file copies into a name of its own and leaves the published file's bytes alone.
#[test]
fn a_copy_never_writes_through_a_leftover_partial_name() {
    let created: Fixture = fixture("store-leftover");
    let store: PathBuf = created.root.join(STORE_DIRECTORY);
    let published: PathBuf = store.join("another-build");
    let leftovers: Vec<PathBuf> = plant_aliasing_leftovers(store.as_path(), published.as_path(), RESERVE_ATTEMPTS - 1);
    let stored: PathBuf = shared_copy(Path::new(WRAPPER), created.root.as_path()).expect("the copy is made");
    assert!(
        std::fs::read(&stored).expect("stored bytes") == std::fs::read(WRAPPER).expect("built bytes"),
        "the stored copy is not the build"
    );
    assert!(still_published(&published), "a leftover name was written through");
    let mut expected: Vec<String> = leftovers
        .iter()
        .map(|leftover| { return leftover.file_name().expect("name").to_string_lossy().into_owned(); })
        .collect();
    expected.push(String::from("another-build"));
    expected.push(content_name(Path::new(WRAPPER)).expect("the build's name"));
    expected.sort();
    assert_eq!(names_in(&store), expected, "the leftovers stay and the copy has no partial file of its own");
}

/// A copy of another build that no fixture has linked for the idle age is removed, a younger one is not, and neither is
/// the current build's copy; a partial file goes only when it is that old too, so a live caller's never does.
#[test]
fn only_idle_copies_of_other_builds_leave_the_store() {
    let created: Fixture = fixture("store-idle");
    let store: PathBuf = created.root.join(STORE_DIRECTORY);
    std::fs::create_dir_all(&store).expect("store");
    let (keep, other, partial): (PathBuf, PathBuf, PathBuf) = (
        store.join("keep"),
        store.join("other"),
        store.join(format!("{PARTIAL_PREFIX}1-0")),
    );
    for path in [&keep, &other, &partial] {
        std::fs::write(path, b"bytes").expect("file");
    }
    remove_idle_builds(store.as_path(), keep.as_path(), IDLE_BUILD_AGE);
    assert_eq!(names_in(&store).len(), 3, "nothing is idle yet: {:?}", names_in(&store));
    remove_idle_builds(store.as_path(), keep.as_path(), Duration::ZERO);
    assert_eq!(names_in(&store), vec![String::from("keep")], "everything but the current build's copy was idle");
}

/// The status-change time of a file, in seconds, which no program can set: only a link, an unlink, a write or an
/// attribute change moves it.
fn changed_at(path: &Path) -> i64 {
    return std::fs::metadata(path).expect("metadata").ctime();
}

/// A store holding the current build's copy and one other copy, both with one name, and a partial file.
fn store_with_an_idle_candidate(created: &Fixture) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let store: PathBuf = created.root.join(STORE_DIRECTORY);
    std::fs::create_dir_all(&store).expect("store");
    let (keep, other, partial): (PathBuf, PathBuf, PathBuf) =
        (store.join("keep"), store.join("other"), store.join(format!("{PARTIAL_PREFIX}1-0")));
    for path in [&keep, &other, &partial] {
        std::fs::write(path, b"bytes").expect("file");
    }
    return (store, keep, other, partial);
}

/// A copy and a partial file are removed when `now` is the idle age after their status-change time and not one second
/// before, and the current build's copy is never removed: the age is judged by the clock handed in, not by sleeping.
#[test]
fn a_copy_leaves_the_store_exactly_when_it_has_been_idle_for_the_idle_age() {
    let created: Fixture = fixture("store-aging");
    let (store, keep, other, partial) = store_with_an_idle_candidate(&created);
    let idle_seconds: i64 = i64::try_from(IDLE_BUILD_AGE.as_secs()).expect("seconds");
    let (first_changed, last_changed): (i64, i64) =
        (changed_at(&other).min(changed_at(&partial)), changed_at(&other).max(changed_at(&partial)));
    remove_idle_builds_at(store.as_path(), keep.as_path(), IDLE_BUILD_AGE, first_changed + idle_seconds - 1, &mut |_| {});
    assert_eq!(names_in(&store).len(), 3, "a second short of the idle age removed one: {:?}", names_in(&store));
    remove_idle_builds_at(store.as_path(), keep.as_path(), IDLE_BUILD_AGE, last_changed + idle_seconds, &mut |_| {});
    assert_eq!(names_in(&store), vec![String::from("keep")], "at the idle age everything but the current build went");
}

/// A copy a fixture linked just now stays however old its contents are: the age is the status-change time, which a
/// link moves, and not the modification time, which a link leaves alone and which any copy of an old build carries.
#[test]
fn a_copy_with_an_old_modification_time_and_a_recent_link_stays() {
    let created: Fixture = fixture("store-recent-link");
    let (store, keep, other, partial) = store_with_an_idle_candidate(&created);
    for path in [&other, &partial] {
        std::fs::File::open(path).expect("open").set_modified(std::time::UNIX_EPOCH).expect("an ancient modification time");
    }
    remove_idle_builds(store.as_path(), keep.as_path(), IDLE_BUILD_AGE);
    assert_eq!(names_in(&store).len(), 3, "a recently linked copy went: {:?}", names_in(&store));
}

/// A copy that a fixture links after the decision to remove it was made, and before the removal, stays, and the
/// fixture's link is that very copy; a copy that a fixture already holds a link to is never a candidate.
#[test]
fn a_link_made_during_the_removal_keeps_the_copy() {
    let created: Fixture = fixture("store-link-during-removal");
    let (store, keep, other, _partial) = store_with_an_idle_candidate(&created);
    let held: PathBuf = store.join("held");
    std::fs::write(&held, b"held").expect("file");
    std::fs::hard_link(&held, created.root.join("held-link")).expect("a fixture holds the copy");
    let link: PathBuf = created.root.join("fixture-link");
    let mut decided: Vec<PathBuf> = Vec::new();
    remove_idle_builds_at(store.as_path(), keep.as_path(), Duration::ZERO, i64::MAX, &mut |candidate| {
        decided.push(candidate.to_path_buf());
        if candidate == other.as_path() {
            std::fs::hard_link(candidate, &link).expect("a fixture links the copy at the moment of the decision");
        }
    });
    assert_eq!(decided, vec![other.clone()], "only a copy nobody holds is a candidate");
    assert!(held.exists(), "a copy with a fixture's link was removed");
    assert!(other.exists(), "the copy a fixture linked during the removal was removed");
    assert_eq!(identity(&other), identity(&link), "the name is another file than the fixture's link");
    assert_eq!(std::fs::metadata(&other).expect("other").nlink(), 2);
    let leftovers: Vec<String> = names_in(&store).into_iter().filter(|name| { return name.starts_with(PARTIAL_PREFIX); }).collect();
    assert!(leftovers.is_empty(), "the set-aside name stayed: {leftovers:?}");
}

/// The copy of another build is removed after a caller got its path and before it links it: the caller gets the copy
/// again, and its link is the stored file itself, not a copy of its own.
#[test]
fn a_stored_copy_that_vanishes_before_the_link_is_made_again() {
    let created: Fixture = fixture("store-vanish");
    let built: &Path = Path::new(WRAPPER);
    let to: PathBuf = created.root.join("linked");
    let mut removals: usize = 0;
    link_stored(built, created.root.as_path(), to.as_path(), &mut |stored| {
        if removals == 0 {
            std::fs::remove_file(stored).expect("the prune removes the copy");
        }
        removals += 1;
    })
    .expect("the link after the copy was made again");
    assert!(removals >= 2, "the stored copy was asked for again: {removals}");
    let stored: PathBuf = created.root.join(STORE_DIRECTORY).join(content_name(built).expect("name"));
    assert_eq!(identity(&to), identity(&stored), "the link is not the stored file");
    assert_eq!(std::fs::metadata(&to).expect("linked").nlink(), 2, "the link is a copy of its own");
    assert_eq!(std::fs::read(&to).expect("linked bytes"), std::fs::read(built).expect("built bytes"));
}

/// A copy whose bytes do not have the name read before it was made is not published. The name of the build is read
/// once per process, so asking for another file under it makes exactly that mismatch.
#[test]
fn a_copy_that_does_not_have_the_builds_name_is_not_published() {
    let created: Fixture = fixture("store-changed");
    shared_copy(Path::new(WRAPPER), created.root.as_path()).expect("the build's name is read and its copy made");
    let changed: PathBuf = created.root.join("changed-build");
    std::fs::write(&changed, b"bytes the name was not read from").expect("changed build");
    // A root with no stored copy yet, so the ask has to make one.
    let empty_root: PathBuf = created.root.join("empty-root");
    let refused = shared_copy(changed.as_path(), empty_root.as_path());
    assert!(refused.is_err(), "a copy that is not the named build was published");
    assert_eq!(names_in(&empty_root.join(STORE_DIRECTORY)), Vec::<String>::new(), "a partial file or a copy stayed");
}

/// How a fixture root gets the built executable, as the filesystem under the root allows.
#[derive(Debug, Eq, PartialEq)]
enum Linking {
    /// A hard link of the built executable works.
    Built,
    /// The link fails across filesystems (`EXDEV`): fixtures link the one stored copy.
    Stored,
    /// The link fails for another reason (a protected-links rule, a filesystem without links): fixtures copy.
    Private(std::io::ErrorKind),
}

/// Ask `directory`'s filesystem the question the fixtures' factory asks, with a link that is removed again.
fn linking_in(directory: &Path) -> Linking {
    let probe: PathBuf = directory.join(format!("link-probe-{}", std::process::id()));
    let linked: std::io::Result<()> = std::fs::hard_link(WRAPPER, &probe);
    if linked.is_ok() {
        std::fs::remove_file(&probe).expect("remove the probe link");
    }
    return match linked {
        Ok(()) => Linking::Built,
        Err(error) if error.raw_os_error() == Some(libc::EXDEV) => Linking::Stored,
        Err(error) => Linking::Private(error.kind()),
    };
}

/// What two fixtures in `directory` must be, given how `directory`'s filesystem links:
/// the built executable itself where it can be linked, one stored copy of its bytes where only a copy on this
/// filesystem can, and private copies with its bytes where no link works. Returns which of the three applied.
fn assert_fixtures_follow_the_links_the_filesystem_allows(directory: &Path, first: &Fixture, second: &Fixture) -> Linking {
    let built_bytes: Vec<u8> = std::fs::read(WRAPPER).expect("built bytes");
    let linking: Linking = linking_in(directory);
    match &linking {
        Linking::Built => {
            for fixture in [first, second] {
                assert_eq!(identity(&fixture.wrapper), identity(Path::new(WRAPPER)), "not the built executable itself");
            }
        }
        Linking::Stored => {
            let stored: PathBuf = directory.join(STORE_DIRECTORY).join(content_name(Path::new(WRAPPER)).expect("name"));
            assert!(
                std::fs::read(&stored).is_ok_and(|bytes| { return bytes == built_bytes; }),
                "the stored copy does not hold the built bytes: {}",
                stored.display()
            );
            for fixture in [first, second] {
                assert_eq!(identity(&fixture.wrapper), identity(&stored), "not the one stored copy");
            }
            assert_ne!(identity(&stored), identity(Path::new(WRAPPER)), "the stored copy cannot be the build");
        }
        Linking::Private(kind) => {
            eprintln!("{} cannot link the build ({kind}); fixtures there copy it, so sharing is not asserted", directory.display());
            for fixture in [first, second] {
                assert!(
                    std::fs::read(&fixture.wrapper).is_ok_and(|bytes| { return bytes == built_bytes; }),
                    "a private copy does not hold the built bytes: {}",
                    fixture.wrapper.display()
                );
            }
        }
    }
    return linking;
}

/// Two fixtures in the default root are the built executable itself where the root's filesystem links it, its one
/// stored copy where it does not (`EXDEV`), and copies of its bytes where no link works.
#[test]
fn fixtures_are_the_built_executable_or_its_one_stored_copy() {
    let (first, second): (Fixture, Fixture) = (fixture("share-first"), fixture("share-second"));
    let root: PathBuf = first.root.parent().expect("the fixture root").to_path_buf();
    assert_fixtures_follow_the_links_the_filesystem_allows(root.as_path(), &first, &second);
}

/// Directories that are usually another filesystem than the build: shared memory and the user's runtime directory.
fn other_filesystem_candidates() -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = vec![PathBuf::from("/dev/shm")];
    candidates.extend(std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from));
    return candidates;
}

/// How many copies of the executable the stored-copy path holds at once on the other filesystem: the stored copy and
/// the partial copy of a caller that is about to publish.
const COPIES_ON_THE_OTHER_FILESYSTEM: u64 = 2;

/// A directory made for one control and removed when the control ends, however it ends: a panic unwinds through
/// this too, so a failed control leaves no executable behind.
struct Scratch {
    /// The directory, which holds the roots of the control's fixtures and their store.
    directory: PathBuf,
}

impl Scratch {
    /// Make the directory, and the guard that removes it.
    fn create(directory: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&directory)?;
        return Ok(Self { directory });
    }
}

/// Removes the directory and everything under it; a directory already gone is not an error.
impl Drop for Scratch {
    fn drop(&mut self) {
        match std::fs::remove_dir_all(&self.directory) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => eprintln!("could not remove {} ({error})", self.directory.display()),
        }
    }
}

/// The bytes that can be written to the filesystem under `path` without root, for an ordinary user.
fn available_bytes(path: &Path) -> std::io::Result<u64> {
    use std::os::unix::ffi::OsStrExt;
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(std::io::Error::other)?;
    let mut status: std::mem::MaybeUninit<libc::statvfs> = std::mem::MaybeUninit::uninit();
    // SAFETY: `name` is a NUL-terminated path that outlives the call, and `status` is a writable `statvfs` that the
    // call fills when it returns zero, which is the only case in which it is read.
    let answered: i32 = unsafe { libc::statvfs(name.as_ptr(), status.as_mut_ptr()) };
    if answered != 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: the call returned zero, so it filled `status`.
    let filled: libc::statvfs = unsafe { status.assume_init() };
    return Ok(filled.f_bavail.saturating_mul(filled.f_frsize));
}

/// What:
///  A scratch directory in the first of `candidates` that is a writable directory on another filesystem than `built`
///  (the probe link of the build fails with `EXDEV`) with room for the copies the stored-copy path holds there; the
///  reasons every candidate was refused when none is.
/// Why:
///  Not every host has shared memory or a runtime directory, and `EXDEV` says nothing about capacity.
fn scratch_on_another_filesystem(candidates: &[PathBuf], built: &Path) -> Result<Scratch, String> {
    let needed: u64 = std::fs::metadata(built).map_err(|error| { return format!("{} is unreadable ({error})", built.display()); })?.len()
        * COPIES_ON_THE_OTHER_FILESYSTEM;
    let mut reasons: Vec<String> = Vec::new();
    for candidate in candidates {
        if !candidate.is_dir() {
            reasons.push(format!("{} is not a directory", candidate.display()));
            continue;
        }
        let scratch: Scratch = match Scratch::create(candidate.join(format!("native-binary-root-{}", std::process::id()))) {
            Ok(created) => created,
            Err(error) => {
                reasons.push(format!("{} is not writable ({error})", candidate.display()));
                continue;
            }
        };
        match std::fs::hard_link(built, scratch.directory.join("probe")) {
            Ok(()) => reasons.push(format!("{} is on the build's filesystem", candidate.display())),
            Err(error) if error.raw_os_error() == Some(libc::EXDEV) => match available_bytes(&scratch.directory) {
                Ok(free) if free >= needed => return Ok(scratch),
                Ok(free) => reasons.push(format!("{} has {free} bytes free and the control needs {needed}", candidate.display())),
                Err(measure_error) => reasons.push(format!("{} could not be measured ({measure_error})", candidate.display())),
            },
            Err(error) => reasons.push(format!("{} refused a probe link of the build ({error})", candidate.display())),
        }
    }
    return Err(reasons.join("; "));
}

/// Two fixtures in a root on another filesystem than the build, where the link of the build itself fails (`EXDEV`),
/// are one stored copy of the build's bytes. Where no such filesystem is available the control says why and ends.
#[test]
fn fixtures_on_another_filesystem_share_the_stored_copy_of_the_build() {
    let scratch: Scratch = match scratch_on_another_filesystem(other_filesystem_candidates().as_slice(), Path::new(WRAPPER)) {
        Ok(found) => found,
        Err(reasons) => {
            eprintln!("SKIPPED, no other filesystem to relocate the fixture root to: {reasons}");
            return;
        }
    };
    let (first, second): (Fixture, Fixture) =
        (fixture_in(scratch.directory.as_path(), "share-first"), fixture_in(scratch.directory.as_path(), "share-second"));
    let linking: Linking = assert_fixtures_follow_the_links_the_filesystem_allows(scratch.directory.as_path(), &first, &second);
    assert_eq!(linking, Linking::Stored, "the scratch directory was chosen because it could not link the build");
}

/// A control that panics leaves no scratch directory behind, and a host with no other filesystem gets every reason it
/// was refused, not a panic.
#[test]
fn a_scratch_directory_goes_with_its_control_and_a_missing_filesystem_is_a_reason() {
    let created: Fixture = fixture("store-scratch");
    let directory: PathBuf = created.root.join("scratch-guard");
    let panicked = std::panic::catch_unwind(|| {
        let scratch: Scratch = Scratch::create(directory.clone()).expect("scratch");
        std::fs::write(scratch.directory.join("executable-sized"), b"bytes").expect("file");
        panic!("the control failed after it wrote");
    });
    assert!(panicked.is_err(), "the control did not panic");
    assert!(!directory.exists(), "a panicking control left its scratch directory");
    let built_directory: PathBuf = Path::new(WRAPPER).parent().expect("the build's directory").to_path_buf();
    let missing: PathBuf = created.root.join("no-such-directory");
    let candidates: Vec<PathBuf> = vec![missing.clone(), built_directory.clone()];
    let refused: Result<Scratch, String> = scratch_on_another_filesystem(candidates.as_slice(), Path::new(WRAPPER));
    let reasons: String = refused.err().expect("neither candidate is another filesystem");
    assert!(!missing.exists(), "the probe made the directory that did not exist");
    assert!(reasons.contains("is not a directory"), "the missing directory is not given as the reason: {reasons}");
    for candidate in [&missing, &built_directory] {
        assert!(reasons.contains(candidate.to_string_lossy().as_ref()), "{} is not named in: {reasons}", candidate.display());
    }
}

/// The time the waiting controls start from, a fixed instant so no control reads the real clock: the clock of a wait
/// moves only when the wait pauses.
fn fake_start() -> std::time::SystemTime {
    return std::time::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
}

/// The progress window, the poll and the cap the waiting controls use, in the fake clock's time.
const WAIT_LIMITS: [Duration; 3] = [Duration::from_secs(10), Duration::from_secs(1), Duration::from_secs(100)];

/// How many pauses a waiting control allows before it fails, so a wait that never ends fails the control instead of
/// holding it: far more than the cap needs.
const MOST_PAUSES: u32 = 1000;

/// A clock that moves only when the wait pauses, and runs `on_pause` with the new time and the number of pauses so far.
struct FakeClock<'a> {
    /// How far the clock has moved from `fake_start`.
    elapsed: std::cell::Cell<Duration>,
    /// How many times the wait paused.
    pauses: std::cell::Cell<u32>,
    /// What happens while the wait sleeps: the other caller's writes and publish.
    on_pause: std::cell::RefCell<Box<dyn FnMut(std::time::SystemTime, u32) + 'a>>,
}

impl<'a> FakeClock<'a> {
    /// A clock at `fake_start` whose pauses run `on_pause`.
    fn new(on_pause: impl FnMut(std::time::SystemTime, u32) + 'a) -> Self {
        return FakeClock { elapsed: std::cell::Cell::new(Duration::ZERO), pauses: std::cell::Cell::new(0), on_pause: std::cell::RefCell::new(Box::new(on_pause)) };
    }

    /// A clock whose pauses do nothing.
    fn idle() -> Self {
        return FakeClock::new(|_, _| {});
    }

    /// How long the wait slept in all, as the clock saw it.
    fn slept(&self) -> Duration {
        return self.elapsed.get();
    }
}

impl WaitClock for FakeClock<'_> {
    /// The start plus the time slept.
    fn now(&self) -> std::time::SystemTime {
        return fake_start() + self.elapsed.get();
    }

    /// Move the clock by the poll and let the other caller act.
    fn pause(&self, poll: Duration) {
        self.pauses.set(self.pauses.get() + 1);
        assert!(self.pauses.get() <= MOST_PAUSES, "the wait never ended");
        self.elapsed.set(self.elapsed.get() + poll);
        (self.on_pause.borrow_mut())(fake_start() + self.elapsed.get(), self.pauses.get());
    }
}

/// Set the modification time of a path.
fn set_written(path: &Path, when: std::time::SystemTime) {
    let file = std::fs::OpenOptions::new().write(true).open(path).expect("open to set the time");
    file.set_modified(when).expect("set the time");
}

/// A store with the partial copy of another caller, last written at `written`.
fn store_with_partial(created: &Fixture, written: std::time::SystemTime) -> (PathBuf, PathBuf, PathBuf) {
    let store: PathBuf = created.root.join(STORE_DIRECTORY);
    std::fs::create_dir_all(&store).expect("store");
    let partial: PathBuf = store.join(format!("{PARTIAL_PREFIX}1-0"));
    std::fs::write(&partial, b"a copy being written").expect("a partial copy begins");
    set_written(partial.as_path(), written);
    return (store.clone(), partial, store.join("the-build"));
}

/// A caller waits while the other caller keeps writing, and ends the wait at once when the copy is published: nothing but
/// the stored name and the recent writes decide it.
#[test]
fn a_caller_waits_while_the_copy_is_written_and_stops_when_it_is_published() {
    let created: Fixture = fixture("store-wait-live");
    let (store, partial, stored) = store_with_partial(&created, fake_start());
    // The writer writes on every pause, for longer than the progress window, and publishes on its last.
    let writes: u32 = u32::try_from(WAIT_LIMITS[0].as_secs() / WAIT_LIMITS[1].as_secs()).expect("fits") * 3;
    let clock = FakeClock::new(|now, pause| {
        if pause < writes {
            set_written(partial.as_path(), now);
        } else {
            publish(partial.as_path(), stored.as_path()).expect("publish");
        }
    });
    assert_eq!(wait_for_publisher_on(store.as_path(), stored.as_path(), WAIT_LIMITS, &clock), WaitEnd::Published);
    assert_eq!(clock.pauses.get(), writes, "the wait ended on the pause that published");
    assert!(clock.slept() > WAIT_LIMITS[0], "the wait outlasted the window because the copy kept being written");
    assert!(stored.exists() && !partial.exists());
}

/// Nobody waits for a partial copy nobody is writing, and the wait does not pause even once: an old one (a killed run's),
/// a leftover that aliases a published file, no partial copy at all, and a stored copy that is there already.
#[test]
fn a_caller_does_not_wait_for_a_partial_copy_nobody_is_writing() {
    let created: Fixture = fixture("store-wait-stale");
    let (store, old, stored) = store_with_partial(&created, fake_start() - WAIT_LIMITS[0] * 10);
    let clock = FakeClock::idle();
    assert_eq!(wait_for_publisher_on(store.as_path(), stored.as_path(), WAIT_LIMITS, &clock), WaitEnd::NobodyCopying, "an old partial");
    assert_eq!(clock.pauses.get(), 0);
    std::fs::remove_file(&old).expect("remove");
    let published: PathBuf = store.join("another-build");
    std::fs::write(&published, b"published bytes").expect("published");
    let alias: PathBuf = store.join(format!("{PARTIAL_PREFIX}1-1"));
    std::fs::hard_link(&published, &alias).expect("a leftover that aliases a published file");
    set_written(alias.as_path(), fake_start());
    assert_eq!(wait_for_publisher_on(store.as_path(), stored.as_path(), WAIT_LIMITS, &clock), WaitEnd::NobodyCopying, "an aliasing leftover, written just now");
    std::fs::remove_file(&alias).expect("remove");
    assert_eq!(wait_for_publisher_on(store.as_path(), stored.as_path(), WAIT_LIMITS, &clock), WaitEnd::NobodyCopying, "no partial copy");
    assert_eq!(clock.pauses.get(), 0, "none of the three paused");
    std::fs::write(&stored, b"the build").expect("a stored copy");
    assert_eq!(wait_for_publisher_on(store.as_path(), stored.as_path(), WAIT_LIMITS, &clock), WaitEnd::Published);
}

/// A copy that stops growing is waited for exactly until the window has passed without a write, and no longer.
#[test]
fn a_copy_that_stops_growing_ends_the_wait_when_the_window_has_passed() {
    let created: Fixture = fixture("store-wait-stopped");
    let (store, _partial, stored) = store_with_partial(&created, fake_start());
    let clock = FakeClock::idle();
    assert_eq!(wait_for_publisher_on(store.as_path(), stored.as_path(), WAIT_LIMITS, &clock), WaitEnd::NobodyCopying);
    let [window, poll, _cap] = WAIT_LIMITS;
    assert!(clock.slept() >= window, "the wait ended before the window had passed: {:?}", clock.slept());
    assert!(clock.slept() < window + poll, "the wait went on past the first look after the window: {:?}", clock.slept());
}

/// A copy written to without end ends the wait at the cap, on the first look after it, whatever the window says.
#[test]
fn a_copy_that_never_finishes_ends_the_wait_at_the_cap() {
    let created: Fixture = fixture("store-wait-cap");
    let (store, partial, stored) = store_with_partial(&created, fake_start());
    let clock = FakeClock::new(|now, _| { set_written(partial.as_path(), now); });
    assert_eq!(wait_for_publisher_on(store.as_path(), stored.as_path(), WAIT_LIMITS, &clock), WaitEnd::CapReached);
    let [_window, poll, cap] = WAIT_LIMITS;
    assert!(clock.slept() >= cap && clock.slept() < cap + poll, "{:?}", clock.slept());
}

/// A partial copy dated in the future (a clock that moved back) counts as being written, and the wait still ends at the cap.
#[test]
fn a_copy_dated_in_the_future_is_waited_for_to_the_cap_only() {
    let created: Fixture = fixture("store-wait-future");
    let (store, _partial, stored) = store_with_partial(&created, fake_start() + Duration::from_secs(24 * 3600));
    let clock = FakeClock::idle();
    assert_eq!(wait_for_publisher_on(store.as_path(), stored.as_path(), WAIT_LIMITS, &clock), WaitEnd::CapReached);
}

/// The wait on the system's clock decides the cases that need no time: nothing in progress, and a stored copy.
#[test]
fn the_wait_on_the_system_clock_answers_without_waiting_when_nothing_is_in_progress() {
    let created: Fixture = fixture("store-wait-system");
    let store: PathBuf = created.root.join(STORE_DIRECTORY);
    std::fs::create_dir_all(&store).expect("store");
    let stored: PathBuf = store.join("the-build");
    assert_eq!(wait_for_publisher(store.as_path(), stored.as_path()), WaitEnd::NobodyCopying);
    std::fs::write(&stored, b"the build").expect("stored");
    assert_eq!(wait_for_publisher(store.as_path(), stored.as_path()), WaitEnd::Published);
}

/// A caller whose wait ends with the copy published writes no copy of its own and returns the published one (its
/// `before_publish` hook, which only a copying caller reaches, never runs); when the wait finds nobody copying it copies.
#[test]
fn shared_copy_waits_for_a_copy_in_progress_instead_of_writing_another() {
    let created: Fixture = fixture("store-wait-shared");
    let built: PathBuf = created.root.join("stand-in-build");
    std::fs::write(&built, vec![0x5A_u8; STAND_IN_BYTES]).expect("stand-in build");
    let name: String = content_name(&built).expect("name");
    let copied_itself = AtomicUsize::new(0);
    let hook = || { copied_itself.fetch_add(1, Ordering::SeqCst); };
    let asked = AtomicUsize::new(0);
    // The wait stands for the time in which the other caller finishes: it publishes a copy and says so.
    let other_caller_publishes = |store: &Path, stored: &Path| -> WaitEnd {
        asked.fetch_add(1, Ordering::SeqCst);
        let partial: PathBuf = store.join(format!("{PARTIAL_PREFIX}1-0"));
        std::fs::copy(&built, &partial).expect("the other caller's copy completes");
        publish(partial.as_path(), stored).expect("publish");
        return WaitEnd::Published;
    };
    let returned: PathBuf = shared_copy_waiting(&built, name.as_str(), created.root.as_path(), &hook, &other_caller_publishes).expect("a stored copy");
    assert_eq!(asked.load(Ordering::SeqCst), 1, "the caller waited once");
    assert_eq!(returned, created.root.join(STORE_DIRECTORY).join(&name));
    assert_eq!(copied_itself.load(Ordering::SeqCst), 0, "the waiting caller wrote a copy of its own");
    assert_eq!(names_in(&created.root.join(STORE_DIRECTORY)), vec![name.clone()]);
    // Nobody copying: the caller copies and publishes itself.
    let other_root: Fixture = fixture("store-wait-alone");
    let nobody = |_store: &Path, _stored: &Path| -> WaitEnd { return WaitEnd::NobodyCopying; };
    let alone = shared_copy_waiting(&built, name.as_str(), other_root.root.as_path(), &hook, &nobody).expect("a copy of its own");
    assert_eq!(copied_itself.load(Ordering::SeqCst), 1);
    assert_eq!(std::fs::read(alone).expect("copy"), std::fs::read(&built).expect("built"));
}
