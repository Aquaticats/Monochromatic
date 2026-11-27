//! What:
//!  Controls for the bytes a commit's content policies read:
//!  the entries of the commit's own index,
//!  which are the bytes that commit writes.
//! Why:
//!  A warning about a file that ends with exactly one line feed names bytes nobody will commit,
//!  and a check that reads another state than the commit writes warns about one file while it
//!  passes over the file the commit really records (issue 6622).
//!  Each control states one state a commit can reach,
//!  reads what the stage before forwarding answered,
//!  and holds that answer against the bytes the commit's own index holds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const findings = theStageBeforeForwarding(scene, mode, pathspecs);
//! // const committed = theCommitIndexBlob(scene, path);
//! // expect(findings.some((f) => f.path === path)).toBe(normalizeFinalNewline(committed) !== undefined);
//! ```

use crate::candidate_object::parse_object_id;
use crate::candidate_prediction::CandidateRequest;
use crate::command_test_support::os_arguments;
use crate::commit_facts::PrivateIndexFacts;
use crate::commit_index::{
    IndexSelection,
    initialize_commit_index,
    list_changed_index_paths,
};
use crate::commit_transaction::staged_at_base;
use crate::effective_target::ToolCacheDirectory;
use crate::policy_checks::shipped_checks;
use crate::policy_content::LifecycleContent;
use crate::policy_engine::{
    PolicyChecks,
    PolicyFinding,
    PolicyOutcome,
};
use crate::policy_final_newline::normalized_final_newline;
use crate::policy_registry::PolicyId;
use crate::policy_trigger::Trigger;
use crate::private_index_request::CaptureIndexes;
use crate::repository_facts::{
    RepositoryFacts,
    git_facts,
};
use crate::start_directory::StartDirectory;
use crate::test_support::{
    REAL_GIT,
    fixture,
    git,
    git_context,
    remove,
    repository,
};
use crate::transaction_git::{
    GitRequest,
    run_git,
};
use crate::transaction_journal::TransactionMode;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// The name every control of this file gives the file it commits.
const COMMITTED_FILE: &str = "notes.txt";

/// What:
///  `Scene` is one fixture repository with a first commit,
///  its three private index files and the base commit's name.
///  `root` is the fixture folder the whole scene lives in,
///  and `repo` the repository inside it;
///  `indexes` are the copies a commit prepares,
///  and `base` is the name of the commit they
///  are compared with.
/// Why:
///  Every state of this file differs only in what the worktree and the real index hold,
///  so the scene is made once per control and each control says only its own change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Scene = { root: string; repo: string; indexes: CaptureIndexes; base: string };
/// ```
struct Scene {
    /// The fixture folder this scene removes when its control ends.
    root: PathBuf,
    /// The repository whose commit the control prepares.
    repo: PathBuf,
    /// The captured index,
    ///  the commit's own index,
    ///  and the object-store folder of the preparation.
    indexes: CaptureIndexes,
    /// The name of the commit every candidate of the preparation is compared with.
    base: String,
}

/// What:
///  `a_scene(name)` makes a repository whose `HEAD` holds `base.txt`,
///  and the three index files a commit prepares beside it.
/// Why:
///  A candidate is a path the commit's own index changes against a commit that already exists,
///  so the scene has to start with one,
///  and an unborn branch would exercise none of the
///  additions these controls are about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aScene(name: string): Scene
/// ```
fn a_scene(name: &str) -> Scene {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    std::fs::write(repo.join("base.txt"), b"base\n").expect("the base file");
    git(repo.as_path(), &["add", "--", "base.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    // `.join(..)` builds the path of each private index inside the repository's Git directory.
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: git_dir.join("captured.index"),
        commit_index: git_dir.join("commit.index"),
        object_directory: git_dir.join("objects"),
    };
    let mut read: GitRequest = GitRequest::new(repo.as_path(), &["rev-parse", "HEAD"]);
    read.without_prefix = true;
    let answer = run_git(&git_context(), &read).expect("the base commit is named");
    assert!(answer.succeeded(), "{}", answer.error_text());
    // `.trim()` drops the line feed Git ends the name with.
    let base: String = String::from_utf8_lossy(answer.stdout.as_slice())
        .trim()
        .to_owned();
    return Scene {
        root,
        repo,
        indexes,
        base,
    };
}

/// What:
///  `overlay(scene)` is the two variables every Git call of the facts runs with,
///  naming the repository and its worktree.
/// Why:
///  The policies ask Git about a repository rather than about the folder a test happens to
///  run in,
///  and a control never changes this process's own directory (issue 6454).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function overlay(scene: Scene): [string, string][]
/// ```
fn overlay(scene: &Scene) -> Vec<(OsString, OsString)> {
    return vec![
        (
            OsString::from("GIT_DIR"),
            scene.repo.join(".git").into_os_string(),
        ),
        (
            OsString::from("GIT_WORK_TREE"),
            scene.repo.as_os_str().to_os_string(),
        ),
    ];
}

/// What:
///  `committed_bytes(scene, path)` reads the blob the commit's own index holds for `path`,
///  which is the content the scratch commit writes.
/// Why:
///  The wrapper writes its commit's tree from this index,
///  so these bytes are the ones a person
///  would find in the commit,
///  and no read of the worktree can say what they are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function committedBytes(scene: Scene, path: string): Uint8Array
/// ```
fn committed_bytes(
    scene: &Scene,
    path: &str,
) -> Vec<u8> {
    let object: String = format!(":{path}");
    let mut show: GitRequest = GitRequest::new(scene.repo.as_path(), &["cat-file", "blob"]);
    // `push(..)` appends the object to ask for, after the command words.
    show.arguments.push(OsString::from(object.as_str()));
    show.without_prefix = true;
    show.index = Some(scene.indexes.commit_index.clone());
    show.object_directory = Some(scene.indexes.object_directory.clone());
    let answer = run_git(&git_context(), &show).expect("the commit's index answers");
    assert!(
        answer.succeeded(),
        "the commit's index holds no bytes for {path}: {}",
        answer.error_text()
    );
    return answer.stdout;
}

/// What:
///  `forward_findings(scene, mode, pathspecs)` builds the commit's own index the way a commit
///  builds it,
///  asks the `final-newline` policy the question the stage before forwarding asks,
///  and answers with the paths the findings name.
///  `mode` is which index the commit commits,
///  and `pathspecs` names the paths an explicit-path commit selects.
/// Why:
///  This is the whole of what the reported warning came from:
///  the entries the commit prepared,
///  compared with its recorded base,
///  read by the rule that warned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function forwardFindings(scene: Scene, mode: TransactionMode, pathspecs: string[]): string[]
/// ```
fn forward_findings(
    scene: &Scene,
    mode: TransactionMode,
    pathspecs: &[OsString],
) -> Vec<String> {
    let selection: IndexSelection<'_> = IndexSelection {
        mode,
        pathspecs,
        pathspec_file: None,
        pathspec_file_nul: false,
        stage_into_index: false,
        stage_tracked_changes: false,
        base_revision: scene.base.as_str(),
    };
    initialize_commit_index(
        &git_context(),
        scene.repo.as_path(),
        scene.repo.as_path(),
        scene.repo.join(".git").join("index").as_path(),
        &scene.indexes,
        &selection,
    )
    .expect("the commit's own index is staged");
    let changed: Vec<Vec<u8>> = list_changed_index_paths(
        &git_context(),
        scene.repo.as_path(),
        scene.indexes.commit_index.as_path(),
        scene.base.as_str(),
        scene.indexes.object_directory.as_path(),
    )
    .expect("the commit's own index lists the paths it changes");
    // What: the rule facts of this repository, wrapped in the two variables that name the commit's own index.
    // Why:  the commit hands its policies `PrivateIndexFacts`, and a control that handed them the plain facts
    //       would read the person's index and answer about bytes this commit never writes.
    let private = PrivateIndexFacts::new(
        git_facts(Path::new(REAL_GIT), &[], overlay(scene).as_slice()),
        scene.indexes.commit_index.as_path(),
        scene.indexes.object_directory.as_path(),
    );
    let findings: Vec<PolicyFinding> = findings_of(
        private,
        os_arguments(&[]),
        LifecycleContent::Requested(staged_at_base(
            parse_object_id(scene.base.as_bytes()),
            changed.as_slice(),
        )),
    );
    return named_paths(findings.as_slice());
}

/// What:
///  `add_findings(scene)` asks the `final-newline` policy what `git add -- notes.txt` would stage,
///  which is the read-only lifecycle whose warning a person sees before a commit is forwarded.
/// Why:
///  That lifecycle corrects nothing,
///  so its finding is the one that tells a person to edit the file
///  themselves,
///  and a warning with that wording is about bytes nothing has corrected yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function addFindings(scene: Scene): PolicyFinding[]
/// ```
fn add_findings(scene: &Scene) -> Vec<PolicyFinding> {
    let mut arguments: Vec<OsString> = vec![OsString::from("add")];
    arguments.extend(the_file());
    return findings_of(
        git_facts(Path::new(REAL_GIT), &[], overlay(scene).as_slice()),
        arguments,
        LifecycleContent::Requested(CandidateRequest::Add(vec![
            OsString::from("--"),
            OsString::from(COMMITTED_FILE),
        ])),
    );
}

/// What:
///  `findings_of(facts, arguments, candidates)` runs the shipped `final-newline` check at the
///  pre-forward point and answers the findings it made.
///  `facts` is the facts provider the lifecycle hands its policies,
///  and `arguments` are the words
///  of the command a person typed.
/// Why:
///  Both lifecycles of this file ask the same rule the same question and differ only in the facts
///  and the candidates,
///  so the one call says what either answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsOf(facts: RepositoryFacts, arguments: string[], candidates: LifecycleContent): PolicyFinding[]
/// ```
fn findings_of<F: RepositoryFacts>(
    facts: F,
    arguments: Vec<OsString>,
    candidates: LifecycleContent,
) -> Vec<PolicyFinding> {
    let mut checks = shipped_checks(
        facts,
        arguments,
        candidates,
        Vec::<ToolCacheDirectory>::new(),
        &StartDirectory::at(Path::new("/")),
    );
    // `match` unpacks the policy's answer; the `Findings` arm is the list a person would read.
    match checks.check(PolicyId::FinalNewline, Trigger::PreForward) {
        PolicyOutcome::Findings(findings) => return findings,
        other => {
            panic!("the final-newline policy answered neither findings nor nothing: {other:?}")
        }
    }
}

/// What:
///  `named_paths(findings)` is the pathname of each finding,
///  in the order the findings came.
/// Why:
///  A control about one named file reads the paths alone,
///  and a finding that names no path is
///  dropped here because these policies always name the file they are about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namedPaths(findings: PolicyFinding[]): string[]
/// ```
fn named_paths(findings: &[PolicyFinding]) -> Vec<String> {
    let mut named: Vec<String> = Vec::new();
    for found in findings {
        // `.as_ref()` lends the optional path; `Some` is "this finding names a file".
        if let Some(path) = found.path.as_ref() {
            named.push(String::from(path.text()));
        }
    }
    return named;
}

/// What:
///  `is_the_committed_file(path)` says whether one finding names the file these controls commit.
/// Why:
///  The crate refuses a closure with an implicit return,
///  so this predicate is a named function,
///  as `is_slash` is beside the final-newline rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isTheCommittedFile = (path: string) => path === COMMITTED_FILE;
/// ```
fn is_the_committed_file(path: &String) -> bool {
    return *path == COMMITTED_FILE;
}

/// What:
///  `assert_the_finding_follows_the_committed_bytes(scene, mode, pathspecs)` holds one state to
///  the rule this file is about:
///  the finding names `COMMITTED_FILE` exactly when the bytes the commit writes are not canonical.
/// Why:
///  A state that warns about canonical bytes,
///  or stays silent about bytes the commit records
///  uncorrected,
///  fails here and the message names both the bytes read and the paths answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertTheFindingFollowsTheCommittedBytes(scene, mode, pathspecs): void
/// ```
fn assert_the_finding_follows_the_committed_bytes(
    scene: &Scene,
    mode: TransactionMode,
    pathspecs: &[OsString],
) {
    let named: Vec<String> = forward_findings(scene, mode, pathspecs);
    let committed: Vec<u8> = committed_bytes(scene, COMMITTED_FILE);
    let owes_a_finding: bool = normalized_final_newline(committed.as_slice()).is_some();
    let found: bool = named.iter().any(is_the_committed_file);
    assert_eq!(
        found,
        owes_a_finding,
        "{mode:?} with {pathspecs:?}: the commit writes {:?} for {COMMITTED_FILE}, and the stage \
         answered {named:?}",
        String::from_utf8_lossy(committed.as_slice())
    );
}

/// The pathspec that names the committed file,
/// as an explicit-path commit's command line names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theFile: string[] = ['notes.txt'];
/// ```
fn the_file() -> Vec<OsString> {
    return vec![OsString::from(COMMITTED_FILE)];
}

/// A new file that ends with exactly one line feed,
/// staged the way the reporting round staged its files,
/// draws no finding from the commit that commits it.
#[test]
fn a_staged_new_file_that_ends_with_one_line_feed_draws_no_finding() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_staged_new_file_that_ends_with_one_line_feed_draws_no_finding",
        || {
            let scene: Scene = a_scene("commit-stage-newline");
            std::fs::write(scene.repo.join(COMMITTED_FILE), b"export const one = 1;\n")
                .expect("the staged file");
            git(scene.repo.as_path(), &["add", "--", COMMITTED_FILE]);
            // `&[]` is "this commit names no path": it commits the whole index.
            assert_the_finding_follows_the_committed_bytes(&scene, TransactionMode::Index, &[]);
            assert_eq!(
                forward_findings(&scene, TransactionMode::Index, &[]),
                Vec::<String>::new(),
                "the file ends with exactly one line feed"
            );
            remove(scene.root.as_path());
        },
    );
}

/// A new file the commit names and nothing staged is read from the worktree,
/// which is where that commit takes it from,
///  and the same file without its line feed draws the finding.
#[test]
fn a_new_file_the_commit_names_and_nothing_staged_is_read_from_the_worktree() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_new_file_the_commit_names_and_nothing_staged_is_read_from_the_worktree",
        || {
            let scene: Scene = a_scene("commit-path-newline");
            std::fs::write(scene.repo.join(COMMITTED_FILE), b"export const one = 1;\n")
                .expect("the named file");
            assert_the_finding_follows_the_committed_bytes(
                &scene,
                TransactionMode::ExplicitPath,
                the_file().as_slice(),
            );
            assert_eq!(
                forward_findings(&scene, TransactionMode::ExplicitPath, the_file().as_slice()),
                Vec::<String>::new(),
                "the worktree copy ends with exactly one line feed"
            );
            std::fs::write(scene.repo.join(COMMITTED_FILE), b"export const one = 1")
                .expect("the named file without its line feed");
            assert_the_finding_follows_the_committed_bytes(
                &scene,
                TransactionMode::ExplicitPath,
                the_file().as_slice(),
            );
            assert_eq!(
                forward_findings(&scene, TransactionMode::ExplicitPath, the_file().as_slice()),
                vec![String::from(COMMITTED_FILE)],
                "the bytes this commit writes end with no line feed at all"
            );
            remove(scene.root.as_path());
        },
    );
}

/// A file staged and then written again is read as the commit writes it:
///  the worktree's bytes for a commit that names it,
///  and the index's bytes for a commit of the whole index.
#[test]
fn a_file_written_again_after_it_was_staged_is_read_as_the_commit_writes_it() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_file_written_again_after_it_was_staged_is_read_as_the_commit_writes_it",
        || {
            // What: the index holds the file without its line feed, and the worktree holds it with one.
            // Why:  a commit that names the file writes the worktree's bytes, so the canonical bytes
            //       are what a person would find, and the older index entry must draw no warning.
            let scene: Scene = a_scene("commit-restaged-newline");
            std::fs::write(scene.repo.join(COMMITTED_FILE), b"one\ntwo").expect("no line feed");
            git(scene.repo.as_path(), &["add", "--", COMMITTED_FILE]);
            std::fs::write(scene.repo.join(COMMITTED_FILE), b"one\ntwo\n").expect("one line feed");
            assert_the_finding_follows_the_committed_bytes(
                &scene,
                TransactionMode::ExplicitPath,
                the_file().as_slice(),
            );
            assert_eq!(
                forward_findings(&scene, TransactionMode::ExplicitPath, the_file().as_slice()),
                Vec::<String>::new(),
                "the commit writes the worktree's canonical bytes"
            );
            remove(scene.root.as_path());

            // What: the index holds the canonical bytes and the worktree holds the file without its
            //       line feed.
            // Why:  a commit of the whole index writes the index's bytes, so that commit records the
            //       canonical file even though the worktree is uncorrected.
            let whole: Scene = a_scene("commit-index-newline");
            std::fs::write(whole.repo.join(COMMITTED_FILE), b"one\ntwo\n").expect("one line feed");
            git(whole.repo.as_path(), &["add", "--", COMMITTED_FILE]);
            std::fs::write(whole.repo.join(COMMITTED_FILE), b"one\ntwo").expect("no line feed");
            assert_the_finding_follows_the_committed_bytes(&whole, TransactionMode::Index, &[]);
            assert_eq!(
                forward_findings(&whole, TransactionMode::Index, &[]),
                Vec::<String>::new(),
                "the commit writes the index's canonical bytes"
            );
            // What: the same state committed by a command that names the file.
            // Why:  that commit writes the worktree's bytes, so the missing line feed is recorded
            //       and the warning is the truth about what lands.
            assert_the_finding_follows_the_committed_bytes(
                &whole,
                TransactionMode::ExplicitPath,
                the_file().as_slice(),
            );
            assert_eq!(
                forward_findings(&whole, TransactionMode::ExplicitPath, the_file().as_slice()),
                vec![String::from(COMMITTED_FILE)],
                "this commit writes the worktree's bytes, which end with no line feed"
            );
            remove(whole.root.as_path());
        },
    );
}

/// A file added with intent only is read for the bytes the commit records for it.
#[test]
fn an_intent_to_add_file_is_read_for_the_bytes_the_commit_records() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "an_intent_to_add_file_is_read_for_the_bytes_the_commit_records",
        || {
            let scene: Scene = a_scene("commit-intent-newline");
            std::fs::write(scene.repo.join(COMMITTED_FILE), b"one\ntwo").expect("no line feed");
            git(
                scene.repo.as_path(),
                &["add", "--intent-to-add", "--", COMMITTED_FILE],
            );
            assert_the_finding_follows_the_committed_bytes(&scene, TransactionMode::Index, &[]);
            remove(scene.root.as_path());
        },
    );
}

/// A repository whose attributes convert line ends is read for the bytes Git records.
#[test]
fn a_repository_that_converts_line_ends_is_read_for_the_bytes_git_records() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_repository_that_converts_line_ends_is_read_for_the_bytes_git_records",
        || {
            let scene: Scene = a_scene("commit-attributes-newline");
            std::fs::write(scene.repo.join(".gitattributes"), b"* text=auto\n")
                .expect("the attributes");
            std::fs::write(scene.repo.join(COMMITTED_FILE), b"one\r\ntwo\r\n")
                .expect("the file of Windows line ends");
            git(scene.repo.as_path(), &["add", "--", COMMITTED_FILE]);
            assert_the_finding_follows_the_committed_bytes(&scene, TransactionMode::Index, &[]);
            assert_eq!(
                forward_findings(&scene, TransactionMode::Index, &[]),
                Vec::<String>::new(),
                "the attribute stores the file with line feeds, so Git's bytes are canonical"
            );
            remove(scene.root.as_path());
        },
    );
}

/// A new file that ends with exactly one line feed draws no finding from the `git add` that stages it,
///  and the same file without its line feed is named,
///  by a finding that corrects nothing
///  and tells the person to edit the file.
#[test]
fn the_add_that_stages_a_new_file_names_only_bytes_without_one_line_feed() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "the_add_that_stages_a_new_file_names_only_bytes_without_one_line_feed",
        || {
            let scene: Scene = a_scene("commit-add-newline");
            std::fs::write(scene.repo.join(COMMITTED_FILE), b"one\ntwo\n")
                .expect("the file with one line feed");
            assert_eq!(
                named_paths(add_findings(&scene).as_slice()),
                Vec::<String>::new(),
                "the file this add stages ends with exactly one line feed"
            );
            // What: the same file, written again without its line feed, then staged by the same add.
            // Why:  the warning a person sees before a commit is forwarded is this one, and it must name
            //       the bytes this add would stage rather than anything the commit will do with them.
            std::fs::write(scene.repo.join(COMMITTED_FILE), b"one\ntwo")
                .expect("the file without its line feed");
            let findings: Vec<PolicyFinding> = add_findings(&scene);
            assert_eq!(
                named_paths(findings.as_slice()),
                vec![String::from(COMMITTED_FILE)],
                "the bytes this add stages end with no line feed at all"
            );
            assert!(
                !findings[0].fix_available,
                "an add corrects nothing, so its finding offers no fix and tells the person to edit the file"
            );
            remove(scene.root.as_path());
        },
    );
}
