//! Controls for the pool of finished shadow repositories:
//!  a fill makes one,
//! a claim takes it once,
//!  a member that is not what a creation would make is refused,
//! and the commits two members build are the same commit.

use super::*;
use crate::capture_invocation::{InvocationCapture, capture_invocation_base, capture_invocation_layout};
use crate::commit_build_part_durations::BuildPartClock;
use crate::commit_index::{IndexSelection, initialize_commit_index};
use crate::commit_preparation::{PreparationRequest, read_prepared_commit, run_native_commit};
use crate::private_index_request::CaptureIndexes;
use crate::shadow_create::{ShadowTemplate, create_shadow_template, take_or_create_shadow};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::transaction_git::GitContext;
use crate::transaction_journal::TransactionMode;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Captures the invocation of a repository's working directory,
/// as the transaction does.
fn capture_of(repo: &Path) -> InvocationCapture {
    let context: GitContext = git_context();
    let layout = capture_invocation_layout(&context, repo, false).expect("layout");
    return capture_invocation_base(&context, repo, layout).expect("capture");
}

/// The facts one invocation's shadow is made from.
fn template_of(repo: &Path) -> ShadowTemplate {
    return ShadowTemplate::of(&git_context(), &capture_of(repo).layout).expect("template");
}

/// Everything a shadow holds,
/// by name,
/// with what a link points at,
/// sorted so two shadows compare entry for entry.
fn shape_of(directory: &Path) -> Vec<String> {
    let mut shape: Vec<String> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![PathBuf::from("")];
    while let Some(relative) = pending.pop() {
        let reading: PathBuf = directory.join(&relative);
        let Ok(entries) = std::fs::read_dir(reading.as_path()) else { continue };
        for entry in entries.flatten() {
            let name: String = entry.file_name().to_string_lossy().into_owned();
            let child: PathBuf = relative.join(&name);
            let metadata = std::fs::symlink_metadata(entry.path()).expect("metadata");
            if metadata.file_type().is_symlink() {
                let target = std::fs::read_link(entry.path()).expect("target");
                shape.push(format!("link {name}: {}", target.display()));
                continue;
            }
            if metadata.is_dir() {
                shape.push(format!("dir  {name}"));
                pending.push(child);
                continue;
            }
            let bytes = std::fs::read(entry.path()).expect("bytes");
            shape.push(format!("file {name}: {} bytes {:x}", bytes.len(), bytes.len()));
        }
    }
    shape.sort();
    return shape;
}

/// The member a fill makes for a repository.
fn a_member(repo: &Path) -> (ShadowTemplate, PathBuf) {
    let template: ShadowTemplate = template_of(repo);
    let member: Option<PathBuf> = fill(&git_context(), &template).expect("fill");
    return (template, member.expect("a member"));
}

/// A repository with one commit on `main`.
fn one_commit_repository(root: &Path, name: &str) -> PathBuf {
    let repo: PathBuf = repository(root, name);
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    return repo;
}

/// A filled member is what a creation makes,
/// a claim takes it,
/// and the pool is empty afterwards.
#[test]
fn a_fill_makes_what_a_creation_makes_and_a_claim_takes_it() {
    let root: PathBuf = fixture("shadow-pool-take");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let (template, member) = a_member(repo.as_path());
    assert_eq!(members(pool_root(&template).as_path()), vec![member.clone()], "one member");
    let same_creation: PathBuf = root.join("created");
    let mut clock: BuildPartClock = BuildPartClock::begin("test");
    create_shadow_template(&git_context(), &template, same_creation.as_path(), &mut clock).expect("creation");
    assert_eq!(shape_of(member.as_path()), shape_of(same_creation.as_path()), "a member is a creation's tree");
    assert_eq!(
        std::fs::read_to_string(member.join("config").as_path()).expect("config"),
        std::fs::read_to_string(same_creation.join("config").as_path()).expect("config"),
        "the same config"
    );
    let destination: PathBuf = root.join("claimed");
    assert!(take(&template, destination.as_path()), "the member is taken");
    assert!(members(pool_root(&template).as_path()).is_empty(), "the pool gave it up");
    assert_eq!(shape_of(destination.as_path()), shape_of(same_creation.as_path()), "the claim is the same tree");
    remove(root.as_path());
}

/// A pool with no member leaves the commit to make its own creation,
/// and says so by answering nothing.
#[test]
fn an_empty_pool_leaves_the_creation_to_the_commit() {
    let root: PathBuf = fixture("shadow-pool-empty");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let template: ShadowTemplate = template_of(repo.as_path());
    let destination: PathBuf = root.join("shadow");
    assert!(!take(&template, destination.as_path()), "nothing to take");
    assert!(std::fs::symlink_metadata(destination.as_path()).is_err(), "and nothing is made");
    remove(root.as_path());
}

/// A member that holds something a creation never makes is refused and removed,
/// so the next claim does not trust it either.
#[test]
fn a_member_the_creation_would_not_make_is_refused_and_removed() {
    let root: PathBuf = fixture("shadow-pool-refused");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let (template, member) = a_member(repo.as_path());
    std::fs::write(member.join("MERGE_HEAD").as_path(), b"0123456789012345678901234567890123456789\n").expect("state");
    let destination: PathBuf = root.join("shadow");
    assert!(!take(&template, destination.as_path()), "a member with state is refused");
    assert!(std::fs::symlink_metadata(member.as_path()).is_err(), "the refused member is removed");
    assert!(std::fs::symlink_metadata(destination.as_path()).is_err(), "and is not left at the claim's path");
    remove(root.as_path());
}

/// Two claims of one member take it once,
/// and the loser makes its own creation.
#[test]
fn a_member_is_claimed_once() {
    let root: PathBuf = fixture("shadow-pool-once");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let (template, _member) = a_member(repo.as_path());
    assert!(take(&template, root.join("first").as_path()), "the first claim takes it");
    assert!(!take(&template, root.join("second").as_path()), "the second finds nothing");
    remove(root.as_path());
}

/// A member made for another worktree root is refused,
/// because its config names the worktree the commit runs in.
#[test]
fn a_member_of_another_worktree_root_is_refused() {
    let root: PathBuf = fixture("shadow-pool-other-worktree");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let (mut template, member) = a_member(repo.as_path());
    // A worktree at another path is another root, and the same pool, because the key is the worktree's Git directory.
    template.repository_root = root.join("elsewhere");
    assert!(!take(&template, member.as_path()), "its config names another worktree");
    assert!(std::fs::symlink_metadata(member.as_path()).is_err(), "and it is removed");
    remove(root.as_path());
}

/// A fill keeps the pool at its limit and makes nothing more.
#[test]
fn a_fill_keeps_the_pool_at_its_limit() {
    let root: PathBuf = fixture("shadow-pool-limit");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let template: ShadowTemplate = template_of(repo.as_path());
    for _round in 0..POOL_LIMIT + 2 {
        let _ = fill(&git_context(), &template).expect("fill");
    }
    assert_eq!(members(pool_root(&template).as_path()).len(), POOL_LIMIT, "the pool's limit holds");
    remove(root.as_path());
}

/// A fill that was killed leaves a name no claim takes,
/// and the next fill removes it once it is old.
#[test]
fn an_abandoned_fill_is_removed_by_a_later_fill() {
    let root: PathBuf = fixture("shadow-pool-abandoned");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let template: ShadowTemplate = template_of(repo.as_path());
    let pool: PathBuf = pool_root(&template);
    std::fs::create_dir_all(pool.as_path()).expect("pool");
    let abandoned: PathBuf = pool.join(format!("{FILLING_PREFIX}abandoned"));
    std::fs::create_dir(abandoned.as_path()).expect("half a member");
    assert!(!take(&template, root.join("shadow").as_path()), "a fill is not a member");
    sweep_abandoned(pool.as_path(), SystemTime::now() + Duration::from_secs(3600));
    assert!(std::fs::symlink_metadata(abandoned.as_path()).is_err(), "the abandoned fill is removed");
    remove(root.as_path());
}

/// A reftable repository is never pooled:
/// its shadow is Git's own tables,
/// which no member holds.
#[test]
fn a_reftable_repository_is_never_pooled() {
    let root: PathBuf = fixture("shadow-pool-reftable");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(repo.as_path()).expect("repo");
    git(repo.as_path(), &["init", "--quiet", "--ref-format=reftable", "--initial-branch=main"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let template: ShadowTemplate = template_of(repo.as_path());
    assert_eq!(fill(&git_context(), &template).expect("fill"), None, "nothing is filled");
    assert!(!take(&template, root.join("shadow").as_path()), "and nothing is taken");
    remove(root.as_path());
}

/// What:
///  The commit a member builds is the commit a creation builds:
///  the same object name and the same stored bytes,
///  from the same private index and the same message.
/// Why:
///  A commit that takes a finished shadow must produce what a commit that makes one produces,
///  which is what the owner's "a member is exactly what a creation makes,
/// or it is refused" asks for.
///  The identities and the dates are fixed,
/// so the two commits are the same bytes rather than only the same tree.
#[test]
fn the_commit_a_member_builds_is_the_commit_a_creation_builds() {
    let root: PathBuf = fixture("shadow-pool-parity");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change");
    let mut context: GitContext = git_context();
    for (name, value) in [
        ("GIT_AUTHOR_DATE", "1700000000 +0000"),
        ("GIT_COMMITTER_DATE", "1700000000 +0000"),
    ] {
        context.overlay.push((std::ffi::OsString::from(name), std::ffi::OsString::from(value)));
    }
    let capture: InvocationCapture = capture_invocation_base(
        &context,
        repo.as_path(),
        capture_invocation_layout(&context, repo.as_path(), false).expect("layout"),
    )
    .expect("capture");
    let common: PathBuf = root.join("transaction");
    std::fs::create_dir(common.as_path()).expect("transaction");
    let private: PathBuf = common.join("commit.index");
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: common.join("captured.index"),
        commit_index: private.clone(),
        object_directory: repo.join(".git").join("objects"),
    };
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
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
    initialize_commit_index(
        &context,
        repo.as_path(),
        capture.layout.real_index_path.as_path(),
        &indexes,
        &selection,
    )
    .expect("private index");
    let built: Vec<(Vec<u8>, String)> = ["created", "pooled"]
        .iter()
        .map(|arm| {
            let scratch: PathBuf = root.join(arm);
            std::fs::create_dir(scratch.as_path()).expect("scratch");
            let index: PathBuf = scratch.join("commit.index");
            std::fs::copy(private.as_path(), index.as_path()).expect("index copy");
            let template: ShadowTemplate = ShadowTemplate::of(&context, &capture.layout).expect("template");
            let pooled: bool = *arm == "pooled";
            if pooled {
                fill(&context, &template).expect("fill");
                assert_eq!(members(pool_root(&template).as_path()).len(), 1, "one member waits");
            }
            let shadow: PathBuf = scratch.join("shadow");
            let mut clock: BuildPartClock = BuildPartClock::begin("test");
            take_or_create_shadow(&context, repo.as_path(), &capture, shadow.as_path(), scratch.as_path(), false, &mut clock)
                .expect("shadow");
            if pooled {
                assert!(
                    members(pool_root(&template).as_path()).is_empty(),
                    "the pooled arm took the member rather than making a creation"
                );
            }
            let request: PreparationRequest = PreparationRequest {
                capture: &capture,
                shadow: shadow.as_path(),
                commit_index: index.as_path(),
                scratch: scratch.as_path(),
                global_args: &[],
                commit_args: &[String::from("-m"), String::from("message")],
                object_directory: None,
            };
            let code: i32 = run_native_commit(&context, repo.as_path(), &request).expect("git commit");
            assert_eq!(code, 0, "{arm} commit");
            let prepared = read_prepared_commit(&context, shadow.as_path()).expect("prepared");
            return (prepared.raw, prepared.oid);
        })
        .collect();
    assert_eq!(built[0].1, built[1].1, "the same commit object");
    assert_eq!(built[0].0, built[1].0, "the same stored bytes");
    remove(root.as_path());
}
