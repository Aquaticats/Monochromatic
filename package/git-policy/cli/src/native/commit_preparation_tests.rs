//! Controls for native preparation: real state unchanged, the shadow seen by hooks,
//!  hook failures propagated with their exit code, and no shadow left behind.
//!
//!  Hooks are configured in the repository (`hook.<name>.event` with `hook.<name>.command`), which is what Git runs
//!  natively inside the preparation; the tests pass a plan whose lock lives in the transaction directory.

use super::*;
use crate::capture_invocation::{capture_invocation_base, capture_invocation_layout};
use crate::capture_order::CaptureIndexes;
use crate::commit_index::{IndexSelection, initialize_commit_index, write_private_tree};
use crate::shadow_create::create_shadow_repository;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::transaction_journal::TransactionMode;
use crate::transaction_journal_parse::parse_prepared;
use crate::candidate_prediction::CandidateRequest;
use crate::command_test_support::os_arguments;
use crate::commit_convergence::{CommitConvergence, CommitConvergenceInput, converge_commit_policies};
use crate::config_schema::PolicyConfig;
use crate::policy_checks::shipped_checks;
use crate::policy_content::{Correction, LifecycleContent};
use crate::policy_convergence::Convergence;
use crate::policy_engine::StageRequest;
use crate::policy_trigger::Trigger;
use crate::private_index_stage::{PrivateStage, stage_correction};
use crate::candidate_object::CandidateMode;
use crate::repository_facts::git_facts;
use crate::signal_relay::SignalGuardActive;
use crate::test_support::REAL_GIT;
use crate::wrapper_controls::no_controls;
use std::ffi::OsString;
use std::path::PathBuf;
use std::rc::Rc;
use crate::commit_preparation::PREPARED_FILENAME;

/// A fixed transaction ID of the shape the registry accepts.
const TRANSACTION: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";

/// Everything one preparation fixture produced.
struct Fixture {
    /// Repository the transaction selects.
    repo: PathBuf,
    /// Shadow repository of the transaction.
    shadow: PathBuf,
    /// Transaction directory, which holds the journal and the hooks.
    transaction: PathBuf,
    /// Invocation capture.
    capture: InvocationCapture,
    /// Private commit index, staged for `a.txt` only.
    commit_index: PathBuf,
    /// Intended tree of the private index.
    intended_tree: String,
}

/// Define the hook `under-test` for `event`: the script is written under `root` and the repository's configuration
///  names it as the hook's command, which is what Git runs natively inside the preparation.
fn configure_hook(root: &Path, repo: &Path, event: &str, script: &str) {
    let scripts: PathBuf = root.join("hook-scripts");
    std::fs::create_dir_all(&scripts).expect("hook scripts");
    let path: PathBuf = scripts.join(event);
    std::fs::write(&path, script).expect("hook");
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("executable");
    git(repo, &["config", "hook.under-test.event", event]);
    git(repo, &["config", "hook.under-test.command", path.to_str().expect("UTF-8")]);
}

/// A repository with `a.txt` committed, the change to `a.txt` staged privately, and a shadow ready to prepare.
fn prepared_fixture(root: &Path, hook_script: Option<(&str, &str)>) -> Fixture {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    git(repo.as_path(), &["remote", "add", "origin", "https://example.invalid/repo.git"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["update-ref", "refs/remotes/origin/main", base.as_str()]);
    git(repo.as_path(), &["config", "branch.main.remote", "origin"]);
    git(repo.as_path(), &["config", "branch.main.merge", "refs/heads/main"]);
    std::fs::write(repo.join(".git").join("branch.config"), "[x]\n\tv = onbranch\n").expect("include");
    std::fs::OpenOptions::new()
        .append(true)
        .open(repo.join(".git").join("config"))
        .and_then(|mut file| {
            use std::io::Write;
            return file.write_all(b"[includeIf \"onbranch:main\"]\n\tpath = branch.config\n");
        })
        .expect("include config");
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change a");
    let context = git_context();
    let layout = capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
    let capture: InvocationCapture = capture_invocation_base(&context, repo.as_path(), layout).expect("capture");
    let transaction: PathBuf = root.join(format!("transaction-{TRANSACTION}"));
    std::fs::create_dir_all(&transaction).expect("transaction");
    let shadow: PathBuf = create_shadow_repository(&context, repo.as_path(), &capture, TRANSACTION, transaction.as_path())
        .expect("shadow");
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: transaction.join("captured.index"),
        commit_index: transaction.join("commit.index"),
        object_directory: shadow.join("objects"),
    };
    let pathspecs: Vec<OsString> = vec![OsString::from("a.txt")];
    let selection: IndexSelection = IndexSelection {
        mode: TransactionMode::ExplicitPath,
        pathspecs: pathspecs.as_slice(),
        pathspec_file: None,
        pathspec_file_nul: false,
        stage_into_index: false,
        stage_tracked_changes: false,
        base_revision: base.as_str(),
    };
    initialize_commit_index(&context, repo.as_path(), capture.layout.real_index_path.as_path(), &indexes, &selection)
        .expect("private index");
    let intended_tree: String = write_private_tree(
        &context,
        repo.as_path(),
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
    )
    .expect("tree");
    if let Some((name, script)) = hook_script {
        configure_hook(root, repo.as_path(), name, script);
    }
    return Fixture {
        repo,
        shadow,
        transaction,
        capture,
        commit_index: indexes.commit_index,
        intended_tree,
    };
}

/// The hook plan of a fixture: configured hooks possible, no event disabled, the lock inside the transaction directory.
fn hook_plan(fixture: &Fixture, skip_hook_lock: bool) -> HookDispatchPlan {
    return HookDispatchPlan {
        disabled_events: Vec::new(),
        lease: String::from("a-lease"),
        hook_lock_path: fixture.transaction.join("hook.lock").display().to_string(),
        skip_hook_lock,
        named_hooks_possible: true,
    };
}

/// Runs the preparation of a fixture with the fixture's hook plan.
fn prepare(fixture: &Fixture, commit_args: &[&str]) -> Result<PreparedCommit, PreparationError> {
    let hooks: HookDispatchPlan = hook_plan(fixture, false);
    let commit_arguments: Vec<String> = commit_args.iter().map(|arg| { return String::from(*arg); }).collect();
    let request: PreparationRequest = PreparationRequest {
        capture: &fixture.capture,
        shadow: fixture.shadow.as_path(),
        hooks: &hooks,
        commit_index: fixture.commit_index.as_path(),
        global_args: &[],
        commit_args: commit_arguments.as_slice(),
    };
    return prepare_native_commit(
        &git_context(),
        fixture.repo.as_path(),
        &request,
        fixture.transaction.as_path(),
        fixture.intended_tree.as_str(),
        &PreparedPaths {
            committed_paths: vec![b"a.txt".to_vec()],
            ..PreparedPaths::default()
        },
    );
}

/// Preparation leaves the real refs, the real index and the worktree bytes unchanged,
///  and the prepared commit lives in the shadow with its journal record.
#[test]
fn preparation_leaves_real_refs_index_and_worktree_unchanged() {
    let root: PathBuf = fixture("commit-preparation-success");
    let fixture: Fixture = prepared_fixture(root.as_path(), None);
    let refs_before: String = git_text(fixture.repo.as_path(), &["for-each-ref"]);
    let index_before: Vec<u8> = std::fs::read(fixture.repo.join(".git").join("index")).expect("index");
    let head_before: String = git_text(fixture.repo.as_path(), &["rev-parse", "HEAD"]);
    let prepared: PreparedCommit = prepare(&fixture, &["-m", "message"]).expect("prepared");
    assert_eq!(git_text(fixture.repo.as_path(), &["for-each-ref"]), refs_before);
    assert_eq!(git_text(fixture.repo.as_path(), &["rev-parse", "HEAD"]), head_before);
    assert_eq!(std::fs::read(fixture.repo.join(".git").join("index")).expect("index"), index_before);
    assert_eq!(std::fs::read_to_string(fixture.repo.join("a.txt")).expect("worktree"), "changed\n");
    assert_eq!(prepared.tree_oid, fixture.intended_tree);
    assert!(!prepared.signed);
    assert_eq!(
        git_text(fixture.shadow.as_path(), &["rev-parse", "HEAD"]),
        prepared.oid
    );
    let record: String = std::fs::read_to_string(fixture.transaction.join(PREPARED_FILENAME)).expect("prepared.json");
    let parsed = parse_prepared(record.as_bytes()).expect("parse");
    assert_eq!(parsed.prepared_oid, prepared.oid);
    assert_eq!(parsed.intended_tree_oid, fixture.intended_tree);
    assert_eq!(parsed.shadow_path, fixture.shadow.display().to_string());
    assert_eq!(parsed.committed_paths, vec![b"a.txt".to_vec()]);
    remove(root.as_path());
}

/// A hook sees the shadow as the incumbent's hook sees it: `HEAD` at the base, the upstream,
///  the `onbranch:` include and the Git directory is the shadow itself.
#[cfg(unix)]
#[test]
fn a_hook_sees_the_shadow_head_upstream_include_and_git_dir() {
    let root: PathBuf = fixture("commit-preparation-hook-view");
    let out: PathBuf = root.join("out");
    std::fs::create_dir(&out).expect("out");
    let script: String = format!(
        "#!/bin/sh\n\
         git rev-parse HEAD > \"{o}/head\"\n\
         git rev-parse --abbrev-ref '@{{upstream}}' > \"{o}/upstream\"\n\
         git config --get x.v > \"{o}/onbranch\"\n\
         git rev-parse --git-dir > \"{o}/gitdir\"\n",
        o = out.display()
    );
    let fixture: Fixture = prepared_fixture(root.as_path(), Some(("pre-commit", script.as_str())));
    let base: String = git_text(fixture.repo.as_path(), &["rev-parse", "HEAD"]);
    prepare(&fixture, &["-m", "message"]).expect("prepared");
    assert_eq!(std::fs::read_to_string(out.join("head")).expect("head").trim(), base);
    assert_eq!(std::fs::read_to_string(out.join("upstream")).expect("upstream").trim(), "origin/main");
    assert_eq!(std::fs::read_to_string(out.join("onbranch")).expect("onbranch").trim(), "onbranch");
    assert_eq!(
        std::fs::read_to_string(out.join("gitdir")).expect("gitdir").trim(),
        fixture.shadow.display().to_string()
    );
    remove(root.as_path());
}

/// A failing `commit-msg` hook stops preparation with Git's exit code and leaves no shadow repository.
#[cfg(unix)]
#[test]
fn a_failing_commit_msg_hook_propagates_the_exit_code_and_removes_the_shadow() {
    let root: PathBuf = fixture("commit-preparation-hook-fails");
    let fixture: Fixture = prepared_fixture(root.as_path(), Some(("commit-msg", "#!/bin/sh\nexit 7\n")));
    let refs_before: String = git_text(fixture.repo.as_path(), &["for-each-ref"]);
    let error: PreparationError = prepare(&fixture, &["-m", "message"]).expect_err("hook fails");
    match error {
        PreparationError::NativeExit { exit_code } => assert_eq!(exit_code, 1),
        other => panic!("expected the native exit code, got {other}"),
    }
    assert!(!fixture.shadow.exists(), "the shadow must be removed");
    assert!(!fixture.transaction.join(PREPARED_FILENAME).exists());
    assert_eq!(git_text(fixture.repo.as_path(), &["for-each-ref"]), refs_before);
    remove(root.as_path());
}

/// A commit with nothing selected fails with Git's own nonzero code and leaves no shadow.
#[test]
fn a_nothing_to_commit_failure_removes_the_shadow() {
    let root: PathBuf = fixture("commit-preparation-nothing");
    let fixture: Fixture = prepared_fixture(root.as_path(), None);
    let commit_index_empty: PathBuf = fixture.transaction.join("empty.index");
    std::fs::copy(fixture.repo.join(".git").join("index"), &commit_index_empty).expect("index");
    let hooks: HookDispatchPlan = hook_plan(&fixture, false);
    let commit_arguments: Vec<String> = vec![String::from("-m"), String::from("message")];
    let request: PreparationRequest = PreparationRequest {
        capture: &fixture.capture,
        shadow: fixture.shadow.as_path(),
        hooks: &hooks,
        commit_index: commit_index_empty.as_path(),
        global_args: &[],
        commit_args: commit_arguments.as_slice(),
    };
    let error: PreparationError = prepare_native_commit(
        &git_context(),
        fixture.repo.as_path(),
        &request,
        fixture.transaction.as_path(),
        fixture.intended_tree.as_str(),
        &PreparedPaths::default(),
    )
    .expect_err("nothing to commit");
    assert!(matches!(error, PreparationError::NativeExit { exit_code } if exit_code != 0));
    assert!(!fixture.shadow.exists());
    remove(root.as_path());
}

/// The native command names the shadow, turns traditional hooks off, and names no work tree and no `post-commit` switch.
#[test]
fn native_commit_arguments_name_the_shadow_and_turn_traditional_hooks_off() {
    let capture_root: PathBuf = fixture("commit-preparation-arguments");
    let repo: PathBuf = repository(capture_root.as_path(), "repo");
    let context = git_context();
    let layout = capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
    let capture: InvocationCapture = capture_invocation_base(&context, repo.as_path(), layout).expect("capture");
    let hooks: HookDispatchPlan = HookDispatchPlan {
        disabled_events: vec![String::from("commit-msg")],
        lease: String::from("a-lease"),
        hook_lock_path: String::from("/unused/hook.lock"),
        skip_hook_lock: true,
        named_hooks_possible: true,
    };
    let commit_arguments: Vec<String> = vec![String::from("-m"), String::from("x")];
    let global: Vec<String> = vec![String::from("-c"), String::from("a.b=1")];
    let request: PreparationRequest = PreparationRequest {
        capture: &capture,
        shadow: Path::new("/s"),
        hooks: &hooks,
        commit_index: Path::new("/i"),
        global_args: global.as_slice(),
        commit_args: commit_arguments.as_slice(),
    };
    assert_eq!(
        native_commit_arguments(&request),
        vec![
            "-c",
            "a.b=1",
            "--git-dir=/s",
            "-c",
            "core.hooksPath=/dev/null/cli-git-no-hooks",
            "commit",
            "-m",
            "x",
        ]
    );
    remove(capture_root.as_path());
}

/// The prepared commit's signature is read from its header block only, not from the message.
#[test]
fn signature_is_read_from_the_header_block_only() {
    let root: PathBuf = fixture("commit-preparation-signed");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir(&shadow).expect("shadow");
    git(&shadow, &["init", "--bare", "--quiet", "--template="]);
    std::fs::write(shadow.join("objects").join("info").join("alternates"), format!("{}\n", repo.join(".git").join("objects").display())).expect("alternates");
    git(&shadow, &["update-ref", "refs/heads/main", head.as_str()]);
    git(&shadow, &["symbolic-ref", "HEAD", "refs/heads/main"]);
    let prepared: PreparedCommit = read_prepared_commit(&git_context(), shadow.as_path()).expect("read");
    assert_eq!(prepared.oid, head);
    assert!(!prepared.signed);
    remove(root.as_path());
}

/// The shadow object store of a fixture, where the commit path writes its corrected blobs.
fn shadow_objects(fixture: &Fixture) -> PathBuf {
    return fixture.shadow.join("objects");
}

/// The exact bytes the prepared commit's tree holds for a path, read from the shadow repository.
fn shadow_text(fixture: &Fixture, commit: &str, path: &str) -> String {
    let shadow: String = fixture.shadow.display().to_string();
    let output: std::process::Output = git(
        fixture.repo.as_path(),
        &["--git-dir", shadow.as_str(), "show", format!("{commit}:{path}").as_str()],
    );
    return String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
}

/// Runs the commit path's convergence over the fixture's private commit index.
fn converge_fixture(fixture: &Fixture) -> CommitConvergence {
    let objects: PathBuf = shadow_objects(fixture);
    let overlay: Vec<(OsString, OsString)> = vec![
        (OsString::from("GIT_DIR"), fixture.repo.join(".git").into_os_string()),
        (OsString::from("GIT_WORK_TREE"), fixture.repo.as_os_str().to_os_string()),
        (OsString::from("GIT_INDEX_FILE"), fixture.commit_index.as_os_str().to_os_string()),
        (OsString::from("GIT_OBJECT_DIRECTORY"), objects.as_os_str().to_os_string()),
    ];
    let facts = git_facts(std::path::Path::new(REAL_GIT), &[], overlay.as_slice());
    let mut checks = shipped_checks(
        facts,
        os_arguments(&[]),
        LifecycleContent::Requested(CandidateRequest::Staged(vec![OsString::from("a.txt")])),
        Vec::<PathBuf>::new(),
    );
    let request: StageRequest = StageRequest {
        trigger: Trigger::PreForward,
        config: PolicyConfig::defaults(),
        controls: no_controls(),
        selected: Vec::new(),
    };
    return converge_commit_policies(CommitConvergenceInput {
        request: &request,
        checks: &mut checks,
        stage: PrivateStage {
            context: &git_context(),
            cwd: fixture.repo.as_path(),
            index: fixture.commit_index.as_path(),
            object_directory: Some(objects.as_path()),
        },
    });
}

/// The final-newline correction the commit path writes reaches the prepared commit's tree.
#[test]
fn the_final_newline_correction_reaches_the_prepared_commit() {
    let root: PathBuf = fixture("preparation-final-newline");
    let mut prepared_for: Fixture = prepared_fixture(root.as_path(), None);
    let objects: PathBuf = shadow_objects(&prepared_for);
    let uncorrected: Correction = Correction {
        path: b"a.txt".to_vec(),
        mode: CandidateMode::Regular,
        before: Rc::from(b"changed\n".as_slice()),
        after: Rc::from(b"changed".as_slice()),
    };
    let stage: PrivateStage<'_> = PrivateStage {
        context: &git_context(),
        cwd: prepared_for.repo.as_path(),
        index: prepared_for.commit_index.as_path(),
        object_directory: Some(objects.as_path()),
    };
    assert_eq!(stage_correction(&stage, &uncorrected), Ok(()), "the uncorrected bytes are staged");
    let outcome: CommitConvergence = converge_fixture(&prepared_for);
    assert_eq!(outcome.convergence, Convergence::Settled { changed_passes: 1 }, "{:?}", outcome.pass.events);
    prepared_for.intended_tree = write_private_tree(
        &git_context(),
        prepared_for.repo.as_path(),
        prepared_for.commit_index.as_path(),
        objects.as_path(),
    )
    .expect("the corrected tree");
    let prepared: PreparedCommit = prepare(&prepared_for, &["--quiet", "-m", "corrected"])
        .expect("the corrected commit prepares");
    assert_eq!(shadow_text(&prepared_for, prepared.oid.as_str(), "a.txt"), "changed\n");
    remove(root.as_path());
}

/// A pre-commit hook that re-stages a selected file changes the prepared commit, and the change is kept.
#[test]
fn a_pre_commit_hook_restaging_a_selected_file_is_kept_in_the_prepared_commit() {
    let root: PathBuf = fixture("preparation-hook-restage");
    let script: &str = "#!/bin/sh\nblob=$(printf 'reformatted\\n' | git hash-object -w --stdin) && git update-index --cacheinfo 100644,\"$blob\",a.txt\n";
    let hooked: Fixture = prepared_fixture(root.as_path(), Some(("pre-commit", script)));
    let prepared: PreparedCommit = prepare(&hooked, &["--quiet", "-m", "hooked"]).expect("the hooked commit prepares");
    assert_eq!(shadow_text(&hooked, prepared.oid.as_str(), "a.txt"), "reformatted\n");
    remove(root.as_path());
}

/// The wait for the guard ends with the first success, passes over only the "already active" failure, and gives up
///  after the limit with the failure itself. Every closure panics past its call cap, so a mutated guard that retries
///  too often fails the test at once instead of looping.
#[test]
fn the_guard_wait_retries_only_while_another_holder_is_active() {
    let delay = std::time::Duration::from_millis(1);
    let active = || { return std::io::Error::other(SignalGuardActive); };
    let mut calls: u32 = 0;
    let waited: std::io::Result<u32> = retry_while_active(
        || {
            calls += 1;
            assert!(calls <= 3, "the success on the third call ends the wait");
            if calls < 3 { return Err(active()); }
            return Ok(calls);
        },
        5,
        delay,
    );
    assert_eq!(waited.expect("success after two active failures"), 3);
    let mut other_calls: u32 = 0;
    let other: std::io::Result<u32> = retry_while_active(
        || {
            other_calls += 1;
            assert!(other_calls <= 1, "another failure is returned at once, never retried");
            return Err(std::io::Error::other("no handler could be installed"));
        },
        5,
        delay,
    );
    assert!(other.is_err());
    assert_eq!(other_calls, 1);
    let mut limited_calls: u32 = 0;
    let limited: std::io::Result<u32> = retry_while_active(
        || {
            limited_calls += 1;
            assert!(limited_calls <= 4, "the first try and three retries are all there is");
            return Err(active());
        },
        3,
        delay,
    );
    assert!(limited.is_err());
    assert_eq!(limited_calls, 4, "the first try and three retries");
    let mut zero_calls: u32 = 0;
    let zero: std::io::Result<u32> = retry_while_active(
        || {
            zero_calls += 1;
            assert!(zero_calls <= 1, "a limit of zero allows no retry");
            return Err(active());
        },
        0,
        delay,
    );
    assert!(zero.is_err());
    assert_eq!(zero_calls, 1);
}

/// Both signature header spellings mark the prepared commit as signed, and a signature-looking message line does not.
#[test]
fn either_signature_header_marks_the_commit_signed() {
    let root: PathBuf = fixture("commit-preparation-headers");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let tree: String = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let parent: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir(&shadow).expect("shadow");
    git(&shadow, &["init", "--bare", "--quiet", "--template="]);
    std::fs::write(
        shadow.join("objects").join("info").join("alternates"),
        format!("{}\n", repo.join(".git").join("objects").display()),
    )
    .expect("alternates");
    git(&shadow, &["symbolic-ref", "HEAD", "refs/heads/main"]);
    let commit_with = |extra: &str, message: &str| -> String {
        let raw: String = format!(
            "tree {}\nparent {}\nauthor A <a@example.invalid> 1 +0000\ncommitter A <a@example.invalid> 1 +0000\n{extra}\n{message}\n",
            tree.trim(),
            parent.trim()
        );
        let mut child = std::process::Command::new(REAL_GIT)
            .current_dir(&repo)
            .args(["hash-object", "-t", "commit", "-w", "--stdin"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("hash-object");
        std::io::Write::write_all(&mut child.stdin.take().expect("stdin"), raw.as_bytes()).expect("raw commit");
        return String::from_utf8_lossy(child.wait_with_output().expect("output").stdout.as_slice()).trim().to_string();
    };
    let signed_with = |extra: &str, message: &str| -> bool {
        let oid: String = commit_with(extra, message);
        git(&shadow, &["update-ref", "refs/heads/main", oid.as_str()]);
        return read_prepared_commit(&git_context(), shadow.as_path()).expect("read").signed;
    };
    assert!(signed_with("gpgsig -----BEGIN PGP SIGNATURE-----\n -----END PGP SIGNATURE-----\n", "m"));
    assert!(signed_with("gpgsig-sha256 -----BEGIN PGP SIGNATURE-----\n -----END PGP SIGNATURE-----\n", "m"));
    assert!(!signed_with("", "gpgsig in the message\n\ngpgsig-sha256 too"));
    remove(root.as_path());
}

/// Each preparation failure displays its own words.
#[test]
fn each_preparation_failure_displays_its_own_words() {
    assert_eq!(
        PreparationError::NativeExit { exit_code: 3 }.to_string(),
        "git commit exited 3 during private preparation; nothing landed."
    );
    assert_eq!(PreparationError::Git(GitFailure(String::from("git words"))).to_string(), "git words");
    assert_eq!(PreparationError::Write(String::from("write words")).to_string(), "write words");
}

/// The wait recognizes the occupied guard by its type: an error that only repeats the wording is final (issue 1346).
#[test]
fn an_error_with_the_guard_wording_but_not_its_type_is_not_retried() {
    let mut calls: u32 = 0;
    let waited: std::io::Result<u32> = retry_while_active(
        || {
            calls += 1;
            assert!(calls <= 1, "a look-alike error is returned at once");
            return Err(std::io::Error::other("a signal guard is already active in this process"));
        },
        5,
        std::time::Duration::from_millis(1),
    );
    assert!(waited.is_err());
    assert_eq!(calls, 1);
}
/// A hook script that records its arguments, the lease it inherited, whether the hook lock exists and the work tree
///  variable Git gave it.
#[cfg(unix)]
fn recording_script(record: &Path, lock: &Path) -> String {
    return format!(
        "#!/bin/sh\n{{ printf 'args=%s\\n' \"$*\"; printf 'lease=%s\\n' \"$CLI_GIT_PREPARATION_LEASE\"; \
         if [ -d '{}' ]; then printf 'locked=yes\\n'; else printf 'locked=no\\n'; fi; \
         printf 'work_tree=[%s]\\n' \"$GIT_WORK_TREE\"; }} >> '{}'\n",
        lock.display(),
        record.display()
    );
}

/// Run only the native commit of a fixture under `hooks`, and return its exit code.
#[cfg(unix)]
fn native_commit(fixture: &Fixture, hooks: &HookDispatchPlan) -> i32 {
    let commit_arguments: Vec<String> = ["--quiet", "-m", "message"].iter().map(|argument| { return String::from(*argument); }).collect();
    let request: PreparationRequest = PreparationRequest {
        capture: &fixture.capture,
        shadow: fixture.shadow.as_path(),
        hooks,
        commit_index: fixture.commit_index.as_path(),
        global_args: &[],
        commit_args: commit_arguments.as_slice(),
    };
    return run_native_commit(&git_context(), fixture.repo.as_path(), &request).expect("commit");
}

/// Git runs the configured `commit-msg` hook inside the preparation with its message file, the plan's lease, and the
///  hook lock held; the transaction's lock is gone once the preparation ends.
#[cfg(unix)]
#[test]
fn a_configured_hook_runs_inside_the_preparation_under_the_lock_and_the_lease() {
    let root: PathBuf = fixture("preparation-hook-lock");
    let record: PathBuf = root.join("record.txt");
    let hooked: Fixture = prepared_fixture(root.as_path(), None);
    let lock: PathBuf = hooked.transaction.join("hook.lock");
    configure_hook(root.as_path(), hooked.repo.as_path(), "commit-msg", recording_script(record.as_path(), lock.as_path()).as_str());
    assert_eq!(native_commit(&hooked, &hook_plan(&hooked, false)), 0);
    let seen: String = std::fs::read_to_string(&record).expect("the hook ran");
    assert!(seen.contains("args=") && seen.contains("COMMIT_EDITMSG"), "{seen}");
    assert!(seen.contains("lease=a-lease"), "{seen}");
    assert!(seen.contains("locked=yes"), "{seen}");
    assert!(seen.contains("work_tree=[]"), "Git leaves GIT_WORK_TREE unset for the hook: {seen}");
    assert!(!lock.exists(), "the lock is released when the native commit ends");
    remove(root.as_path());
}

/// A plan that skips the lock (concurrent commits, or a hook's own nested commit) holds none while the hook runs.
#[cfg(unix)]
#[test]
fn a_plan_that_skips_the_lock_holds_none_around_the_native_commit() {
    let root: PathBuf = fixture("preparation-hook-no-lock");
    let record: PathBuf = root.join("record.txt");
    let hooked: Fixture = prepared_fixture(root.as_path(), None);
    let lock: PathBuf = hooked.transaction.join("hook.lock");
    configure_hook(root.as_path(), hooked.repo.as_path(), "pre-commit", recording_script(record.as_path(), lock.as_path()).as_str());
    assert_eq!(native_commit(&hooked, &hook_plan(&hooked, true)), 0);
    let seen: String = std::fs::read_to_string(&record).expect("the hook ran");
    assert!(seen.contains("locked=no"), "{seen}");
    remove(root.as_path());
}

/// A plan whose configuration can define no hook takes no lock and the native commit still succeeds.
#[cfg(unix)]
#[test]
fn a_plan_without_possible_hooks_takes_no_lock() {
    let root: PathBuf = fixture("preparation-no-hooks-no-lock");
    let plain: Fixture = prepared_fixture(root.as_path(), None);
    let hooks: HookDispatchPlan = HookDispatchPlan { named_hooks_possible: false, ..hook_plan(&plain, false) };
    assert_eq!(native_commit(&plain, &hooks), 0);
    assert!(!plain.transaction.join("hook.lock").exists());
    remove(root.as_path());
}
