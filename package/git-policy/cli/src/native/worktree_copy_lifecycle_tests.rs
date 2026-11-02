//! Lifecycle checks against real Git: applicability, bypasses, settlement, the summary line and exit statuses.

use super::*;
use crate::test_support::{fixture, git, git_context, git_output, remove, repository};
use crate::worktree_copy_journal::{ensure_journal_root, read_pending_journals};
use crate::worktree_copy_lock::SETTLEMENT_LOCK_NAME;
use std::process::Output;

/// A repository with an ignored `cache` directory, returned with its canonical common directory.
fn source(root: &Path) -> (PathBuf, PathBuf) {
    let main: PathBuf = repository(root, "main");
    std::fs::write(main.join(".gitignore"), b"cache/\n").expect("gitignore");
    std::fs::create_dir_all(main.join("cache")).expect("cache");
    std::fs::write(main.join("cache/data"), b"payload").expect("data");
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("common");
    return (std::fs::canonicalize(main).expect("canonical main"), common);
}

/// Runs real Git with the given arguments in a directory and reports its status.
fn run_real(directory: &Path, arguments: &[String]) -> GitStatus {
    let output: Output = git_output(directory, arguments);
    return GitStatus {
        code: output.status.code(),
    };
}

/// The identity of the main worktree of a source.
fn main_identity(main: &Path, common: &Path) -> WorktreeIdentity {
    return WorktreeIdentity::MainWorktree {
        common_dir: common.to_path_buf(),
        git_dir: common.to_path_buf(),
        worktree_root: main.to_path_buf(),
    };
}

/// The summary line names the count, the source, and the destinations, with the incumbent's nouns.
#[test]
fn summary_line_uses_exact_nouns() {
    assert_eq!(
        summary_line(1, 1, Some(Path::new("/repo"))),
        "cli-git: copied 1 ignored filesystem entry from \"/repo\" into 1 new worktree.\n"
    );
    assert_eq!(
        summary_line(3, 2, None),
        "cli-git: copied 3 ignored filesystem entries from bare repository with an empty source set into 2 new worktrees.\n"
    );
}

/// The exit status follows the incumbent: a copy-only failure is 2, Git's own status is kept, and a signal is 1.
#[test]
fn exit_status_follows_the_settlement_table() {
    let ok: GitStatus = GitStatus { code: Some(0) };
    let failed: GitStatus = GitStatus { code: Some(5) };
    let signal: GitStatus = GitStatus { code: None };
    assert_eq!(
        exit_code(&Settlement {
            git: ok,
            copy_failure: None
        }),
        0
    );
    assert_eq!(
        exit_code(&Settlement {
            git: failed,
            copy_failure: None
        }),
        5
    );
    assert_eq!(
        exit_code(&Settlement {
            git: signal,
            copy_failure: None
        }),
        1
    );
    let copy_failure: WorktreeCopyError = WorktreeCopyError(String::from("copy failed"));
    assert_eq!(
        exit_code(&Settlement {
            git: ok,
            copy_failure: Some(copy_failure.clone())
        }),
        2
    );
    assert_eq!(
        exit_code(&Settlement {
            git: failed,
            copy_failure: Some(copy_failure.clone())
        }),
        5
    );
    assert_eq!(
        exit_code(&Settlement {
            git: signal,
            copy_failure: Some(copy_failure)
        }),
        1
    );
}

/// A worktree added from a linked worktree gets its ignored files, the summary line, and leaves no lock behind.
#[cfg(unix)]
#[test]
fn added_worktree_receives_ignored_state_and_lock_is_released() {
    let root: PathBuf = fixture("lifecycle-add");
    let (main, common) = source(root.as_path());
    std::fs::write(common.join("info/exclude"), b"cache/\n").expect("exclude");
    let source_root: PathBuf = root.join("source");
    git(
        main.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "source",
            source_root.to_str().expect("utf8"),
        ],
    );
    let source_canonical: PathBuf = std::fs::canonicalize(source_root).expect("canonical source");
    std::fs::create_dir_all(source_canonical.join("cache")).expect("cache");
    std::fs::write(source_canonical.join("cache/data"), b"payload").expect("data");
    let topic: PathBuf = root.join("topic");
    let identity: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: common.clone(),
        git_dir: common.join("worktrees").join("source"),
        worktree_root: source_canonical.clone(),
    };
    let mut stderr: Vec<u8> = Vec::new();
    let args: Vec<String> = vec![
        String::from("worktree"),
        String::from("add"),
        String::from("--quiet"),
        String::from("-b"),
        String::from("topic"),
        topic.to_string_lossy().into_owned(),
    ];
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity: identity.clone(),
            effective_cwd: source_canonical.clone(),
            creates_or_moves: true,
            opted_out: false,
            short_circuit: false,
            inherited_token: None,
            context: git_context(),
        },
        &mut |_token| return Ok(run_real(source_canonical.as_path(), args.as_slice())),
        &mut || return Ok(identity.clone()),
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0);
    assert_eq!(
        std::fs::read(topic.join("cache/data")).expect("copied"),
        b"payload"
    );
    assert_eq!(
        String::from_utf8(stderr).expect("utf8"),
        format!(
            "cli-git: copied 2 ignored filesystem entries from {} into 1 new worktree.\n",
            quote(source_canonical.to_string_lossy().as_ref())
        )
    );
    assert!(
        !ensure_journal_root(common.as_path())
            .expect("root")
            .join(SETTLEMENT_LOCK_NAME)
            .exists()
    );
    assert!(
        read_pending_journals(common.as_path())
            .expect("pending")
            .is_empty()
    );
    remove(root.as_path());
}

/// The opt-out flag skips synchronization and recovery, and the flag never reaches the runner.
#[cfg(unix)]
#[test]
fn opted_out_invocation_skips_synchronization() {
    let root: PathBuf = fixture("lifecycle-opt-out");
    let (main, common) = source(root.as_path());
    let topic: PathBuf = root.join("topic");
    let identity: WorktreeIdentity = main_identity(main.as_path(), common.as_path());
    let mut seen: Vec<Option<String>> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity: identity.clone(),
            effective_cwd: main.clone(),
            creates_or_moves: true,
            opted_out: true,
            short_circuit: false,
            inherited_token: None,
            context: git_context(),
        },
        &mut |token| {
            seen.push(token.map(str::to_string));
            return Ok(run_real(
                main.as_path(),
                &[
                    String::from("worktree"),
                    String::from("add"),
                    String::from("--quiet"),
                    String::from("-b"),
                    String::from("topic"),
                    topic.to_string_lossy().into_owned(),
                ],
            ));
        },
        &mut || return Ok(identity.clone()),
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0);
    assert_eq!(
        seen,
        vec![None],
        "no settlement lease is handed to a skipped run"
    );
    assert!(!topic.join("cache").exists());
    assert!(stderr.is_empty());
    remove(root.as_path());
}

/// A main worktree command that creates or moves no worktree runs without any settlement or lock.
#[test]
fn main_worktree_command_that_creates_nothing_is_not_settled() {
    let root: PathBuf = fixture("lifecycle-main");
    let (main, common) = source(root.as_path());
    let identity: WorktreeIdentity = main_identity(main.as_path(), common.as_path());
    let mut seen: Vec<Option<String>> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity: identity.clone(),
            effective_cwd: main.clone(),
            creates_or_moves: false,
            opted_out: false,
            short_circuit: false,
            inherited_token: None,
            context: git_context(),
        },
        &mut |token| {
            seen.push(token.map(str::to_string));
            return Ok(GitStatus { code: Some(7) });
        },
        &mut || return Ok(identity.clone()),
        &mut Vec::new(),
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 7);
    assert_eq!(seen, vec![None]);
    assert!(
        !ensure_journal_root(common.as_path())
            .expect("root")
            .join(SETTLEMENT_LOCK_NAME)
            .exists()
    );
    remove(root.as_path());
}

/// A command that creates no worktrees runs outside the lock; a pending journal alone takes it for recovery.
#[cfg(unix)]
#[test]
fn non_worktree_command_runs_without_settlement() {
    let root: PathBuf = fixture("lifecycle-status");
    let (main, common) = source(root.as_path());
    let identity: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: common.clone(),
        git_dir: common.clone(),
        worktree_root: main.clone(),
    };
    let mut seen: Vec<Option<String>> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity: identity.clone(),
            effective_cwd: main.clone(),
            creates_or_moves: false,
            opted_out: false,
            short_circuit: false,
            inherited_token: None,
            context: git_context(),
        },
        &mut |token| {
            seen.push(token.map(str::to_string));
            return Ok(GitStatus { code: Some(0) });
        },
        &mut || return Ok(identity.clone()),
        &mut Vec::new(),
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0);
    assert_eq!(seen, vec![None]);
    remove(root.as_path());
}

/// A valid inherited lease forwards without taking the lock again, while a stale token is not trusted.
#[cfg(unix)]
#[test]
fn inherited_lease_forwards_without_settlement() {
    let root: PathBuf = fixture("lifecycle-inherited");
    let (main, common) = source(root.as_path());
    let identity: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: common.clone(),
        git_dir: common.clone(),
        worktree_root: main.clone(),
    };
    let held = crate::worktree_copy_lock::try_acquire_worktree_copy_lock(common.as_path())
        .expect("attempt")
        .expect("free");
    let token: String = held.lease_token().to_string();
    let mut seen: Vec<Option<String>> = Vec::new();
    // A valid inherited token would wait forever if the lock were taken again, so this returns only if the lock is skipped.
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity: identity.clone(),
            effective_cwd: main.clone(),
            creates_or_moves: true,
            opted_out: false,
            short_circuit: false,
            inherited_token: Some(token),
            context: git_context(),
        },
        &mut |lease| {
            seen.push(lease.map(str::to_string));
            return Ok(GitStatus { code: Some(0) });
        },
        &mut || return Ok(identity.clone()),
        &mut Vec::new(),
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0);
    assert_eq!(seen, vec![None]);
    held.release().expect("released");
    remove(root.as_path());
}

/// The repository path of the generated rules file that sits in the repository root's own `.cache`.
const RULES_PATH: &str = ".cache/forbidden-strings.rules.txt";

/// A main checkout whose ignored state has the shapes the rule must tell apart, returned with its canonical root and common
/// directory: installed dependencies with a pnpm-style link and a hard-linked store file, build output with a cache inside it,
/// caches, run records, a local file, a separate checkout and a registered worktree nested in an ignored directory.
#[cfg(unix)]
fn main_with_state(root: &Path) -> (PathBuf, PathBuf) {
    let main: PathBuf = repository(root, "main");
    std::fs::write(
        main.join(".gitignore"),
        b".cache/\nnode_modules/\ntarget/\n.pnpm-store/\n.claude/\nmutants.out/\nclone/\n*.local.*\n",
    )
    .expect("gitignore");
    let write = |path: &str, bytes: &[u8]| {
        let full: PathBuf = main.join(path);
        std::fs::create_dir_all(full.parent().expect("parent")).expect("directories");
        std::fs::write(full, bytes).expect("file");
    };
    write(RULES_PATH, b"rules");
    write(".cache/test-scratch/keep.txt", b"scratch");
    write("node_modules/.pnpm/dep@1.0.0/node_modules/dep/index.js", b"module");
    write("node_modules/.pnpm/dep@1.0.0/node_modules/dep/data.bin", b"data");
    write("node_modules/.pnpm-store/v11/files/ab", b"stored");
    write("node_modules/.cache/jiti/x.js", b"jiti");
    write("node_modules/.monochromatic/2026-11-01/log.jsonl", b"log");
    write("package/a/node_modules/.monochromatic/log.jsonl", b"log");
    write("package/a/node_modules/.bin/tool", b"#!/bin/sh\n");
    write("package/a/.cache/typescript/tsbuildinfo", b"cache");
    write("target/debug/deps/libx.rlib", b"rlib");
    write("target/debug/.fingerprint/x/fp", b"fp");
    write("target/debug/incremental/x/state", b"incremental");
    write("target/verification/run/out.txt", b"record");
    write("mutants.out/outcomes.json", b"{}");
    write(".pnpm-store/v11/files/cd", b"store");
    write("mise.local.toml", b"[env]\n");
    write("clone/.git/HEAD", b"ref: refs/heads/main\n");
    write("clone/file.txt", b"another checkout");
    // The pnpm layout: a package link to the virtual store, relative, and a store file with a second hard link.
    std::os::unix::fs::symlink(
        ".pnpm/dep@1.0.0/node_modules/dep",
        main.join("node_modules/dep"),
    )
    .expect("link");
    std::os::unix::fs::symlink(
        "../../../node_modules/.pnpm/dep@1.0.0/node_modules/dep",
        main.join("package/a/node_modules/dep"),
    )
    .expect("package link");
    std::fs::hard_link(
        main.join("node_modules/.pnpm/dep@1.0.0/node_modules/dep/data.bin"),
        main.join("target/debug/deps/data.bin"),
    )
    .expect("hard link");
    git(main.as_path(), &["add", ".gitignore"]);
    git(main.as_path(), &["commit", "--message", "ignore"]);
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "nested", ".claude/worktrees/nested"]);
    write(".claude/worktrees/nested/inside.txt", b"nested worktree file");
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("common");
    return (std::fs::canonicalize(main).expect("canonical main"), common);
}

/// Runs `git worktree add` for a new `topic` worktree from the main checkout through the lifecycle.
fn add_from_main(root: &Path, main: &Path, common: &Path, flags: (bool, bool)) -> (Settlement, String, PathBuf) {
    let topic: PathBuf = root.join("topic");
    let identity: WorktreeIdentity = main_identity(main, common);
    let args: Vec<String> = vec![
        String::from("worktree"),
        String::from("add"),
        String::from("--quiet"),
        String::from("-b"),
        String::from("topic"),
        topic.to_string_lossy().into_owned(),
    ];
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity: identity.clone(),
            effective_cwd: main.to_path_buf(),
            creates_or_moves: flags.0,
            opted_out: flags.1,
            short_circuit: false,
            inherited_token: None,
            context: git_context(),
        },
        &mut |_token| return Ok(run_real(main, args.as_slice())),
        &mut || return Ok(identity.clone()),
        &mut stderr,
    )
    .expect("settled");
    return (settlement, String::from_utf8(stderr).expect("utf8"), topic);
}

/// Nothing of the copy stays behind: no lock, no pending journal and no private stage beside the worktree.
fn assert_nothing_left(root: &Path, common: &Path) {
    assert!(
        !ensure_journal_root(common)
            .expect("root")
            .join(SETTLEMENT_LOCK_NAME)
            .exists()
    );
    assert!(read_pending_journals(common).expect("pending").is_empty());
    let leftovers: Vec<String> = std::fs::read_dir(root)
        .expect("listing")
        .map(|entry| return entry.expect("entry").file_name().to_string_lossy().into_owned())
        .filter(|name| return name.starts_with(crate::worktree_copy_selection::STAGE_PREFIX))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

/// A worktree added from the main checkout receives the dependencies and builds it needs, links and all.
#[cfg(unix)]
#[test]
fn main_checkout_source_carries_dependencies_and_builds() {
    let root: PathBuf = fixture("lifecycle-main-carries");
    let (main, common) = main_with_state(root.as_path());
    let (settlement, stderr, topic) = add_from_main(root.as_path(), main.as_path(), common.as_path(), (true, false));
    assert_eq!(exit_code(&settlement), 0, "{stderr} {:?}", settlement.copy_failure);
    assert!(settlement.copy_failure.is_none());
    let dependency: &str = "node_modules/.pnpm/dep@1.0.0/node_modules/dep/index.js";
    assert_eq!(std::fs::read(topic.join(dependency)).expect("root dependency"), b"module");
    assert_eq!(
        std::fs::read_link(topic.join("node_modules/dep")).expect("root link"),
        PathBuf::from(".pnpm/dep@1.0.0/node_modules/dep"),
        "a link into the virtual store stays a link with the same target"
    );
    assert_eq!(
        std::fs::read_link(topic.join("package/a/node_modules/dep")).expect("package link"),
        PathBuf::from("../../../node_modules/.pnpm/dep@1.0.0/node_modules/dep"),
    );
    assert_eq!(std::fs::read(topic.join("package/a/node_modules/dep/index.js")).expect("through the link"), b"module");
    assert!(topic.join("package/a/node_modules/.bin/tool").is_file());
    assert_eq!(std::fs::read(topic.join("target/debug/deps/libx.rlib")).expect("build"), b"rlib");
    assert!(topic.join("target/debug/.fingerprint/x/fp").is_file());
    assert_eq!(std::fs::read(topic.join("target/debug/deps/data.bin")).expect("hard linked file"), b"data");
    assert_eq!(std::fs::read(topic.join("node_modules/.pnpm/dep@1.0.0/node_modules/dep/data.bin")).expect("its twin"), b"data");
    assert_eq!(std::fs::read(topic.join(RULES_PATH)).expect("the root cache"), b"rules");
    assert!(topic.join(".cache/test-scratch/keep.txt").is_file());
    assert!(topic.join("mise.local.toml").is_file(), "local files are carried");
    assert!(
        stderr.starts_with("cli-git: copied ")
            && stderr.contains(quote(main.to_string_lossy().as_ref()).as_str())
            && stderr.ends_with(" into 1 new worktree.\n"),
        "{stderr}"
    );
    assert_nothing_left(root.as_path(), common.as_path());
    remove(root.as_path());
}

/// A worktree added from the main checkout never receives its nested worktrees, another checkout, caches or run records.
#[cfg(unix)]
#[test]
fn main_checkout_source_leaves_out_checkouts_caches_and_run_records() {
    let root: PathBuf = fixture("lifecycle-main-leaves-out");
    let (main, common) = main_with_state(root.as_path());
    let (settlement, stderr, topic) = add_from_main(root.as_path(), main.as_path(), common.as_path(), (true, false));
    assert_eq!(exit_code(&settlement), 0, "{stderr} {:?}", settlement.copy_failure);
    for absent in [
        ".claude/worktrees/nested/inside.txt",
        "clone",
        "node_modules/.pnpm-store",
        "node_modules/.cache",
        "node_modules/.monochromatic",
        "package/a/node_modules/.monochromatic",
        "package/a/.cache",
        "target/debug/incremental",
        "target/verification",
        "mutants.out",
        ".pnpm-store",
    ] {
        assert!(std::fs::symlink_metadata(topic.join(absent)).is_err(), "{absent} must not be carried");
    }
    assert_nothing_left(root.as_path(), common.as_path());
    remove(root.as_path());
}

/// A file the user cannot read is left out with one notice line, and the add still succeeds.
#[cfg(unix)]
#[test]
fn main_checkout_source_skips_unreadable_entries_with_a_notice() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("lifecycle-main-unreadable");
    let (main, common) = main_with_state(root.as_path());
    let hidden: PathBuf = main.join("node_modules/.pnpm/dep@1.0.0/node_modules/dep/secret.bin");
    std::fs::write(hidden.as_path(), b"secret").expect("secret");
    std::fs::set_permissions(hidden.as_path(), std::fs::Permissions::from_mode(0o000)).expect("mode");
    // A process with the privilege to read anything cannot show the case, so the control proves the file is unreadable.
    if std::fs::File::open(hidden.as_path()).is_ok() {
        remove(root.as_path());
        return;
    }
    let (settlement, stderr, topic) = add_from_main(root.as_path(), main.as_path(), common.as_path(), (true, false));
    assert_eq!(exit_code(&settlement), 0, "{stderr} {:?}", settlement.copy_failure);
    assert!(settlement.copy_failure.is_none());
    assert!(!topic.join("node_modules/.pnpm/dep@1.0.0/node_modules/dep/secret.bin").exists());
    assert!(topic.join("node_modules/.pnpm/dep@1.0.0/node_modules/dep/index.js").is_file());
    assert!(stderr.contains("cli-git: left out 1 unreadable ignored entry from"), "{stderr}");
    assert!(stderr.contains("secret.bin"), "{stderr}");
    assert_eq!(stderr.matches("left out").count(), 1, "one notice line: {stderr}");
    assert_nothing_left(root.as_path(), common.as_path());
    std::fs::set_permissions(hidden.as_path(), std::fs::Permissions::from_mode(0o600)).expect("restore");
    remove(root.as_path());
}

/// The opt-out flag leaves the main checkout's ignored state behind and prints nothing.
#[cfg(unix)]
#[test]
fn main_checkout_source_honors_the_opt_out() {
    let root: PathBuf = fixture("lifecycle-main-opt-out");
    let (main, common) = main_with_state(root.as_path());
    let (settlement, stderr, topic) = add_from_main(root.as_path(), main.as_path(), common.as_path(), (true, true));
    assert_eq!(exit_code(&settlement), 0);
    assert!(stderr.is_empty(), "{stderr}");
    assert!(topic.exists(), "Git itself still created the worktree");
    assert!(!topic.join("node_modules").exists());
    assert!(!topic.join(RULES_PATH).exists());
    remove(root.as_path());
}

/// A command the caller reports as creating no worktree is never settled, even from a main checkout with ignored state.
#[cfg(unix)]
#[test]
fn main_checkout_source_ignores_commands_that_create_no_worktree() {
    let root: PathBuf = fixture("lifecycle-main-not-creating");
    let (main, common) = main_with_state(root.as_path());
    let (settlement, stderr, topic) = add_from_main(root.as_path(), main.as_path(), common.as_path(), (false, false));
    assert_eq!(exit_code(&settlement), 0);
    assert!(stderr.is_empty(), "{stderr}");
    assert!(topic.exists(), "the runner still created the worktree");
    assert!(!topic.join("node_modules").exists());
    remove(root.as_path());
}
