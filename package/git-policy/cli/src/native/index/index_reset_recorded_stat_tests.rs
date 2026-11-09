//! Controls for the stat data the landing records on the entries its in-process reset writes:
//! a path whose worktree file still holds the committed bytes records the file's stat data,
//! a path edited after
//!  its capture records none,
//! a path as new as the index records none,
//! and a path written earlier in the index file's own second records its stat data,
//! and the commit that follows a landing asks for no refresh for the paths
//!  the previous commit landed.

use super::support::{Edit, Plan, Scene, copy_of, entry_of, finish, landing_reset, named, scene, stat_of_file};
use super::*;
use crate::commit_index_stat_staleness::{WorktreeStaging, index_stat_data_needs_refresh};
use crate::index_stat::StatData;
use crate::test_support::{age, git_context, git_output};
use std::path::{Path, PathBuf};

/// The files of the first commit.
const FILES: [(&str, &[u8], bool); 3] = [("a", b"a one\n", false), ("d/x", b"x one\n", false), ("z", b"z one\n", false)];

/// What:
///  Settle into the middle of a second,
/// so a file's times and an index file written just after it are separated by that half second.
/// Why:
///   The racy rule compares to the nanosecond,
/// and a control for an entry older or newer than the index within one second needs a
///  second it can spend.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const second = settleIntoASecond(); // waits out the last half of a second
/// ```
fn settle_into_a_second() -> u64 {
    let mut now: std::time::Duration = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("after the epoch");
    while now.subsec_millis() > 500 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("after the epoch");
    }
    return now.as_secs();
}

/// What:
///  Set the modification and access times of `path` to `second` and `nanoseconds`,
/// without following a link.
/// Why:
///   A control states how the file's time compares with the index file's own to the nanosecond.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// utimes(path, { seconds: second, nanoseconds });
/// ```
fn set_times(path: &Path, second: u64, nanoseconds: u32) {
    use std::os::unix::ffi::OsStrExt as _;
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("no NUL in a fixture path");
    let times: [libc::timespec; 2] = [
        libc::timespec { tv_sec: second as libc::time_t, tv_nsec: libc::c_long::from(nanoseconds) },
        libc::timespec { tv_sec: second as libc::time_t, tv_nsec: libc::c_long::from(nanoseconds) },
    ];
    // SAFETY: `name` is a NUL-terminated string that outlives the call and `times` holds the two timestamps the call reads.
    let result: libc::c_int = unsafe { libc::utimensat(libc::AT_FDCWD, name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) };
    assert_eq!(result, 0, "the times of {}", path.display());
}

/// What `git status --porcelain=v1` prints in `repo`.
fn status(repo: &Path) -> String {
    let output: std::process::Output = git_output(repo, &["status", "--porcelain=v1"]);
    assert!(output.status.success(), "git status: {}", String::from_utf8_lossy(&output.stderr));
    return String::from_utf8(output.stdout).expect("UTF-8");
}

/// The real index of `repo`.
fn real_index(repo: &Path) -> PathBuf {
    return repo.join(".git").join("index");
}

/// What:
///  Whether a commit that stages `named` from the worktree needs Git's stat refresh,
/// asked as the client asks it before it
///  prepares a commit.
/// Why:
///   This is the cost the recording exists to remove:
/// a stale entry costs one Git start and one hash of its file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stale = indexStatDataNeedsRefresh({ context, cwd: repo, index: realIndex(repo), staging: { named: paths } });
/// ```
fn needs_refresh_for(repo: &Path, typed: &[&str]) -> bool {
    let owned: Vec<std::ffi::OsString> = typed.iter().map(|path| { return std::ffi::OsString::from(*path); }).collect();
    return index_stat_data_needs_refresh(&git_context(), repo, real_index(repo).as_path(), WorktreeStaging::Named(owned.as_slice()));
}

/// A scene whose second commit changes `a` from `a one` to `a two`,
/// with the worktree left holding the first commit's bytes.
fn captures_a_two(name: &str) -> Scene {
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }];
    return scene(name, &Plan { files: &FILES, edits: edits.as_slice(), ..Plan::default() });
}

/// A path the worktree still holds the committed bytes for records the file's stat data,
/// and a path edited after its capture keeps none,
/// so the edit stays visible.
#[test]
fn a_path_the_worktree_holds_records_the_files_stat_data_and_one_edited_after_its_capture_records_none() {
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }, Edit::Put { path: "d/x", mode: "100644", content: b"x two\n" }];
    let scene: Scene = scene("index-reset-recorded-stat-both", &Plan { files: &FILES, edits: edits.as_slice(), ..Plan::default() });
    // The worktree holds what was captured for `a`, while a write after the capture left `d/x` with other bytes of the same size.
    std::fs::write(scene.repo.join("a"), b"a two\n").expect("a");
    age(scene.repo.join("a").as_path());
    std::fs::write(scene.repo.join("d/x"), b"x TWO\n").expect("d/x");
    age(scene.repo.join("d/x").as_path());
    let copy: PathBuf = copy_of(&scene);
    assert!(landing_reset(&git_context(), copy.as_path(), scene.new.as_str(), IndexPaths::Named(&named(&["a", "d/x"]))), "the reset is the plain case");
    assert_eq!(entry_of(copy.as_path(), "a").stat, stat_of_file(copy.join("a").as_path()), "the path the worktree holds records the file's stat data");
    assert_eq!(entry_of(copy.as_path(), "d/x").stat, StatData::default(), "a path edited after its capture keeps the zero stat data Git hashes");
    assert_eq!(status(copy.as_path()), " M d/x\n", "the edit after the capture is still the person's");
    finish(scene);
}

/// A file written earlier in the index file's own second is older than the index,
/// so its entry records the file's stat data.
#[test]
fn a_file_written_earlier_in_the_indexs_own_second_records_its_stat_data() {
    let scene: Scene = captures_a_two("index-reset-recorded-stat-same-second");
    // The worktree holds the committed bytes, written early in the second the landing's index is dated with.
    std::fs::write(scene.repo.join("a"), b"a two\n").expect("a");
    let second: u64 = settle_into_a_second();
    set_times(scene.repo.join("a").as_path(), second, 1);
    assert!(landing_reset(&git_context(), scene.repo.as_path(), scene.new.as_str(), IndexPaths::Named(&named(&["a"]))), "the reset is the plain case");
    assert_eq!(entry_of(scene.repo.as_path(), "a").stat, stat_of_file(scene.repo.join("a").as_path()), "the file is older than the index by nanoseconds, so its stat data is recorded");
    finish(scene);
}

/// A file as new as the index the entry goes into records nothing,
/// so a same-size edit in that instant is still seen,
/// while the same
///  file two minutes older records its stat data.
#[test]
fn a_file_as_new_as_the_index_records_nothing() {
    let scene: Scene = captures_a_two("index-reset-recorded-stat-racy");
    // The worktree holds the committed bytes, and its time is the last nanosecond of the second the index is dated with.
    std::fs::write(scene.repo.join("a"), b"a two\n").expect("a");
    let second: u64 = settle_into_a_second();
    set_times(scene.repo.join("a").as_path(), second, 999_999_999);
    let copy: PathBuf = copy_of(&scene);
    assert!(landing_reset(&git_context(), copy.as_path(), scene.new.as_str(), IndexPaths::Named(&named(&["a"]))), "the reset is the plain case");
    assert_eq!(entry_of(copy.as_path(), "a").stat, StatData::default(), "an entry as new as the index records nothing");
    // A same-size edit after the landing is still the person's, and the next commit hashes it.
    std::fs::write(copy.join("a"), b"a TWO\n").expect("same-size edit");
    assert_eq!(status(copy.as_path()), " M a\n", "a same-size edit is not hidden by an entry as new as the index");
    assert!(needs_refresh_for(copy.as_path(), &["z"]), "the next commit hashes an entry as new as the index");
    // The same file, two minutes old, is older than the index the entry goes into and records.
    age(scene.repo.join("a").as_path());
    assert!(landing_reset(&git_context(), scene.repo.as_path(), scene.new.as_str(), IndexPaths::Named(&named(&["a"]))), "the reset is the plain case");
    assert_eq!(entry_of(scene.repo.as_path(), "a").stat, stat_of_file(scene.repo.join("a").as_path()), "an older file records its stat data");
    finish(scene);
}

/// After a landing that recorded the stat data of the paths it landed,
/// the commit that follows asks for no refresh for them,
/// and the positive control is an edit after the landing,
/// which makes the question true again.
#[test]
fn the_commit_after_a_landing_asks_for_no_refresh_for_the_paths_it_landed() {
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }, Edit::Put { path: "d/x", mode: "100644", content: b"x two\n" }];
    let scene: Scene = scene("index-reset-recorded-stat-next", &Plan { files: &FILES, edits: edits.as_slice(), ..Plan::default() });
    for (path, content) in [("a", b"a two\n".as_slice()), ("d/x", b"x two\n".as_slice())] {
        std::fs::write(scene.repo.join(path), content).expect("the worktree holds what was captured");
        age(scene.repo.join(path).as_path());
    }
    assert!(landing_reset(&git_context(), scene.repo.as_path(), scene.new.as_str(), IndexPaths::Named(&named(&["a", "d/x"]))), "the reset is the plain case");
    // The commit that follows lands `z`, so neither landed path is staged from the worktree afresh.
    assert!(!needs_refresh_for(scene.repo.as_path(), &["z"]), "no Git start and no hash for the paths the previous commit landed");
    std::fs::write(scene.repo.join("a"), b"a two and more\n").expect("an edit after the landing");
    assert!(needs_refresh_for(scene.repo.as_path(), &["z"]), "an edit after the landing is stale again");
    finish(scene);
}
