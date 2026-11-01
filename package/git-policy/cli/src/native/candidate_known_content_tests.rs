//! What:
//!  Controls for the lookup of candidates whose content a parent commit's tree already holds.
//! Why:
//!  The lookup decides which bytes a policy never reads,
//!  so its basis per request,
//!  its comparison by
//!      object name alone,
//!  its stop at published history and its refusal to vouch for corrected files each get a
//!      control on a disposable repository with real Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(known.entry(indexOf('copy.txt'))?.parent).toBe(head); expect(known.entry(indexOf('edited.txt'))).toBeUndefined();
//! ```
#![cfg(unix)]

/// Import the items under test.
use super::{ContentBasis, KnownContent, KnownEntry, find_known_content};
use crate::candidate_object::{CandidateMode, ObjectId, parse_object_id};
use crate::candidate_prediction::{CandidateRequest, PushedScope};
use crate::candidate_store::CandidateStore;
use crate::candidate_version::CandidateVersion;
use crate::policy_content::{ContentState, Correction, LifecycleContent};
use crate::policy_test_support::{ScriptedFacts, main_worktree, scripted_facts};
use crate::test_support::{REAL_GIT, fixture, git, git_text, remove, repository};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// An object name from hexadecimal text.
fn name(hex: &str) -> ObjectId {
    return parse_object_id(hex.as_bytes()).expect("an object name");
}

/// The object name `revision` resolves to in `repo`.
fn oid(repo: &Path, revision: &str) -> ObjectId {
    return name(git_text(repo, &["rev-parse", revision]).as_str());
}

/// A store over real Git that runs in `repo`.
fn store_in(repo: &Path) -> CandidateStore {
    return CandidateStore::new(
        Path::new(REAL_GIT),
        &[OsString::from("-C"), repo.as_os_str().to_os_string()],
        &[],
    );
}

/// The request for every worktree file of the repository.
fn whole_worktree() -> CandidateRequest {
    return CandidateRequest::Add(vec![OsString::from("--all")]);
}

/// Facts that prepare candidates in `repo` with real Git.
fn facts_in(repo: &Path) -> ScriptedFacts {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.location = Ok(main_worktree(
        repo.to_str().expect("fixture path is UTF-8"),
        "",
    ));
    facts.candidates_repository = Some(repo.to_path_buf());
    return facts;
}

/// The position of the candidate at `path` in `version`.
fn index_of(version: &CandidateVersion, path: &str) -> usize {
    return version
        .candidate_at_path(path.as_bytes())
        .expect("a candidate at the path")
        .identity
        .index;
}

/// Each request kind has the basis the lifecycle defines:
///  a direct check compares with nothing,
///  the staged requests with their baseline,
///  a landed commit with its parents,
///  a push with the parents outside the pushed commits unless a tip has no published history.
#[test]
fn the_basis_follows_the_kind_of_request() {
    let commit: ObjectId = name(&"1".repeat(40));
    let tip: ObjectId = name(&"2".repeat(40));
    assert_eq!(
        ContentBasis::of(&CandidateRequest::Direct(Vec::new())),
        ContentBasis::Nothing
    );
    assert_eq!(
        ContentBasis::of(&CandidateRequest::Add(Vec::new())),
        ContentBasis::Baseline
    );
    assert_eq!(
        ContentBasis::of(&CandidateRequest::Staged(Vec::new())),
        ContentBasis::Baseline
    );
    assert_eq!(
        ContentBasis::of(&CandidateRequest::StagedAtBase {
            base: None,
            pathspecs: Vec::new(),
        }),
        ContentBasis::Baseline
    );
    assert_eq!(
        ContentBasis::of(&CandidateRequest::Landed(commit.clone())),
        ContentBasis::ParentsOutside(vec![commit.clone()])
    );
    assert_eq!(
        ContentBasis::of(&CandidateRequest::Pushed(PushedScope {
            commits: vec![commit.clone()],
            trees: Vec::new(),
        })),
        ContentBasis::ParentsOutside(vec![commit.clone()])
    );
    assert_eq!(
        ContentBasis::of(&CandidateRequest::Pushed(PushedScope {
            commits: vec![commit],
            trees: vec![tip],
        })),
        ContentBasis::Nothing
    );
}

/// The decision's text names the masked pathname,
///  the blob and the parent,
///  and nothing else.
#[test]
fn a_decision_is_described_with_its_inputs() {
    let entry: KnownEntry = KnownEntry {
        object: name(&"a".repeat(40)),
        parent: name(&"b".repeat(40)),
    };
    assert_eq!(
        entry.describe("dir/[REDACTED]"),
        format!(
            "skipped the content of dir/[REDACTED]: blob {} is already in the tree of commit {}; only the pathname was checked.",
            "a".repeat(40),
            "b".repeat(40)
        )
    );
    assert!(KnownContent::default().is_empty());
    assert_eq!(KnownContent::default().len(), 0);
    assert_eq!(KnownContent::default().entry(0), None);
}

/// Content is known by object name alone:
///  a copy at a new path is known,
///  a file with one changed byte is not,
///  a new file is not,
///  and a file this invocation corrected is not even when its object is.
#[test]
fn content_is_known_by_object_name_and_never_when_corrected() {
    let root: PathBuf = fixture("known-content-head");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"known\nbytes\n").expect("a");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=known"]);
    let head: ObjectId = oid(repo.as_path(), "HEAD");
    std::fs::write(repo.join("copy.txt"), b"known\nbytes\n").expect("copy");
    std::fs::write(repo.join("edited.txt"), b"known\nbyteS\n").expect("edited");
    std::fs::write(repo.join("fresh.txt"), b"never seen\n").expect("fresh");
    let request: CandidateRequest = whole_worktree();
    let lifecycle: LifecycleContent = LifecycleContent::Requested(request.clone());
    let mut content: ContentState = ContentState::new();
    let mut facts: ScriptedFacts = facts_in(repo.as_path());
    let version: Rc<CandidateVersion> = content
        .version(&lifecycle, &mut facts)
        .expect("prepared")
        .expect("candidates");
    let known: KnownContent = content.known_content(&request, &[]).expect("known content");
    assert_eq!(
        known.entry(index_of(&version, "copy.txt")),
        Some(&KnownEntry {
            object: oid(repo.as_path(), "HEAD:a.txt"),
            parent: head,
        })
    );
    assert_eq!(known.entry(index_of(&version, "edited.txt")), None);
    assert_eq!(known.entry(index_of(&version, "fresh.txt")), None);
    assert_eq!(known.len(), 1);
    // A correction replaces the file's bytes, so the object name no longer describes them.
    content.apply(vec![Correction {
        path: b"copy.txt".to_vec(),
        mode: CandidateMode::Regular,
        before: Rc::from(&b"known\nbytes\n"[..]),
        after: Rc::from(&b"known\nbytes\n\n"[..]),
    }]);
    let corrected: KnownContent = content.known_content(&request, &[]).expect("known content");
    assert!(corrected.is_empty(), "{corrected:?}");
    // A direct check reads what the files hold now and compares with nothing.
    let mut direct: ContentState = ContentState::new();
    let direct_request: CandidateRequest = CandidateRequest::Direct(vec![OsString::from(":/")]);
    direct
        .version(
            &LifecycleContent::Requested(direct_request.clone()),
            &mut facts,
        )
        .expect("prepared");
    assert!(
        direct
            .known_content(&direct_request, &[])
            .expect("known content")
            .is_empty()
    );
    remove(root.as_path());
}

/// Before preparation the lookup refuses to answer:
///  nothing may be treated as known for candidates that were never listed.
#[test]
fn nothing_is_known_before_the_candidates_are_prepared() {
    let mut content: ContentState = ContentState::new();
    assert!(content.known_content(&whole_worktree(), &[]).is_err());
}

/// The parents outside the given commits are listed once each,
///  in the commits' order,
///  and a root commit has none.
#[test]
fn parents_outside_the_given_commits_are_listed_once() {
    let root: PathBuf = fixture("known-content-parents");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let first: ObjectId = oid(repo.as_path(), "HEAD");
    git(repo.as_path(), &["switch", "--quiet", "--create", "side"]);
    git(
        repo.as_path(),
        &["commit", "--quiet", "--allow-empty", "--message=side"],
    );
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    git(
        repo.as_path(),
        &["commit", "--quiet", "--allow-empty", "--message=main"],
    );
    git(
        repo.as_path(),
        &["merge", "--quiet", "--no-ff", "--no-edit", "side"],
    );
    let merge: ObjectId = oid(repo.as_path(), "HEAD");
    let main_tip: ObjectId = oid(repo.as_path(), "HEAD^1");
    let side_tip: ObjectId = oid(repo.as_path(), "HEAD^2");
    let mut subject: CandidateStore = store_in(repo.as_path());
    // The merge has two parents, and both are outside the commits asked about.
    assert_eq!(
        subject.parents_outside(std::slice::from_ref(&merge)).expect("parents"),
        vec![main_tip.clone(), side_tip.clone()]
    );
    // A parent that is itself asked about is not outside; the other commits' parents still count once.
    assert_eq!(
        subject
            .parents_outside(&[merge.clone(), side_tip.clone(), main_tip.clone()])
            .expect("parents"),
        vec![first.clone()]
    );
    // A root commit has no parent, and no commits ask nothing.
    assert_eq!(
        subject.parents_outside(&[first]).expect("parents"),
        Vec::<ObjectId>::new()
    );
    assert_eq!(
        subject.parents_outside(&[]).expect("parents"),
        Vec::<ObjectId>::new()
    );
    remove(root.as_path());
}

/// A basis with no parent to compare with finds nothing and lists no tree.
#[test]
fn no_basis_means_nothing_is_known() {
    let root: PathBuf = fixture("known-content-nothing");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"bytes\n").expect("a");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=known"]);
    std::fs::write(repo.join("copy.txt"), b"bytes\n").expect("copy");
    let mut content: ContentState = ContentState::new();
    let mut facts: ScriptedFacts = facts_in(repo.as_path());
    let lifecycle: LifecycleContent = LifecycleContent::Requested(whole_worktree());
    let version: Rc<CandidateVersion> = content
        .version(&lifecycle, &mut facts)
        .expect("prepared")
        .expect("candidates");
    let mut subject: CandidateStore = store_in(repo.as_path());
    let corrected: BTreeMap<Vec<u8>, crate::direct_fix_install::InstallChange> = BTreeMap::new();
    let found: KnownContent = find_known_content(
        &mut subject,
        &version,
        &ContentBasis::Nothing,
        &[],
        &corrected,
    )
    .expect("lookup");
    assert!(found.is_empty());
    remove(root.as_path());
}
