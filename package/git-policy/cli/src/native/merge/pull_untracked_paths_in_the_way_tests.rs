//! What:
//!  Controls for the untracked files a pull's branch move would overwrite:
//!  the copy made of each one,
//!  the numbered folder that holds them,
//!  and the words that name where they are.
//! Why:
//!  A pull that stops must not lose the file that made it stop,
//!  and the reset Git's own words offer deletes it
//!  (the owner's rule "never discard user data").
//!  A file or a link standing where the move needs a folder is such a file:
//!  the reset replaces it with that folder.

use super::{KEPT_FOLDER_NAME, PathLeftInTheWay, UntrackedInTheWay, keep_the_untracked_paths_in_the_way, recovery_words};
use crate::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::{Path, PathBuf};

/// The bytes only the untracked file holds.
const PRECIOUS: &str = "the only copy of this line\n";

/// The command the words offer,
/// which is the hatch in the one place the words mean to run it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const offered = `git reset ${WORKTREE_ENFORCEMENT_ESCAPE_HATCH} --hard`;
/// ```
fn offered() -> String {
    return format!("git reset {WORKTREE_ENFORCEMENT_ESCAPE_HATCH} --hard");
}

/// Where the words offer the reset,
/// as an offset into them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const offeredAt = (words) => words.indexOf(offered);
/// ```
fn offered_at(words: &str) -> usize {
    return words.find(offered().as_str()).unwrap_or_else(|| panic!("the words offer the reset: {words}"));
}

/// What:
///  A repository whose `main` holds one commit,
///  a branch `topic` that adds `name` as a file (making the folders above it),
///  and the worktree back on `main` so that `name` is untracked there.
/// Why:
///   The state the pull's move fails in is one commit that adds a path,
///  and a file Git does not track standing at it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { root, from, to } = aCommitThatAdds('new.txt');
/// ```
fn a_commit_that_adds(fixture_root: &Path, name: &str) -> (PathBuf, String, String) {
    return a_commit_that_adds_every(fixture_root, &[name]);
}

/// What:
///  `a_commit_that_adds` for several paths,
///  which is what a move whose entry stands on the way to more than one of them needs.
/// Why:
///   Two added paths below one entry must keep that entry once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { root, from, to } = aCommitThatAddsEvery(['dir/one.txt', 'dir/two.txt']);
/// ```
fn a_commit_that_adds_every(fixture_root: &Path, names: &[&str]) -> (PathBuf, String, String) {
    let root: PathBuf = repository(fixture_root, "repo");
    let from: String = git_text(root.as_path(), &["rev-parse", "HEAD"]);
    git(root.as_path(), &["branch", "topic"]);
    for name in names {
        let path: PathBuf = root.join(name);
        if let Some(above) = path.parent() {
            std::fs::create_dir_all(above).expect("the folders the new commit writes below");
        }
        std::fs::write(path, b"the commit's own bytes\n").expect("the file the new commit adds");
    }
    let list: Vec<&str> = names.to_vec();
    let mut added: Vec<&str> = vec!["add", "--"];
    added.extend(list.iter().copied());
    git(root.as_path(), added.as_slice());
    git(root.as_path(), &["commit", "--quiet", "--message=adds-the-path"]);
    let to: String = git_text(root.as_path(), &["rev-parse", "HEAD"]);
    // The worktree goes back to the commit without the path, and the commit stays on `topic`.
    git(root.as_path(), &["reset", "--quiet", "--hard", from.as_str()]);
    return (root, from, to);
}

/// A file Git does not track,
/// at a path the new commit adds,
/// is copied beside the Git directory,
/// and the words name both paths.
#[test]
fn a_file_the_move_would_overwrite_is_copied_beside_the_git_directory() {
    let fixture_root: PathBuf = fixture("pull-untracked-file");
    let (root, from, to): (PathBuf, String, String) = a_commit_that_adds(fixture_root.as_path(), "new.txt");
    std::fs::write(root.join("new.txt"), PRECIOUS.as_bytes()).expect("the untracked file");

    let in_the_way: UntrackedInTheWay = keep_the_untracked_paths_in_the_way(&git_context(), root.as_path(), root.join(".git").as_path(), from.as_str(), to.as_str());

    assert_eq!(in_the_way.kept.len(), 1, "{in_the_way:?}");
    assert!(in_the_way.left.is_empty(), "{in_the_way:?}");
    let copy: &Path = in_the_way.kept[0].in_the_copy.as_path();
    assert_eq!(std::fs::read(copy).expect("the copy is there"), PRECIOUS.as_bytes(), "the copy holds the untracked bytes");
    assert_eq!(copy.file_name().expect("a name"), std::ffi::OsStr::new("new.txt"));
    let folder: &Path = copy.parent().expect("the copy's folder");
    assert_eq!(folder.file_name().expect("a name"), std::ffi::OsStr::new("1"), "the first numbered folder");
    assert_eq!(folder.parent().expect("the folder's parent").file_name().expect("a name"), std::ffi::OsStr::new(KEPT_FOLDER_NAME));

    let words: String = recovery_words(from.as_str(), &in_the_way);
    assert!(words.contains(&format!("`{}`", copy.display())), "the words name the copy: {words}");
    assert!(words.contains("git diff "), "Git's own recovery words stay: {words}");
    assert_eq!(words.matches(offered().as_str()).count(), 1, "the reset is offered once: {words}");
    remove(fixture_root.as_path());
}

/// A second pull that fails does not write over the first pull's copies.
#[test]
fn a_second_keep_makes_a_second_numbered_folder() {
    let fixture_root: PathBuf = fixture("pull-untracked-twice");
    let (root, from, to): (PathBuf, String, String) = a_commit_that_adds(fixture_root.as_path(), "new.txt");
    std::fs::write(root.join("new.txt"), PRECIOUS.as_bytes()).expect("the untracked file");

    let first: UntrackedInTheWay = keep_the_untracked_paths_in_the_way(&git_context(), root.as_path(), root.join(".git").as_path(), from.as_str(), to.as_str());
    let second: UntrackedInTheWay = keep_the_untracked_paths_in_the_way(&git_context(), root.as_path(), root.join(".git").as_path(), from.as_str(), to.as_str());

    assert_eq!(first.kept.len(), 1, "{first:?}");
    assert_eq!(second.kept.len(), 1, "{second:?}");
    assert_ne!(first.kept[0].in_the_copy, second.kept[0].in_the_copy, "the second keep writes another folder");
    assert_eq!(std::fs::read(second.kept[0].in_the_copy.as_path()).expect("the second copy is there"), PRECIOUS.as_bytes());
    assert_eq!(std::fs::read(first.kept[0].in_the_copy.as_path()).expect("the first copy stays"), PRECIOUS.as_bytes());
    remove(fixture_root.as_path());
}

/// A symbolic link is kept as a link with its own target,
/// never copied by following it.
#[test]
fn a_symbolic_link_is_kept_as_a_link() {
    let fixture_root: PathBuf = fixture("pull-untracked-link");
    let (root, from, to): (PathBuf, String, String) = a_commit_that_adds(fixture_root.as_path(), "link");
    std::os::unix::fs::symlink("target.txt", root.join("link")).expect("the untracked link");

    let in_the_way: UntrackedInTheWay = keep_the_untracked_paths_in_the_way(&git_context(), root.as_path(), root.join(".git").as_path(), from.as_str(), to.as_str());

    assert_eq!(in_the_way.kept.len(), 1, "{in_the_way:?}");
    let copy: &Path = in_the_way.kept[0].in_the_copy.as_path();
    assert!(std::fs::symlink_metadata(copy).expect("the copy is there").file_type().is_symlink(), "the copy is a link");
    assert_eq!(std::fs::read_link(copy).expect("the copy's target"), PathBuf::from("target.txt"));
    remove(fixture_root.as_path());
}

/// A directory standing where the new commit writes a file is named,
/// and nothing is copied from inside it.
#[test]
fn a_directory_in_the_way_is_named_and_not_copied() {
    let fixture_root: PathBuf = fixture("pull-untracked-directory");
    let (root, from, to): (PathBuf, String, String) = a_commit_that_adds(fixture_root.as_path(), "blocker");
    std::fs::create_dir(root.join("blocker")).expect("the directory in the way");
    std::fs::write(root.join("blocker").join("inside.txt"), PRECIOUS.as_bytes()).expect("a file inside it");

    let in_the_way: UntrackedInTheWay = keep_the_untracked_paths_in_the_way(&git_context(), root.as_path(), root.join(".git").as_path(), from.as_str(), to.as_str());

    assert!(in_the_way.kept.is_empty(), "{in_the_way:?}");
    assert_eq!(in_the_way.left.len(), 1, "{in_the_way:?}");
    assert_eq!(in_the_way.left[0].in_the_worktree, root.join("blocker"));
    assert!(in_the_way.left[0].why.contains("directory"), "{}", in_the_way.left[0].why);

    let words: String = recovery_words(from.as_str(), &in_the_way);
    let named: usize = words.find("blocker").expect("the words name the directory");
    assert!(named < offered_at(words.as_str()), "the directory is named before the reset is offered: {words}");
    remove(fixture_root.as_path());
}

/// A listing that could not run keeps nothing and says so,
/// and the reset is offered only after the untracked files are saved.
#[test]
fn a_listing_that_could_not_run_offers_the_reset_last() {
    let in_the_way: UntrackedInTheWay = UntrackedInTheWay {
        listing_failure: Some(String::from("git diff-tree answered status 128")),
        kept: Vec::new(),
        left: Vec::new(),
        folder: None,
    };
    let words: String = recovery_words("0123456789abcdef0123456789abcdef01234567", &in_the_way);

    assert!(words.contains("git ls-files --others"), "the words name how to list what Git does not track: {words}");
    assert_eq!(words.matches(offered().as_str()).count(), 1, "the reset is offered once: {words}");
    let saved: usize = words.find("git ls-files --others").expect("the listing command");
    assert!(saved < offered_at(words.as_str()), "the reset is offered after saving: {words}");
    assert!(words.contains("could not be listed"), "{words}");
}

/// Nothing in the way keeps the words Git's failure had,
/// with the one line that names the hatch.
#[test]
fn nothing_in_the_way_keeps_the_words_the_wrapper_had() {
    let in_the_way: UntrackedInTheWay = UntrackedInTheWay { listing_failure: None, kept: Vec::new(), left: Vec::new(), folder: None };
    let words: String = recovery_words("0123456789abcdef0123456789abcdef01234567", &in_the_way);

    assert!(words.starts_with("fatal: Cannot fast-forward your working tree.\n"), "{words}");
    assert!(words.contains("$ git reset --hard\nto recover.\n"), "{words}");
    assert!(words.ends_with(&format!("; run `{}` if you mean it.\n", offered())), "{words}");
}

/// A kept path and a left path in one answer:
///  the kept sentence names the copy,
///  and the left sentence names what to move and offers the reset there.
#[test]
fn kept_and_left_paths_are_worded_together() {
    let in_the_way: UntrackedInTheWay = UntrackedInTheWay {
        listing_failure: None,
        kept: vec![super::KeptPath { in_the_worktree: PathBuf::from("/r/new.txt"), in_the_copy: PathBuf::from("/r/.git/data/pull-kept/1/new.txt") }],
        left: vec![PathLeftInTheWay { in_the_worktree: PathBuf::from("/r/blocker"), why: String::from("it is a directory, and the wrapper copies plain files and symbolic links") }],
        folder: Some(PathBuf::from("/r/.git/data/pull-kept/1")),
    };
    let words: String = recovery_words("0123456789abcdef0123456789abcdef01234567", &in_the_way);

    assert!(words.contains("`/r/.git/data/pull-kept/1/new.txt`"), "{words}");
    assert!(words.contains("nothing was copied from `/r/blocker`"), "{words}");
    assert_eq!(words.matches(offered().as_str()).count(), 1, "the reset is offered once: {words}");
    let moved: usize = words.find("Move what stands there out of the worktree first").expect("the move comes first");
    assert!(moved < offered_at(words.as_str()), "the reset is offered after the move: {words}");
}

/// A file a person can run keeps its permission bits in the copy,
/// because an entry is its bytes and its mode.
#[test]
fn a_runnable_file_keeps_its_mode_in_the_copy() {
    use std::os::unix::fs::PermissionsExt as _;
    let fixture_root: PathBuf = fixture("pull-untracked-mode");
    let (root, from, to): (PathBuf, String, String) = a_commit_that_adds(fixture_root.as_path(), "run.sh");
    std::fs::write(root.join("run.sh"), PRECIOUS.as_bytes()).expect("the untracked file");
    std::fs::set_permissions(root.join("run.sh"), std::fs::Permissions::from_mode(0o755)).expect("the file runs");

    let in_the_way: UntrackedInTheWay = keep_the_untracked_paths_in_the_way(&git_context(), root.as_path(), root.join(".git").as_path(), from.as_str(), to.as_str());

    assert_eq!(in_the_way.kept.len(), 1, "{in_the_way:?}");
    let mode: u32 = std::fs::metadata(in_the_way.kept[0].in_the_copy.as_path()).expect("the copy is there").permissions().mode() & 0o777;
    assert_eq!(mode, 0o755, "the copy runs as the original did");
    remove(fixture_root.as_path());
}

/// An empty directory holds no bytes,
/// so the words do not ask the person to move one,
/// and they still offer the reset.
#[test]
fn an_empty_directory_in_the_way_is_not_named() {
    let fixture_root: PathBuf = fixture("pull-untracked-empty-directory");
    let (root, from, to): (PathBuf, String, String) = a_commit_that_adds(fixture_root.as_path(), "blocker");
    std::fs::create_dir(root.join("blocker")).expect("the empty directory in the way");

    let in_the_way: UntrackedInTheWay = keep_the_untracked_paths_in_the_way(&git_context(), root.as_path(), root.join(".git").as_path(), from.as_str(), to.as_str());

    assert!(in_the_way.kept.is_empty(), "{in_the_way:?}");
    assert!(in_the_way.left.is_empty(), "{in_the_way:?}");
    let words: String = recovery_words(from.as_str(), &in_the_way);
    assert!(!words.contains("blocker"), "an empty directory is not named: {words}");
    assert_eq!(words.matches(offered().as_str()).count(), 1, "the reset is still offered: {words}");
    remove(fixture_root.as_path());
}

/// A file Git does not track standing where an added path needs a folder
/// is the entry the reset replaces,
/// so its bytes are copied and the words name the path that stands there.
#[test]
fn a_file_on_the_way_to_an_added_path_is_copied() {
    let fixture_root: PathBuf = fixture("pull-untracked-parent-file");
    let (root, from, to): (PathBuf, String, String) = a_commit_that_adds(fixture_root.as_path(), "dir/new.txt");
    // The reset that put the worktree back on the old commit removed the folder the commit made.
    std::fs::write(root.join("dir"), PRECIOUS.as_bytes()).expect("the file standing on the way");

    let in_the_way: UntrackedInTheWay = keep_the_untracked_paths_in_the_way(&git_context(), root.as_path(), root.join(".git").as_path(), from.as_str(), to.as_str());

    assert!(in_the_way.listing_failure.is_none(), "{in_the_way:?}");
    assert_eq!(in_the_way.kept.len(), 1, "{in_the_way:?}");
    assert_eq!(in_the_way.kept[0].in_the_worktree, root.join("dir"), "the words name the entry that stands there");
    let copy: &Path = in_the_way.kept[0].in_the_copy.as_path();
    assert_eq!(std::fs::read(copy).expect("the copy is there"), PRECIOUS.as_bytes(), "the copy holds the file's bytes");
    assert_eq!(copy.file_name().expect("a name"), std::ffi::OsStr::new("dir"), "the copy keeps the standing entry's own name");

    let words: String = recovery_words(from.as_str(), &in_the_way);
    assert!(words.contains(&format!("`{}`", copy.display())), "the words name the copy: {words}");
    assert!(!words.contains("new.txt"), "the words name the entry the reset removes, not the path below it: {words}");
    assert_eq!(words.matches(offered().as_str()).count(), 1, "the reset is offered once, and its copy is kept: {words}");
    remove(fixture_root.as_path());
}

/// A symbolic link standing where an added path needs a folder
/// is kept as a link with its own target,
/// which is the only record of that target when it leads nowhere.
#[test]
fn a_link_on_the_way_to_an_added_path_is_kept_as_a_link() {
    let fixture_root: PathBuf = fixture("pull-untracked-parent-link");
    let (root, from, to): (PathBuf, String, String) = a_commit_that_adds(fixture_root.as_path(), "dir/new.txt");
    let target: &str = "the-only-record-of-the-link-target";
    std::os::unix::fs::symlink(target, root.join("dir")).expect("the link standing on the way");

    let in_the_way: UntrackedInTheWay = keep_the_untracked_paths_in_the_way(&git_context(), root.as_path(), root.join(".git").as_path(), from.as_str(), to.as_str());

    assert!(in_the_way.listing_failure.is_none(), "{in_the_way:?}");
    assert_eq!(in_the_way.kept.len(), 1, "{in_the_way:?}");
    assert_eq!(in_the_way.kept[0].in_the_worktree, root.join("dir"));
    let copy: &Path = in_the_way.kept[0].in_the_copy.as_path();
    assert!(std::fs::symlink_metadata(copy).expect("the copy is there").file_type().is_symlink(), "the copy is a link");
    assert_eq!(std::fs::read_link(copy).expect("the copy's target"), PathBuf::from(target));

    let words: String = recovery_words(from.as_str(), &in_the_way);
    assert!(words.contains(&format!("`{}`", copy.display())), "the words name the copy: {words}");
    assert!(!words.contains("nothing was copied from"), "everything the reset removes was kept: {words}");
    remove(fixture_root.as_path());
}

/// Two added paths below one standing entry
/// are one line for the person and one copy.
#[test]
fn one_entry_on_the_way_of_two_added_paths_is_copied_once() {
    let fixture_root: PathBuf = fixture("pull-untracked-parent-twice");
    let (root, from, to): (PathBuf, String, String) = a_commit_that_adds_every(fixture_root.as_path(), &["dir/one.txt", "dir/two.txt"]);
    std::fs::write(root.join("dir"), PRECIOUS.as_bytes()).expect("the file standing on the way");

    let in_the_way: UntrackedInTheWay = keep_the_untracked_paths_in_the_way(&git_context(), root.as_path(), root.join(".git").as_path(), from.as_str(), to.as_str());

    assert_eq!(in_the_way.kept.len(), 1, "one entry, one copy: {in_the_way:?}");
    assert!(in_the_way.left.is_empty(), "{in_the_way:?}");
    assert_eq!(std::fs::read(in_the_way.kept[0].in_the_copy.as_path()).expect("the copy is there"), PRECIOUS.as_bytes());
    remove(fixture_root.as_path());
}

/// An entry the previous commit tracked is the move's own to replace,
/// and `git diff <old head>` holds its bytes,
/// so nothing is copied for it and nothing is named.
#[test]
fn a_file_the_previous_commit_tracked_on_the_way_is_not_copied() {
    let fixture_root: PathBuf = fixture("pull-untracked-tracked-parent");
    let root: PathBuf = repository(fixture_root.as_path(), "repo");
    std::fs::write(root.join("dir"), b"the old commit's own bytes\n").expect("the tracked file");
    git(root.as_path(), &["add", "--", "dir"]);
    git(root.as_path(), &["commit", "--quiet", "--message=a-file-called-dir"]);
    let from: String = git_text(root.as_path(), &["rev-parse", "HEAD"]);
    git(root.as_path(), &["branch", "topic"]);
    git(root.as_path(), &["rm", "--quiet", "--", "dir"]);
    std::fs::create_dir(root.join("dir")).expect("the folder the new commit writes below");
    std::fs::write(root.join("dir").join("new.txt"), b"the commit's own bytes\n").expect("the file the new commit adds");
    git(root.as_path(), &["add", "--", "dir/new.txt"]);
    git(root.as_path(), &["commit", "--quiet", "--message=a-folder-called-dir"]);
    let to: String = git_text(root.as_path(), &["rev-parse", "HEAD"]);
    git(root.as_path(), &["reset", "--quiet", "--hard", from.as_str()]);

    let in_the_way: UntrackedInTheWay = keep_the_untracked_paths_in_the_way(&git_context(), root.as_path(), root.join(".git").as_path(), from.as_str(), to.as_str());

    assert!(in_the_way.kept.is_empty(), "the tracked file's bytes are in `git diff` and not the wrapper's to copy: {in_the_way:?}");
    assert!(in_the_way.left.is_empty(), "{in_the_way:?}");
    let words: String = recovery_words(from.as_str(), &in_the_way);
    assert!(words.ends_with(&format!("; run `{}` if you mean it.\n", offered())), "the reset is offered at once: {words}");
    assert!(!words.contains("would have overwritten"), "no untracked file stands there: {words}");
    assert!(!words.contains("nothing was copied from"), "{words}");
    remove(fixture_root.as_path());
}

/// The listing keeps none of the caller's global options,
/// so a relative `-C` on the command line is not applied a second time
/// from the top of the worktree it already reached.
#[test]
fn a_relative_directory_change_of_the_caller_is_not_applied_twice() {
    let fixture_root: PathBuf = fixture("pull-untracked-relative-c");
    let (root, from, to): (PathBuf, String, String) = a_commit_that_adds(fixture_root.as_path(), "new.txt");
    std::fs::write(root.join("new.txt"), PRECIOUS.as_bytes()).expect("the untracked file");
    // The directory need not exist: what failed was the caller's `-C` reaching a start that already runs in the worktree.
    let mut context: crate::transaction_git::GitContext = git_context();
    context.global_prefix = vec![std::ffi::OsString::from("-C"), std::ffi::OsString::from("a-directory-the-caller-named")];

    let in_the_way: UntrackedInTheWay = keep_the_untracked_paths_in_the_way(&context, root.as_path(), root.join(".git").as_path(), from.as_str(), to.as_str());

    assert!(in_the_way.listing_failure.is_none(), "the listing runs whatever directories the caller's options name: {in_the_way:?}");
    assert_eq!(in_the_way.kept.len(), 1, "{in_the_way:?}");
    assert_eq!(std::fs::read(in_the_way.kept[0].in_the_copy.as_path()).expect("the copy is there"), PRECIOUS.as_bytes());
    remove(fixture_root.as_path());
}
