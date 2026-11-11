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
use crate::scratch_location::{SCRATCH_STALE_AFTER, prepare_repository_scratch};
use crate::wrapper_names_old_and_new::{REPOSITORY_SCRATCH, SERVICE_SCRATCH_PREFIX, SHADOW_POOL_FOLDER};
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
        capture.layout.repository_root.as_path(),
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

/// What:
///  The pool is its own folder beside the repository scratch directory,
///  and never an entry of it.
/// Why:
///  A commit's scratch directory is emptied once its landing is over,
///  and the check that it is empty is the one that catches leaked scratch
///  (`binary_service_scratch_tests.rs`,
///  `binary_markdown_tests.rs`).
///  A member is standing state the owner asked for,
///  so it stays where a scratch sweep,
/// a release of a commit's tree and a reclamation of a removed worktree cannot call it a leftover.
#[test]
fn the_pool_is_its_own_folder_beside_the_scratch_directory() {
    let root: PathBuf = fixture("shadow-pool-place");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    let scratch: PathBuf = common.join(REPOSITORY_SCRATCH.current);
    let (template, member) = a_member(repo.as_path());
    let pool: PathBuf = pool_root(&template);
    assert_eq!(pool, common.join(SHADOW_POOL_FOLDER).join("main"), "the pool's own folder, named after the worktree");
    assert_eq!(member.parent(), Some(pool.as_path()), "a member is inside the pool");
    assert!(!pool.starts_with(scratch.as_path()), "the pool is not inside the scratch directory");
    assert!(!member.starts_with(scratch.as_path()), "and neither is a member");
    assert!(pool.starts_with(common.as_path()), "the pool is the repository's own state, on its device");
    remove(root.as_path());
}

/// What:
///  A pooled shadow survives a sweep of the repository scratch directory,
///  while a stale entry of that directory does not.
/// Why:
///  A sweep of that directory runs at the start of each command that takes scratch
///  (`prepare_repository_scratch`),
///  and it removes an entry of it once the entry is old enough.
///  A member is not an entry of it at all,
///  so no age and no removal the sweep makes reaches one.
#[test]
fn a_pooled_shadow_survives_a_sweep_of_the_scratch_directory() {
    let root: PathBuf = fixture("shadow-pool-survives-sweep");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let (template, member) = a_member(repo.as_path());
    let common: PathBuf = repo.join(".git");
    let scratch: PathBuf = prepare_repository_scratch(common.as_path()).expect("the repository scratch directory");
    assert_eq!(scratch, common.join(REPOSITORY_SCRATCH.current), "the scratch directory a command takes");
    // The stale tree is one a killed command left, and the member beside the directory is older still.
    let stale: PathBuf = scratch.join(format!("{SERVICE_SCRATCH_PREFIX}a-command-that-was-killed"));
    std::fs::create_dir(stale.as_path()).expect("a stale tree");
    std::fs::File::open(stale.as_path()).expect("open").set_modified(SystemTime::now() - SCRATCH_STALE_AFTER * 3).expect("its age");
    std::fs::File::open(member.as_path()).expect("open").set_modified(SystemTime::now() - SCRATCH_STALE_AFTER * 3).expect("its age");
    let _again: PathBuf = prepare_repository_scratch(common.as_path()).expect("the scratch directory again");
    assert!(std::fs::symlink_metadata(stale.as_path()).is_err(), "the sweep removed the stale scratch tree");
    assert_eq!(members(pool_root(&template).as_path()), vec![member.clone()], "and left the pooled shadow alone");
    remove(root.as_path());
}

/// What:
///  A member stays usable when a common directory gains an entry no commit reads.
/// Why:
///  A repository with many agents collects these all the time
///  (`FETCH_HEAD`,
/// a timing-log generation,
/// another tool's file),
///  and a member that refused on one would send its commit back to the full creation
///  the pool exists to skip.
#[test]
fn a_member_stays_usable_when_an_entry_no_commit_reads_appears() {
    let root: PathBuf = fixture("shadow-pool-unread-state-appears");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let (template, member) = a_member(repo.as_path());
    let common: PathBuf = repo.join(".git");
    for name in ["FETCH_HEAD", "monochromatic-git-policy-cli-commit-timing.jsonl", "cli-git-captures"] {
        std::fs::write(common.join(name), b"state").expect("an entry no commit reads");
    }
    std::fs::create_dir_all(common.join("worktrees").join("another")).expect("another worktree's state");
    let destination: PathBuf = root.join("shadow");
    assert!(take(&template, destination.as_path()), "the member is still what this invocation's creation makes");
    for name in ["FETCH_HEAD", "monochromatic-git-policy-cli-commit-timing.jsonl", "cli-git-captures", "worktrees"] {
        assert!(std::fs::symlink_metadata(destination.join(name)).is_err(), "the claimed shadow holds no {name}");
    }
    assert!(std::fs::symlink_metadata(member.as_path()).is_err(), "and the pool gave the member up");
    remove(root.as_path());
}

/// What:
///  A member stays usable when a common directory loses an entry no commit reads.
/// Why:
///   The member holds a link to every entry the common directory had when it was filled;
///  a link to a name that is no longer there resolves to nothing,
/// which is the view a creation of this moment gives,
///  so the entry going away is no reason to refuse the member.
#[test]
fn a_member_stays_usable_when_an_entry_no_commit_reads_goes_away() {
    let root: PathBuf = fixture("shadow-pool-unread-state-goes");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    for name in ["FETCH_HEAD", "monochromatic-git-policy-cli-audit.jsonl"] {
        std::fs::write(common.join(name), b"state").expect("an entry no commit reads");
    }
    std::fs::create_dir_all(common.join("lost-found")).expect("an entry no commit reads");
    let (template, _member) = a_member(repo.as_path());
    for name in ["FETCH_HEAD", "monochromatic-git-policy-cli-audit.jsonl"] {
        std::fs::remove_file(common.join(name)).expect("gone");
    }
    std::fs::remove_dir_all(common.join("lost-found")).expect("gone");
    let destination: PathBuf = root.join("shadow");
    assert!(take(&template, destination.as_path()), "the member is what a creation of this moment makes");
    remove(root.as_path());
}

/// What:
///  A member that would miss an entry a commit reads is still refused.
/// Why:
///  The tolerance above is about entries no commit reads;
/// an entry a commit reads that the repository gained since the fill
///  is exactly the member that would give the commit a view of an older moment.
#[test]
fn a_member_that_would_miss_an_entry_a_commit_reads_is_refused() {
    let root: PathBuf = fixture("shadow-pool-read-state-appears");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    std::fs::remove_dir_all(common.join("info")).expect("this repository has no info directory");
    let (template, _member) = a_member(repo.as_path());
    std::fs::create_dir(common.join("info")).expect("an info directory appears");
    std::fs::write(common.join("info").join("attributes"), "*.rs text\n").expect("and holds attributes");
    let destination: PathBuf = root.join("shadow");
    assert!(!take(&template, destination.as_path()), "a member that misses info is refused");
    assert!(std::fs::symlink_metadata(destination.as_path()).is_err(), "and the claim's path is left empty");
    remove(root.as_path());
}

/// What:
///  A member that holds a real entry where a link belongs is still refused,
///  even for a name the repository no longer holds.
/// Why:
///   A link to an entry that is gone gives the commit the view a creation gives;
/// a directory or a file of its own there is what a use of the member left,
///  and the next commit would trust it.
#[test]
fn a_member_that_holds_a_real_entry_where_a_link_belongs_is_refused() {
    let root: PathBuf = fixture("shadow-pool-real-entry");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let (template, member) = a_member(repo.as_path());
    assert!(
        std::fs::symlink_metadata(repo.join(".git").join("rr-cache").as_path()).is_err(),
        "this repository has no rr-cache"
    );
    std::fs::create_dir(member.join("rr-cache").as_path()).expect("a real entry where a link belongs");
    let destination: PathBuf = root.join("shadow");
    assert!(!take(&template, destination.as_path()), "a real directory at a shared name is refused");
    remove(root.as_path());
}

/// What:
///  A refused member is named in the commit's own timing record,
///  where a slow commit explains itself,
///  and nowhere a person's terminal reads.
/// Why:
///  A member is the wrapper's own cache:
/// the person can neither cause a refusal nor fix it,
///  so the evidence belongs in the record that names the creation it forced
///  (`shadow_pool.rs`,
/// `note_refusal`).
#[test]
fn a_refused_member_is_named_in_the_build_parts() {
    let root: PathBuf = fixture("shadow-pool-refusal-evidence");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let (template, member) = a_member(repo.as_path());
    std::fs::write(member.join("MERGE_HEAD").as_path(), b"0123456789012345678901234567890123456789\n").expect("state");
    let _before = crate::commit_build_part_durations::drain_build_parts();
    assert!(!take(&template, root.join("shadow").as_path()), "a member with state is refused");
    let (parts, _dropped) = crate::commit_build_part_durations::drain_build_parts();
    assert!(
        parts.iter().any(|part| { return part.name == "shadow pool refused"; }),
        "the record names the refusal: {parts:?}"
    );
    remove(root.as_path());
}

/// What:
///  A fill makes no member for a linked worktree that keeps its own `info` directory.
/// Why:
///   Such a worktree's shadows are made where the commit is,
///  so every member the service made for it was refused by the very next commit,
///  which removed it and built its own creation:
/// the fill made work for nobody and the pool churned.
#[test]
fn a_fill_makes_no_member_for_a_worktree_that_keeps_its_own_info() {
    let root: PathBuf = fixture("shadow-pool-own-info");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let linked: PathBuf = root.join("linked");
    git(
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "feature", linked.to_str().expect("fixture")],
    );
    let git_dir: PathBuf = repo.join(".git").join("worktrees").join("linked");
    std::fs::create_dir_all(git_dir.join("info")).expect("worktree info");
    std::fs::write(git_dir.join("info").join("sparse-checkout"), "/*\n").expect("sparse");
    let template: ShadowTemplate = template_of(linked.as_path());
    assert_eq!(fill(&git_context(), &template).expect("fill"), None, "no member is made");
    assert!(members(pool_root(&template).as_path()).is_empty(), "and the pool holds none");
    assert!(!take(&template, root.join("shadow").as_path()), "nothing is taken either");
    remove(root.as_path());
}
