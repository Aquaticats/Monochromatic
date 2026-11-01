//! Lifecycle checks against real Git: applicability, bypasses, settlement, the summary line and exit statuses.

use super::*;
use crate::test_support::{fixture, git, git_context, git_output, remove, repository};
use crate::service_copy::tests::DirectRequester;
use crate::worktree_copy_journal::{journal_root, read_pending_journals};
use crate::worktree_copy_request::{CopyRequest, requests_exist};
use crate::worktree_copy_selection::CopySelection;
use std::process::Output;

/// The requester that is the service in the same process.
fn direct() -> DirectRequester {
    return DirectRequester { context: git_context(), wakes: 0 };
}

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

/// A worktree added from a linked worktree gets its ignored files, the summary line, and leaves no request behind.
#[cfg(unix)]
#[test]
fn added_worktree_receives_ignored_state_through_the_service_and_leaves_no_request() {
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
            added_path: None,
            context: git_context(),
            main_checkout_paths: Vec::new(),
        },
        &mut || return Ok(run_real(source_canonical.as_path(), args.as_slice())),
        &mut direct(),
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
    assert!(!requests_exist(common.as_path()).expect("requests"));
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
            added_path: None,
            context: git_context(),
            main_checkout_paths: Vec::new(),
        },
        &mut || {
            seen.push(None);
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
        &mut direct(),
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0);
    assert_eq!(
        seen,
        vec![None],
        "the runner is called once for a skipped run"
    );
    assert!(!topic.join("cache").exists());
    assert!(stderr.is_empty());
    remove(root.as_path());
}

/// The main worktree is not applicable: the command runs without any settlement.
#[test]
fn main_worktree_invocation_is_not_applicable() {
    let root: PathBuf = fixture("lifecycle-main");
    let (main, common) = source(root.as_path());
    let identity: WorktreeIdentity = main_identity(main.as_path(), common.as_path());
    let mut seen: Vec<Option<String>> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity: identity.clone(),
            effective_cwd: main.clone(),
            creates_or_moves: true,
            opted_out: false,
            short_circuit: false,
            added_path: None,
            context: git_context(),
            main_checkout_paths: Vec::new(),
        },
        &mut || {
            seen.push(None);
            return Ok(GitStatus { code: Some(7) });
        },
        &mut direct(),
        &mut Vec::new(),
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 7);
    assert_eq!(seen, vec![None]);
    assert!(!requests_exist(common.as_path()).expect("requests"));
    remove(root.as_path());
}

/// A command that creates no worktrees forwards after recovering what is pending, without a copy.
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
            added_path: None,
            context: git_context(),
            main_checkout_paths: Vec::new(),
        },
        &mut || {
            seen.push(None);
            return Ok(GitStatus { code: Some(0) });
        },
        &mut direct(),
        &mut Vec::new(),
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0);
    assert_eq!(seen, vec![None]);
    remove(root.as_path());
}

/// A linked worktree identity for a source at `root` whose administrative directory is `admin`.
fn linked_identity(root: &Path, common: &Path, admin: &Path) -> WorktreeIdentity {
    return WorktreeIdentity::LinkedWorktree {
        common_dir: common.to_path_buf(),
        git_dir: admin.to_path_buf(),
        worktree_root: root.to_path_buf(),
    };
}

/// A creating command run from a linked worktree that Git is still creating (a filter or hook of an enclosing
///  `git worktree add`) forwards without observing, recovering or copying: the enclosing command copies.
#[cfg(unix)]
#[test]
fn a_creating_command_inside_an_unfinished_source_only_forwards() {
    let root: PathBuf = fixture("lifecycle-unfinished-source");
    let (main, common) = source(root.as_path());
    let admin: PathBuf = common.join("worktrees").join("enclosing");
    std::fs::create_dir_all(admin.as_path()).expect("administrative directory");
    std::fs::write(admin.join("locked"), b"initializing").expect("Git's mark of a worktree being created");
    let mut ran: usize = 0;
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity: linked_identity(main.as_path(), common.as_path(), admin.as_path()),
            effective_cwd: main.clone(),
            creates_or_moves: true,
            opted_out: false,
            short_circuit: false,
            added_path: None,
            context: git_context(),
            main_checkout_paths: Vec::new(),
        },
        &mut || {
            ran += 1;
            return Ok(GitStatus { code: Some(0) });
        },
        &mut direct(),
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0);
    assert_eq!(ran, 1);
    assert!(stderr.is_empty());
    assert!(!journal_root(common.as_path()).exists(), "nothing was observed or created");
    remove(root.as_path());
}

/// A command that creates nothing nudges the service only when a copy is pending, runs its own Git at once, and never
///  waits for the copy.
#[cfg(unix)]
#[test]
fn a_command_that_creates_nothing_nudges_the_service_only_when_a_copy_is_pending() {
    let root: PathBuf = fixture("lifecycle-wake");
    let (main, common) = source(root.as_path());
    let identity: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: common.clone(),
        git_dir: common.clone(),
        worktree_root: main.clone(),
    };
    let mut requester: DirectRequester = direct();
    let mut ran: usize = 0;
    let mut run = |requester: &mut DirectRequester| -> Settlement {
        return run_with_worktree_copy(
            &Invocation {
                identity: identity.clone(),
                effective_cwd: main.clone(),
                creates_or_moves: false,
                opted_out: false,
                short_circuit: false,
                added_path: None,
                context: git_context(),
                main_checkout_paths: Vec::new(),
            },
            &mut || {
                ran += 1;
                return Ok(GitStatus { code: Some(0) });
            },
            requester,
            &mut Vec::new(),
        )
        .expect("settled");
    };
    assert_eq!(exit_code(&run(&mut requester)), 0);
    assert_eq!(requester.wakes, 0, "nothing was pending");
    // A copy a killed wrapper or service left: a request on disk.
    let topic: PathBuf = root.join("topic");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "topic", topic.to_str().expect("utf8")]);
    let request: CopyRequest = CopyRequest {
        id: crate::worktree_copy_request::new_request_id().expect("identity"),
        admin_id: String::from("topic"),
        registration: String::from("token"),
        destination_root: std::fs::canonicalize(topic).expect("canonical"),
        source_root: main.clone(),
        selection: CopySelection::Ignored,
        tries: 0,
    };
    crate::worktree_copy_request::write_request(common.as_path(), &request).expect("written");
    assert_eq!(exit_code(&run(&mut requester)), 0);
    assert_eq!(requester.wakes, 1, "a pending copy is the service's to finish");
    assert_eq!(ran, 2, "the command ran its own Git both times");
    remove(root.as_path());
}

/// A worktree this command asked for with `--lock` is filled, and a locked worktree another command is creating is not:
///  Git's lock decides, not the text in it.
#[cfg(unix)]
#[test]
fn a_locked_worktree_this_command_added_is_copied_and_a_locked_peer_is_not() {
    let root: PathBuf = fixture("lifecycle-locked");
    let (main, common) = source(root.as_path());
    let topic: PathBuf = root.join("topic");
    let peer: PathBuf = root.join("peer");
    let identity: WorktreeIdentity = main_identity(main.as_path(), common.as_path());
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity,
            effective_cwd: main.clone(),
            creates_or_moves: true,
            opted_out: false,
            short_circuit: false,
            added_path: Some(topic.clone()),
            context: git_context(),
            main_checkout_paths: vec![String::from("cache/data")],
        },
        // Two registrations appear while Git runs: the one asked for, locked by `--lock` with the sentinel text itself,
        // and a peer's, locked with whatever text its language gives.
        &mut || {
            git(main.as_path(), &["worktree", "add", "--quiet", "--lock", "--reason", "initializing", "-b", "topic", topic.to_str().expect("utf8")]);
            git(main.as_path(), &["worktree", "add", "--quiet", "--lock", "--reason", "initialisation", "-b", "peer", peer.to_str().expect("utf8")]);
            return Ok(GitStatus { code: Some(0) });
        },
        &mut direct(),
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0, "{}", String::from_utf8_lossy(stderr.as_slice()));
    assert!(topic.join("cache/data").exists(), "the worktree this command asked for was not filled");
    assert!(!peer.join("cache").exists(), "a locked peer was filled by a command that did not create it");
    assert!(!requests_exist(common.as_path()).expect("requests"));
    remove(root.as_path());
}

/// The path of the configured rules file, as the caller supplies it for a main-checkout source.
const RULES_PATH: &str = ".cache/forbidden-strings.rules.txt";

/// A main checkout that ignores its generated cache and a second ignored directory,
/// returned with its canonical root and common directory.
/// The cache holds the rules file only when `with_rules` is set.
fn main_with_cache(root: &Path, with_rules: bool) -> (PathBuf, PathBuf) {
    let main: PathBuf = repository(root, "main");
    std::fs::write(main.join(".gitignore"), b".cache/\ncache/\n").expect("gitignore");
    std::fs::create_dir_all(main.join("cache")).expect("cache");
    std::fs::write(main.join("cache/data"), b"payload").expect("data");
    std::fs::create_dir_all(main.join(".cache")).expect("generated cache");
    std::fs::write(main.join(".cache/other.txt"), b"other").expect("other cache file");
    if with_rules {
        std::fs::write(main.join(RULES_PATH), b"rules").expect("rules");
    }
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("common");
    return (std::fs::canonicalize(main).expect("canonical main"), common);
}

/// Runs `git worktree add` for a new `topic` worktree from the main checkout through the lifecycle.
fn add_from_main(
    root: &Path,
    main: &Path,
    common: &Path,
    flags: (bool, bool),
    named: &[&str],
) -> (Settlement, String, PathBuf) {
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
            added_path: None,
            context: git_context(),
            main_checkout_paths: named.iter().map(|path| return String::from(*path)).collect(),
        },
        &mut || return Ok(run_real(main, args.as_slice())),
        &mut direct(),
        &mut stderr,
    )
    .expect("settled");
    return (settlement, String::from_utf8(stderr).expect("utf8"), topic);
}

/// Nothing of the copy stays behind: no request, no pending journal and no private stage beside the worktree.
fn assert_nothing_left(root: &Path, common: &Path) {
    assert!(!requests_exist(common).expect("requests"));
    assert!(read_pending_journals(common).expect("pending").is_empty());
    let leftovers: Vec<String> = std::fs::read_dir(root)
        .expect("listing")
        .map(|entry| return entry.expect("entry").file_name().to_string_lossy().into_owned())
        .filter(|name| return name.starts_with(crate::worktree_copy_selection::STAGE_PREFIX))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

/// A worktree added from the main checkout receives the named rules file and nothing else the checkout ignores.
#[cfg(unix)]
#[test]
fn main_checkout_source_copies_only_the_named_rules_file() {
    let root: PathBuf = fixture("lifecycle-main-rules");
    let (main, common) = main_with_cache(root.as_path(), true);
    let (settlement, stderr, topic) = add_from_main(
        root.as_path(),
        main.as_path(),
        common.as_path(),
        (true, false),
        &[RULES_PATH],
    );
    assert_eq!(exit_code(&settlement), 0);
    assert_eq!(std::fs::read(topic.join(RULES_PATH)).expect("copied"), b"rules");
    assert!(!topic.join("cache").exists(), "other ignored roots stay behind");
    assert!(
        !topic.join(".cache/other.txt").exists(),
        "siblings of the rules file stay behind"
    );
    assert!(
        stderr.starts_with("cli-git: copied ")
            && stderr.contains(quote(main.to_string_lossy().as_ref()).as_str())
            && stderr.ends_with(" into 1 new worktree.\n"),
        "{stderr}"
    );
    assert_nothing_left(root.as_path(), common.as_path());
    remove(root.as_path());
}

/// A main checkout without the rules file copies nothing, creates no rules file, and leaves nothing behind.
#[cfg(unix)]
#[test]
fn main_checkout_source_without_the_rules_file_copies_nothing() {
    let root: PathBuf = fixture("lifecycle-main-no-rules");
    let (main, common) = main_with_cache(root.as_path(), false);
    let (settlement, stderr, topic) = add_from_main(
        root.as_path(),
        main.as_path(),
        common.as_path(),
        (true, false),
        &[RULES_PATH],
    );
    assert_eq!(exit_code(&settlement), 0);
    assert!(settlement.copy_failure.is_none());
    assert!(!topic.join(".cache").exists());
    assert!(!topic.join("cache").exists());
    assert!(stderr.contains("copied 0 ignored filesystem entries"), "{stderr}");
    assert_nothing_left(root.as_path(), common.as_path());
    remove(root.as_path());
}

/// A rules file the checkout tracks, or that no ignore rule covers, is the checkout's own content and is not copied.
#[cfg(unix)]
#[test]
fn main_checkout_source_does_not_copy_a_rules_file_git_does_not_ignore() {
    let root: PathBuf = fixture("lifecycle-main-unignored");
    let (main, common) = main_with_cache(root.as_path(), true);
    std::fs::write(main.join("custom-rules.txt"), b"custom").expect("custom");
    let (settlement, _stderr, topic) = add_from_main(
        root.as_path(),
        main.as_path(),
        common.as_path(),
        (true, false),
        &["custom-rules.txt"],
    );
    assert_eq!(exit_code(&settlement), 0);
    assert!(!topic.join("custom-rules.txt").exists());
    assert!(!topic.join(RULES_PATH).exists());
    assert_nothing_left(root.as_path(), common.as_path());
    remove(root.as_path());
}

/// The opt-out flag leaves the main checkout's rules file behind and prints nothing.
#[cfg(unix)]
#[test]
fn main_checkout_source_honors_the_opt_out() {
    let root: PathBuf = fixture("lifecycle-main-opt-out");
    let (main, common) = main_with_cache(root.as_path(), true);
    let (settlement, stderr, topic) = add_from_main(
        root.as_path(),
        main.as_path(),
        common.as_path(),
        (true, true),
        &[RULES_PATH],
    );
    assert_eq!(exit_code(&settlement), 0);
    assert!(stderr.is_empty(), "{stderr}");
    assert!(topic.exists(), "Git itself still created the worktree");
    assert!(!topic.join(RULES_PATH).exists());
    remove(root.as_path());
}

/// A command the caller reports as creating no worktree is never settled, even from a main checkout with a rules file.
#[cfg(unix)]
#[test]
fn main_checkout_source_ignores_commands_that_create_no_worktree() {
    let root: PathBuf = fixture("lifecycle-main-not-creating");
    let (main, common) = main_with_cache(root.as_path(), true);
    let (settlement, stderr, topic) = add_from_main(
        root.as_path(),
        main.as_path(),
        common.as_path(),
        (false, false),
        &[RULES_PATH],
    );
    assert_eq!(exit_code(&settlement), 0);
    assert!(stderr.is_empty(), "{stderr}");
    assert!(topic.exists(), "the runner still created the worktree");
    assert!(!topic.join(RULES_PATH).exists());
    remove(root.as_path());
}
