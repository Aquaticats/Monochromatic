//! Unit tests of the shared named helpers.

use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use super::fixtures::sample_lock;
use super::util::{all_ascii_digits, contains_any, ends_with_any, file_name_os, file_name_text, find_tool, number_or_zero, starts_with_ascii_digit};

#[test]
fn digit_predicates_accept_only_ascii_digits() {
    assert!(all_ascii_digits("0123"));
    assert!(all_ascii_digits(""));
    assert!(!all_ascii_digits("12a"));
    assert!(!all_ascii_digits("\u{0663}"));
    assert!(starts_with_ascii_digit("4.1"));
    assert!(!starts_with_ascii_digit("v4"));
    assert!(!starts_with_ascii_digit(""));
}

#[test]
fn any_of_predicates_match_substrings_and_endings() {
    assert!(contains_any("tool-linux-x64", &["darwin", "linux"]));
    assert!(!contains_any("tool-linux-x64", &["darwin", "windows"]));
    assert!(!contains_any("tool", &[]));
    assert!(ends_with_any("a.tar.gz", &[".zip", ".gz"]));
    assert!(!ends_with_any("a.tar.gz", &[".zip", ".xz"]));
}

#[test]
fn a_tool_is_found_by_its_id() {
    let lock = sample_lock();
    assert_eq!(find_tool(&lock.tools, "node").map(|tool| return tool.version.as_str()), Some("1.2.3"));
    assert!(find_tool(&lock.tools, "absent").is_none());
}

#[test]
fn file_names_convert_and_missing_ones_are_empty() {
    assert_eq!(file_name_text(Path::new("/a/b/c.txt")), "c.txt");
    assert_eq!(file_name_text(Path::new("/")), "");
    assert_eq!(file_name_os(Path::new("/a/b")).to_string_lossy(), "b");
    assert_eq!(number_or_zero("17"), 17);
    assert_eq!(number_or_zero("rc1"), 0);
}

//region The mode of the temporary an atomic write builds

/// What:
/// The process's umask, set for one test and put back when the guard drops.
/// Why:
/// The mode the kernel gives a created file is the mode `open` was asked for,
/// less the umask,
/// so proving "never wider than the kept mode, whatever the umask" needs two umasks in one run.
/// The tested write never widens a file beyond the mode it is asked for,
/// so a test that lowers the umask cannot make a sibling test's file wider than it asked for;
/// nextest also gives each test a process of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withUmask = (mask: number, body: () => void) => { const previous = process.umask(mask); try { body() } finally { process.umask(previous) } };
/// ```
#[cfg(unix)]
struct Umask(
    /// The umask this test found, put back when it ends.
    u32,
);

// Sets the umask: a POSIX call the standard library does not carry, declared here for the tests alone.
#[cfg(unix)]
unsafe extern "C" {
    fn umask(mask: u32) -> u32;
}

/// Sets the umask for the lifetime of the returned guard.
#[cfg(unix)]
fn set_umask(mask: u32) -> Umask {
    // SAFETY: `umask` takes one integer and returns one; it touches no memory of ours,
    // and the previous value is put back when the guard drops.
    return Umask(unsafe { umask(mask) });
}

#[cfg(unix)]
impl Drop for Umask {
    fn drop(&mut self) {
        // SAFETY: the same one-integer POSIX call, with the value this test found.
        unsafe {
            umask(self.0);
        }
    }
}

/// Every `.<target name>.<pid>.tmp` beside `target`,
/// in the order the directory lists them.
#[cfg(unix)]
fn temporaries_beside(target: &Path) -> Vec<std::path::PathBuf> {
    let parent = target.parent().expect("a parent directory");
    let prefix = format!(".{}.", file_name_text(target));
    let mut found: Vec<std::path::PathBuf> = Vec::new();
    for entry in std::fs::read_dir(parent).expect("read the directory").flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(&prefix) && name.ends_with(".tmp") {
            found.push(entry.path());
        }
    }
    return found;
}

/// The mode bits of one file.
#[cfg(unix)]
fn mode_of(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    return std::fs::metadata(path).expect("the file exists").permissions().mode() & 0o7777;
}

/// What:
/// The new content is never written into a file wider than the mode the finished file keeps,
/// whichever umask the process has.
/// Why:
/// The window this closes is the one a reader of another account could open:
/// under umask 022 the old write asked for 0666 and the kernel gave 0644,
/// wrote the whole plan,
/// and only then narrowed it to 0600.
/// The kill point reads the temporary at the only instant a reader could open it,
/// which is also the instant the review's `strace` shows the `write` at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('never writes the plan into a wider file', () => { ... })
/// ```
#[test]
#[cfg(unix)]
fn the_new_content_is_only_ever_in_a_file_of_the_kept_mode_whatever_the_umask() {
    use std::cell::RefCell;
    // The kept mode 0644 under the ambient umask is the third arm: even there the old write landed on 0644,
    // while a kept mode of 0640 must not become 0644 in the others' bit.
    let arms: [(u32, Option<u32>); 3] = [(0o600, Some(0o022)), (0o600, Some(0o000)), (0o640, None)];
    // Every arm is measured, and the failures are reported together: one run then says which umasks go wrong.
    let mut problems: Vec<String> = Vec::new();
    for (kept, umask_asked) in arms {
        let directory = crate::test_scratch::scratch();
        let target = directory.path().join("toolchain.plan.jsonc");
        std::fs::write(&target, "the plan the person wrote\n").expect("write the plan");
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(kept)).expect("set the plan's mode");
        let plan = "the plan the person wrote, plus the entry this run adds\n";
        let seen: RefCell<Option<(u32, String)>> = RefCell::new(None);
        let _umask = umask_asked.map(set_umask);
        let reader = |temporary: &Path| -> Result<(), crate::toolchain::error::ToolchainError> {
            let content = std::fs::read_to_string(temporary).expect("read the temporary");
            *seen.borrow_mut() = Some((mode_of(temporary), content));
            return Ok(());
        };
        super::util::write_atomically_at(&target, plan.as_bytes(), Some(kept), &reader).expect("the write lands");
        let (seen_mode, seen_content) = seen.borrow().clone().expect("the kill point ran while the temporary held the bytes");
        if seen_content != plan {
            problems.push(format!("the temporary holds every byte of the new plan (kept mode {kept:o})"));
        }
        if seen_mode != kept {
            problems.push(format!(
                "the file holding the new plan carries the kept mode {kept:o} from its first byte, and it carried {seen_mode:o} under umask {umask_asked:?}"
            ));
        }
        if mode_of(&target) != kept {
            problems.push(format!("the renamed plan keeps the mode {kept:o}, and it has {:o}", mode_of(&target)));
        }
        if std::fs::read_to_string(&target).expect("read the plan") != plan {
            problems.push(format!("the renamed plan holds the new content (kept mode {kept:o})"));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// What:
/// A write that ends where a kill lands leaves the temporary holding the new content,
/// readable no more widely than the plan,
/// and the next write of the same plan removes it.
/// Why:
/// The cleanup on this crate's error paths does not run when the process is killed,
/// so the leftover is a file nobody owns;
/// a temporary whose process is gone must be removed by the next writer,
/// and one whose process still runs must be left for that process to rename.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('leaves only a copy as private as the plan', () => { ... })
/// ```
#[test]
#[cfg(unix)]
fn a_killed_write_leaves_a_copy_no_wider_than_the_plan_and_the_next_write_removes_it() {
    let _umask = set_umask(0o022);
    let directory = crate::test_scratch::scratch();
    let target = directory.path().join("toolchain.plan.jsonc");
    std::fs::write(&target, "the plan the person wrote\n").expect("write the plan");
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o600)).expect("set the plan's mode");
    let plan = "the plan the person wrote, plus the entry this run adds\n";
    let killed = |_temporary: &Path| -> Result<(), crate::toolchain::error::ToolchainError> {
        return Err(crate::toolchain::error::ToolchainError::failure("killed between the write and the rename"));
    };
    super::util::write_atomically_at(&target, plan.as_bytes(), Some(0o600), &killed).expect_err("the run ends");
    assert_eq!(std::fs::read_to_string(&target).expect("read the plan"), "the plan the person wrote\n", "the plan is the old one");
    let left = temporaries_beside(&target);
    assert_eq!(left.len(), 1, "one temporary is left: {left:?}");
    assert_eq!(mode_of(&left[0]), 0o600, "another account cannot read the plan a kill left behind");
    assert_eq!(std::fs::read_to_string(&left[0]).expect("read the leftover"), plan);
    // The leftover carries this process's pid, so this process is the one that will rename it: the next write
    // of the same plan reuses that name and leaves nothing behind.
    super::util::write_atomically_with_mode(&target, b"the plan after the next run\n", Some(0o600)).expect("the write lands");
    assert!(temporaries_beside(&target).is_empty(), "the next write leaves no temporary behind");
    assert_eq!(mode_of(&target), 0o600);
}

/// What:
/// A temporary whose process is gone is removed by the next write of the same target,
/// and one whose process is still running is left alone.
/// Why:
/// This is what makes the kill window a window and not a permanent leak,
/// and the second half is what keeps a concurrent writer's temporary from being deleted under it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('clears what a dead process left and keeps a live one', () => { ... })
/// ```
#[test]
#[cfg(target_os = "linux")]
fn the_next_write_clears_a_temporary_of_a_process_that_is_gone_and_keeps_a_live_one() {
    let directory = crate::test_scratch::scratch();
    let target = directory.path().join("toolchain.lock.json");
    std::fs::write(&target, "the old lock\n").expect("write the lock");
    let gone = directory.path().join(format!(".{}.{}.tmp", file_name_text(&target), super::fixtures::pid_of_a_process_that_ended()));
    std::fs::write(&gone, "what a killed run left\n").expect("write the leftover");
    // Another writer that is still running: a program of its own, so this write's temporary name differs from it.
    let mut other = std::process::Command::new("sleep").arg("30").spawn().expect("start a program that stays");
    let live = directory.path().join(format!(".{}.{}.tmp", file_name_text(&target), other.id()));
    std::fs::write(&live, "another writer's temporary\n").expect("write the live temporary");
    super::util::write_atomically(&target, b"the new lock\n").expect("the write lands");
    let kept = std::fs::read_to_string(&live);
    let _ = other.kill();
    let _ = other.wait();
    assert!(!gone.exists(), "the leftover of a process that is gone is removed");
    assert_eq!(kept.expect("read the live temporary"), "another writer's temporary\n", "a live run's temporary stays");
    assert_eq!(std::fs::read_to_string(&target).expect("read the lock"), "the new lock\n");
}

//endregion The mode of the temporary an atomic write builds
