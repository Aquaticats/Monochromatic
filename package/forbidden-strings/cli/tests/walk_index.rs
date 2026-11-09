//! Binary-boundary tests for the index the `--all` walk unions its file list
//! with (issue 1232).
//!
//! What:
//! `--all` walks the working tree honouring `.gitignore`,
//!  then unions in the paths this working tree's Git index records,
//!  so a tracked file that `.gitignore` hides is still scanned.
//! Why:
//! the index a linked worktree,
//!  a submodule,
//!  or a subfolder scan must read is not `<walked directory>/.git/index`,
//!  and reading the wrong file silently dropped exactly those force-added
//!  files,
//!  so a tracked secret in an ignored file passed the gate with no
//!  diagnostic.
//!  These tests ask the real binary,
//! from the real working directory shapes,
//!  because the defect is in which file the process reads.

/// Imports ordinary filesystem operations for isolated fixtures.
use std::fs;
/// Imports the absolute path type for disposable fixtures.
use std::path::{Path, PathBuf};
/// Imports the process runner to exercise the actual CLI.
use std::process::{Command, Output};

/// Binary artifact built by Cargo for this integration test.
const BIN: &str = env!("CARGO_BIN_EXE_forbidden-strings");

/// The literal the fixture rules file forbids.
///
/// It is long and unusual so the embedded baseline never matches it on its
///  own,
/// which keeps a finding unambiguously the fixture rule's.
const SENTINEL: &str = "VAULTTOKEN_ISSUE_1232";

/// The system Git binary.
///
/// The developer's `git` on `PATH` is this repository's policy wrapper,
/// which
///  adds hooks and copies that a fixture must not run.
const GIT: &str = "/usr/bin/git";

/// Creates one disposable fixture directory under the test target directory.
fn fixture(label: &str) -> PathBuf {
    let root: PathBuf = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("fs-walk-index-{}-{}", label, std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create fixture root");
    return root
}

/// Runs the real Git binary in `directory`,
/// failing the test on a non-zero
/// exit.
fn run_git(directory: &Path, args: &[&str]) {
    let status = Command::new(GIT)
        .args(args)
        .current_dir(directory)
        .status()
        .expect("git failed to spawn");
    assert!(status.success(), "git {:?} failed in {:?}", args, directory);
}

/// Runs the scanner with `--all` from `directory`,
/// using a disposable cache root so no test touches the developer's cache.
fn scan_all(directory: &Path, rules: &Path) -> Output {
    let cache_root: PathBuf = directory.join("cache");
    return Command::new(BIN)
        .args([
            "--rules",
            rules.to_str().expect("rules path utf8"),
            "--all",
        ])
        .current_dir(directory)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", cache_root)
        .output()
        .expect("spawn the scanner");
}

/// Writes the fixture rules file forbidding [`SENTINEL`] and returns its path.
fn write_rules(root: &Path) -> PathBuf {
    let rules: PathBuf = root.join("rules.txt");
    fs::write(&rules, format!("{}\n", SENTINEL)).expect("write the fixture rules");
    return rules
}

/// Creates a repository whose `.gitignore` hides `*.ignored`,
///  with `plain.txt` and a force-added `tracked.ignored` both holding the
///  sentinel.
fn seed_repository(repository: &Path) {
    fs::create_dir_all(repository).expect("create repository dir");
    run_git(repository, &["init", "-q"]);
    run_git(repository, &["config", "user.email", "t@t"]);
    run_git(repository, &["config", "user.name", "t"]);
    fs::write(repository.join(".gitignore"), "*.ignored\n").expect("write .gitignore");
    fs::write(repository.join("plain.txt"), format!("{}\n", SENTINEL)).expect("write plain.txt");
    fs::write(repository.join("tracked.ignored"), format!("{}\n", SENTINEL))
        .expect("write tracked.ignored");
    run_git(repository, &["add", "-f", ".gitignore", "plain.txt", "tracked.ignored"]);
    run_git(
        repository,
        &["commit", "-q", "-m", "initial", ".gitignore", "plain.txt", "tracked.ignored"],
    );
}

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
        &["worktree", "add", "-q", worktree.to_str().expect("worktree utf8"), "-b", "wt-branch"],
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
    fs::write(repository.join("sub/tracked.ignored"), format!("{}\n", SENTINEL))
        .expect("write the ignored tracked file");
    run_git(&repository, &["add", "-f", ".gitignore", "sub/tracked.ignored"]);
    run_git(&repository, &["commit", "-q", "-m", "initial", ".gitignore", "sub/tracked.ignored"]);
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
        stderr.contains("git reset"),
        "a missing index inside a repository must be reported, with its remedy; got: {}",
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
