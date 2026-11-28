//! What:
//!  Plan step 21's watch arm:
//!  a watch set on a slot of the checkout before the clone is mounted,
//!  read after the fill,
//!  and what the comparison in `worktree_clone_watch_rebinding/` says of it.
//! Why:
//!  The plan asks for the watch rebinding after a clone fill,
//!  and the comparison is reached by no command of this release
//!  (`SPEC.md` "The clone route's repair and its watches":
//!  "What plan step 21 builds is reached by no command"),
//!  so the arm calls the published comparison itself and reads its sentence.
//!  An inotify watch belongs to the directory object it was set on,
//!  so the arm proves the premise too:
//!  a watch set before the mount sees nothing written through the slot
//!  afterwards,
//!  while a watch set after the mount sees the same write.
//!  The comparison before the mount is the arm's own control:
//!  the same call on the same path answers that the watch still sees the tree
//!  and writes no sentence,
//!  so a sentence after the mount is the mount's and not the call's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const before = afterTheMount(slot, identityOf(slot)); const probe = Probe.on(slot); mountTheClone(); writeThrough(slot); assert(!probe.sees()); assert(freshProbe.sees()); assert(afterTheMount(slot, recorded).watches === 'blind');
//! ```

#![cfg(target_os = "linux")]

use super::zfs_step_21_guest_fixture::{
    AddRound,
    Fixture,
};
use git_policy_cli::subvolume::fs_beneath_identity::ObjectId;
use git_policy_cli::worktree_clone_watch_rebinding::{
    AfterTheMount,
    Watches,
    after_the_mount,
    the_object_the_path_shows,
};
use std::ffi::{
    CString,
    OsStr,
};
use std::os::unix::ffi::OsStrExt as _;
use std::path::{
    Path,
    PathBuf,
};
use std::time::{
    Duration,
    Instant,
};

/// The name the arm writes through the slot,
/// so an event for it is the event the arm looks for.
const WRITTEN: &str = "written-through-the-slot";

/// How long the arm waits for one event before it calls the wait a silence.
/// Why:
///  The write follows at once,
///  so the wait only has to outlast the kernel's own delivery.
const PATIENCE: Duration = Duration::from_millis(500);

/// What:
///  An inotify watch on one directory's new entries,
///  closed however the arm ends.
/// Why:
///  The release's own watch answers whether to look again and never whether an
///  event arrived (`landing/service_index_watch.rs`),
///  and this arm needs the second answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const probe = Probe.on(directory); // closed in Drop
/// ```
struct Probe {
    /// The inotify descriptor this probe owns.
    watch: std::ffi::c_int,
}

impl Probe {
    /// A watch for new entries in `directory`.
    fn on(directory: &Path) -> Probe {
        let named: CString =
            CString::new(directory.as_os_str().as_bytes()).expect("a path without NUL");
        // SAFETY: the descriptor is created here and closed by `Drop`, whatever the waits answered.
        let watch: std::ffi::c_int = unsafe { libc::inotify_init1(libc::IN_CLOEXEC) };
        assert!(
            watch >= 0,
            "inotify_init1 refused: {}",
            std::io::Error::last_os_error()
        );
        // SAFETY: `named` is a NUL-terminated name of a directory the fixture holds.
        let added: std::ffi::c_int =
            unsafe { libc::inotify_add_watch(watch, named.as_ptr(), libc::IN_CREATE) };
        let probe: Probe = Probe {
            watch,
        };
        assert!(
            added >= 0,
            "inotify_add_watch refused for {}: {}",
            directory.display(),
            std::io::Error::last_os_error()
        );
        return probe;
    }

    /// Whether an event naming the arm's file arrives inside `PATIENCE`.
    fn sees(&self) -> bool {
        let deadline: Instant = Instant::now() + PATIENCE;
        loop {
            let left: Duration = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return false;
            }
            let mut entry: libc::pollfd = libc::pollfd {
                fd: self.watch,
                events: libc::POLLIN,
                revents: 0,
            };
            let milliseconds: std::ffi::c_int = std::ffi::c_int::try_from(left.as_millis())
                .unwrap_or(std::ffi::c_int::MAX)
                .max(1);
            // SAFETY: `entry` names this probe's one descriptor and lives for the call.
            if unsafe { libc::poll(&raw mut entry, 1, milliseconds) } <= 0 {
                return false;
            }
            let mut buffer: [u8; 4096] = [0; 4096];
            // SAFETY: the buffer is valid for its length.
            let read: isize =
                unsafe { libc::read(self.watch, buffer.as_mut_ptr().cast(), buffer.len()) };
            let Ok(count) = usize::try_from(read) else {
                return false;
            };
            if count == 0 {
                return false;
            }
            if names_the_file(&buffer[..count], OsStr::new(WRITTEN).as_bytes()) {
                return true;
            }
        }
    }
}

impl Drop for Probe {
    fn drop(&mut self) {
        // SAFETY: the descriptor belongs to this probe and is closed once.
        let closed: std::ffi::c_int = unsafe { libc::close(self.watch) };
        assert_eq!(
            closed,
            0,
            "the watch closes: {}",
            std::io::Error::last_os_error()
        );
    }
}

/// Whether `buffer` holds an inotify event naming `wanted`.
fn names_the_file(
    buffer: &[u8],
    wanted: &[u8],
) -> bool {
    let mut rest: &[u8] = buffer;
    while rest.len() >= 16 {
        let length: usize =
            usize::try_from(u32::from_ne_bytes([rest[12], rest[13], rest[14], rest[15]]))
                .expect("a length");
        let payload: &[u8] = &rest[16..];
        if payload.len() < length {
            return false;
        }
        let name: &[u8] = payload[..length]
            .split(|byte| return *byte == 0)
            .next()
            .unwrap_or(&[]);
        if name == wanted {
            return true;
        }
        rest = &rest[16 + length..];
    }
    return false;
}

/// The write the arm makes through the slot's path.
fn write_through_the_slot(slot: &Path) {
    std::fs::write(slot.join(WRITTEN), b"a write in the tree the path names\n").expect("the write");
}

/// What:
///  Add a worktree,
///  setting one watch and taking one comparison just before the clone is
///  mounted,
///  then read what a watch from before the mount sees and what the comparison
///  says afterwards.
/// Why:
///  The article under test is the directory object the path names,
///  so the arm takes the mount's own moment instead of a later one,
///  and the watch it holds is the one an editor would have opened on the
///  registration Git made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watchAndCompareJustBeforeTheMount(); mount(); writeThrough(); assertWatchesAndSentence();
/// ```
pub(super) fn the_watch_arm(fixture: &Fixture) {
    let name: String = String::from("watch-0");
    let mut covered: Option<Probe> = None;
    let mut recorded: Option<ObjectId> = None;
    let mut before_the_mount: Option<AfterTheMount> = None;
    let mut hook = |slot: &Path| {
        covered = Some(Probe::on(slot));
        let identity: ObjectId =
            the_object_the_path_shows(slot).expect("the slot's own identity before the mount");
        before_the_mount = Some(after_the_mount(slot, identity).expect("the comparison before it"));
        recorded = Some(identity);
    };
    let added: AddRound = fixture.add(name.as_str(), Some(&mut hook));
    drop(hook);
    assert_eq!(
        added.status,
        Some(0),
        "the add of {name} was refused: {}",
        added.said
    );
    let slot: PathBuf = fixture.slot(name.as_str());
    let the_covered: Probe = covered.expect("the watch the arm set before the mount");
    let the_recorded: ObjectId = recorded.expect("the object the slot showed before the mount");
    let before: AfterTheMount = before_the_mount.expect("the comparison before the mount");
    write_through_the_slot(slot.as_path());
    let covered_sees: bool = the_covered.sees();
    drop(the_covered);
    let fresh: Probe = Probe::on(slot.as_path());
    write_through_the_slot(slot.as_path());
    let fresh_sees: bool = fresh.sees();
    drop(fresh);
    let after: AfterTheMount =
        after_the_mount(slot.as_path(), the_recorded).expect("the comparison after the mount");
    println!(
        "STEP21 watch before-mount={:?} before-line={} after-mount={:?}",
        before.watches,
        before.line.is_some(),
        after.watches
    );
    println!("STEP21 watch covered-watch-saw={covered_sees} fresh-watch-saw={fresh_sees}");
    let line: Option<String> = after.line.clone();
    match line.as_ref() {
        Some(sentence) => println!("STEP21 watch sentence={sentence}"),
        None => println!("STEP21 watch sentence=none"),
    }
    assert_eq!(
        before.watches,
        Watches::StillTheSameTree,
        "the comparison before the mount already called the watches blind"
    );
    assert_eq!(
        before.line, None,
        "the comparison before the mount wrote a sentence"
    );
    assert_eq!(
        after.watches,
        Watches::Blind,
        "the comparison after the mount still called the watches seeing"
    );
    assert!(
        !covered_sees,
        "a watch set on the covered directory saw the write"
    );
    assert!(
        fresh_sees,
        "a watch set on the tree the path names saw no write, so the probe cannot see anything"
    );
    let sentence: String = line.expect("the sentence for the person");
    for needle in [
        slot.to_str().expect("a text path"),
        "editor",
        "git fsmonitor--daemon status",
        "git fsmonitor--daemon stop",
    ] {
        assert!(
            sentence.contains(needle),
            "the sentence names no {needle:?}: {sentence}"
        );
    }
    println!("STEP21 watch names=slot,editor,fsmonitor");
    fixture.clear(name.as_str());
}
