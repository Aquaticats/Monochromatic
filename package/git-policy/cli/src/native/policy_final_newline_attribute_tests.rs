//! What:
//!  Controls for the part of the final-newline policy that honors Git's `text` attribute:
//!  a file whose `text` attribute is unset keeps its exact bytes.
//! Why:
//!  A fuzz seed's trailing bytes are the test input,
//!  and a path list goes stale whenever a package moves.
//!  The package's own `.gitattributes` travels with it,
//!  so the policy reads the attribute from the same state the candidates come from:
//!  the index for staged and projected candidates,
//!  and the commit's tree for landed and pushed ones.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(findings(repo, 'seed/** -text')).not.toContainEqual(finding('seed/case'));
//! ```
#![cfg(unix)]

/// Import the module under test.
use super::{FINAL_NEWLINE_CODE, FINAL_NEWLINE_MESSAGE, check_final_newline};
use crate::candidate_object::{ObjectId, parse_object_id};
use crate::candidate_version::CandidateVersion;
use crate::candidate_prediction::{CandidateRequest, PushedScope};
use crate::diagnostics::EngineFailureCode;
use crate::event_path::EventPath;
use crate::policy_content::{ContentState, Correction, LifecycleContent};
use crate::policy_engine::{PolicyFinding, PolicyOutcome};
use crate::policy_test_support::{ScriptedFacts, scripted_facts};
use crate::policy_trigger::Trigger;
use crate::test_support::{fixture, git, git_text, remove, repository};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// The finding about the file at `path` of a lifecycle that does not correct.
fn finding(path: &str) -> PolicyFinding {
    return PolicyFinding {
        code: FINAL_NEWLINE_CODE,
        message: String::from(FINAL_NEWLINE_MESSAGE),
        path: Some(EventPath::from_git_bytes(path.as_bytes())),
        location: None,
        fix_available: false,
    };
}

/// Write `bytes` at `relative` below `repo`,
///  creating the directories above it.
fn write(repo: &Path, relative: &str, bytes: &[u8]) {
    let target: PathBuf = repo.join(relative);
    std::fs::create_dir_all(target.parent().expect("a parent directory")).expect("directories");
    std::fs::write(target, bytes).expect("file");
}

/// The lifecycle that stages every file of the repository.
fn add_all() -> LifecycleContent {
    return LifecycleContent::Requested(CandidateRequest::Add(vec![OsString::from("--all")]));
}

/// The outcome of one check of `lifecycle` run from `directory`.
fn check_from(directory: &Path, lifecycle: &LifecycleContent, trigger: Trigger) -> PolicyOutcome {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.candidates_repository = Some(directory.to_path_buf());
    return check_final_newline(
        &mut ContentState::new(),
        lifecycle,
        &mut facts,
        trigger,
        false,
    );
}

/// The outcome of one check of `lifecycle` run at the top of `repo`.
fn check(repo: &Path, lifecycle: &LifecycleContent) -> PolicyOutcome {
    return check_from(repo, lifecycle, Trigger::PreForward);
}

/// The object name `revision` resolves to in `repo`.
fn object(repo: &Path, revision: &str) -> ObjectId {
    let text: String = git_text(repo, &["rev-parse", revision]);
    return parse_object_id(text.trim().as_bytes()).expect("an object name");
}

/// Commit everything in `repo` with `message`.
fn commit_all(repo: &Path, message: &str) {
    git(repo, &["add", "--all"]);
    git(
        repo,
        &["commit", "--quiet", &format!("--message={message}")],
    );
}

/// Make every Git process started in `repo` from now on fail on its configuration,
///  while a process that already runs keeps reading objects.
fn break_git_configuration(repo: &Path) {
    std::fs::write(repo.join(".git/config"), b"[broken\n").expect("broken configuration");
}

/// Prepare the candidates of `lifecycle` and read one candidate's bytes,
///  which starts the long-lived object reader that later reads keep using.
fn start_object_reader(
    content: &mut ContentState,
    lifecycle: &LifecycleContent,
    facts: &mut ScriptedFacts,
) {
    let version: Rc<CandidateVersion> = content
        .prepare(lifecycle, facts)
        .expect("prepared")
        .expect("candidates");
    content
        .bytes(&version.candidates()[0])
        .expect("the first candidate's bytes");
}

/// A seed under a path the attributes mark `-text` keeps its exact bytes,
///  while a neighbour outside the pattern is still reported.
#[test]
fn a_file_whose_text_attribute_is_unset_keeps_its_bytes() {
    let root: PathBuf = fixture("final-newline-attribute-unset");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), ".gitattributes", b"seed/** -text\n");
    write(repo.as_path(), "seed/boundary-512", b"no final newline");
    write(repo.as_path(), "seed/extra", b"extra\n\n");
    write(repo.as_path(), "other.txt", b"no final newline");
    assert_eq!(
        check(repo.as_path(), &add_all()),
        PolicyOutcome::Findings(vec![finding("other.txt")])
    );
    remove(root.as_path());
}

/// The `binary` macro unsets `text`,
///  so a UTF-8 file it names keeps its bytes.
#[test]
fn a_file_marked_binary_keeps_its_bytes() {
    let root: PathBuf = fixture("final-newline-attribute-binary");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), ".gitattributes", b"*.dat binary\n");
    write(repo.as_path(), "blob.dat", b"utf-8 without a final newline");
    write(repo.as_path(), "blob.txt", b"utf-8 without a final newline");
    assert_eq!(
        check(repo.as_path(), &add_all()),
        PolicyOutcome::Findings(vec![finding("blob.txt")])
    );
    remove(root.as_path());
}

/// A file no attribute names is still corrected,
///  with the proposal a direct fix applies,
///  beside files that keep their bytes.
#[test]
fn a_plain_text_file_is_still_corrected() {
    let root: PathBuf = fixture("final-newline-attribute-plain");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), ".gitattributes", b"seed/** -text\n");
    write(repo.as_path(), "seed/case", b"exact");
    write(repo.as_path(), "plain.txt", b"plain");
    let mut facts: ScriptedFacts = scripted_facts();
    facts.candidates_repository = Some(repo.clone());
    let mut content: ContentState = ContentState::new();
    let outcome: PolicyOutcome = check_final_newline(
        &mut content,
        &add_all(),
        &mut facts,
        Trigger::DirectFix,
        false,
    );
    assert_eq!(
        outcome,
        PolicyOutcome::Findings(vec![PolicyFinding {
            fix_available: true,
            ..finding("plain.txt")
        }])
    );
    let proposals: Vec<Correction> = content.take_proposals();
    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0].path, b"plain.txt".to_vec());
    assert_eq!(&*proposals[0].after, b"plain\n");
    remove(root.as_path());
}

/// The `.gitattributes` of `package/forbidden-strings/cli.fuzz`,
///  which is why the path list no longer names its seeds:
///  its seed, corpus and artifact files keep their bytes with no other exemption.
#[test]
fn the_forbidden_strings_fuzz_sidecar_declaration_preserves_its_files() {
    let root: PathBuf = fixture("final-newline-attribute-sidecar");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let sidecar: &str = "package/forbidden-strings/cli.fuzz";
    write(
        repo.as_path(),
        ".gitattributes",
        b"* text=auto eol=lf\n",
    );
    write(
        repo.as_path(),
        &format!("{sidecar}/.gitattributes"),
        b"seed/** -text\ncorpus/** -text\nartifacts/** -text\n",
    );
    write(repo.as_path(), &format!("{sidecar}/seed/fuzz_depth_envelope/boundary-512"), b"[[[");
    write(repo.as_path(), &format!("{sidecar}/seed/malformed-deep"), b"extra\n\n");
    write(repo.as_path(), &format!("{sidecar}/corpus/entry"), b"corpus");
    write(repo.as_path(), &format!("{sidecar}/artifacts/crash"), b"crash");
    write(repo.as_path(), &format!("{sidecar}/src/lib.rs"), b"source");
    assert_eq!(
        check(repo.as_path(), &add_all()),
        PolicyOutcome::Findings(vec![finding(&format!("{sidecar}/src/lib.rs"))])
    );
    remove(root.as_path());
}

/// A `text` attribute that is set,
///  or set to `auto`,
///  does not preserve the file: only an unset one does.
#[test]
fn a_file_whose_text_attribute_is_set_is_still_corrected() {
    let root: PathBuf = fixture("final-newline-attribute-set");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(
        repo.as_path(),
        ".gitattributes",
        b"* text=auto eol=lf\nexplicit.txt text\nseed/** -text\n",
    );
    write(repo.as_path(), "explicit.txt", b"explicit");
    write(repo.as_path(), "automatic.txt", b"automatic");
    write(repo.as_path(), "seed/case", b"exact");
    assert_eq!(
        check(repo.as_path(), &add_all()),
        PolicyOutcome::Findings(vec![finding("automatic.txt"), finding("explicit.txt")])
    );
    remove(root.as_path());
}

/// A `.gitattributes` in a sidecar directory applies to its own subtree only,
///  as Git resolves it.
#[test]
fn a_nested_gitattributes_applies_to_its_own_subtree() {
    let root: PathBuf = fixture("final-newline-attribute-nested");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(
        repo.as_path(),
        "package/sidecar.fuzz/.gitattributes",
        b"seed/** -text\n",
    );
    write(repo.as_path(), "package/sidecar.fuzz/seed/case", b"exact");
    write(repo.as_path(), "package/sidecar.fuzz/src/lib.rs", b"source");
    write(repo.as_path(), "package/other.fuzz/seed/case", b"exact");
    write(repo.as_path(), "seed/case", b"exact");
    assert_eq!(
        check(repo.as_path(), &add_all()),
        PolicyOutcome::Findings(vec![
            finding("package/other.fuzz/seed/case"),
            finding("package/sidecar.fuzz/src/lib.rs"),
            finding("seed/case"),
        ])
    );
    remove(root.as_path());
}

/// A `.gitattributes` staged by the same command applies to that command's candidates,
///  although no commit holds it yet.
#[test]
fn a_gitattributes_staged_with_the_seed_applies_to_it() {
    let root: PathBuf = fixture("final-newline-attribute-same-add");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), ".gitattributes", b"seed/** -text\n");
    write(repo.as_path(), "seed/case", b"exact");
    let lifecycle: LifecycleContent = LifecycleContent::Requested(CandidateRequest::Add(vec![
        OsString::from("--"),
        OsString::from(".gitattributes"),
        OsString::from("seed/case"),
    ]));
    assert_eq!(
        check(repo.as_path(), &lifecycle),
        PolicyOutcome::Findings(Vec::new())
    );
    remove(root.as_path());
}

/// Staged candidates read the index's `.gitattributes`,
///  not the worktree's: an unstaged edit does not preserve a staged file,
///  and an unstaged removal does not expose one.
#[test]
fn staged_candidates_read_the_index_attributes_not_the_worktree() {
    let root: PathBuf = fixture("final-newline-attribute-index");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), ".gitattributes", b"seed/** -text\n");
    write(repo.as_path(), "seed/case", b"exact");
    write(repo.as_path(), "plain.txt", b"plain");
    git(
        repo.as_path(),
        &["add", ".gitattributes", "seed/case", "plain.txt"],
    );
    // The worktree now disagrees with the index in both directions.
    write(repo.as_path(), ".gitattributes", b"plain.txt -text\n");
    let lifecycle: LifecycleContent =
        LifecycleContent::Requested(CandidateRequest::Staged(vec![OsString::from(":/")]));
    assert_eq!(
        check(repo.as_path(), &lifecycle),
        PolicyOutcome::Findings(vec![finding("plain.txt")])
    );
    remove(root.as_path());
}

/// The candidates' paths are relative to the top level while Git resolves attribute paths from the directory
///  it runs in, so a check started below the top level still finds a rule written at the top.
#[test]
fn a_check_started_below_the_top_level_resolves_top_level_paths() {
    let root: PathBuf = fixture("final-newline-attribute-subdirectory");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), ".gitattributes", b"sub/seed/** -text\n");
    write(repo.as_path(), "sub/seed/case", b"exact");
    write(repo.as_path(), "sub/plain.txt", b"plain");
    let lifecycle: LifecycleContent = LifecycleContent::Requested(CandidateRequest::Add(vec![
        OsString::from("--"),
        OsString::from(":/"),
    ]));
    assert_eq!(
        check_from(repo.join("sub").as_path(), &lifecycle, Trigger::PreForward),
        PolicyOutcome::Findings(vec![finding("sub/plain.txt")])
    );
    remove(root.as_path());
}

/// A landed commit reads the `.gitattributes` of its own tree,
///  whatever the index and the worktree hold afterwards.
#[test]
fn a_landed_commit_reads_the_attributes_of_its_own_tree() {
    let root: PathBuf = fixture("final-newline-attribute-landed");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), ".gitattributes", b"seed/** -text\n");
    write(repo.as_path(), "seed/case", b"exact");
    write(repo.as_path(), "plain.txt", b"plain");
    commit_all(repo.as_path(), "landed");
    let landed: ObjectId = object(repo.as_path(), "HEAD");
    // Later states drop the rule from the index and the worktree.
    write(repo.as_path(), ".gitattributes", b"");
    git(repo.as_path(), &["add", ".gitattributes"]);
    write(repo.as_path(), ".gitattributes", b"plain.txt -text\n");
    let lifecycle: LifecycleContent =
        LifecycleContent::Requested(CandidateRequest::Landed(landed));
    assert_eq!(
        check_from(repo.as_path(), &lifecycle, Trigger::PostCommit),
        PolicyOutcome::Findings(vec![finding("plain.txt")])
    );
    remove(root.as_path());
}

/// Each pushed commit is read against its own tree:
///  a path first listed by a commit that marked it `-text` is preserved,
///  and a path of a commit that did not is reported.
#[test]
fn a_pushed_commit_reads_the_attributes_of_its_own_tree() {
    let root: PathBuf = fixture("final-newline-attribute-pushed");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), ".gitattributes", b"seed/** -text\n");
    write(repo.as_path(), "seed/case", b"exact");
    commit_all(repo.as_path(), "marked");
    let marked: ObjectId = object(repo.as_path(), "HEAD");
    write(repo.as_path(), ".gitattributes", b"");
    write(repo.as_path(), "seed/later", b"exact");
    commit_all(repo.as_path(), "unmarked");
    let unmarked: ObjectId = object(repo.as_path(), "HEAD");
    let lifecycle: LifecycleContent =
        LifecycleContent::Requested(CandidateRequest::Pushed(PushedScope {
            commits: vec![marked, unmarked],
            trees: Vec::new(),
        }));
    assert_eq!(
        check_from(repo.as_path(), &lifecycle, Trigger::ManualPush),
        PolicyOutcome::Findings(vec![finding("seed/later")])
    );
    remove(root.as_path());
}

/// A pushed tip tree is read against itself.
#[test]
fn a_pushed_tree_reads_its_own_attributes() {
    let root: PathBuf = fixture("final-newline-attribute-tree");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), ".gitattributes", b"seed/** -text\n");
    write(repo.as_path(), "seed/case", b"exact");
    write(repo.as_path(), "plain.txt", b"plain");
    commit_all(repo.as_path(), "tip");
    let tree: ObjectId = object(repo.as_path(), "HEAD^{tree}");
    let lifecycle: LifecycleContent =
        LifecycleContent::Requested(CandidateRequest::Pushed(PushedScope {
            commits: Vec::new(),
            trees: vec![tree],
        }));
    assert_eq!(
        check_from(repo.as_path(), &lifecycle, Trigger::ManualPush),
        PolicyOutcome::Findings(vec![finding("plain.txt")])
    );
    remove(root.as_path());
}

/// No attribute is asked about a file whose bytes are already canonical,
///  so a commit of canonical files starts no attribute query:
///  a repository whose configuration Git can no longer read still passes.
#[test]
fn a_canonical_file_needs_no_attribute_lookup() {
    let root: PathBuf = fixture("final-newline-attribute-lazy");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), ".gitattributes", b"seed/** -text\n");
    write(repo.as_path(), "fine.txt", b"fine\n");
    commit_all(repo.as_path(), "canonical");
    let landed: ObjectId = object(repo.as_path(), "HEAD");
    let lifecycle: LifecycleContent =
        LifecycleContent::Requested(CandidateRequest::Landed(landed));
    let mut facts: ScriptedFacts = scripted_facts();
    facts.candidates_repository = Some(repo.clone());
    let mut content: ContentState = ContentState::new();
    // Prepare first: the running object reader keeps reading, and any later Git process fails.
    start_object_reader(&mut content, &lifecycle, &mut facts);
    break_git_configuration(repo.as_path());
    assert_eq!(
        check_final_newline(&mut content, &lifecycle, &mut facts, Trigger::PostCommit, false),
        PolicyOutcome::Findings(Vec::new())
    );
    remove(root.as_path());
}

/// An attribute lookup Git cannot complete fails the policy instead of treating the file as text.
#[test]
fn an_attribute_lookup_that_fails_fails_the_policy() {
    let root: PathBuf = fixture("final-newline-attribute-failure");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), ".gitattributes", b"seed/** -text\n");
    write(repo.as_path(), "plain.txt", b"plain");
    commit_all(repo.as_path(), "landed");
    let landed: ObjectId = object(repo.as_path(), "HEAD");
    let lifecycle: LifecycleContent =
        LifecycleContent::Requested(CandidateRequest::Landed(landed));
    let mut facts: ScriptedFacts = scripted_facts();
    facts.candidates_repository = Some(repo.clone());
    let mut content: ContentState = ContentState::new();
    start_object_reader(&mut content, &lifecycle, &mut facts);
    break_git_configuration(repo.as_path());
    match check_final_newline(&mut content, &lifecycle, &mut facts, Trigger::PostCommit, false) {
        PolicyOutcome::Failed { code, message, .. } => {
            assert_eq!(code, EngineFailureCode::ContentUnavailable);
            // The failure is the attribute lookup's, not the object reader's.
            assert!(
                message.starts_with("cli-git could not read the text attribute of candidate files: git rev-parse failed"),
                "{message}"
            );
        }
        other => panic!("expected a failure, got {other:?}"),
    }
    remove(root.as_path());
}
