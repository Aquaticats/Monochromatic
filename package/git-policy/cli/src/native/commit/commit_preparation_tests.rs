//! Controls for native preparation:
//! real state unchanged,
//! the shadow's shape,
//! no hook of any kind run,
//! and no shadow
//!  left behind.

use super::*;
use crate::capture_invocation::{capture_invocation_base, capture_invocation_layout};
/// The tool-cache exemption entry these checks are built with.
use crate::effective_target::ToolCacheDirectory;
use crate::private_index_request::CaptureIndexes;
use crate::commit_index::{IndexSelection, initialize_commit_index, write_private_tree};
use crate::shadow_create::create_shadow_repository;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::transaction_journal::TransactionMode;
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
use crate::test_support::REAL_GIT;
use crate::wrapper_controls::no_controls;
use std::ffi::OsString;
use std::path::PathBuf;
use std::rc::Rc;

/// A fixed transaction ID of the shape the registry accepts.
const TRANSACTION: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";

/// Everything one preparation fixture produced.
struct Fixture {
    /// Repository the transaction selects.
    repo: PathBuf,
    /// Shadow repository of the transaction.
    shadow: PathBuf,
    /// Invocation capture.
    capture: InvocationCapture,
    /// Private commit index,
    /// staged for `a.txt` only.
    commit_index: PathBuf,
    /// The commit's own scratch directory,
    /// which holds the file the native-commit configuration travels in.
    scratch: PathBuf,
    /// Intended tree of the private index.
    intended_tree: String,
}

/// The repository one preparation fixture commits in:
/// `a.txt` committed on `main`,
/// an upstream and an `onbranch` include,
/// and a change to `a.txt` in the worktree.
fn prepared_repository(root: &Path) -> PathBuf {
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
    return repo;
}

/// A repository with `a.txt` committed,
/// the change to `a.txt` staged privately,
/// and a shadow ready to prepare.
fn prepared_fixture(root: &Path) -> Fixture {
    return preparation_fixture(root, prepared_repository(root));
}

/// The private index,
/// the capture and the shadow of a repository,
/// taken after everything the invocation would have read is in place.
fn preparation_fixture(root: &Path, repo: PathBuf) -> Fixture {
    let context: GitContext = git_context();
    return preparation_fixture_with(&context, root, repo);
}

/// `preparation_fixture` in the configuration layers `context` names,
/// so a control captures and prepares under the same global configuration real Git read.
fn preparation_fixture_with(context: &GitContext, root: &Path, repo: PathBuf) -> Fixture {
    // The base is whatever the repository's branch holds now, which is the state the invocation saw.
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let layout = capture_invocation_layout(context, repo.as_path(), false).expect("layout");
    let capture: InvocationCapture = capture_invocation_base(context, repo.as_path(), layout).expect("capture");
    let transaction: PathBuf = root.join(format!("transaction-{TRANSACTION}"));
    std::fs::create_dir_all(&transaction).expect("transaction");
    let shadow: PathBuf = create_shadow_repository(context, repo.as_path(), &capture, TRANSACTION, transaction.as_path())
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
    initialize_commit_index(context, repo.as_path(), capture.layout.repository_root.as_path(), capture.layout.real_index_path.as_path(), &indexes, &selection)
        .expect("private index");
    let intended_tree: String = write_private_tree(
        context,
        repo.as_path(),
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
    )
    .expect("tree");
    return Fixture {
        repo,
        shadow,
        capture,
        commit_index: indexes.commit_index,
        scratch: transaction.clone(),
        intended_tree,
    };
}

/// What:
///  A commit is built in the shadow of a repository whose directory name is not UTF-8.
/// Why:
///   The shadow lives under that Git directory,
///  and its name reached Git through `display()`,
///  which spelled the byte as U+FFFD and named a directory that does not exist:
///  Git answered "not a git repository" and the commit was not made.
#[test]
fn a_commit_names_the_shadow_by_its_bytes() {
    use std::os::unix::ffi::OsStringExt;
    let root: PathBuf = fixture("commit-preparation-not-utf8");
    let repo: PathBuf = root.join(std::ffi::OsString::from_vec(b"rep\xefo".to_vec()));
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change a");
    let prepared: Fixture = preparation_fixture(root.as_path(), repo.clone());
    let commit: PreparedCommit = prepare(&prepared, &["--message=prepared"])
        .expect("the shadow builds the commit for a repository whose name is not UTF-8");
    assert!(!commit.oid.is_empty(), "the commit names an object");
    remove(root.as_path());
}

/// Runs the preparation of a fixture.
fn prepare(fixture: &Fixture, commit_args: &[&str]) -> Result<PreparedCommit, PreparationError> {
    let context: GitContext = git_context();
    return prepare_with(&context, fixture, commit_args);
}

/// `prepare` with the configuration layers `context` names.
fn prepare_with(context: &GitContext, fixture: &Fixture, commit_args: &[&str]) -> Result<PreparedCommit, PreparationError> {
    let commit_arguments: Vec<OsString> = commit_args.iter().map(OsString::from).collect();
    let request: PreparationRequest = PreparationRequest {
        capture: &fixture.capture,
        shadow: fixture.shadow.as_path(),
        commit_index: fixture.commit_index.as_path(),
        scratch: fixture.scratch.as_path(),
        global_args: &[],
        commit_args: commit_arguments.as_slice(),
        object_directory: None,
    };
    let code: i32 = run_native_commit(context, fixture.repo.as_path(), &request)?;
    if code != 0 {
        return Err(PreparationError::Git(GitFailure(format!("git commit exited {code}"))));
    }
    return read_prepared_commit(context, fixture.shadow.as_path()).map_err(PreparationError::Git);
}

/// The context of `git_context` with `GIT_CONFIG_GLOBAL` naming a file holding `text`,
/// so the capture and the native commit read the same global layer real Git reads.
fn context_with_global(root: &Path, text: &str) -> GitContext {
    let global: PathBuf = root.join("global.config");
    std::fs::write(global.as_path(), text).expect("global configuration");
    let mut context: GitContext = git_context();
    context.overlay.retain(|(name, _)| { return name != "GIT_CONFIG_GLOBAL"; });
    context.overlay.push((std::ffi::OsString::from("GIT_CONFIG_GLOBAL"), global.into_os_string()));
    return context;
}

/// Run real Git with the layers a context names,
/// so a control sees the configuration the person's Git saw.
fn git_in_layers(context: &GitContext, directory: &Path, arguments: &[&str]) -> std::process::Output {
    let mut command = std::process::Command::new(REAL_GIT);
    command.current_dir(directory).env_clear().env("PATH", "/usr/bin:/bin");
    for (name, value) in context.overlay.iter() {
        command.env(name, value);
    }
    return command.args(arguments).output().expect("native Git control");
}

/// How many times the stand-in signer of a fixture ran.
fn signer_calls(calls: &Path) -> usize {
    return std::fs::read_to_string(calls).map_or(0, |text| { return text.lines().count(); });
}

/// Append `bytes` to a file a fixture already wrote.
fn append_to(path: &Path, bytes: &[u8]) {
    use std::io::Write;
    std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .and_then(|mut file| { return file.write_all(bytes); })
        .expect("append to the file");
}

/// Preparation leaves the real refs,
/// the real index and the worktree bytes unchanged,
///  and the prepared commit lives in the shadow.
#[test]
fn preparation_leaves_real_refs_index_and_worktree_unchanged() {
    let root: PathBuf = fixture("commit-preparation-success");
    let fixture: Fixture = prepared_fixture(root.as_path());
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
    remove(root.as_path());
}

/// The native command names the shadow and turns traditional hooks off,
/// and names no work tree.
#[test]
fn native_commit_arguments_name_the_shadow_and_turn_traditional_hooks_off() {
    let capture_root: PathBuf = fixture("commit-preparation-arguments");
    let repo: PathBuf = repository(capture_root.as_path(), "repo");
    let context = git_context();
    let layout = capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
    let capture: InvocationCapture = capture_invocation_base(&context, repo.as_path(), layout).expect("capture");
    let commit_arguments: Vec<OsString> = vec![OsString::from("-m"), OsString::from("x")];
    let global: Vec<OsString> = vec![OsString::from("-c"), OsString::from("a.b=1")];
    let request: PreparationRequest = PreparationRequest {
        capture: &capture,
        shadow: Path::new("/s"),
        commit_index: Path::new("/i"),
        scratch: Path::new("/c"),
        global_args: global.as_slice(),
        commit_args: commit_arguments.as_slice(),
        object_directory: None,
    };
    assert_eq!(
        native_commit_arguments(&request).expect("the configuration travels"),
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

/// The prepared commit's signature is read from its header block only,
/// not from the message.
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

/// The shadow object store of a fixture,
/// where the commit path writes its corrected blobs.
fn shadow_objects(fixture: &Fixture) -> PathBuf {
    return fixture.shadow.join("objects");
}

/// The exact bytes the prepared commit's tree holds for a path,
/// read from the shadow repository.
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
        Vec::<ToolCacheDirectory>::new(),
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
    let mut prepared_for: Fixture = prepared_fixture(root.as_path());
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

/// Both signature header spellings mark the prepared commit as signed,
/// and a signature-looking message line does not.
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
    assert_eq!(PreparationError::Git(GitFailure(String::from("git words"))).to_string(), "git words");
}

/// The shadow shows what a policy and a commit need:
/// `HEAD` at the base,
/// the upstream,
/// the `onbranch:` include,
/// and its
///  own Git directory.
#[test]
fn the_shadow_shows_head_upstream_include_and_git_dir() {
    let root: PathBuf = fixture("commit-preparation-shadow-view");
    let prepared: Fixture = prepared_fixture(root.as_path());
    let base: String = git_text(prepared.repo.as_path(), &["rev-parse", "HEAD"]);
    let shadow: String = format!("--git-dir={}", prepared.shadow.display());
    let in_shadow = |arguments: &[&str]| {
        let mut all: Vec<&str> = vec![shadow.as_str()];
        all.extend_from_slice(arguments);
        return git_text(prepared.repo.as_path(), all.as_slice());
    };
    assert_eq!(in_shadow(&["rev-parse", "HEAD"]).trim(), base);
    assert_eq!(in_shadow(&["rev-parse", "--abbrev-ref", "@{upstream}"]).trim(), "origin/main");
    assert_eq!(in_shadow(&["config", "--get", "x.v"]).trim(), "onbranch");
    assert_eq!(in_shadow(&["rev-parse", "--git-dir"]).trim(), prepared.shadow.display().to_string());
    remove(root.as_path());
}

/// What:
///  A commit whose signing program never answers,
///  prepared in `root`,
///  with the signer the commit command names.
/// Why:
///   The two ceiling controls differ only in whether the notice thread may start,
/// so they share the step that outlasts its ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const { fixture, globalArgs, commitArgs } = aCommitWhoseSignerNeverAnswers(root);
/// ```
#[cfg(unix)]
fn a_commit_whose_signer_never_answers(root: &Path) -> (Fixture, Vec<OsString>, Vec<OsString>) {
    let fixture: Fixture = prepared_fixture(root);
    let signer: PathBuf = root.join("slow-signer");
    crate::test_support::executable(signer.as_path(), b"#!/bin/sh\nexec sleep 60\n");
    let global_args: Vec<OsString> = vec![OsString::from("-c"), OsString::from(format!("gpg.program={}", signer.display()))];
    let commit_args: Vec<OsString> = vec![OsString::from("-S"), OsString::from("--message=x")];
    return (fixture, global_args, commit_args);
}

/// A configured hook for every hook event,
/// whatever its name,
/// does not run in the native commit,
/// and the commit
///  succeeds:
/// the shadow's own config switches every event off.
#[cfg(unix)]
#[test]
fn a_configured_hook_for_every_event_does_not_run_in_the_native_commit() {
    let root: PathBuf = fixture("commit-preparation-no-hooks");
    let prepared: Fixture = prepared_fixture(root.as_path());
    let marker: PathBuf = root.join("hook-ran");
    for (index, event) in crate::hooks_off::HOOK_EVENTS.iter().enumerate() {
        let script: String = format!("#!/bin/sh\necho {event} >> '{}'\nexit 7\n", marker.display());
        let path: PathBuf = root.join(format!("hook-{index}"));
        crate::test_support::executable(path.as_path(), script.as_bytes());
        git(prepared.repo.as_path(), &["config", format!("hook.named-{index}.event").as_str(), event]);
        git(prepared.repo.as_path(), &["config", format!("hook.named-{index}.command").as_str(), path.to_str().expect("UTF-8")]);
    }
    prepare(&prepared, &["--quiet", "-m", "message"]).expect("no hook runs, so none can refuse the commit");
    assert!(!marker.exists(), "a hook ran: {:?}", std::fs::read_to_string(&marker));
    remove(root.as_path());
}

/// A scratch commit whose signing program never answers is stopped at its ceiling,
/// says so in words that name what was kept,
/// leaves no process below the command,
/// and changes neither the real refs nor the real index:
/// the person's work is as it was and the same command can be run again.
#[test]
fn a_scratch_commit_that_outlasts_its_ceiling_is_stopped_and_changes_nothing() {
    let root: PathBuf = fixture("commit-preparation-ceiling");
    let (fixture, global_args, commit_args): (Fixture, Vec<OsString>, Vec<OsString>) = a_commit_whose_signer_never_answers(root.as_path());
    let refs_before: String = git_text(fixture.repo.as_path(), &["for-each-ref"]);
    let index_before: Vec<u8> = std::fs::read(fixture.repo.join(".git").join("index")).expect("index");
    let request: PreparationRequest = PreparationRequest {
        capture: &fixture.capture,
        shadow: fixture.shadow.as_path(),
        commit_index: fixture.commit_index.as_path(),
        scratch: fixture.scratch.as_path(),
        global_args: global_args.as_slice(),
        commit_args: commit_args.as_slice(),
        object_directory: None,
    };
    let began: std::time::Instant = std::time::Instant::now();
    let outcome = run_native_commit_within(&git_context(), fixture.repo.as_path(), &request, std::time::Duration::from_millis(1500));
    let lasted: std::time::Duration = began.elapsed();
    let Err(PreparationError::Git(GitFailure(said))) = outcome else {
        panic!("a step that outlasted its ceiling was not turned into a stop: {outcome:?}");
    };
    assert!(lasted < std::time::Duration::from_secs(20), "the stop did not end the step: {lasted:?}");
    assert!(said.contains("was stopped after") && said.contains("nothing was committed"), "{said}");
    assert!(said.contains("signs the commit"), "a signed commit's stop names the signer: {said}");
    assert!(crate::process_tree::descendants_of(i32::try_from(std::process::id()).expect("pid")).is_empty(), "a process was left below the command");
    assert_eq!(git_text(fixture.repo.as_path(), &["for-each-ref"]), refs_before);
    assert_eq!(std::fs::read(fixture.repo.join(".git").join("index")).expect("index"), index_before);
    remove(root.as_path());
}

/// The repository's own `gpg.openpgp.program` beats a global legacy `gpg.program`,
/// because both name the OpenPGP program and the repository's entry comes later,
/// so the shadow's native commit must sign with the repository's program and never start the global one.
#[test]
fn the_repository_openpgp_program_beats_the_legacy_global_program() {
    let root: PathBuf = fixture("commit-preparation-legacy-gpg-program");
    let repo: PathBuf = prepared_repository(root.as_path());
    let calls: PathBuf = root.join("signer-calls");
    let signer: PathBuf = root.join("stand-in-gpg");
    let script: String = format!(
        "#!/bin/sh\ncat > /dev/null\necho call >> '{}'\necho '[GNUPG:] SIG_CREATED ' >&2\nprintf -- '-----BEGIN PGP SIGNATURE-----\\n\\nstand-in\\n-----END PGP SIGNATURE-----\\n'\n",
        calls.display()
    );
    crate::test_support::executable(signer.as_path(), script.as_bytes());
    let context: GitContext = context_with_global(root.as_path(), "[gpg]\n\tprogram = /does-not-exist/cli-git-fixture\n");
    for (setting, value) in [
        ("gpg.openpgp.program", signer.to_str().expect("UTF-8")),
        ("commit.gpgsign", "true"),
        ("user.signingkey", "fixture"),
    ] {
        git(repo.as_path(), &["config", setting, value]);
    }
    // The positive control: real Git signs this repository's commit with its own program, and starts the global one never.
    let control: std::process::Output = git_in_layers(&context, repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=real control"]);
    assert!(control.status.success(), "{}", String::from_utf8_lossy(&control.stderr));
    assert!(
        git_text(repo.as_path(), &["cat-file", "commit", "HEAD"]).contains("\ngpgsig "),
        "the repository's program must sign the control, or this control proves nothing"
    );
    let after_control: usize = signer_calls(calls.as_path());
    assert_eq!(after_control, 1, "real Git ran the repository's program once");
    let fixture: Fixture = preparation_fixture_with(&context, root.as_path(), repo);
    let prepared: PreparedCommit = prepare_with(&context, &fixture, &["--allow-empty", "--message=shadow control"]).expect("prepared");
    assert!(prepared.signed, "the repository's program must sign in the shadow too");
    assert_eq!(signer_calls(calls.as_path()), after_control + 1, "the shadow ran the repository's program, not the global one");
    remove(root.as_path());
}

/// A trailer subsection holding an equals sign,
/// such as `[trailer "review=bot"]`,
/// names a key that a `key=value` setting cannot carry:
/// Git splits such a setting at its first equals sign,
/// so the configured trailer has to reach the shadow another way.
#[test]
fn a_trailer_subsection_holding_an_equals_sign_keeps_its_configuration() {
    let root: PathBuf = fixture("commit-preparation-trailer-equals");
    let repo: PathBuf = prepared_repository(root.as_path());
    let included: PathBuf = root.join("trailer.config");
    std::fs::write(included.as_path(), "[trailer \"review=bot\"]\n\tkey = Reviewed-by\n\tcmd = printf reviewer\n").expect("trailer configuration");
    // Only this exact Git directory selects the file, which is what no longer matches in the shadow.
    let condition: String = format!("includeIf.gitdir:{}/.git.path", repo.display());
    git(repo.as_path(), &["config", condition.as_str(), included.to_str().expect("UTF-8")]);
    let trailer: &str = "review=bot: given";
    // The positive control: real Git adds the configured trailer for this token.
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=real control", "--trailer", trailer]);
    let control: String = git_text(repo.as_path(), &["log", "-1", "--format=%B"]);
    assert!(control.contains("Reviewed-by: reviewer"), "the configured trailer must be added, or this control proves nothing: {control}");
    let fixture: Fixture = preparation_fixture(root.as_path(), repo);
    prepare(&fixture, &["--allow-empty", "--message=shadow control", "--trailer", trailer]).expect("prepared");
    let shadow: String = git_text(fixture.shadow.as_path(), &["log", "-1", "--format=%B"]);
    assert!(shadow.contains("Reviewed-by: reviewer"), "the shadow must add the configured trailer too: {shadow}");
    remove(root.as_path());
}

/// A configured value that no single argument can carry stays in the configuration files,
/// so the shadow's native commit starts and commits as real Git does.
#[test]
fn a_configuration_value_longer_than_one_argument_does_not_stop_the_native_commit() {
    let root: PathBuf = fixture("commit-preparation-long-value");
    let repo: PathBuf = prepared_repository(root.as_path());
    let value: String = "x".repeat(crate::argument_limit::one_argument_limit() * 2);
    append_to(
        repo.join(".git").join("config").as_path(),
        format!("[trailer \"unused\"]\n\tkey = {value}\n").as_bytes(),
    );
    // The positive control: real Git reads the value from its file, never uses it, and commits.
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=real control"]);
    let fixture: Fixture = preparation_fixture(root.as_path(), repo);
    let prepared: PreparedCommit = prepare(&fixture, &["--message=shadow control"]).expect("prepared");
    assert_eq!(prepared.tree_oid, fixture.intended_tree);
    remove(root.as_path());
}

/// A throwaway SSH signing key and the configuration that signs with it,
/// in a file an exact `gitdir` condition includes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const key = sshKeygen(root); const included = writeSigningConfig(root, key);
/// git(repo, ['config', `includeIf.gitdir:${repo}/.git.path`, included]);
/// ```
fn signing_included_by_exact_gitdir(root: &Path, repo: &Path) -> PathBuf {
    let key: PathBuf = root.join("signing-key");
    let made = std::process::Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-C", "fixture", "-f"])
        .arg(&key)
        .output()
        .expect("ssh-keygen starts");
    assert!(made.status.success(), "{}", String::from_utf8_lossy(&made.stderr));
    let included: PathBuf = root.join("signing.config");
    std::fs::write(
        included.as_path(),
        format!("[commit]\n\tgpgSign = true\n[gpg]\n\tformat = ssh\n[user]\n\tsigningKey = {}\n", key.display()),
    )
    .expect("signing configuration");
    // Only this exact Git directory selects the file, which is what no longer matches in the shadow.
    let condition: String = format!("includeIf.gitdir:{}/.git.path", repo.display());
    git(repo, &["config", condition.as_str(), included.to_str().expect("UTF-8")]);
    return included;
}

/// A commit whose signing comes from an exact `gitdir` conditional include is signed by the shadow's native commit too:
/// the person asked for a signature,
/// and a clean unsigned commit would answer a different request.
#[test]
fn signing_from_an_exact_gitdir_include_reaches_the_shadow() {
    let root: PathBuf = fixture("commit-preparation-gitdir-signing");
    let repo: PathBuf = prepared_repository(root.as_path());
    signing_included_by_exact_gitdir(root.as_path(), repo.as_path());
    // The positive control: real Git, in the real Git directory, signs this repository's commits.
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=native control"]);
    let native: String = git_text(repo.as_path(), &["cat-file", "commit", "HEAD"]);
    assert!(native.contains("\ngpgsig "), "the exact gitdir include must sign, or this control proves nothing");
    // The capture reads the configuration after the include is in place, as the invocation's does.
    let fixture: Fixture = preparation_fixture(root.as_path(), repo);
    // The shadow's own native commit must sign for the same reason.
    let prepared: PreparedCommit = prepare(&fixture, &["--allow-empty", "--message=shadow control"]).expect("prepared");
    assert!(prepared.signed, "the shadow must not turn the person's signing request into an unsigned commit");
    remove(root.as_path());
}

/// A configured `[trailer ""]` is valid configuration real Git accepts,
/// so a commit carrying it through the shadow must not fail:
/// the key `trailer..key` is a trailer whose subsection is empty,
/// and it is written as the quoted empty subsection it is.
#[test]
fn an_empty_trailer_subsection_does_not_stop_the_native_commit() {
    let root: PathBuf = fixture("commit-preparation-empty-subsection");
    let repo: PathBuf = prepared_repository(root.as_path());
    // Git writes the empty subsection itself, as `[trailer ""]`.
    git(repo.as_path(), &["config", "trailer..key", "X-Empty"]);
    // The positive control: real Git commits with this configuration in place.
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=real control"]);
    let fixture: Fixture = preparation_fixture(root.as_path(), repo);
    let prepared: PreparedCommit = prepare(&fixture, &["--message=shadow control"]).expect("prepared");
    assert_eq!(prepared.tree_oid, fixture.intended_tree);
    remove(root.as_path());
}

/// An `x509` signing program named in the exact `gitdir` include is the one the shadow's native commit runs:
/// a person who configured a signer does not get the default program,
/// which on this host does not exist at all.
#[test]
fn an_x509_program_from_an_exact_gitdir_include_reaches_the_shadow() {
    let root: PathBuf = fixture("commit-preparation-x509-signing");
    let repo: PathBuf = prepared_repository(root.as_path());
    let calls: PathBuf = root.join("signer-calls");
    let signer: PathBuf = root.join("stand-in-gpgsm");
    let script: String = format!(
        "#!/bin/sh\ncat > /dev/null\necho call >> '{}'\necho '[GNUPG:] SIG_CREATED ' >&2\nprintf -- '-----BEGIN SIGNED MESSAGE-----\\n\\nstand-in\\n-----END SIGNED MESSAGE-----\\n'\n",
        calls.display()
    );
    crate::test_support::executable(signer.as_path(), script.as_bytes());
    let included: PathBuf = root.join("signing.config");
    std::fs::write(
        included.as_path(),
        format!(
            "[commit]\n\tgpgSign = true\n[gpg]\n\tformat = x509\n[gpg \"x509\"]\n\tprogram = {}\n[user]\n\tsigningKey = fixture\n",
            signer.display()
        ),
    )
    .expect("signing configuration");
    // Only this exact Git directory selects the file, which is what no longer matches in the shadow.
    let condition: String = format!("includeIf.gitdir:{}/.git.path", repo.display());
    git(repo.as_path(), &["config", condition.as_str(), included.to_str().expect("UTF-8")]);
    // The positive control: real Git signs this repository's commit with the configured program.
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=real control"]);
    assert!(
        git_text(repo.as_path(), &["cat-file", "commit", "HEAD"]).contains("\ngpgsig "),
        "the configured x509 program must sign the control, or this control proves nothing"
    );
    let after_control: usize = signer_calls(calls.as_path());
    assert_eq!(after_control, 1, "real Git ran the configured x509 program once");
    let fixture: Fixture = preparation_fixture(root.as_path(), repo);
    let prepared: PreparedCommit = prepare(&fixture, &["--allow-empty", "--message=shadow control"]).expect("prepared");
    assert!(prepared.signed, "the shadow must sign with the configured x509 program too");
    assert_eq!(signer_calls(calls.as_path()), after_control + 1, "the shadow ran the configured x509 program, not the default one");
    remove(root.as_path());
}

/// What:
///  A commit whose notice thread the system refused still has a ceiling,
///  because the wait that owns the step ends it.
/// Why:
///   The notice thread is what ends a step at its ceiling,
/// and a process at its thread limit starts none.
///  The ceiling then had no enforcer at all:
/// a step that outlived it waited with no bound over it,
/// which is finding 1 of the fifth review of this step.
///  The bound must not depend on a thread that the same resource pressure refuses,
/// so the wait that owns the child ends it,
/// and it can,
/// because a number cannot be reused while its child is unreaped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // refuseTheNextNoticeThread(); await expect(runNativeCommitWithin({ ceiling: 1500 })).rejects.toThrow('was stopped after')
/// ```
#[cfg(unix)]
#[test]
fn a_scratch_commit_whose_notice_thread_was_refused_is_stopped_at_its_ceiling() {
    let root: PathBuf = fixture("commit-preparation-ceiling-no-notice");
    let (fixture, global_args, commit_args): (Fixture, Vec<OsString>, Vec<OsString>) = a_commit_whose_signer_never_answers(root.as_path());
    let refs_before: String = git_text(fixture.repo.as_path(), &["for-each-ref"]);
    let request: PreparationRequest = PreparationRequest {
        capture: &fixture.capture,
        shadow: fixture.shadow.as_path(),
        commit_index: fixture.commit_index.as_path(),
        scratch: fixture.scratch.as_path(),
        global_args: global_args.as_slice(),
        commit_args: commit_args.as_slice(),
        object_directory: None,
    };
    crate::native_commit_waiting_notice::refuse_the_next_notice_thread(1);
    let began: std::time::Instant = std::time::Instant::now();
    let outcome = run_native_commit_within(&git_context(), fixture.repo.as_path(), &request, std::time::Duration::from_millis(1500));
    let lasted: std::time::Duration = began.elapsed();
    let Err(PreparationError::Git(GitFailure(said))) = outcome else {
        panic!("a step that outlasted its ceiling waited with nothing to stop it: {outcome:?} after {lasted:?}");
    };
    assert!(
        lasted < std::time::Duration::from_secs(20),
        "the step ran on past its ceiling although no notice thread enforced it: {lasted:?}"
    );
    assert!(said.contains("was stopped after") && said.contains("nothing was committed"), "{said}");
    assert!(crate::process_tree::descendants_of(i32::try_from(std::process::id()).expect("pid")).is_empty(), "a process was left below the command");
    assert_eq!(git_text(fixture.repo.as_path(), &["for-each-ref"]), refs_before);
    remove(root.as_path());
}
