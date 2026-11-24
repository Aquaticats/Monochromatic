//! What:
//!  Rung 2 of the cheap-worktree ladder through the built executable:
//! `git worktree add` from a clean worktree that is no
//!  subvolume is served by clones of its tracked files,
//! and the result equals what Git's own checkout makes.
//! Why:
//!  A clone shares data blocks with the source,
//! which Git's checkout never does,
//! so a shared physical extent is the sign
//!  that clones served the command;
//! contents,
//! status,
//! words and the ignored-state copy are compared with a control that
//!  real Git made from the same commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(physicalExtent('../topic/big.bin')).toBe(physicalExtent('big.bin'));
//! ```

/// Import the shared fixtures and the helpers of the snapshot controls.
use super::support::{Fixture, Observed, fixture, first_physical_extent, git, on_a_reflink_capable_fixture, remove, repository, run_direct, run_wrapped_verbose, stderr_of};
use super::support::on_btrfs_or_acknowledged;
use super::worktree_snapshot::{add_pair, age, commit_all, tracked, tracked_content};
use git_policy_cli::fs_kind::is_subvolume_root;
use std::ffi::OsStr;
use std::path::PathBuf;

/// A repository that is an ordinary directory with a large aged file,
/// a second commit and an ignored directory.
fn ordinary_repository(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    tracked(repo.as_path(), ".gitignore", b"ignored/\n");
    tracked(repo.as_path(), "old.txt", b"old\n");
    tracked(repo.as_path(), "shared.txt", b"first\n");
    tracked(repo.as_path(), "big.bin", &(0..200_000u32).flat_map(|value: u32| { return value.to_le_bytes(); }).collect::<Vec<u8>>());
    commit_all(fixture, repo.as_path(), "first");
    std::fs::remove_file(repo.join("old.txt")).expect("delete");
    tracked(repo.as_path(), "shared.txt", b"second, longer\n");
    tracked(repo.as_path(), "new/added.txt", b"added\n");
    commit_all(fixture, repo.as_path(), "second");
    std::fs::create_dir_all(repo.join("ignored/pkg")).expect("ignored directory");
    std::fs::write(repo.join("ignored/pkg/index.js"), b"module.exports = 1;\n").expect("ignored file");
    age(repo.join("ignored/pkg/index.js").as_path());
    return repo;
}

/// A clean worktree that is no subvolume is cloned:
/// the data blocks are shared,
/// the content equals the control,
/// the first
///  status is clean,
/// and the caller sees Git's own words.
#[test]
fn a_clean_ordinary_main_checkout_is_cloned_and_equals_gits_own_checkout() {
    let fixture: Fixture = fixture("clone-main");
    if !on_btrfs_or_acknowledged(&fixture, "clone-main") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = ordinary_repository(&fixture);
    let (created, control) = add_pair(&fixture, repo.as_path(), "topic", "control", &["-b", "NAME", "../NAME"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let wrapped: String = stderr_of(&created).replace("topic", "NAME");
    let git_words: String = stderr_of(&control).replace("control", "NAME");
    assert!(wrapped.starts_with(git_words.as_str()), "Git's own words come first: {wrapped}");
    // Then the line that says why this was not a snapshot (the main checkout here is no subvolume), then the summary line of the
    //  ignored-state copy.
    let after_git: Vec<&str> = wrapped[git_words.len()..].lines().collect();
    assert!(after_git.first().is_some_and(|line: &&str| { return line.starts_with("monochromatic-git-policy-cli: this worktree was not made as a snapshot"); }), "{wrapped}");
    assert!(after_git.get(1).is_some_and(|line: &&str| { return line.starts_with("monochromatic-git-policy-cli: copied "); }), "then the summary line of the ignored-state copy: {wrapped}");
    assert_eq!(String::from_utf8_lossy(&created.stdout), String::from_utf8_lossy(&control.stdout));
    let topic: PathBuf = fixture.root.join("topic");
    assert!(is_subvolume_root(topic.as_path()).expect("probe"), "the clones went into a subvolume");
    assert_eq!(first_physical_extent(topic.join("big.bin").as_path()), first_physical_extent(repo.join("big.bin").as_path()), "a clone shares the extent");
    assert_ne!(first_physical_extent(fixture.root.join("control/big.bin").as_path()), first_physical_extent(repo.join("big.bin").as_path()), "Git's checkout wrote its own");
    assert_eq!(tracked_content(&fixture, topic.as_path()), tracked_content(&fixture, fixture.root.join("control").as_path()));
    assert_eq!(git(&fixture, topic.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]).stdout, b"", "clean on the first status");
    assert!(topic.join("ignored").exists(), "the clones carry no ignored state; the copy of the main checkout does");
    remove(&fixture);
}

/// An older target gets the difference applied by Git on top of the clones,
/// and equals the control.
#[test]
fn an_older_target_commit_is_cloned_and_updated() {
    let fixture: Fixture = fixture("clone-older");
    if !on_btrfs_or_acknowledged(&fixture, "clone-older") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = ordinary_repository(&fixture);
    let (created, _control) = add_pair(&fixture, repo.as_path(), "old", "oldcontrol", &["--detach", "../NAME", "HEAD~1"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let old: PathBuf = fixture.root.join("old");
    assert_eq!(first_physical_extent(old.join("big.bin").as_path()), first_physical_extent(repo.join("big.bin").as_path()));
    assert_eq!(std::fs::read(old.join("shared.txt")).expect("shared"), b"first\n");
    assert!(old.join("old.txt").is_file() && !old.join("new").exists());
    assert_eq!(tracked_content(&fixture, old.as_path()), tracked_content(&fixture, fixture.root.join("oldcontrol").as_path()));
    assert_eq!(git(&fixture, old.as_path(), &["status", "--porcelain=v1"]).stdout, b"");
    remove(&fixture);
}

/// A source whose only change is a tracked edit and whose tree is below the whole-tree limit is cloned from by
/// nothing:
/// Git's checkout runs and the log names no clone.
/// The control needs a root the rung could serve on, so its answer is the limit's and not a filesystem's refusal.
#[test]
fn a_dirty_ordinary_source_below_the_whole_tree_limit_is_not_cloned() {
    let fixture: Fixture = fixture("clone-dirty");
    if !on_a_reflink_capable_fixture(&fixture, "a_dirty_ordinary_source_below_the_whole_tree_limit_is_not_cloned") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = ordinary_repository(&fixture);
    // A tracked change, which is the shape the whole-tree limit was measured on: the tree's few hundred kilobytes sit
    // far below the limit's bytes, so the add takes Git's own checkout.
    std::fs::write(repo.join("shared.txt"), b"the person's edit\n").expect("the edit");
    let created: Observed = run_wrapped_verbose(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "plain", "../plain"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert!(!stderr_of(&created).contains("clones served"), "the whole-tree limit sent the add to Git's own checkout: {}", stderr_of(&created));
    let plain: PathBuf = fixture.root.join("plain");
    let control: Observed = run_direct(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "control", "../control"]);
    assert_eq!(control.code, Some(0), "{}", stderr_of(&control));
    assert_eq!(tracked_content(&fixture, plain.as_path()), tracked_content(&fixture, fixture.root.join("control").as_path()), "Git's own checkout wrote the worktree");
    assert_eq!(git(&fixture, plain.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]).stdout, b"", "clean on the first status");
    remove(&fixture);
}

/// A source whose only difference is an untracked file is cloned:
/// the untracked file is no tracked change, so the rule for a source with no tracked change judges it,
/// the log names the clone,
/// and the new worktree holds Git's own bytes.
#[test]
fn an_untracked_only_source_is_cloned() {
    let fixture: Fixture = fixture("clone-untracked-only");
    if !on_a_reflink_capable_fixture(&fixture, "an_untracked_only_source_is_cloned") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = ordinary_repository(&fixture);
    std::fs::write(repo.join("untracked.txt"), b"u").expect("untracked");
    let created: Observed = run_wrapped_verbose(&fixture, repo.as_path(), &["worktree", "add", "-b", "topic", "../topic"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let topic: PathBuf = fixture.root.join("topic");
    assert!(stderr_of(&created).contains(format!("clones served {}", topic.display()).as_str()), "the clone rung served the add: {}", stderr_of(&created));
    let control: Observed = run_direct(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "control", "../control"]);
    assert_eq!(control.code, Some(0), "{}", stderr_of(&control));
    assert_eq!(tracked_content(&fixture, topic.as_path()), tracked_content(&fixture, fixture.root.join("control").as_path()), "the bytes are Git's own checkout's");
    assert_eq!(git(&fixture, topic.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]).stdout, b"", "clean on the first status");
    assert!(!topic.join("untracked.txt").exists(), "an untracked file of the source reaches no worktree");
    remove(&fixture);
}
