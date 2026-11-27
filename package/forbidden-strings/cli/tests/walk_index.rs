//!  Binary-boundary tests for the index the `--all` walk unions its file list
//!  with (issue 1232).
//!
//! What:
//!  `--all` walks the working tree honouring `.gitignore`,
//!  then unions in the paths this working tree's Git index records,
//!  so a tracked file that `.gitignore` hides is still scanned.
//! Why:
//!  the index a linked worktree,
//!  a submodule,
//!  or a subfolder scan must read is not `<walked directory>/.git/index`,
//!  and reading the wrong file silently dropped exactly those force-added
//!  files,
//!  so a tracked secret in an ignored file passed the gate with no
//!  diagnostic.
//!  These tests ask the real binary,
//!  from the real working directory shapes,
//!  because the defect is in which file the process reads.
//!
//!  The cases sit in two role-named modules of this folder;
//!  this file holds the fixture harness they call,
//!  then lists the parts in the order they build on it.
//!  Each part needs `#[path]` because a crate root always looks for its
//!  modules beside itself,
//!  in `tests/`,
//!  not in this folder.

/// Imports ordinary filesystem operations for isolated fixtures.
use std::fs;
/// Imports the absolute path type for disposable fixtures.
use std::path::{
    Path,
    PathBuf,
};
/// Imports the process runner to exercise the actual CLI.
use std::process::{
    Command,
    Output,
};

/// Binary artifact built by Cargo for this integration test.
const BIN: &str = env!("CARGO_BIN_EXE_forbidden-strings");

///  The literal the fixture rules file forbids.
///
///  It is long and unusual so the embedded baseline never matches it on its
///  own,
///  which keeps a finding unambiguously the fixture rule's.
const SENTINEL: &str = "VAULTTOKEN_ISSUE_1232";

///  The system Git binary,
///  or `git` from `PATH` where the platform keeps it elsewhere.
///
///  The developer's `git` on `PATH` is this repository's policy wrapper,
///  whose hooks and copies a fixture must not run,
///  so the platform's own location wins when it exists.
fn git_binary() -> &'static str {
    if Path::new("/usr/bin/git").exists() {
        return "/usr/bin/git";
    }
    return "git";
}

/// Creates one disposable fixture directory under the test target directory.
fn fixture(label: &str) -> PathBuf {
    let root: PathBuf = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "fs-walk-index-{}-{}",
        label,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create fixture root");
    return root;
}

///  Runs the real Git binary in `directory`,
///  failing the test on a non-zero
///  exit.
fn run_git(
    directory: &Path,
    args: &[&str],
) {
    let status = Command::new(git_binary())
        .args(args)
        .current_dir(directory)
        .status()
        .expect("git failed to spawn");
    assert!(status.success(), "git {:?} failed in {:?}", args, directory);
}

///  Runs the scanner with `--all` from `directory`,
///  using a disposable cache root beside the rules file,
///  which is outside every scanned directory and keeps the developer's cache
///  untouched.
fn scan_all(
    directory: &Path,
    rules: &Path,
) -> Output {
    return configured(directory, rules, None);
}

///  Runs the scanner with `--all` and an explicit child environment,
///  the only way to exercise a variable that is process-global.
fn scan_all_with(
    directory: &Path,
    rules: &Path,
    variable: (&str, &Path),
) -> Output {
    return configured(directory, rules, Some(variable));
}

///  Runs the scanner with `--all`,
///  optionally setting one environment variable
///  on the child only.
fn configured(
    directory: &Path,
    rules: &Path,
    variable: Option<(&str, &Path)>,
) -> Output {
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
    return rules;
}

///  Creates a repository whose `.gitignore` hides `*.ignored`,
///  with `plain.txt` and a force-added `tracked.ignored` both holding the
///  sentinel.
fn seed_repository(repository: &Path) {
    fs::create_dir_all(repository).expect("create repository dir");
    run_git(repository, &["init", "-q"]);
    run_git(repository, &["config", "user.email", "t@t"]);
    run_git(repository, &["config", "user.name", "t"]);
    fs::write(repository.join(".gitignore"), "*.ignored\n").expect("write .gitignore");
    fs::write(repository.join("plain.txt"), format!("{}\n", SENTINEL)).expect("write plain.txt");
    fs::write(
        repository.join("tracked.ignored"),
        format!("{}\n", SENTINEL),
    )
    .expect("write tracked.ignored");
    run_git(
        repository,
        &["add", "-f", ".gitignore", "plain.txt", "tracked.ignored"],
    );
    run_git(
        repository,
        &[
            "commit",
            "-q",
            "-m",
            "initial",
            ".gitignore",
            "plain.txt",
            "tracked.ignored",
        ],
    );
}

/// What:
///  `#[path = "walk_index/index_availability_tests.rs"] mod
///  index_availability_tests;` registers the module covering an index that is
///  missing,
///  named by `GIT_INDEX_FILE`,
///  or unreachable.
/// Why:
///  The union is best effort,
///  and the scan must say so rather than pass
///  silently.
#[path = "walk_index/index_availability_tests.rs"]
mod index_availability_tests;
/// What:
///  `#[path = "walk_index/linked_worktree_tests.rs"] mod
///  linked_worktree_tests;` registers the module covering a linked worktree
///  and a scan started in a subfolder.
/// Why:
///  Those two shapes are what the wrong-index defect dropped files from.
#[path = "walk_index/linked_worktree_tests.rs"]
mod linked_worktree_tests;
