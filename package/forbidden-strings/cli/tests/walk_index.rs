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

/// The system Git binary,
/// or `git` from `PATH` where the platform keeps it elsewhere.
///
/// The developer's `git` on `PATH` is this repository's policy wrapper,
/// whose hooks and copies a fixture must not run,
/// so the platform's own location wins when it exists.
fn git_binary() -> &'static str {
    if Path::new("/usr/bin/git").exists() {
        return "/usr/bin/git"
    }
    return "git"
}

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
    let status = Command::new(git_binary())
        .args(args)
        .current_dir(directory)
        .status()
        .expect("git failed to spawn");
    assert!(status.success(), "git {:?} failed in {:?}", args, directory);
}

/// Runs the scanner with `--all` from `directory`,
///  using a disposable cache root beside the rules file,
///  which is outside every scanned directory and keeps the developer's cache
///  untouched.
fn scan_all(directory: &Path, rules: &Path) -> Output {
    return configured(directory, rules, None)
}

/// Runs the scanner with `--all` and an explicit child environment,
///  the only way to exercise a variable that is process-global.
fn scan_all_with(directory: &Path, rules: &Path, variable: (&str, &Path)) -> Output {
    return configured(directory, rules, Some(variable))
}

/// Runs the scanner with `--all`,
/// optionally setting one environment variable
///  on the child only.
fn configured(directory: &Path, rules: &Path, variable: Option<(&str, &Path)>) -> Output {
    let cache_root: PathBuf = rules.parent().expect("rules parent").join("cache");
    let mut command = Command::new(BIN);
    command
        .args(["--rules", rules.to_str().expect("rules path utf8"), "--all"])
        .current_dir(directory)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", cache_root);
    if let Some((name, value)) = variable {
        command.env(name, value);
    }
    return command.output().expect("spawn the scanner");
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
    fs::write(repository.join("index-only.ignored"), format!("{}\n", SENTINEL))
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
    fs::write(repository.join("tracked.ignored"), format!("{}\n", SENTINEL))
        .expect("write the ignored tracked file");
    run_git(&repository, &["add", "-f", ".gitignore", "clean.txt", "tracked.ignored"]);
    run_git(
        &repository,
        &["commit", "-q", "-m", "initial", ".gitignore", "clean.txt", "tracked.ignored"],
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
