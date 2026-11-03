//! Controls for who owns a conversion and how long Git's `index.lock` is held: no kernel lock file, one live owner per worktree
//!  named in the skip, and the index lock held only from the proving pass to the end of the index rewrite.

use super::*;
use crate::subvolume_test_support::{PlainPlatform, backdate, checkout_with_worktree};
use crate::test_support::{git, remove};

/// The options of a real run.
const RUN: ConvertOptions = ConvertOptions { dry_run: false, abandon: false, command_line_attributes: false };

/// The name the conversion's kernel lock file had before the conversion stopped taking one.
const RETIRED_KERNEL_LOCK_NAME: &str = "cli-git-convert.lock";

/// A checkout with one settled detached linked worktree `wt`: old files, a refreshed index, so it is quiet.
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

/// A conversion creates no kernel lock file at all, at any moment of the run and after it.
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

/// Git's `index.lock` is absent while the first copy runs and after the refresh, and present from the proving pass to the
///  refresh, which is the only span in which a Git writer could invalidate the proof or the rewrite.
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
