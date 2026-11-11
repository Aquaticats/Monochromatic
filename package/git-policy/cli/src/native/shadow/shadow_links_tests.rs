//! Controls for the shadow links:
//! the named entries a commit reads link,
//! and every other entry of a common Git directory stays absent.

use super::*;
use crate::candidate_private_index::{PRIVATE_INDEX_PREFIX, STAGING_SUFFIX};
use crate::test_support::{fixture, git, remove, repository};
use crate::wrapper_names_old_and_new::{REPOSITORY_SCRATCH, SHADOW_POOL_FOLDER};

/// Every entry a shadow shares,
/// beside the ones a fixture repository has,
/// written into a common directory so the control names them itself.
const ENTRIES_A_COMMIT_DOES_NOT_READ: [&str; 12] = [
    "FETCH_HEAD",
    "ORIG_HEAD",
    "description",
    "lost-found",
    "worktrees",
    "fsmonitor--daemon",
    "monochromatic-git-policy-cli-audit.jsonl",
    "monochromatic-git-policy-cli-commit-timing.jsonl",
    "monochromatic-git-policy-cli-commit-timing.jsonl.1",
    "cli-git-captures",
    "MERGE_RR",
    ".factory-scm-trigger",
];

/// The names of the entries one directory holds,
/// in the order the directory lists them.
fn entry_names(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for listed in std::fs::read_dir(directory).expect("a readable directory") {
        let entry: std::fs::DirEntry = listed.expect("a readable entry");
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    return names;
}

/// A shadow links the entries a commit reads and nothing else,
/// so a file a command of the repository writes costs it no link.
#[test]
fn only_the_named_shared_entries_link() {
    let root: PathBuf = fixture("shadow-links-named");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    for name in ENTRIES_A_COMMIT_DOES_NOT_READ {
        std::fs::write(common.join(name), b"state").expect("an entry no commit reads");
    }
    std::fs::create_dir(common.join("threshold")).expect("a folder no commit reads");
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir(shadow.as_path()).expect("shadow");
    link_shadow_entries(shadow.as_path(), common.as_path(), common.as_path()).expect("link");
    assert_eq!(
        std::fs::read_link(shadow.join("hooks")).expect("hooks link"),
        common.join("hooks"),
        "the hooks directory is the real one"
    );
    assert_eq!(
        std::fs::read_link(shadow.join("info")).expect("info link"),
        common.join("info"),
        "the info directory is the real one"
    );
    let mut expected: Vec<String> = SHARED_ENTRY_NAMES
        .iter()
        .map(|name| { return String::from(*name); })
        .filter(|name| { return common.join(name).exists(); })
        .collect();
    let mut linked: Vec<String> = entry_names(shadow.as_path());
    expected.sort();
    linked.sort();
    assert_eq!(linked, expected, "a creation links the named entries the repository holds, and nothing else");
    for name in ENTRIES_A_COMMIT_DOES_NOT_READ {
        assert!(std::fs::symlink_metadata(shadow.join(name)).is_err(), "an entry no commit reads reached the shadow as {name}");
    }
    remove(root.as_path());
}

/// What:
///  The named list holds every entry a commit's shadow reads through a link,
///  and none of the state a busy common directory collects.
/// Why:
///  This is the decision the pool rests on:
///  a member is refused when one of these appears,
///  and never because of the others
///  (`shadow_pool_member.rs`,
/// `doc/planning/cli-git-commit-latency-under-many-agents.md`,
/// "The choices this leaves to the owner").
#[test]
fn the_named_list_is_what_a_commit_reads() {
    for name in ["branches", "hooks", "info", "lfs", "modules", "remotes", "rr-cache", "shallow", "svn"] {
        assert!(is_shared_shadow_entry(name), "{name} is an entry a commit reads");
    }
    for name in [
        // The shadow's own.
        "HEAD",
        "refs",
        "packed-refs",
        "logs",
        "index",
        "config",
        "config.worktree",
        "objects",
        "COMMIT_EDITMSG",
        // The wrapper's own state, under this release's name and an earlier release's.
        "monochromatic-git-policy-cli",
        "cli-git",
        "monochromatic-git-policy-cli-audit.jsonl",
        REPOSITORY_SCRATCH.current,
        REPOSITORY_SCRATCH.earlier,
        SHADOW_POOL_FOLDER,
        "cli-git-add-policy-1-2-3",
        // Conclusion state, which the creation copies.
        "MERGE_HEAD",
        "MERGE_RR",
        "sequencer",
        // Locks and the markers a command stages them in.
        "index.lock",
        "ORIG_HEAD.lock",
        // State of Git and of other tools that no commit reads.
        "FETCH_HEAD",
        "ORIG_HEAD",
        "description",
        "lost-found",
        "worktrees",
        "fsmonitor--daemon",
        "cli-git-captures",
        ".factory-scm-trigger",
    ] {
        assert!(!is_shared_shadow_entry(name), "{name} is not an entry a commit reads");
    }
}

/// A linked worktree keeps its `info` private,
/// links the shared entries inside it,
///  and copies its own sparse checkout and `config.worktree`.
#[test]
fn linked_worktree_copies_its_per_worktree_entries() {
    let root: PathBuf = fixture("shadow-links-linked");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let linked: PathBuf = root.join("linked");
    git(
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "feature", linked.to_str().expect("fixture")],
    );
    let common: PathBuf = repo.join(".git");
    let git_dir: PathBuf = common.join("worktrees").join("linked");
    std::fs::create_dir_all(git_dir.join("info")).expect("worktree info");
    std::fs::write(git_dir.join("info").join("sparse-checkout"), "/*\n").expect("sparse");
    std::fs::write(git_dir.join("config.worktree"), "[core]\n\tbare = false\n").expect("config");
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir(&shadow).expect("shadow");
    link_shadow_entries(shadow.as_path(), common.as_path(), git_dir.as_path()).expect("link");
    let shadow_info: PathBuf = shadow.join("info");
    assert!(std::fs::symlink_metadata(&shadow_info).expect("info").is_dir());
    assert_eq!(
        std::fs::read_link(shadow_info.join("exclude")).expect("exclude link"),
        common.join("info").join("exclude")
    );
    assert_eq!(
        std::fs::read_to_string(shadow_info.join("sparse-checkout")).expect("sparse copy"),
        "/*\n"
    );
    assert!(std::fs::symlink_metadata(shadow.join("info").join("sparse-checkout"))
        .expect("copy")
        .is_file());
    assert_eq!(
        std::fs::read_to_string(shadow.join("config.worktree")).expect("config copy"),
        "[core]\n\tbare = false\n"
    );
    remove(root.as_path());
}

/// The wrapper's own private copy of an index never reaches the shadow,
/// and the entries a commit reads still link.
///
/// A common directory collects one such copy for every command that died before it removed it,
/// so the number of names a shadow must link would grow without a limit the person chose.
#[test]
fn the_wrappers_private_index_copies_stay_out_of_the_shadow() {
    let root: PathBuf = fixture("shadow-links-private-index");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    let dead: PathBuf = common.join(format!("{PRIVATE_INDEX_PREFIX}1-2-3"));
    let staging: PathBuf = common.join(format!("{PRIVATE_INDEX_PREFIX}1-2-4{STAGING_SUFFIX}"));
    std::fs::create_dir(dead.as_path()).expect("dead copy");
    std::fs::write(dead.join("index"), b"index").expect("index");
    std::fs::create_dir(staging.as_path()).expect("staging copy");
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir(shadow.as_path()).expect("shadow");
    link_shadow_entries(shadow.as_path(), common.as_path(), common.as_path()).expect("link");
    assert!(std::fs::symlink_metadata(shadow.join(dead.file_name().expect("name"))).is_err());
    assert!(std::fs::symlink_metadata(shadow.join(staging.file_name().expect("name"))).is_err());
    assert_eq!(
        std::fs::read_link(shadow.join("hooks")).expect("hooks link"),
        common.join("hooks")
    );
    remove(root.as_path());
}

/// The private `info` directory of a linked worktree is private to the account.
#[cfg(unix)]
#[test]
fn private_info_directory_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("shadow-links-info-mode");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "feature", linked.to_str().expect("fixture")]);
    let common: PathBuf = repo.join(".git");
    let git_dir: PathBuf = common.join("worktrees").join("linked");
    std::fs::create_dir_all(git_dir.join("info")).expect("worktree info");
    std::fs::write(git_dir.join("info").join("sparse-checkout"), "/*\n").expect("sparse");
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir(&shadow).expect("shadow");
    link_shadow_entries(shadow.as_path(), common.as_path(), git_dir.as_path()).expect("link");
    let mode: u32 = std::fs::metadata(shadow.join("info")).expect("info").permissions().mode() & 0o777;
    assert_eq!(mode, 0o700);
    remove(root.as_path());
}

/// A linked worktree whose common directory has no shared `info` directory still links,
///  with its own sparse checkout copied in.
#[test]
fn linked_worktree_without_a_shared_info_directory_still_links() {
    let root: PathBuf = fixture("shadow-links-no-info");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "feature", linked.to_str().expect("fixture")]);
    let common: PathBuf = repo.join(".git");
    let git_dir: PathBuf = common.join("worktrees").join("linked");
    std::fs::create_dir_all(git_dir.join("info")).expect("worktree info");
    std::fs::write(git_dir.join("info").join("sparse-checkout"), "/*\n").expect("sparse");
    std::fs::remove_dir_all(common.join("info")).expect("remove shared info");
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir(&shadow).expect("shadow");
    link_shadow_entries(shadow.as_path(), common.as_path(), git_dir.as_path()).expect("link");
    assert_eq!(
        std::fs::read_to_string(shadow.join("info").join("sparse-checkout")).expect("sparse copy"),
        "/*\n"
    );
    remove(root.as_path());
}

/// A template entry the shadow already holds is replaced by the real link,
/// whether it is a file or a directory.
#[cfg(unix)]
#[test]
fn existing_shadow_entries_are_replaced_by_links() {
    let root: PathBuf = fixture("shadow-links-replace");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir_all(shadow.join("hooks")).expect("template hooks");
    std::fs::write(shadow.join("hooks").join("sample"), "x").expect("sample");
    std::fs::create_dir_all(shadow.join("info")).expect("template info");
    std::fs::write(shadow.join("info").join("exclude"), "template\n").expect("template exclude");
    link_shadow_entries(shadow.as_path(), common.as_path(), common.as_path()).expect("link");
    assert_eq!(std::fs::read_link(shadow.join("hooks")).expect("hooks link"), common.join("hooks"));
    assert_eq!(std::fs::read_link(shadow.join("info")).expect("info link"), common.join("info"));
    remove(root.as_path());
}

/// What:
///  A shared entry that a command removes between the listing and the link does not fail the link.
/// Why:
///  `lfs` appears when somebody starts using it and can go again;
/// a removal that lands inside a commit's creation must leave the commit building the shadow of a
///  repository that does not hold the entry, never failing it.
///  Unix makes a link to a name whether or not that name is there,
/// and Git reads a link to nothing as nothing,
///  which is what a creation of this moment holds.
#[test]
fn a_shared_entry_that_is_gone_before_it_is_linked_is_not_an_error() {
    let root: PathBuf = fixture("shadow-links-gone");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    std::fs::create_dir(common.join("lfs")).expect("lfs");
    let (shared, own_info): (Vec<SharedEntry>, bool) = shared_entries_of(common.as_path(), common.as_path()).expect("listed");
    assert!(!own_info);
    assert!(shared.iter().any(|entry| { return entry.name == "lfs"; }), "the listing holds lfs");
    std::fs::remove_dir_all(common.join("lfs")).expect("another command removes lfs");
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir(shadow.as_path()).expect("shadow");
    link_directory_entries(shared.as_slice(), shadow.as_path()).expect("a vanished entry is no failure");
    assert!(!common.join("lfs").exists(), "the real entry is gone");
    if let Ok(metadata) = std::fs::symlink_metadata(shadow.join("lfs").as_path()) {
        assert!(metadata.file_type().is_symlink(), "a gone entry is a link to nothing, never a copy of something");
    }
    assert!(std::fs::symlink_metadata(shadow.join("hooks").as_path()).is_ok(), "the entries that are there still link");
    remove(root.as_path());
}

/// A location whose metadata cannot be read,
/// other than an absent one,
/// is an error and not an absent file.
#[test]
fn an_unreadable_sparse_location_is_an_error() {
    let root: PathBuf = fixture("shadow-links-unreadable");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "feature", linked.to_str().expect("fixture")]);
    let common: PathBuf = repo.join(".git");
    let git_dir: PathBuf = common.join("worktrees").join("linked");
    std::fs::write(git_dir.join("info"), "not a directory").expect("info as a file");
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir(&shadow).expect("shadow");
    assert!(link_shadow_entries(shadow.as_path(), common.as_path(), git_dir.as_path()).is_err());
    remove(root.as_path());
}

/// A shared `info` location that is not a directory is an error,
/// because only an absent one is tolerated.
#[test]
fn a_shared_info_that_is_not_a_directory_is_an_error() {
    let root: PathBuf = fixture("shadow-links-shared-info-file");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "feature", linked.to_str().expect("fixture")]);
    let common: PathBuf = repo.join(".git");
    let git_dir: PathBuf = common.join("worktrees").join("linked");
    std::fs::create_dir_all(git_dir.join("info")).expect("worktree info");
    std::fs::write(git_dir.join("info").join("sparse-checkout"), "/*\n").expect("sparse");
    std::fs::remove_dir_all(common.join("info")).expect("remove shared info");
    std::fs::write(common.join("info"), "not a directory").expect("shared info as a file");
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir(&shadow).expect("shadow");
    assert!(link_shadow_entries(shadow.as_path(), common.as_path(), git_dir.as_path()).is_err());
    remove(root.as_path());
}

/// What:
///  A file one command stages a lock in never reaches a shadow,
///  under any name the lock it belongs to has.
/// Why:
///   Every command of a repository leaves one of these while it runs,
///  so a shadow that linked one held an entry a creation made a moment ago and no other creation would make.
///  The name is `<lock>.cli-git-marker-<pid>.<namespace>-<batch>-<token>`,
///  and the lock can be a ref's,
/// not only the index's.
#[test]
fn a_staged_lock_marker_never_reaches_the_shadow() {
    let root: PathBuf = fixture("shadow-links-lock-marker");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    for name in [
        "index.lock.cli-git-marker-4242.4026531836-1794310871725332-cf320c2d-d336-4336-b19e-d9e64c8c9d9e",
        "HEAD.lock",
    ] {
        std::fs::write(common.join(name), b"marker").expect("marker");
    }
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir(shadow.as_path()).expect("shadow");
    link_shadow_entries(shadow.as_path(), common.as_path(), common.as_path()).expect("link");
    for entry in std::fs::read_dir(shadow.as_path()).expect("shadow entries").flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        assert!(!name.contains("cli-git-marker"), "a staged marker reached the shadow as {name}");
        assert!(!name.ends_with(".lock"), "a lock reached the shadow as {name}");
    }
    remove(root.as_path());
}

/// What:
///  The folder of finished shadows beside the scratch root never reaches a shadow.
/// Why:
///   A link to the pool would let a shadow reach every member,
///   and whether the folder exists would decide whether a claim trusts a member.
///  It is the wrapper's own state that holds shadow trees,
///  like the scratch root beside it,
///  and not an entry a commit reads.
#[test]
fn the_pool_of_finished_shadows_never_reaches_the_shadow() {
    let root: PathBuf = fixture("shadow-links-pool");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    let pool: PathBuf = common.join(SHADOW_POOL_FOLDER);
    std::fs::create_dir_all(pool.join("main").join("member-x")).expect("the pool and one member");
    assert!(!is_shared_shadow_entry(SHADOW_POOL_FOLDER), "the pool's own name is never linked");
    assert!(is_shared_shadow_entry("hooks"), "an entry a commit reads is linked");
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir(shadow.as_path()).expect("shadow");
    link_shadow_entries(shadow.as_path(), common.as_path(), common.as_path()).expect("link");
    assert!(
        std::fs::symlink_metadata(shadow.join(SHADOW_POOL_FOLDER).as_path()).is_err(),
        "the pool is not in the shadow"
    );
    assert!(std::fs::symlink_metadata(shadow.join("hooks").as_path()).is_ok(), "a real shared entry is");
    remove(root.as_path());
}
