//! What:
//!  The index of a working tree that is not a plain main checkout:
//!  a linked worktree's force-added,
//!  `.gitignore`-hidden file is reported from inside that worktree,
//!  and a scan started in a subfolder still reports the tracked file
//!  `.gitignore` hides below it.
//! Why:
//!  Neither tree keeps its index at `<walked directory>/.git/index`,
//!  so joining that path onto the walked directory silently dropped exactly
//!  the force-added files the union exists for,
//!  and a tracked secret in an ignored file passed the gate with no
//!  diagnostic.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // linked-worktree.test.ts: a worktree's gitdir names another directory
//! ```

use std::fs;
use std::path::PathBuf;
use std::process::Output;

use crate::{
    SENTINEL,
    fixture,
    run_git,
    scan_all,
    seed_repository,
    write_rules,
};

/// A scan of a linked worktree from inside it reports the force-added
/// tracked file that `.gitignore` hides.
#[test]
fn all_reports_the_tracked_ignored_file_of_a_linked_worktree() {
    let root: PathBuf = fixture("linked-worktree");
    let main: PathBuf = root.join("main");
    seed_repository(&main);
    let worktree: PathBuf = root.join("wt");
    run_git(
        &main,
        &[
            "worktree",
            "add",
            "-q",
            worktree.to_str().expect("worktree utf8"),
            "-b",
            "wt-branch",
        ],
    );
    assert!(
        worktree.join(".git").is_file(),
        "fixture precondition: a linked worktree's .git is a file"
    );
    let rules: PathBuf = write_rules(&root);

    let output: Output = scan_all(&worktree, &rules);
    let stderr: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "the tracked secret in a linked worktree must fail the scan.\nstderr: {}",
        stderr
    );
    assert!(
        stderr.contains("tracked.ignored:1 rule="),
        "the worktree's tracked, ignored file must be scanned and reported; got: {}",
        stderr
    );
    assert!(
        stderr.contains("plain.txt:1 rule="),
        "the ordinary tracked file must still be scanned; got: {}",
        stderr
    );

    let _ = fs::remove_dir_all(&root);
}

/// A scan started in a subfolder of a main repository reports the tracked
///  file that `.gitignore` hides below that subfolder.
#[test]
fn all_reports_the_tracked_ignored_file_below_a_repository_subfolder() {
    let root: PathBuf = fixture("subfolder");
    let repository: PathBuf = root.join("repository");
    fs::create_dir_all(repository.join("sub")).expect("create subfolder");
    run_git(&repository, &["init", "-q"]);
    run_git(&repository, &["config", "user.email", "t@t"]);
    run_git(&repository, &["config", "user.name", "t"]);
    fs::write(repository.join(".gitignore"), "*.ignored\n").expect("write .gitignore");
    fs::write(
        repository.join("sub/tracked.ignored"),
        format!("{}\n", SENTINEL),
    )
    .expect("write the ignored tracked file");
    run_git(
        &repository,
        &["add", "-f", ".gitignore", "sub/tracked.ignored"],
    );
    run_git(
        &repository,
        &[
            "commit",
            "-q",
            "-m",
            "initial",
            ".gitignore",
            "sub/tracked.ignored",
        ],
    );
    let rules: PathBuf = write_rules(&root);

    let output: Output = scan_all(&repository.join("sub"), &rules);
    let stderr: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "the tracked secret under a scanned subfolder must fail the scan.\nstderr: {}",
        stderr
    );
    assert!(
        stderr.contains("tracked.ignored:1 rule="),
        "the subfolder scan must report the tracked, ignored file it walked over; got: {}",
        stderr
    );

    let _ = fs::remove_dir_all(&root);
}
