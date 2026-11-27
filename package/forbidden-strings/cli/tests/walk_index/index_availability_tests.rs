//! What:
//!  What a scan says and does when the index it would union is missing,
//!  named by `GIT_INDEX_FILE`,
//!  or unreachable,
//!  and when a `.git` entry itself cannot be read.
//! Why:
//!  The union is best effort by design,
//!  but the contract is that the scan says so on stderr instead of passing
//!  silently over the files the index records,
//!  and that an index Git was told to use is the one it reads.
//!  A gate that cannot see a warning is a gate that passes on a file it never
//!  inspected.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // index-availability.test.ts: missing, named, and unreadable index cases
//! ```

use std::fs;
use std::path::PathBuf;
use std::process::{
    Command,
    Output,
};

use crate::{
    SENTINEL,
    fixture,
    git_binary,
    run_git,
    scan_all,
    scan_all_with,
    seed_repository,
    write_rules,
};

/// A repository whose index is gone still scans the working tree and says on
///  stderr that the tracked files it records were not covered.
#[test]
fn all_says_the_tracked_union_is_unavailable_when_the_index_is_missing() {
    let root: PathBuf = fixture("missing-index");
    let repository: PathBuf = root.join("repository");
    seed_repository(&repository);
    fs::remove_file(repository.join(".git/index")).expect("remove the index");
    let rules: PathBuf = write_rules(&root);

    let output: Output = scan_all(&repository, &rules);
    let stderr: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("tracked-file union is unavailable")
            && stderr.contains("it cannot be read")
            && stderr.contains(".git/index"),
        "a missing index inside a repository must be reported, naming the file and why; got: {}",
        stderr
    );
    assert!(
        stderr.contains("plain.txt:1 rule="),
        "the working-tree scan must still run; got: {}",
        stderr
    );
    assert!(
        !stderr.contains("tracked.ignored"),
        "without an index the force-added file cannot be reached, and the diagnostic says so; got: {}",
        stderr
    );

    let _ = fs::remove_dir_all(&root);
}

/// Git's own `GIT_INDEX_FILE` names the active index,
///  so a scan must read that file rather than the repository's default one.
#[test]
fn all_reads_the_index_that_git_index_file_names() {
    let root: PathBuf = fixture("index-file-environment");
    let repository: PathBuf = root.join("repository");
    seed_repository(&repository);
    // This entry goes into a separate index file only, so the repository's own
    // index holds no record of it: the two indexes answer differently, which
    // is what makes the scan's choice observable.
    fs::write(
        repository.join("index-only.ignored"),
        format!("{}\n", SENTINEL),
    )
    .expect("write the ignored tracked file");
    let alternate: PathBuf = root.join("alternate-index");
    let status = Command::new(git_binary())
        .args(["add", "-f", "index-only.ignored"])
        .current_dir(&repository)
        .env("GIT_INDEX_FILE", &alternate)
        .status()
        .expect("git add failed to spawn");
    assert!(status.success(), "the alternate index must be written");
    let rules: PathBuf = write_rules(&root);

    let with_override: Output = scan_all_with(&repository, &rules, ("GIT_INDEX_FILE", &alternate));
    let stderr: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&with_override.stderr);
    assert!(
        stderr.contains("index-only.ignored:1 rule="),
        "the index GIT_INDEX_FILE names must be the one scanned; got: {}",
        stderr
    );

    let default: Output = scan_all(&repository, &rules);
    let default_stderr: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&default.stderr);
    assert!(
        !default_stderr.contains("index-only.ignored"),
        "without the variable the repository's own index is the one read; got: {}",
        default_stderr
    );

    let _ = fs::remove_dir_all(&root);
}

/// With the index gone and a secret only in a tracked,
/// ignored file,
///  the scan cannot see it:
///  it exits clean and says the tracked-file union is unavailable,
///  which is the documented best-effort contract rather than a silent pass.
#[test]
fn all_exits_clean_but_says_the_union_is_unavailable_when_only_the_index_hides_a_secret() {
    let root: PathBuf = fixture("clean-without-the-union");
    let repository: PathBuf = root.join("repository");
    fs::create_dir_all(&repository).expect("create repository dir");
    run_git(&repository, &["init", "-q"]);
    run_git(&repository, &["config", "user.email", "t@t"]);
    run_git(&repository, &["config", "user.name", "t"]);
    fs::write(repository.join(".gitignore"), "*.ignored\n").expect("write .gitignore");
    fs::write(repository.join("clean.txt"), "ordinary content\n").expect("write clean.txt");
    fs::write(
        repository.join("tracked.ignored"),
        format!("{}\n", SENTINEL),
    )
    .expect("write the ignored tracked file");
    run_git(
        &repository,
        &["add", "-f", ".gitignore", "clean.txt", "tracked.ignored"],
    );
    run_git(
        &repository,
        &[
            "commit",
            "-q",
            "-m",
            "initial",
            ".gitignore",
            "clean.txt",
            "tracked.ignored",
        ],
    );
    fs::remove_file(repository.join(".git/index")).expect("remove the index");
    let rules: PathBuf = write_rules(&root);

    let output: Output = scan_all(&repository, &rules);
    let stderr: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "the working-tree scan itself finds nothing, so it exits clean; stderr: {}",
        stderr
    );
    assert!(
        stderr.contains("tracked-file union is unavailable") && stderr.contains("gitignore"),
        "the clean exit must be qualified by the union diagnostic; got: {}",
        stderr
    );

    let _ = fs::remove_dir_all(&root);
}

/// A `.git` entry the scan cannot use is reported,
///  and the walking scan still runs.
#[test]
fn all_reports_a_git_file_it_cannot_read_and_keeps_walking() {
    let root: PathBuf = fixture("unusable-git-file");
    let repository: PathBuf = root.join("repository");
    seed_repository(&repository);
    // A nested directory holding a `.git` file Git refuses: scanning it must
    // say the tracked-file union is unavailable rather than fall back to the
    // repository above with no word.
    let nested: PathBuf = repository.join("nested");
    fs::create_dir_all(&nested).expect("create nested dir");
    fs::write(nested.join("visible.txt"), format!("{}\n", SENTINEL)).expect("write visible.txt");
    fs::write(nested.join(".git"), "not a git file\n").expect("write a malformed .git file");
    let rules: PathBuf = write_rules(&root);

    let output: Output = scan_all(&nested, &rules);
    let stderr: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("tracked-file union is unavailable"),
        "a .git entry the scan cannot use must be reported; got: {}",
        stderr
    );
    assert!(
        stderr.contains("visible.txt:1 rule="),
        "the walking scan must still cover the files it can see; got: {}",
        stderr
    );

    let _ = fs::remove_dir_all(&root);
}
