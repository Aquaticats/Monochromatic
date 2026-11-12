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

/// A shadow of a fixture,
/// made exactly as a commit makes it,
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

/// A per-worktree configuration that differs from the one a creation would copy is refused,
/// byte for byte:
/// two files whose bytes differ in a way that decodes to the same text are still two configurations,
/// and a commit that took the older one would run the editor,
/// the hook or the command the person has since changed.
/// Why:
///   The copy a creation makes is the bytes of the file,
/// so the comparison over them is the one that answers whether
///  the member is what a creation would have made.
#[test]
fn a_worktree_configuration_is_compared_by_its_bytes() {
    let root: PathBuf = fixture("shadow-pool-member-worktree-configuration");
    let repo: PathBuf = repository_of(root.as_path());
    git(repo.as_path(), &["config", "extensions.worktreeConfig", "true"]);
    // Git reads the per-worktree file as bytes, so an editor path that is not text is a configuration it accepts.
    let first: &[u8] = b"[core]\n\teditor = \x80\n";
    let second: &[u8] = b"[core]\n\teditor = \x81\n";
    std::fs::write(repo.join(".git/config.worktree").as_path(), first).expect("the per-worktree file");
    let template: ShadowTemplate = template_of(repo.as_path());
    let shadow: PathBuf = creation(root.as_path(), &template, "created");
    assert_eq!(refusal(&template, shadow.as_path()).expect("checked"), None, "the copy a creation made is a member");
    std::fs::write(repo.join(".git/config.worktree").as_path(), second).expect("another editor, in bytes");
    assert_eq!(
        refusal(&template, shadow.as_path()).expect("checked"),
        Some(String::from("its copy of the worktree's configuration is not the one a creation would copy")),
        "two configurations that decode to the same text are still two configurations"
    );
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
            Some(format!("it holds {leftover}, which this invocation's creation does not make")),
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
        Some(String::from("it holds a pack, which no creation leaves"))
    );
    let referenced: PathBuf = creation(root.as_path(), &template, "created-ref");
    std::fs::write(
        referenced.join("refs").join("heads").join("main").as_path(),
        b"0123456789012345678901234567890123456789\n",
    )
    .expect("ref");
    assert_eq!(
        refusal(&template, referenced.as_path()).expect("checked"),
        Some(String::from("it holds a ref, which no creation leaves"))
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
        Some(String::from("the shared entry info links somewhere other than the repository's own entry"))
    );
    let missing: PathBuf = creation(root.as_path(), &template, "created-missing");
    std::fs::remove_file(missing.join("hooks").as_path()).expect("the real link");
    assert_eq!(
        refusal(&template, missing.as_path()).expect("checked"),
        Some(String::from("the shared entry hooks is not in the member"))
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
        Some(String::from("its config is not the one this invocation would write"))
    );
    let mut other_object_directory: ShadowTemplate = template.clone();
    other_object_directory.object_directory = root.join("objects-elsewhere");
    assert_eq!(
        refusal(&other_object_directory, shadow.as_path()).expect("checked"),
        Some(String::from("its alternates do not name this invocation's object directory"))
    );
    std::fs::write(shadow.join("HEAD").as_path(), b"ref: refs/heads/other\n").expect("head");
    assert_eq!(
        refusal(&template, shadow.as_path()).expect("checked"),
        Some(String::from("its HEAD is not the one a creation leaves before a commit names its branch"))
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
        Some(String::from("this worktree keeps its own info directory, so its shadows are made where the commit is"))
    );
    remove(root.as_path());
}

/// What:
///  An entry a commit does not read that the repository has since lost is no refusal.
/// Why:
///   The member holds the link a creation made,
/// and a link to a name that is gone resolves to nothing,
///  which is the view a creation of this moment gives:
/// a common directory that collects and drops state all day
///  (`FETCH_HEAD`,
/// its own log generations,
/// a removed worktree)
///  must not refuse a member over it.
#[test]
fn an_entry_no_commit_reads_that_the_repository_lost_is_no_refusal() {
    let root: PathBuf = fixture("shadow-pool-member-unread-state-gone");
    let repo: PathBuf = repository_of(root.as_path());
    let common: PathBuf = repo.join(".git");
    let went: [&str; 4] = [
        "FETCH_HEAD",
        "monochromatic-git-policy-cli-commit-timing.jsonl",
        "lost-found",
        "cli-git-captures",
    ];
    for name in went {
        std::fs::write(common.join(name), b"state").expect("an entry no commit reads");
    }
    std::fs::create_dir_all(common.join("worktrees").join("another")).expect("another worktree's state");
    let template: ShadowTemplate = template_of(repo.as_path());
    let shadow: PathBuf = creation(root.as_path(), &template, "created");
    for name in went {
        std::fs::remove_file(common.join(name)).expect("gone");
    }
    std::fs::remove_dir_all(common.join("worktrees")).expect("gone");
    assert_eq!(
        refusal(&template, shadow.as_path()).expect("checked"),
        None,
        "state no commit reads is no reason to refuse a member"
    );
    remove(root.as_path());
}

/// What:
///  An entry a commit reads that appeared after the member was made is a refusal,
///  named as the entry it would miss.
/// Why:
///   A member made before `info` or `rr-cache` existed would give the commit a view of the
///  repository as it was then,
///   so the member that would miss one is removed rather than trusted.
#[test]
fn an_entry_a_commit_reads_that_appeared_after_the_member_is_a_refusal() {
    let root: PathBuf = fixture("shadow-pool-member-read-state-appeared");
    let repo: PathBuf = repository_of(root.as_path());
    let common: PathBuf = repo.join(".git");
    assert!(
        std::fs::symlink_metadata(common.join("rr-cache").as_path()).is_err(),
        "this repository has no rr-cache"
    );
    let template: ShadowTemplate = template_of(repo.as_path());
    let shadow: PathBuf = creation(root.as_path(), &template, "created");
    std::fs::create_dir(common.join("rr-cache")).expect("rerere records something");
    assert_eq!(
        refusal(&template, shadow.as_path()).expect("checked"),
        Some(String::from("the shared entry rr-cache is not in the member"))
    );
    remove(root.as_path());
}

/// What:
///  A real entry at a shared name the repository no longer holds is a refusal,
///  because only the link a creation made gives the view a creation gives.
/// Why:
///   A directory or a file of the member's own there is what a use of it left,
///  and the next commit would trust bytes this invocation's creation never made.
#[test]
fn a_real_entry_at_a_shared_name_is_a_refusal() {
    let root: PathBuf = fixture("shadow-pool-member-real-entry");
    let repo: PathBuf = repository_of(root.as_path());
    let template: ShadowTemplate = template_of(repo.as_path());
    let shadow: PathBuf = creation(root.as_path(), &template, "created");
    std::fs::create_dir(shadow.join("rr-cache").as_path()).expect("a real entry where a link belongs");
    assert_eq!(
        refusal(&template, shadow.as_path()).expect("checked"),
        Some(String::from("the shared entry rr-cache is not a link in the member"))
    );
    remove(root.as_path());
}

/// What:
///  A shared entry the repository has lost since the fill refuses the member,
///  and the refusal names it.
/// Why:
///   The link a fill made points at a directory that is gone,
///  while a creation of this moment makes that directory and links it,
///  so the two trees are not the same view:
/// what writes into the directory a link names fails on one that resolves to nothing
///  (`git-lfs`'s clean filter writes its object into `<git-dir>/lfs`).
///  A member that is not the tree a creation makes is removed,
///  and the commit makes its own creation.
#[test]
fn a_shared_entry_the_repository_lost_is_a_refusal() {
    let root: PathBuf = fixture("shadow-pool-member-shared-entry-gone");
    let repo: PathBuf = repository_of(root.as_path());
    let common: PathBuf = repo.join(".git");
    std::fs::create_dir(common.join("lfs")).expect("git-lfs has run in this repository");
    let template: ShadowTemplate = template_of(repo.as_path());
    let shadow: PathBuf = creation(root.as_path(), &template, "created");
    assert_eq!(
        std::fs::read_link(shadow.join("lfs")).expect("the link a creation makes"),
        common.join("lfs"),
        "the creation links the repository's own lfs"
    );
    assert_eq!(
        refusal(&template, shadow.as_path()).expect("checked"),
        None,
        "a member whose every link resolves is the tree a creation makes"
    );
    std::fs::remove_dir_all(common.join("lfs")).expect("another command removes lfs");
    assert_eq!(
        refusal(&template, shadow.as_path()).expect("checked"),
        Some(String::from("the shared entry lfs points at an entry the repository no longer holds")),
        "a link to nothing is not the tree a creation of this moment makes"
    );
    let fresh: PathBuf = creation(root.as_path(), &template, "created-after");
    assert_eq!(
        std::fs::read_link(fresh.join("lfs")).expect("the link a creation of this moment makes"),
        common.join("lfs"),
        "a creation makes the store the member's link could not reach"
    );
    assert!(
        std::fs::symlink_metadata(common.join("lfs").as_path()).is_ok_and(|metadata| { return metadata.is_dir(); }),
        "and the store it links is there"
    );
    remove(root.as_path());
}
