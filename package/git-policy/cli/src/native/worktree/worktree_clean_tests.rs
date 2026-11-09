//! Controls for the cleanliness check against real Git 2.56.0 repositories:
//! a clean worktree is proven clean,
//! and each
//!  kind of difference (content,
//! mode,
//! type,
//! deletion,
//! staging,
//! untracked file,
//! operation in progress,
//! submodule,
//!  racy entry) is refused with its reason,
//! while an ignored file changes nothing.

use super::*;
use crate::parallel_test_support::{PATIENCE, hold_the_only_place};
use crate::subvolume_index::StatFields;
use crate::test_support::{age, fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// A repository with two tracked files,
/// a tracked symbolic link and an ignored directory,
/// all older than the index.
fn aged_repository(root: &Path) -> PathBuf {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join(".gitignore"), b"ignored/\n").expect("gitignore");
    std::fs::write(repo.join("a.txt"), b"alpha\n").expect("a");
    std::fs::create_dir(repo.join("dir")).expect("dir");
    std::fs::write(repo.join("dir/b.txt"), b"bravo\n").expect("b");
    std::os::unix::fs::symlink("a.txt", repo.join("link")).expect("link");
    for name in [".gitignore", "a.txt", "dir/b.txt", "link"] {
        age(repo.join(name).as_path());
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    return repo;
}

/// The check for a main checkout,
/// whose Git directory is its `.git`.
fn check(repo: &Path) -> Result<CleanSource, CleanRefusal> {
    let git_dir: PathBuf = repo.join(".git");
    return check_clean(&git_context(), repo, git_dir.as_path(), git_dir.as_path());
}

/// A clean,
/// aged worktree is clean and reports the commit and tree of `HEAD`.
#[test]
fn a_clean_worktree_is_clean_and_names_its_head() {
    let root: PathBuf = fixture("clean-ok");
    let repo: PathBuf = aged_repository(root.as_path());
    std::fs::create_dir(repo.join("ignored")).expect("ignored directory");
    std::fs::write(repo.join("ignored/cache"), b"cache\n").expect("ignored file");
    let clean: CleanSource = check(repo.as_path()).expect("clean");
    assert_eq!(clean.head_commit, git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim());
    assert_eq!(to_hex_for_test(clean.head_tree.as_slice()), git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]).trim());
    assert_eq!(clean.tracked, 4);
    remove(root.as_path());
}

/// The raw object name as hexadecimal.
fn to_hex_for_test(name: &[u8]) -> String {
    return name.iter().map(|byte: &u8| { return format!("{byte:02x}"); }).collect();
}

/// Each difference is refused,
/// and the reason names the path or the state.
#[test]
fn each_difference_is_refused_with_its_reason() {
    let root: PathBuf = fixture("clean-dirty");
    type Make = fn(&Path);
    let cases: [(&str, &str, CleanRefusalKind, Make); 10] = [
        ("rewritten content", "a.txt differs from its index entry", CleanRefusalKind::StatStale, |tree| { std::fs::write(tree.join("a.txt"), b"ALPHA\n").expect("write"); }),
        ("deleted", "dir/b.txt is missing", CleanRefusalKind::FileChanged, |tree| { std::fs::remove_file(tree.join("dir/b.txt")).expect("remove"); }),
        ("executable bit", "a.txt changed its executable bit", CleanRefusalKind::FileChanged, |tree| {
            std::fs::set_permissions(tree.join("a.txt"), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("chmod");
        }),
        ("type change", "link changed its type", CleanRefusalKind::FileChanged, |tree| {
            std::fs::remove_file(tree.join("link")).expect("remove");
            std::fs::write(tree.join("link"), b"a.txt").expect("write");
        }),
        ("untracked", "new.txt is untracked and not ignored", CleanRefusalKind::Untracked, |tree| { std::fs::write(tree.join("new.txt"), b"n").expect("write"); }),
        ("untracked in a directory", "dir/", CleanRefusalKind::Untracked, |tree| { std::fs::write(tree.join("dir/new.txt"), b"n").expect("write"); }),
        ("staged change", "a.txt is staged with a change", CleanRefusalKind::StagedChange, |tree| {
            std::fs::write(tree.join("a.txt"), b"staged\n").expect("write");
            git(tree, &["add", "a.txt"]);
        }),
        ("staged deletion", "the index and the tree of HEAD hold different numbers", CleanRefusalKind::StagedChange, |tree| { git(tree, &["rm", "--quiet", "--cached", "a.txt"]); }),
        ("merge in progress", "MERGE_HEAD exists", CleanRefusalKind::OperationInProgress("MERGE_HEAD"), |tree| {
            let head: String = git_text(tree, &["rev-parse", "HEAD"]);
            std::fs::write(tree.join(".git/MERGE_HEAD"), head.as_bytes()).expect("write");
        }),
        ("held index lock", "index.lock exists", CleanRefusalKind::OperationInProgress("index.lock"), |tree| { std::fs::write(tree.join(".git/index.lock"), b"").expect("write"); }),
    ];
    for (label, expected, kind, make) in cases {
        let one: PathBuf = repository(root.as_path(), format!("case-{}", label.replace(' ', "-")).as_str());
        std::fs::write(one.join(".gitignore"), b"ignored/\n").expect("gitignore");
        std::fs::write(one.join("a.txt"), b"alpha\n").expect("a");
        std::fs::create_dir(one.join("dir")).expect("dir");
        std::fs::write(one.join("dir/b.txt"), b"bravo\n").expect("b");
        std::os::unix::fs::symlink("a.txt", one.join("link")).expect("link");
        for name in [".gitignore", "a.txt", "dir/b.txt", "link"] {
            age(one.join(name).as_path());
        }
        git(one.as_path(), &["add", "--all"]);
        git(one.as_path(), &["commit", "--quiet", "--message=files"]);
        assert!(check(one.as_path()).is_ok(), "{label}: clean before the change");
        make(one.as_path());
        let refusal: CleanRefusal = check(one.as_path()).expect_err(label);
        assert!(refusal.reason.contains(expected), "{label}: {refusal}");
        assert_eq!(refusal.kind, kind, "{label}: the kind is what the callers decide by");
    }
    remove(root.as_path());
}

/// An entry written in the same second as the index proves nothing by its stat fields,
/// so it is refused.
#[test]
fn a_racy_entry_is_refused() {
    let root: PathBuf = fixture("clean-racy");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("fresh.txt"), b"fresh\n").expect("fresh");
    git(repo.as_path(), &["add", "fresh.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=fresh"]);
    let refusal: CleanRefusal = check(repo.as_path()).expect_err("racy");
    assert!(refusal.reason.contains("as new as the index"), "{refusal}");
    assert_eq!(refusal.kind, CleanRefusalKind::Racy);
    remove(root.as_path());
}

/// A submodule entry is refused:
/// its working directory would point at the source's module directory.
#[test]
fn a_submodule_entry_is_refused() {
    let root: PathBuf = fixture("clean-submodule");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["update-index", "--add", "--cacheinfo", format!("160000,{},sub", head.trim()).as_str()]);
    git(repo.as_path(), &["commit", "--quiet", "--message=submodule"]);
    let refusal: CleanRefusal = check(repo.as_path()).expect_err("submodule");
    assert!(refusal.reason.contains("sub is a submodule"), "{refusal}");
    assert_eq!(refusal.kind, CleanRefusalKind::NotPlain);
    remove(root.as_path());
}

/// A linked worktree is checked against its own Git directory and index.
#[test]
fn a_linked_worktree_is_checked_against_its_own_git_directory() {
    let root: PathBuf = fixture("clean-linked");
    let repo: PathBuf = aged_repository(root.as_path());
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "linked", "../linked"]);
    let linked: PathBuf = std::fs::canonicalize(root.join("linked")).expect("canonical");
    for name in ["a.txt", "dir/b.txt", ".gitignore", "link"] {
        age(linked.join(name).as_path());
    }
    git(linked.as_path(), &["update-index", "--refresh"]);
    let admin: PathBuf = repo.join(".git/worktrees/linked");
    // Git wrote the linked index within the second the files were aged, so refresh made it newer than the entries.
    let outcome: Result<CleanSource, CleanRefusal> = check_clean(&git_context(), linked.as_path(), admin.as_path(), repo.join(".git").as_path());
    assert!(outcome.is_ok(), "{outcome:?}");
    std::fs::write(linked.join("untracked.txt"), b"u").expect("write");
    assert!(check_clean(&git_context(), linked.as_path(), admin.as_path(), repo.join(".git").as_path()).is_err());
    remove(root.as_path());
}

/// The comparison below a held directory finds a tracked directory that was replaced by a link,
/// which a walk by path would
///  follow to a copy;
/// with the real directories it agrees with the path comparison.
#[test]
fn the_comparison_below_a_handle_refuses_a_link_in_place_of_a_directory() {
    let root: PathBuf = fixture("clean-beneath");
    let repo: PathBuf = aged_repository(root.as_path());
    let clean: CleanSource = check(repo.as_path()).expect("clean");
    let entries: Vec<StatEntry> = stored_stats(clean.index_bytes.as_slice(), clean.hash_length).expect("entries");
    let held: std::fs::File = crate::fs_beneath::open_root(std::fs::canonicalize(repo.as_path()).expect("canonical").as_path()).expect("handle");
    assert_eq!(tree_differs_from_index_beneath(&held, entries.as_slice(), clean.index_seconds), None, "positive control");
    std::fs::create_dir(root.join("copy")).expect("copy");
    std::fs::copy(repo.join("dir/b.txt"), root.join("copy/b.txt")).expect("copy of the file");
    std::fs::rename(repo.join("dir"), root.join("dir.orig")).expect("moved away");
    std::os::unix::fs::symlink(root.join("copy"), repo.join("dir")).expect("link in its place");
    let reason: String = tree_differs_from_index_beneath(&held, entries.as_slice(), clean.index_seconds).expect("refused");
    assert!(reason.contains("dir/b.txt") && reason.contains("link"), "{reason}");
    remove(root.as_path());
}

/// A metadata that equals the entry is clean and one that does not names its path.
#[test]
fn metadata_of_a_file_in_hand_is_judged_like_its_path() {
    let root: PathBuf = fixture("clean-metadata");
    let repo: PathBuf = aged_repository(root.as_path());
    let clean: CleanSource = check(repo.as_path()).expect("clean");
    let entries: Vec<StatEntry> = stored_stats(clean.index_bytes.as_slice(), clean.hash_length).expect("entries");
    let entry: &StatEntry = entries.iter().find(|entry: &&StatEntry| { return entry.path == b"a.txt"; }).expect("a.txt");
    let held = std::fs::File::open(repo.join("a.txt")).expect("open").metadata().expect("metadata");
    assert_eq!(metadata_differs_from_entry(entry, &held, clean.index_seconds), None);
    std::fs::write(repo.join("a.txt"), b"ALPHA\n").expect("rewritten");
    let changed = std::fs::File::open(repo.join("a.txt")).expect("open").metadata().expect("metadata");
    assert!(metadata_differs_from_entry(entry, &changed, clean.index_seconds).expect("changed").contains("a.txt"));
    remove(root.as_path());
}

/// Which kinds a reset can undo:
/// ordinary changes,
/// and neither an operation in progress nor an entry Git keeps outside the files.
#[test]
fn only_ordinary_changes_are_restorable() {
    let restorable = [CleanRefusalKind::StagedChange, CleanRefusalKind::FileChanged, CleanRefusalKind::StatStale, CleanRefusalKind::Racy, CleanRefusalKind::Untracked];
    let kept = [CleanRefusalKind::OperationInProgress("index.lock"), CleanRefusalKind::Unreadable, CleanRefusalKind::NotPlain];
    assert!(restorable.iter().all(|kind: &CleanRefusalKind| { return kind.restorable(); }));
    assert!(kept.iter().all(|kind: &CleanRefusalKind| { return !kind.restorable(); }));
}

/// A staged change that comes first in the index does not hide a skip-worktree,
/// assume-unchanged or submodule entry that
///  comes later,
/// and an operation in progress is never restorable;
/// an ordinary change is,
/// and names `HEAD` with no index bytes.
#[test]
fn a_dirty_source_is_found_only_when_nothing_a_reset_cannot_undo_hides_in_it() {
    let root: PathBuf = fixture("clean-dirty-source");
    let repo: PathBuf = aged_repository(root.as_path());
    let git_dir: PathBuf = repo.join(".git");
    let why = |tree: &Path| -> CleanRefusal { return check(tree).expect_err("not clean"); };
    // A staged change on the first path in index order.
    std::fs::write(repo.join(".gitignore"), b"ignored/
staged
").expect("write");
    git(repo.as_path(), &["add", ".gitignore"]);
    let refusal: CleanRefusal = why(repo.as_path());
    assert_eq!(refusal.kind, CleanRefusalKind::StagedChange);
    let found: DirtySource = dirty_source(git_dir.as_path(), git_dir.as_path(), refusal.clone()).expect("an ordinary change");
    assert_eq!(found.source.head_commit, git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim());
    assert_eq!(found.source.index_bytes, std::fs::read(git_dir.join("index")).expect("index"), "the index as read, which proves nothing about the files");
    assert_eq!(found.why, refusal);
    for (label, arguments, expected) in [
        ("skip-worktree", vec!["update-index", "--skip-worktree", "dir/b.txt"], "skip-worktree"),
        ("assume-unchanged", vec!["update-index", "--assume-unchanged", "a.txt"], "assume-unchanged"),
    ] {
        git(repo.as_path(), arguments.as_slice());
        let hidden = dirty_source(git_dir.as_path(), git_dir.as_path(), why(repo.as_path())).expect_err(label);
        assert_eq!(hidden.kind, CleanRefusalKind::NotPlain, "{label}");
        assert!(hidden.reason.contains(expected), "{label}: {hidden}");
        git(repo.as_path(), &["update-index", if label == "skip-worktree" { "--no-skip-worktree" } else { "--no-assume-unchanged" }, if label == "skip-worktree" { "dir/b.txt" } else { "a.txt" }]);
    }
    std::fs::write(repo.join(".git/MERGE_HEAD"), git_text(repo.as_path(), &["rev-parse", "HEAD"]).as_bytes()).expect("merge mark");
    let merging = why(repo.as_path());
    assert_eq!(dirty_source(git_dir.as_path(), git_dir.as_path(), merging.clone()), Err(merging), "an operation in progress stays a refusal");
    remove(root.as_path());
}

/// The entries of a clean fixture and the index the walk compares them with.
fn walk_inputs(clean: &CleanSource) -> (Vec<StatEntry>, Vec<crate::repo_read_index::IndexEntry>) {
    let entries: Vec<StatEntry> = stored_stats(clean.index_bytes.as_slice(), clean.hash_length).expect("entries");
    let indexed: Vec<crate::repo_read_index::IndexEntry> = crate::repo_read_index::parse_index(clean.index_bytes.as_slice(), crate::repo_read_layout::ObjectFormat::Sha1).expect("the index of the fixture is a plain one");
    return (entries, indexed);
}

/// The content decision's target for a fixture worktree,
/// whose Git directory is the one named.
fn content_target<'a>(worktree: &'a Path, git_dir: &'a Path, environment: &'a crate::worktree_clean_content::ConversionEnvironment) -> ContentTarget<'a> {
    return ContentTarget { root: worktree, git_dir, common_dir: git_dir, environment, format: crate::repo_read_layout::ObjectFormat::Sha1 };
}

/// One walk below the worktree root both judges every tracked file against its entry and returns the stat fields the snapshot's
///  index takes,
/// equal to what a lookup of each path alone returns,
/// so the files are looked at once.
#[test]
fn one_walk_judges_the_files_and_returns_their_stat_fields() {
    let root: PathBuf = fixture("clean-one-walk");
    let repo: PathBuf = aged_repository(root.as_path());
    let clean: CleanSource = check(repo.as_path()).expect("clean");
    let (entries, indexed) = walk_inputs(&clean);
    let environment = crate::fixture_content_decision::content_environment();
    let canonical: PathBuf = std::fs::canonicalize(repo.as_path()).expect("canonical");
    let git_dir: PathBuf = canonical.join(".git");
    let fields: Vec<StatFields> = tree_fields_by_content(&content_target(canonical.as_path(), git_dir.as_path(), &environment), entries.as_slice(), indexed.as_slice(), clean.index_seconds).expect("every file agrees");
    let held: std::fs::File = crate::fs_beneath::open_root(canonical.as_path()).expect("handle");
    assert_eq!(fields.len(), entries.len());
    for (entry, found) in entries.iter().zip(&fields) {
        assert_eq!(Some(found.clone()), stat_fields_beneath(&held, entry.path.as_slice()), "{}", String::from_utf8_lossy(entry.path.as_slice()));
    }
    remove(root.as_path());
}

/// A file whose stat data cannot vouch for it,
/// and whose content still equals its committed blob,
/// is settled by content instead
///  of refused,
/// and the walk returns its stat fields.
#[test]
fn one_walk_settles_a_stale_entry_by_content() {
    let root: PathBuf = fixture("clean-one-walk-stale");
    let repo: PathBuf = aged_repository(root.as_path());
    let clean: CleanSource = check(repo.as_path()).expect("clean");
    let (entries, indexed) = walk_inputs(&clean);
    let environment = crate::fixture_content_decision::content_environment();
    let canonical: PathBuf = std::fs::canonicalize(repo.as_path()).expect("canonical");
    // The same bytes written again: the size and the content stand, the times the index recorded do not.
    std::fs::write(repo.join("a.txt"), b"alpha\n").expect("rewrite");
    let git_dir: PathBuf = canonical.join(".git");
    let fields: Vec<StatFields> = tree_fields_by_content(&content_target(canonical.as_path(), git_dir.as_path(), &environment), entries.as_slice(), indexed.as_slice(), clean.index_seconds).expect("the content settles the entry");
    assert_eq!(fields.len(), entries.len());
    remove(root.as_path());
}

/// The one walk refuses a changed file and names its path.
#[test]
fn the_one_walk_refuses_a_changed_file() {
    let root: PathBuf = fixture("clean-one-walk-changed");
    let repo: PathBuf = aged_repository(root.as_path());
    let clean: CleanSource = check(repo.as_path()).expect("clean");
    let (entries, indexed) = walk_inputs(&clean);
    let environment = crate::fixture_content_decision::content_environment();
    let canonical: PathBuf = std::fs::canonicalize(repo.as_path()).expect("canonical");
    std::fs::write(repo.join("a.txt"), b"alpha, changed and longer\n").expect("change");
    let git_dir: PathBuf = canonical.join(".git");
    let changed: CleanRefusal = tree_fields_by_content(&content_target(canonical.as_path(), git_dir.as_path(), &environment), entries.as_slice(), indexed.as_slice(), clean.index_seconds).expect_err("a changed file");
    assert!(changed.reason.contains("a.txt"), "{}", changed.reason);
    assert_eq!(changed.kind, CleanRefusalKind::FileChanged);
    remove(root.as_path());
}

/// The one walk does not read through a link that stands where a directory was.
#[test]
fn the_one_walk_refuses_a_link_in_place_of_a_directory() {
    let root: PathBuf = fixture("clean-one-walk-link");
    let repo: PathBuf = aged_repository(root.as_path());
    let clean: CleanSource = check(repo.as_path()).expect("clean");
    let (entries, indexed) = walk_inputs(&clean);
    let environment = crate::fixture_content_decision::content_environment();
    let canonical: PathBuf = std::fs::canonicalize(repo.as_path()).expect("canonical");
    std::fs::create_dir(root.join("copy")).expect("copy");
    std::fs::copy(repo.join("dir/b.txt"), root.join("copy/b.txt")).expect("copy of the file");
    std::fs::rename(repo.join("dir"), root.join("dir.orig")).expect("moved away");
    std::os::unix::fs::symlink(root.join("copy"), repo.join("dir")).expect("link in its place");
    let git_dir: PathBuf = canonical.join(".git");
    let linked: CleanRefusal = tree_fields_by_content(&content_target(canonical.as_path(), git_dir.as_path(), &environment), entries.as_slice(), indexed.as_slice(), clean.index_seconds).expect_err("a link");
    assert!(linked.reason.contains("dir/b.txt"), "{}", linked.reason);
    remove(root.as_path());
}

/// A cursor keeps the directory it opened for the next path in it,
/// and opens another when the parent changes.
#[test]
fn the_cursor_answers_like_a_lookup_of_each_path_alone() {
    let root: PathBuf = fixture("clean-cursor");
    let repo: PathBuf = aged_repository(root.as_path());
    let held: std::fs::File = crate::fs_beneath::open_root(std::fs::canonicalize(repo.as_path()).expect("canonical").as_path()).expect("handle");
    let mut cursor: StatCursor<'_> = StatCursor::new(&held);
    for path in ["a.txt", "dir/b.txt", "a.txt", "link", "dir/b.txt", "dir/absent", "absent/dir/file"] {
        assert_eq!(cursor.fields(path.as_bytes()), stat_fields_beneath(&held, path.as_bytes()), "{path}");
    }
    remove(root.as_path());
}

/// What:
///  The tracked files are proven as a job of the process's blocking pool,
/// beside the untracked scan.
/// Why:
///   The proof walks every tracked file,
/// so a filesystem that stopped answering it would hold the scan's bounded wait
///  open if it ran on the task that awaits that scan;
/// a job of the pool is the only place it may run.
/// Only a job of the pool asks for a place of the process's bound on blocking jobs,
///  so an ask is the proof that the work left the calling thread:
/// the holder gives its place back on that ask and never on a clock.
#[test]
fn the_tracked_files_are_proven_as_a_job_of_the_processs_blocking_pool() {
    let root: PathBuf = fixture("clean-a-place-of-the-bound");
    let repo: PathBuf = aged_repository(root.as_path());
    let (_given_back, finished) = hold_the_only_place();
    let clean: Result<CleanSource, CleanRefusal> = check(repo.as_path());
    assert!(clean.is_ok(), "{:?}", clean.err());
    assert!(
        finished.recv_timeout(PATIENCE).expect("the job that holds the only place never returned"),
        "the tracked files were proven without asking for a place of the process's bound on blocking jobs"
    );
    remove(root.as_path());
}

/// What:
///  Where the system refuses every thread,
/// the tracked files are still proven,
/// and the words of the refusal are the proof's own.
/// Why:
///   The owner's rule is "Commits should never fail.",
/// so a pool that cannot start a thread costs the check its overlap and never its answer.
///  The staged change is the tracked proof's refusal,
/// and a check that gave up on the pool would name the pool instead.
/// The body runs in a child process that is refused every thread it asks for.
#[test]
fn a_worktree_that_is_not_clean_is_still_refused_where_no_thread_can_start() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "a_worktree_that_is_not_clean_is_still_refused_where_no_thread_can_start") {
        return;
    }
    // The fixture's Git runs are processes of their own, so the limit is lifted while the repository is made and put back
    //  for the part this control measures.
    crate::no_threads_child_test_support::let_threads_start_again().expect("the process limit rises");
    let root: PathBuf = fixture("clean-no-thread");
    let repo: PathBuf = aged_repository(root.as_path());
    std::fs::write(repo.join("a.txt"), b"edited\n").expect("edit");
    git(repo.as_path(), &["add", "a.txt"]);
    crate::no_threads_child_test_support::refuse_threads_again().expect("the process limit falls");
    let refused: Result<CleanSource, CleanRefusal> = check(repo.as_path());
    crate::no_threads_child_test_support::let_threads_start_again().expect("the process limit rises again");
    let why: CleanRefusal = refused.expect_err("a staged change is not clean");
    assert!(why.reason.contains("a.txt is staged with a change"), "{}", why.reason);
    remove(root.as_path());
}
