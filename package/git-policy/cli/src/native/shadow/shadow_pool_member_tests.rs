//! Controls for what a finished member of the pool must be:
//!  a creation's tree is accepted,
//! and every leftover of a use is refused by name.

use super::*;
use crate::commit_build_part_durations::BuildPartClock;
use crate::shadow_create::{ShadowTemplate, create_shadow_template};
use crate::test_support::{fixture, git, git_context, remove, repository};
use std::path::{Path, PathBuf};

/// A repository with one commit on `main`,
/// whose shadow the creation can be made from.
fn repository_of(root: &Path) -> PathBuf {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    return repo;
}

/// The facts of a repository's shadow.
fn template_of(repo: &Path) -> ShadowTemplate {
    let context = git_context();
    let layout = crate::capture_invocation::capture_invocation_layout(&context, repo, false).expect("layout");
    return ShadowTemplate::of(&context, &layout).expect("template");
}

/// A shadow of a fixture, made exactly as a commit makes it,
/// at a path of the control's choosing.
fn creation(root: &Path, template: &ShadowTemplate, name: &str) -> PathBuf {
    let shadow: PathBuf = root.join(name);
    let mut clock: BuildPartClock = BuildPartClock::begin("test");
    create_shadow_template(&git_context(), template, shadow.as_path(), &mut clock).expect("creation");
    return shadow;
}

/// A creation's own tree is what a member must be.
#[test]
fn a_creation_is_what_a_member_must_be() {
    let root: PathBuf = fixture("shadow-pool-member-creation");
    let repo: PathBuf = repository_of(root.as_path());
    let template: ShadowTemplate = template_of(repo.as_path());
    let shadow: PathBuf = creation(root.as_path(), &template, "created");
    assert_eq!(refusal(&template, shadow.as_path()).expect("checked"), None, "a creation is a member");
    remove(root.as_path());
}

/// A leftover of a use is refused,
/// and the refusal names it rather than guessing.
#[test]
fn a_leftover_of_a_use_is_refused() {
    let root: PathBuf = fixture("shadow-pool-member-leftovers");
    let repo: PathBuf = repository_of(root.as_path());
    let template: ShadowTemplate = template_of(repo.as_path());
    for (name, leftover) in [
        ("state", "MERGE_HEAD"),
        ("index", "index"),
        ("log", "COMMIT_EDITMSG"),
        ("sequencer", "sequencer"),
    ] {
        let shadow: PathBuf = creation(root.as_path(), &template, format!("created-{name}").as_str());
        std::fs::write(shadow.join(leftover).as_path(), b"x\n").expect("leftover");
        assert_eq!(
            refusal(&template, shadow.as_path()).expect("checked"),
            Some("it holds an entry this invocation's creation does not make"),
            "{leftover}"
        );
    }
    remove(root.as_path());
}

/// A member that holds a pack or a ref of an earlier commit is refused,
/// because no creation leaves one.
#[test]
fn a_pack_or_a_ref_is_refused() {
    let root: PathBuf = fixture("shadow-pool-member-objects");
    let repo: PathBuf = repository_of(root.as_path());
    let template: ShadowTemplate = template_of(repo.as_path());
    let packed: PathBuf = creation(root.as_path(), &template, "created-pack");
    std::fs::write(packed.join("objects").join("pack").join("pack-x.pack").as_path(), b"x").expect("pack");
    assert_eq!(
        refusal(&template, packed.as_path()).expect("checked"),
        Some("it holds a pack, which no creation leaves")
    );
    let referenced: PathBuf = creation(root.as_path(), &template, "created-ref");
    std::fs::write(
        referenced.join("refs").join("heads").join("main").as_path(),
        b"0123456789012345678901234567890123456789\n",
    )
    .expect("ref");
    assert_eq!(
        refusal(&template, referenced.as_path()).expect("checked"),
        Some("it holds a ref, which no creation leaves")
    );
    remove(root.as_path());
}

/// A link that points somewhere other than the repository's own entry is refused,
/// and so is a missing one.
#[test]
fn a_link_of_another_repository_is_refused() {
    let root: PathBuf = fixture("shadow-pool-member-links");
    let repo: PathBuf = repository_of(root.as_path());
    let template: ShadowTemplate = template_of(repo.as_path());
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir(elsewhere.as_path()).expect("elsewhere");
    let moved: PathBuf = creation(root.as_path(), &template, "created-moved");
    let link: PathBuf = moved.join("info");
    std::fs::remove_file(link.as_path()).expect("the real link");
    std::os::unix::fs::symlink(elsewhere.as_path(), link.as_path()).expect("another link");
    assert_eq!(
        refusal(&template, moved.as_path()).expect("checked"),
        Some("a shared entry of the member links somewhere other than the repository's own entry")
    );
    let missing: PathBuf = creation(root.as_path(), &template, "created-missing");
    std::fs::remove_file(missing.join("hooks").as_path()).expect("the real link");
    assert_eq!(
        refusal(&template, missing.as_path()).expect("checked"),
        Some("a shared entry of the repository is not a link in the member")
    );
    remove(root.as_path());
}

/// A member whose config names another worktree root,
/// another object directory or another `HEAD` is refused.
#[test]
fn a_config_of_another_invocation_is_refused() {
    let root: PathBuf = fixture("shadow-pool-member-config");
    let repo: PathBuf = repository_of(root.as_path());
    let template: ShadowTemplate = template_of(repo.as_path());
    let shadow: PathBuf = creation(root.as_path(), &template, "created");
    let mut other: ShadowTemplate = template.clone();
    other.repository_root = root.join("elsewhere");
    assert_eq!(
        refusal(&other, shadow.as_path()).expect("checked"),
        Some("its config is not the one this invocation would write")
    );
    let mut other_object_directory: ShadowTemplate = template.clone();
    other_object_directory.object_directory = root.join("objects-elsewhere");
    assert_eq!(
        refusal(&other_object_directory, shadow.as_path()).expect("checked"),
        Some("its alternates do not name this invocation's object directory")
    );
    std::fs::write(shadow.join("HEAD").as_path(), b"ref: refs/heads/other\n").expect("head");
    assert_eq!(
        refusal(&template, shadow.as_path()).expect("checked"),
        Some("its HEAD is not the one a creation leaves before a commit names its branch")
    );
    remove(root.as_path());
}

/// A linked worktree that keeps its own `info` is never served by the pool.
#[test]
fn a_worktree_that_keeps_its_own_info_is_refused() {
    let root: PathBuf = fixture("shadow-pool-member-sparse");
    let repo: PathBuf = repository_of(root.as_path());
    let linked: PathBuf = root.join("linked");
    git(
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "feature", linked.to_str().expect("fixture")],
    );
    let git_dir: PathBuf = repo.join(".git").join("worktrees").join("linked");
    std::fs::create_dir_all(git_dir.join("info")).expect("worktree info");
    std::fs::write(git_dir.join("info").join("sparse-checkout"), "/*\n").expect("sparse");
    let template: ShadowTemplate = template_of(linked.as_path());
    let shadow: PathBuf = creation(root.as_path(), &template, "created");
    assert_eq!(
        refusal(&template, shadow.as_path()).expect("checked"),
        Some("this worktree keeps its own info directory, so its shadows are made where the commit is")
    );
    remove(root.as_path());
}
