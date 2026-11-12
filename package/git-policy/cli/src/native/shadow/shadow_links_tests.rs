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
///  `rr-cache` appears when somebody starts resolving conflicts and can go again;
/// a removal that lands inside a commit's creation must leave the commit building the shadow of a
///  repository that does not hold the entry,
/// never failing it.
///  Unix makes a link to a name whether or not that name is there,
/// and Git reads a link to nothing as nothing,
///  which is what a creation of this moment holds;
/// a pooled member that still holds one is refused at a claim,
///  because nothing may write into the directory it names.
#[test]
fn a_shared_entry_that_is_gone_before_it_is_linked_is_not_an_error() {
    let root: PathBuf = fixture("shadow-links-gone");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    std::fs::create_dir(common.join("rr-cache")).expect("rerere has run here");
    let (shared, own_info): (Vec<SharedEntry>, bool) = shared_entries_of(common.as_path(), common.as_path()).expect("listed");
    assert!(!own_info);
    assert!(shared.iter().any(|entry| { return entry.name == "rr-cache"; }), "the listing holds rr-cache");
    std::fs::remove_dir_all(common.join("rr-cache")).expect("another command removes rr-cache");
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir(shadow.as_path()).expect("shadow");
    link_directory_entries(shared.as_slice(), shadow.as_path()).expect("a vanished entry is no failure");
    assert!(!common.join("rr-cache").exists(), "the real entry is gone");
    if let Ok(metadata) = std::fs::symlink_metadata(shadow.join("rr-cache").as_path()) {
        assert!(metadata.file_type().is_symlink(), "a gone entry is a link to nothing, never a copy of something");
    }
    assert!(std::fs::symlink_metadata(shadow.join("hooks").as_path()).is_ok(), "the entries that are there still link");
    remove(root.as_path());
}

/// What:
///  The directory a clean filter of the commit writes its object into is made when the repository has
///  not got it yet,
///  and the shadow links it.
/// Why:
///   A filter that stores into `<git-dir>/lfs` as `git-lfs` does needs that directory to be there:
///  a shadow that links nothing leaves the object in the private tree,
///  which is released with it,
///  and a link that reaches nothing makes the filter fail,
///  and the commit with it.
///  `git-lfs` makes the directory on its first write,
///  so a creation that makes it first holds what the next command in the real repository would hold.
#[test]
fn the_store_a_filter_writes_into_is_made_and_linked() {
    let root: PathBuf = fixture("shadow-links-filter-store");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    assert!(
        std::fs::symlink_metadata(common.join(LFS_STORE_ENTRY_NAME).as_path()).is_err(),
        "this repository has no store yet"
    );
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir(shadow.as_path()).expect("shadow");
    link_shadow_entries(shadow.as_path(), common.as_path(), common.as_path()).expect("link");
    assert!(
        std::fs::symlink_metadata(common.join(LFS_STORE_ENTRY_NAME).as_path()).is_ok_and(|metadata| { return metadata.is_dir(); }),
        "the creation makes the real store"
    );
    assert_eq!(
        std::fs::read_link(shadow.join(LFS_STORE_ENTRY_NAME)).expect("the store link"),
        common.join(LFS_STORE_ENTRY_NAME),
        "and the shadow links that one directory"
    );
    std::fs::write(shadow.join(LFS_STORE_ENTRY_NAME).join("written-by-a-filter"), b"x").expect("a write through the link");
    assert!(
        std::fs::symlink_metadata(common.join(LFS_STORE_ENTRY_NAME).join("written-by-a-filter").as_path()).is_ok(),
        "what a filter writes through the shadow's link is the real store's"
    );
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

/// What:
///  A listing of the shared `info` that reports an error is the creation's to report,
///  never a directory quietly linked short of the entries the repository holds.
/// Why:
///   The shadow's `info` is where a commit finds `attributes`,
///  `exclude` and `grafts`,
///  so a listing that dropped a failed read would give it a view of the repository's filters and
///  excludes that no one chose,
///  and the commit would be built from bytes the person's attributes never named.
///  The two arms are one control:
/// a written directory lists every entry,
/// so the error this reads is the only reason the first arm is empty.
#[test]
fn an_error_while_listing_the_shared_info_is_reported() {
    let root: PathBuf = fixture("shadow-links-shared-info-error");
    let info: PathBuf = root.join("info");
    std::fs::create_dir_all(info.as_path()).expect("a shared info");
    std::fs::write(info.join("attributes").as_path(), "* text\n").expect("attributes");
    std::fs::write(info.join("exclude").as_path(), "*.log\n").expect("exclude");
    std::fs::write(info.join("sparse-checkout").as_path(), "/*\n").expect("the per-worktree pattern");
    let written: std::fs::ReadDir = std::fs::read_dir(info.as_path()).expect("a written directory");
    let listed: Vec<SharedEntry> = shared_info_entries(written).expect("a directory that reads lists");
    let mut names: Vec<String> = listed.iter().map(|entry| { return entry.name.to_string_lossy().into_owned(); }).collect();
    names.sort();
    assert_eq!(
        names,
        vec![String::from("attributes"), String::from("exclude")],
        "every shared entry is listed, and never the per-worktree pattern"
    );
    let failed: std::io::Result<Vec<SharedEntry>> =
        shared_info_entries(vec![Err(std::io::Error::other("a directory read that failed"))].into_iter());
    let error: std::io::Error = failed.expect_err("an error the listing reports is the creation's, never an entry left out");
    assert_eq!(error.kind(), std::io::ErrorKind::Other, "the creation reports the error the listing gave it");
    remove(root.as_path());
}

/// What:
///  One repository whose `<git-dir>/lfs` holds something that is not a directory of the wrapper's making:
///  a regular file,
/// a symbolic link that reaches nothing,
///  or a symbolic link to a file.
/// Why:
///  The three are one control,
///  because the creation must treat them the same:
/// leave the name as the person has it and link it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { repo, elsewhere } = storeHeldAs(root, 'file')
/// ```
#[cfg(unix)]
fn store_held_as(root: &Path, slug: &str, state: &str) -> (PathBuf, PathBuf) {
    let repo: PathBuf = repository(root, slug);
    let common: PathBuf = repo.join(".git");
    let elsewhere: PathBuf = root.join(format!("{slug}-elsewhere"));
    std::fs::create_dir(elsewhere.as_path()).expect("a place beside the repository");
    let store: PathBuf = common.join(LFS_STORE_ENTRY_NAME);
    match state {
        "file" => {
            std::fs::write(store.as_path(), b"a file where the store belongs\n").expect("a regular file at the store's name");
        }
        "dangling" => {
            std::os::unix::fs::symlink(elsewhere.join("not-mounted"), store.as_path()).expect("a link that reaches nothing");
        }
        "link-to-file" => {
            std::fs::write(elsewhere.join("store-file"), b"a file where the store belongs\n").expect("a file to link to");
            std::os::unix::fs::symlink(elsewhere.join("store-file"), store.as_path()).expect("a link to a file");
        }
        other => panic!("{other} is not a store state this control names"),
    }
    return (repo, elsewhere);
}

/// What:
///  A store the repository holds as something that is not a directory is linked as it is,
///  and the creation replaces,
/// moves and deletes nothing of it.
/// Why:
///  `git-lfs` keeps its objects at `<git-dir>/lfs`,
/// and people point that name at one store several clones share,
/// at a store on a volume that is not mounted,
///  or leave a file there.
///  Real Git's commit runs with every one of those layouts,
/// so a wrapped commit that makes a directory at that name fails where real Git's succeeds:
///  the commit need not touch LFS at all for the creation to fail,
///  because the creation makes the store on every commit.
#[cfg(unix)]
#[test]
fn a_store_that_is_not_a_directory_is_linked_as_it_is() {
    let root: PathBuf = fixture("shadow-links-store-not-a-directory");
    for state in ["file", "dangling", "link-to-file"] {
        let (repo, elsewhere): (PathBuf, PathBuf) = store_held_as(root.as_path(), state, state);
        let common: PathBuf = repo.join(".git");
        let store: PathBuf = common.join(LFS_STORE_ENTRY_NAME);
        let shadow: PathBuf = root.join(format!("shadow-{state}"));
        std::fs::create_dir(shadow.as_path()).expect("shadow");
        link_shadow_entries(shadow.as_path(), common.as_path(), common.as_path())
            .unwrap_or_else(|error| panic!("a store held as {state} failed the creation: {error}"));
        assert_eq!(
            std::fs::read_link(shadow.join(LFS_STORE_ENTRY_NAME)).expect("the shadow's link to the store"),
            store,
            "the creation links the name the repository holds, held as {state}"
        );
        let held: std::fs::Metadata = std::fs::symlink_metadata(store.as_path()).expect("the store's name is still taken");
        match state {
            "file" => {
                assert!(held.is_file(), "a regular file stands where the store would be");
                assert_eq!(
                    std::fs::read(store.as_path()).expect("the file"),
                    b"a file where the store belongs\n",
                    "and the creation wrote nothing into it"
                );
            }
            "dangling" => {
                assert!(held.file_type().is_symlink(), "the link that reaches nothing is still a link");
                assert_eq!(
                    std::fs::read_link(store.as_path()).expect("the link"),
                    elsewhere.join("not-mounted"),
                    "and it still names what the person pointed it at"
                );
                assert!(std::fs::metadata(store.as_path()).is_err(), "and it still reaches nothing");
            }
            "link-to-file" => {
                assert!(held.file_type().is_symlink(), "the link to the file is still a link");
                assert_eq!(
                    std::fs::read_link(store.as_path()).expect("the link"),
                    elsewhere.join("store-file"),
                    "and it still names the file the person pointed it at"
                );
                assert!(std::fs::metadata(store.as_path()).expect("the file").is_file(), "and still reaches that file");
            }
            other => panic!("{other} is not a store state this control names"),
        }
    }
    remove(root.as_path());
}

/// What:
///  A store another command removes and takes with a link that reaches nothing,
/// between the listing of the entries and their linking,
///  is no failure of the creation,
/// and the creation still replaces nothing.
/// Why:
///  The store is listed whether the repository holds it or not,
/// so the entry the creation links can name something other than the directory the listing read:
/// a `git-lfs` that starts, a sweep of a scratch volume,
///  or a person's own command can change the name in that window.
///  A commit that fails there fails where real Git's commit, which never reads this name,
///  lands.
#[cfg(unix)]
#[test]
fn a_store_another_command_takes_while_the_creation_links_is_not_a_failure() {
    let root: PathBuf = fixture("shadow-links-store-taken-while-linking");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    std::fs::create_dir(common.join(LFS_STORE_ENTRY_NAME)).expect("git-lfs has run in this repository");
    let (shared, own_info): (Vec<SharedEntry>, bool) = shared_entries_of(common.as_path(), common.as_path()).expect("listed");
    assert!(!own_info);
    assert!(
        shared.iter().any(|entry| { return entry.name == LFS_STORE_ENTRY_NAME; }),
        "the listing holds the store"
    );
    // Another command removes the store and takes the name with a link that reaches nothing.
    std::fs::remove_dir_all(common.join(LFS_STORE_ENTRY_NAME)).expect("another command removes the store");
    std::os::unix::fs::symlink(root.join("not-mounted"), common.join(LFS_STORE_ENTRY_NAME)).expect("another command takes the name");
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir(shadow.as_path()).expect("shadow");
    link_directory_entries(shared.as_slice(), shadow.as_path())
        .expect("a store another command changed while the creation ran is no failure of the commit");
    assert_eq!(
        std::fs::read_link(shadow.join(LFS_STORE_ENTRY_NAME)).expect("the shadow's link to the store"),
        common.join(LFS_STORE_ENTRY_NAME)
    );
    let held: std::fs::Metadata = std::fs::symlink_metadata(common.join(LFS_STORE_ENTRY_NAME)).expect("the name is still taken");
    assert!(held.file_type().is_symlink(), "the name another command took is left as it is");
    assert_eq!(
        std::fs::read_link(common.join(LFS_STORE_ENTRY_NAME)).expect("the link"),
        root.join("not-mounted"),
        "the creation did not replace what another command put there"
    );
    remove(root.as_path());
}

/// What:
///  A Git directory this account cannot write leaves the creation without a store,
///  and the commit is not what fails for it.
/// Why:
///  A commit with nothing to do with LFS lands with real Git even when `<git-dir>/lfs` cannot be made,
/// and a filter that needs the store reports what it cannot write itself,
///  which is where real Git's commit fails too.
///  The gate's containers run this suite as root,
/// where no mode refuses this account, so the control names that and returns.
#[cfg(unix)]
#[test]
fn a_store_the_account_cannot_make_does_not_fail_the_creation() {
    use std::os::unix::fs::PermissionsExt;
    // SAFETY: `geteuid` takes no arguments, touches no memory and cannot fail.
    if unsafe { libc::geteuid() } == 0 {
        eprintln!("skipped: a mode this account is refused by needs an account the kernel refuses, and this run is root");
        return;
    }
    let root: PathBuf = fixture("shadow-links-store-unwritable-git-directory");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    assert!(
        std::fs::symlink_metadata(common.join(LFS_STORE_ENTRY_NAME)).is_err(),
        "this repository has no store yet"
    );
    std::fs::set_permissions(common.as_path(), std::fs::Permissions::from_mode(0o500)).expect("the Git directory made unwritable");
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir(shadow.as_path()).expect("shadow");
    let linked: std::io::Result<()> = link_shadow_entries(shadow.as_path(), common.as_path(), common.as_path());
    std::fs::set_permissions(common.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the Git directory given back");
    linked.unwrap_or_else(|error| panic!("a store that cannot be made failed the creation: {error}"));
    assert!(
        std::fs::symlink_metadata(common.join(LFS_STORE_ENTRY_NAME)).is_err(),
        "nothing stands where the account cannot write"
    );
    assert_eq!(
        std::fs::read_link(shadow.join(LFS_STORE_ENTRY_NAME)).expect("the shadow's link to the store"),
        common.join(LFS_STORE_ENTRY_NAME),
        "the shadow still links the name, so a filter that cannot write there fails as real Git's does"
    );
    remove(root.as_path());
}
/// What:
///  The creation's step that makes the store:
///  a link that reaches nothing is made to reach a directory,
/// and a name that already holds one is left exactly as it is.
/// Why:
///  The creation links the store before it makes it,
/// so the moment a removal can land in is the one before that step:
/// a store another command removed after the shadow's link was made leaves the commit's filter a name it cannot
/// write through,
///  and `git-lfs`'s clean filter then fails the commit (`fatal: <path>: clean filter 'lfs' failed`),
///  where real Git's own filter makes the directory and the commit lands.
///  A store that is there is the person's,
/// so the step makes one only when the name is absent.
#[cfg(unix)]
#[test]
fn the_store_step_makes_a_link_that_reaches_nothing_reach_the_store() {
    use std::os::unix::fs::MetadataExt;
    let root: PathBuf = fixture("shadow-links-store-step");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir(shadow.as_path()).expect("shadow");
    // The link a creation makes while the repository has no store: what a removal leaves behind it.
    std::os::unix::fs::symlink(common.join(LFS_STORE_ENTRY_NAME), shadow.join(LFS_STORE_ENTRY_NAME))
        .expect("the link a creation makes");
    assert!(
        std::fs::metadata(shadow.join(LFS_STORE_ENTRY_NAME)).is_err(),
        "the link reaches nothing, which is the state a store removed after the link leaves"
    );
    make_the_store_a_filter_writes_into(common.as_path());
    let made: std::fs::Metadata = std::fs::symlink_metadata(common.join(LFS_STORE_ENTRY_NAME)).expect("the step made the store");
    assert!(made.is_dir(), "the step makes a directory at the store's name");
    assert!(
        shadow.join(LFS_STORE_ENTRY_NAME).metadata().is_ok_and(|metadata| { return metadata.is_dir(); }),
        "and the link the creation made reaches it, which is what the commit's filter writes through"
    );
    // A store that is there is not made again: only an absent name is the creation's to make.
    let inode: u64 = made.ino();
    make_the_store_a_filter_writes_into(common.as_path());
    assert_eq!(
        std::fs::metadata(common.join(LFS_STORE_ENTRY_NAME)).expect("the store is still there").ino(),
        inode,
        "a store the repository holds is left as it is"
    );
    remove(root.as_path());
}
