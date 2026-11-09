//! Controls for who owns a conversion and how long Git's `index.lock` is held:
//! no kernel lock file,
//! one live owner per worktree
//!  named in the skip,
//! and the index lock held only from the proving pass to the end of the index rewrite.

use super::*;
use crate::subvolume_lock::acquire;
use crate::subvolume_test_support::{PlainPlatform, backdate, checkout_with_worktree};
use crate::test_support::{git, remove};
use crate::worktree_convert_journal::CLAIM_PREFIX;

/// The options of a real run.
const RUN: ConvertOptions = ConvertOptions { dry_run: false, abandon: false, command_line_attributes: false };

/// The name the conversion's kernel lock file had before the conversion stopped taking one.
const RETIRED_KERNEL_LOCK_NAME: &str = "cli-git-convert.lock";

/// A checkout with one settled detached linked worktree `wt`:
/// old files,
/// a refreshed index,
/// so it is quiet.
fn quiet_worktree(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let (parent, root) = checkout_with_worktree(name);
    let worktree: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical worktree");
    backdate(worktree.as_path());
    git(worktree.as_path(), &["update-index", "--refresh"]);
    return (parent, root, worktree);
}

/// The administrative directory of a worktree.
fn admin_of(worktree: &Path) -> PathBuf {
    return inspect(worktree).expect("registered").admin;
}

/// Whether the administrative directory holds the `index.lock` that carries the journal's token.
fn index_lock_is_the_conversions(worktree: &Path) -> bool {
    let admin: PathBuf = admin_of(worktree);
    return match Journal::load(admin.as_path()).expect("journal") {
        Some(journal) => is_ours(admin.join("index.lock").as_path(), journal.token.as_str()),
        None => false,
    };
}

/// A conversion creates no kernel lock file at all,
/// at any moment of the run and after it.
#[test]
fn a_conversion_takes_no_kernel_lock_file() {
    let (parent, _root, worktree) = quiet_worktree("ownership-no-kernel-lock");
    let platform: PlainPlatform = PlainPlatform::new();
    let admin: PathBuf = admin_of(worktree.as_path());
    let mut seen_during_the_run: bool = false;
    let verdict = convert_worktree(&platform, worktree.as_path(), RUN, &Interrupt::never(), &mut |_text: &str| {
        seen_during_the_run |= admin.join(RETIRED_KERNEL_LOCK_NAME).exists();
    });
    assert_eq!(verdict.expect("converted"), Verdict::Converted);
    assert!(!seen_during_the_run, "no kernel lock file existed while the conversion ran");
    assert!(!admin.join(RETIRED_KERNEL_LOCK_NAME).exists(), "none is left behind");
    remove(parent.as_path());
}

/// A second command that starts while one is running skips the worktree and names the live owner by its process id.
#[test]
fn a_second_conversion_while_one_runs_is_skipped_naming_the_owner() {
    let (parent, _root, worktree) = quiet_worktree("ownership-second-command");
    let platform: PlainPlatform = PlainPlatform::new();
    let mut inner: Option<Result<Verdict, MigrateError>> = None;
    let outer = convert_worktree(&platform, worktree.as_path(), RUN, &Interrupt::never(), &mut |text: &str| {
        if inner.is_none() && text.contains("copied") {
            inner = Some(convert_worktree(&platform, worktree.as_path(), RUN, &Interrupt::never(), &mut |_inner_text: &str| {}));
        }
    });
    assert_eq!(outer.expect("the first command converts"), Verdict::Converted);
    match inner.expect("the second command ran while the first copied").expect("no failure") {
        Verdict::Skipped(reasons) => {
            let text: String = reasons.join("\n");
            assert!(text.contains("another conversion") && text.contains(format!("process {}", std::process::id()).as_str()), "{text}");
        }
        other => panic!("expected a skip, got {other:?}"),
    }
    remove(parent.as_path());
}

/// Git's `index.lock` is absent while the first copy runs and after the refresh,
/// and present from the proving pass to the
///  refresh,
/// which is the only span in which a Git writer could invalidate the proof or the rewrite.
#[test]
fn the_index_lock_is_held_only_from_the_proving_pass_to_the_end_of_the_refresh() {
    let held_at = [("created", false), ("verified", true), ("synced", true), ("exchanged", true), ("refreshed", true), ("removed", false)];
    for (point, held) in held_at {
        let (parent, _root, worktree) = quiet_worktree(format!("ownership-lock-span-{point}").as_str());
        let platform: PlainPlatform = PlainPlatform::new();
        let killed = convert_worktree(&platform, worktree.as_path(), RUN, &Interrupt::at(point, 0), &mut |_text: &str| {});
        assert_eq!(killed.expect_err("stopped"), MigrateError::Interrupted(String::from(point)));
        assert_eq!(index_lock_is_the_conversions(worktree.as_path()), held, "the index lock at {point}");
        remove(parent.as_path());
    }
}

/// What a conversion that release 0.5.20 or older left behind looked like:
/// the kernel lock file in the administrative
///  directory and Git's `index.lock` with the journal's token,
/// taken at the start and kept to the end.
fn leave_as_release_0_5_20_did(worktree: &Path) {
    let admin: PathBuf = admin_of(worktree);
    std::fs::write(admin.join(RETIRED_KERNEL_LOCK_NAME), b"").expect("the kernel lock file an older release leaves");
    let journal: Journal = Journal::load(admin.as_path()).expect("journal").expect("a journal was left");
    if journal.phase < Phase::Removed {
        acquire(admin.join("index.lock").as_path(), journal.token.as_str()).expect("the older release's index lock");
    }
}

/// A conversion interrupted by release 0.5.20 or older,
/// at any phase,
/// is finished by this release:
/// it reads the journal,
///  accepts the index lock that carries its token,
/// and retires the kernel lock file.
#[test]
fn a_conversion_an_older_release_left_unfinished_is_finished() {
    for point in ["journaled", "created", "verified", "synced", "exchanged", "refreshed", "removed"] {
        let (parent, root, worktree) = quiet_worktree(format!("ownership-older-release-{point}").as_str());
        let platform: PlainPlatform = PlainPlatform::new();
        let git_file: Vec<u8> = std::fs::read(worktree.join(".git")).expect(".git");
        let killed = convert_worktree(&platform, worktree.as_path(), RUN, &Interrupt::at(point, 0), &mut |_text: &str| {});
        assert_eq!(killed.expect_err("stopped"), MigrateError::Interrupted(String::from(point)));
        leave_as_release_0_5_20_did(worktree.as_path());
        let finished = convert_worktree(&platform, worktree.as_path(), RUN, &Interrupt::never(), &mut |_text: &str| {});
        assert_eq!(finished.expect("finished"), Verdict::Resumed, "{point}");
        let admin: PathBuf = admin_of(worktree.as_path());
        assert!(platform.is_subvolume_root(worktree.as_path()).expect("examined"), "{point}");
        assert_eq!(std::fs::read(worktree.join(".git")).expect(".git"), git_file, "{point}");
        assert!(!admin.join(RETIRED_KERNEL_LOCK_NAME).exists() && !admin.join("index.lock").exists() && !admin.join(JOURNAL_NAME).exists(), "{point}: nothing is left");
        assert!(std::fs::symlink_metadata(converting_path(worktree.as_path())).is_err(), "{point}: the old tree is gone");
        assert!(root.join(".git").exists());
        remove(parent.as_path());
    }
}

/// A command of an older release that still holds the kernel lock owns the conversion:
/// this release starts nothing and writes
///  nothing,
/// and converts once the older command has ended.
#[test]
fn a_conversion_held_by_an_older_release_is_not_started_twice() {
    let (parent, _root, worktree) = quiet_worktree("ownership-older-release-running");
    let platform: PlainPlatform = PlainPlatform::new();
    let admin: PathBuf = admin_of(worktree.as_path());
    let older: std::fs::File = std::fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(admin.join(RETIRED_KERNEL_LOCK_NAME)).expect("the older release's lock file");
    // SAFETY: `flock` takes an open descriptor, which `older` keeps open for the call.
    let taken: libc::c_int = unsafe { libc::flock(std::os::unix::io::AsRawFd::as_raw_fd(&older), libc::LOCK_EX | libc::LOCK_NB) };
    assert_eq!(taken, 0, "the stand-in for the older command holds the lock");
    match convert_worktree(&platform, worktree.as_path(), RUN, &Interrupt::never(), &mut |_text: &str| {}).expect("no failure") {
        Verdict::Skipped(reasons) => assert!(reasons.join("\n").contains("another conversion"), "{reasons:?}"),
        other => panic!("expected a skip, got {other:?}"),
    }
    assert!(!admin.join(JOURNAL_NAME).exists() && !admin.join("index.lock").exists(), "nothing was written");
    drop(older);
    let converted = convert_worktree(&platform, worktree.as_path(), RUN, &Interrupt::never(), &mut |_text: &str| {});
    assert_eq!(converted.expect("converted"), Verdict::Converted);
    assert!(!admin.join(RETIRED_KERNEL_LOCK_NAME).exists(), "the leftover kernel lock file was retired");
    remove(parent.as_path());
}

/// A killed command's claim file (its process is gone) does not stop the next command,
/// and a live process's claim does.
#[test]
fn a_dead_owners_claim_is_replaced_and_a_live_owners_claim_stops_the_conversion() {
    use crate::process_identity::process_birth_identity;
    use crate::single_owner_claim::{plant_claim_for_tests, unused_pid_for_tests};
    let (parent, _root, worktree) = quiet_worktree("ownership-claims");
    let platform: PlainPlatform = PlainPlatform::new();
    let admin: PathBuf = admin_of(worktree.as_path());
    let prefix: &std::ffi::OsStr = std::ffi::OsStr::new(CLAIM_PREFIX);
    let mut other: std::process::Child = crate::test_support::spawn_owned(std::process::Command::new("sleep").arg("30")).expect("sleep");
    let other_pid: i64 = i64::from(other.id());
    let birth: String = process_birth_identity(other_pid).expect("identity").expect("alive");
    plant_claim_for_tests(admin.as_path(), prefix, 1, other_pid, birth.as_str());
    match convert_worktree(&platform, worktree.as_path(), RUN, &Interrupt::never(), &mut |_text: &str| {}).expect("no failure") {
        Verdict::Skipped(reasons) => assert!(reasons.join("\n").contains(format!("process {other_pid}").as_str()), "{reasons:?}"),
        other_verdict => panic!("expected a skip, got {other_verdict:?}"),
    }
    assert!(!admin.join(JOURNAL_NAME).exists(), "nothing was written");
    other.kill().expect("kill");
    other.wait().expect("wait");
    plant_claim_for_tests(admin.as_path(), prefix, 2, unused_pid_for_tests(), birth.as_str());
    let converted = convert_worktree(&platform, worktree.as_path(), RUN, &Interrupt::never(), &mut |_text: &str| {});
    assert_eq!(converted.expect("converted"), Verdict::Converted);
    remove(parent.as_path());
}

/// A copy that changed while it was made is not swapped,
/// and Git's `index.lock` is given back at once:
/// nothing needs it.
#[test]
fn a_failed_proof_gives_the_index_lock_back() {
    use crate::subvolume_test_support::Hooked;
    let (parent, _root, worktree) = quiet_worktree("ownership-failed-proof");
    let watched: PathBuf = worktree.clone();
    let platform: Hooked = Hooked::new(Box::new(move |call: u32| {
        // After the first few files were copied, rewrite every tracked file of the source; the proving pass then finds them changed.
        if call == 3 {
            for name in ["README.md", "src/main.rs", "script.sh", "empty"] {
                std::fs::write(watched.join(name), b"rewritten while copying\n").expect("rewrite");
            }
        }
    }));
    let verdict = convert_worktree(&platform, worktree.as_path(), RUN, &Interrupt::never(), &mut |_text: &str| {});
    let message: String = verdict.expect_err("the proving pass refuses").to_string();
    assert!(message.contains("changed while it was copied"), "{message}");
    assert!(!admin_of(worktree.as_path()).join("index.lock").exists(), "the lock is not kept after a proof that failed");
    remove(parent.as_path());
}

/// A conversion that stopped after the swap,
/// whose index lock is gone,
/// takes the lock again before it rewrites the index.
#[test]
fn a_resumed_conversion_takes_the_index_lock_before_the_rewrite() {
    let (parent, _root, worktree) = quiet_worktree("ownership-lock-before-rewrite");
    let platform: PlainPlatform = PlainPlatform::new();
    let killed = convert_worktree(&platform, worktree.as_path(), RUN, &Interrupt::at("exchanged", 0), &mut |_text: &str| {});
    assert!(killed.is_err());
    std::fs::remove_file(admin_of(worktree.as_path()).join("index.lock")).expect("the lock is gone");
    let finished = convert_worktree(&platform, worktree.as_path(), RUN, &Interrupt::never(), &mut |_text: &str| {});
    assert_eq!(finished.expect("finished"), Verdict::Resumed);
    remove(parent.as_path());
}

/// A migration of the main checkout that is running owns every worktree in it:
/// a conversion refuses the worktree,
/// names the
///  migration's process,
/// and writes nothing.
#[test]
fn a_conversion_is_refused_while_the_main_checkout_is_being_migrated() {
    use crate::process_identity::process_birth_identity;
    use crate::single_owner_claim::plant_claim_for_tests;
    use crate::subvolume_preflight::layout_of;
    let (parent, root, worktree) = quiet_worktree("ownership-migration-running");
    let platform: PlainPlatform = PlainPlatform::new();
    let layout = layout_of(root.as_path()).expect("layout");
    // The stand-in must outlive this test however slow the host is: the conversion reads the claim's owner when it looks
    // at it, and a stand-in that has ended by then makes the claim stale, so the conversion is not refused. Five minutes is
    // six times the longest run seen on a loaded host, and the process group is ended with the test thread anyway.
    let mut migration: std::process::Child = crate::test_support::spawn_owned(std::process::Command::new("sleep").arg("300")).expect("sleep");
    let pid: i64 = i64::from(migration.id());
    let birth: String = process_birth_identity(pid).expect("identity").expect("alive");
    plant_claim_for_tests(layout.parent.as_path(), layout.claim_prefix.as_os_str(), 1, pid, birth.as_str());
    match convert_worktree(&platform, worktree.as_path(), RUN, &Interrupt::never(), &mut |_text: &str| {}).expect("no failure") {
        Verdict::Skipped(reasons) => assert!(reasons.join("\n").contains(format!("process {pid}").as_str()), "{reasons:?}"),
        other => panic!("expected a skip, got {other:?}"),
    }
    assert!(!admin_of(worktree.as_path()).join(JOURNAL_NAME).exists(), "nothing was written");
    migration.kill().expect("kill");
    migration.wait().expect("wait");
    remove(parent.as_path());
}
