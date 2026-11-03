//! Differential controls: the in-process reset and `git reset --no-refresh --pathspec-from-file` run on two copies of one
//! repository, and the two index files must be equal byte for byte.
//!
//! `--no-refresh` is the comparison because the in-process update never refreshes: a refresh rewrites the stat data of the
//! entries a copy's changed inode numbers made stale, which the landing does not want and the fallback command does not do.

use super::support::{Edit, Plan, Scene, copy_of, finish, git_reset, git_with, index_bytes, landing_reset, listing, named, scene};
use super::*;
use crate::test_support::{git, git_context, git_text};
use std::path::PathBuf;

/// The two index files after the same reset by Git and in process, equal; `label` names the case.
fn same_as_git(scene: &Scene, names: &[Vec<u8>], label: &str) {
    let copy: PathBuf = copy_of(scene);
    let before: Vec<u8> = index_bytes(scene.repo.as_path());
    git_reset(scene.repo.as_path(), scene.new.as_str(), names);
    let done: bool = landing_reset(&git_context(), copy.as_path(), scene.new.as_str(), IndexPaths::Named(names));
    assert!(done, "{label}: the in-process update declined");
    let (git_bytes, our_bytes) = (index_bytes(scene.repo.as_path()), index_bytes(copy.as_path()));
    assert!(git_bytes != before, "{label}: the case changes nothing, so it proves nothing");
    assert!(git_bytes == our_bytes, "{label}: the index files differ\nGit:\n{}\nin process:\n{}", listing(scene.repo.as_path()), listing(copy.as_path()));
}

/// Sources of the first commit used by most cases.
const BASE: [(&str, &[u8], bool); 6] = [
    ("a", b"a one\n", false),
    ("b", b"b one\n", false),
    ("c", b"c one\n", false),
    ("d/x", b"x one\n", false),
    ("d/y", b"y one\n", true),
    ("z", b"z one\n", false),
];

/// Added, changed, removed and renamed paths, and a named path that did not change.
#[test]
fn added_changed_removed_and_renamed_paths_equal_git() {
    let edits: Vec<Edit<'_>> = vec![
        Edit::Put { path: "a", mode: "100644", content: b"a two\n" },
        Edit::Delete("b"),
        Edit::Put { path: "e", mode: "100644", content: b"e new\n" },
        Edit::Delete("c"),
        Edit::Put { path: "c2", mode: "100644", content: b"c one\n" },
        Edit::Put { path: "d/new/deep.txt", mode: "100644", content: b"deep\n" },
    ];
    let scene: Scene = scene("index-reset-basic", &Plan { files: &BASE, edits: edits.as_slice(), ..Plan::default() });
    same_as_git(&scene, &named(&["a", "b", "c", "c2", "e", "z", "d"]), "named files and a directory");
    finish(scene);
}

/// A directory name selects everything below it, in the tree and in the index.
#[test]
fn a_directory_pathspec_selects_what_is_below_it() {
    let edits: Vec<Edit<'_>> = vec![Edit::Delete("d/x"), Edit::Put { path: "d/y", mode: "100644", content: b"y two\n" }, Edit::Put { path: "d/n/m", mode: "100644", content: b"m\n" }];
    let scene: Scene = scene("index-reset-directory", &Plan { files: &BASE, edits: edits.as_slice(), ..Plan::default() });
    same_as_git(&scene, &named(&["d"]), "directory");
    finish(scene);
}

/// Mode changes in both directions, content unchanged and changed.
#[test]
fn mode_changes_equal_git() {
    let edits: Vec<Edit<'_>> = vec![
        Edit::Put { path: "a", mode: "100755", content: b"a one\n" },
        Edit::Put { path: "d/y", mode: "100644", content: b"y one\n" },
        Edit::Put { path: "b", mode: "100755", content: b"b two\n" },
    ];
    let scene: Scene = scene("index-reset-modes", &Plan { files: &BASE, edits: edits.as_slice(), ..Plan::default() });
    same_as_git(&scene, &named(&["a", "d/y", "b"]), "modes");
    finish(scene);
}

/// A new symbolic link, a file that became one, and one that became a file.
#[test]
fn symbolic_links_equal_git() {
    let edits: Vec<Edit<'_>> = vec![
        Edit::Put { path: "fresh-link", mode: "120000", content: b"a" },
        Edit::Put { path: "a", mode: "120000", content: b"elsewhere" },
        Edit::Put { path: "old-link", mode: "100644", content: b"now a file\n" },
        Edit::Put { path: "retargeted", mode: "120000", content: b"b" },
    ];
    let plan: Plan<'_> = Plan { files: &BASE, links: &[("old-link", "a"), ("retargeted", "z")], edits: edits.as_slice(), ..Plan::default() };
    let scene: Scene = scene("index-reset-links", &plan);
    same_as_git(&scene, &named(&["fresh-link", "a", "old-link", "retargeted"]), "links");
    finish(scene);
}

/// Submodule entries: one added, one moved to another commit, one removed, one replaced by a file.
#[test]
fn submodule_entries_equal_git() {
    let first: String = "1".repeat(40);
    let second: String = "2".repeat(40);
    let edits: Vec<Edit<'_>> = vec![
        Edit::Gitlink { path: "added-sub", id: first.as_str() },
        Edit::Gitlink { path: "moved-sub", id: second.as_str() },
        Edit::Delete("gone-sub"),
        Edit::Put { path: "file-now", mode: "100644", content: b"file\n" },
        Edit::Gitlink { path: "d-now-sub", id: first.as_str() },
    ];
    let plan: Plan<'_> = Plan {
        files: &BASE,
        gitlinks: &[("moved-sub", first.as_str()), ("gone-sub", first.as_str()), ("file-now", second.as_str())],
        edits: edits.as_slice(),
        ..Plan::default()
    };
    let scene: Scene = scene("index-reset-submodules", &plan);
    same_as_git(&scene, &named(&["added-sub", "moved-sub", "gone-sub", "file-now", "d-now-sub"]), "gitlinks");
    finish(scene);
}

/// Unmerged entries of a path that the pathspec does not name stay as they are, with the extensions Git keeps.
#[test]
fn an_unmerged_path_outside_the_pathspec_is_left_alone() {
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }];
    let scene: Scene = scene("index-reset-unmerged-outside", &Plan { files: &BASE, edits: edits.as_slice(), ..Plan::default() });
    make_unmerged(scene.repo.as_path(), "u");
    same_as_git(&scene, &named(&["a"]), "unmerged outside");
    finish(scene);
}

/// Unmerged entries of a named path are replaced by the tree's entry, or removed when the tree has none, and resolve-undo
/// records the sides.
#[test]
fn unmerged_paths_inside_the_pathspec_equal_git() {
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "u", mode: "100644", content: b"u in the tree\n" }];
    let scene: Scene = scene("index-reset-unmerged-inside", &Plan { files: &BASE, edits: edits.as_slice(), ..Plan::default() });
    make_unmerged(scene.repo.as_path(), "u");
    make_unmerged(scene.repo.as_path(), "v");
    same_as_git(&scene, &named(&["u", "v"]), "unmerged inside, one in the tree and one not");
    assert!(index_bytes(scene.repo.as_path()).windows(4).any(|window| { return window == b"REUC"; }), "Git recorded resolve-undo");
    finish(scene);
}

/// Put `path` into the index of `repo` as a conflict with three sides.
fn make_unmerged(repo: &Path, path: &str) {
    let mut lines: String = String::new();
    for (stage, text) in [(1, "base"), (2, "ours"), (3, "theirs")] {
        let blob: String = String::from_utf8(git_with(repo, &[], &["hash-object", "-w", "--stdin"], Some(format!("{path} {text}\n").as_bytes())).stdout).expect("UTF-8").trim_end().to_string();
        lines.push_str(format!("100644 {blob} {stage}\t{path}\n").as_str());
    }
    let output = git_with(repo, &[], &["update-index", "--index-info"], Some(lines.as_bytes()));
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

/// Index versions 2 and 4 keep their layout (version 3 is the extended-flag test).
#[test]
fn every_index_version_equals_git() {
    for version in ["2", "4"] {
        let edits: Vec<Edit<'_>> = vec![Edit::Delete("d/x"), Edit::Put { path: "d/q", mode: "100644", content: b"q\n" }, Edit::Put { path: "a", mode: "100644", content: b"a two\n" }];
        let scene: Scene = scene(format!("index-reset-version-{version}").as_str(), &Plan { files: &BASE, edits: edits.as_slice(), ..Plan::default() });
        git(scene.repo.as_path(), &["update-index", "--index-version", version]);
        assert_eq!(&index_bytes(scene.repo.as_path())[4..8], &[0, 0, 0, version.parse::<u8>().expect("number")], "the fixture has version {version}");
        same_as_git(&scene, &named(&["d/x", "d/q", "a"]), format!("version {version}").as_str());
        finish(scene);
    }
}

/// Extended flags: intent-to-add and skip-worktree entries (version 3) and the assume-unchanged bit.
#[test]
fn extended_and_assume_unchanged_flags_equal_git() {
    let edits: Vec<Edit<'_>> = vec![
        Edit::Put { path: "a", mode: "100644", content: b"a two\n" },
        Edit::Put { path: "b", mode: "100644", content: b"b two\n" },
        Edit::Put { path: "added-later", mode: "100644", content: b"later\n" },
    ];
    let scene: Scene = scene("index-reset-flags", &Plan { files: &BASE, edits: edits.as_slice(), ..Plan::default() });
    std::fs::write(scene.repo.join("added-later"), b"later\n").expect("write");
    git(scene.repo.as_path(), &["add", "--intent-to-add", "added-later"]);
    git(scene.repo.as_path(), &["update-index", "--skip-worktree", "a", "d/x"]);
    git(scene.repo.as_path(), &["update-index", "--assume-unchanged", "b", "z"]);
    assert_eq!(&index_bytes(scene.repo.as_path())[4..8], &[0, 0, 0, 3], "extended flags make a version 3 file");
    same_as_git(&scene, &named(&["a", "b", "added-later", "z"]), "flags");
    finish(scene);
}

/// The cache-tree and the untracked cache are kept and invalidated as Git does, and an fsmonitor extension is dropped because
/// the fixture has no monitor.
#[test]
fn kept_extensions_equal_git() {
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "d/x", mode: "100644", content: b"x two\n" }, Edit::Put { path: "d/n/new", mode: "100644", content: b"n\n" }, Edit::Delete("z")];
    let scene: Scene = scene("index-reset-extensions", &Plan { files: &BASE, edits: edits.as_slice(), ..Plan::default() });
    git(scene.repo.as_path(), &["update-index", "--force-untracked-cache"]);
    std::fs::create_dir_all(scene.repo.join("d/untracked-dir")).expect("directory");
    std::fs::write(scene.repo.join("d/untracked-dir/file"), b"u\n").expect("write");
    std::fs::write(scene.repo.join("top-untracked"), b"u\n").expect("write");
    git(scene.repo.as_path(), &["status", "--short"]);
    add_monitor_extension(scene.repo.as_path());
    let before: Vec<u8> = index_bytes(scene.repo.as_path());
    let cache = crate::index_untracked_io::read_untracked(&crate::index_reset_support_probe::extension_of(before.as_slice(), b"UNTR", 20).expect("UNTR"), 20).expect("untracked cache");
    let root = cache.root.as_ref().expect("the status run listed the worktree");
    assert!(root.valid && !root.untracked.is_empty() && root.dirs.iter().any(|dir| { return dir.valid; }), "the cache holds listings that the reset must clear");
    for signature in [b"TREE", b"UNTR", b"FSMN"] {
        assert!(before.windows(4).any(|window| { return window == signature; }), "the fixture has {}", String::from_utf8_lossy(signature));
    }
    same_as_git(&scene, &named(&["d/x", "d/n/new", "z"]), "extensions");
    let after: Vec<u8> = index_bytes(scene.repo.as_path());
    assert!(after.windows(4).any(|window| { return window == b"UNTR"; }) && !after.windows(4).any(|window| { return window == b"FSMN"; }));
    finish(scene);
}

/// Parse an index, give it an fsmonitor extension that lists its first entry as dirty, and write it back.
pub(crate) fn add_monitor_extension(repo: &Path) {
    let path: PathBuf = repo.join(".git").join("index");
    let bytes: Vec<u8> = std::fs::read(&path).expect("index");
    let mut index = crate::index_full::parse_full_index(bytes.as_slice(), 20).expect("parsed");
    for (position, entry) in index.entries.iter_mut().enumerate() {
        entry.monitor_valid = position != 0;
    }
    index.fsmonitor = Some(crate::index_fsmonitor::Fsmonitor { token: b"builtin:fixture-token".to_vec(), dirty: vec![0] });
    let written = crate::index_full_write::write_full_index(&index, &|data: &[u8]| { return crate::subvolume_sha::sha1(data).to_vec(); }).expect("written");
    std::fs::write(path, written).expect("index written");
}

/// `core.untrackedCache=false` makes Git drop the extension while reading; so does the update.
#[test]
fn a_configuration_that_removes_the_untracked_cache_equals_git() {
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }];
    let scene: Scene = scene("index-reset-untracked-off", &Plan { files: &BASE, edits: edits.as_slice(), ..Plan::default() });
    git(scene.repo.as_path(), &["update-index", "--force-untracked-cache"]);
    git(scene.repo.as_path(), &["status", "--short"]);
    git(scene.repo.as_path(), &["config", "core.untrackedCache", "false"]);
    same_as_git(&scene, &named(&["a"]), "untracked cache off");
    assert!(!index_bytes(scene.repo.as_path()).windows(4).any(|window| { return window == b"UNTR"; }));
    finish(scene);
}

/// A SHA-256 repository: names, checksum and extensions use the longer hash.
#[test]
fn a_sha256_repository_equals_git() {
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }, Edit::Delete("b")];
    let scene: Scene = scene("index-reset-sha256", &Plan { files: &BASE, edits: edits.as_slice(), sha256: true, ..Plan::default() });
    same_as_git(&scene, &named(&["a", "b"]), "sha256");
    finish(scene);
}

/// The follow-up's third way to name paths, the ones that differ between two commits, resets the same entries Git's reset of
/// `diff-tree`'s list does.
#[test]
fn paths_derived_from_two_commits_equal_git() {
    let edits: Vec<Edit<'_>> = vec![Edit::Delete("b"), Edit::Put { path: "a", mode: "100644", content: b"a two\n" }, Edit::Put { path: "n/m", mode: "100644", content: b"m\n" }];
    let scene: Scene = scene("index-reset-derived", &Plan { files: &BASE, edits: edits.as_slice(), ..Plan::default() });
    let copy: PathBuf = copy_of(&scene);
    let listed: String = git_text(scene.repo.as_path(), &["diff-tree", "-r", "--name-only", "--no-renames", scene.old.as_str(), scene.new.as_str()]);
    let names: Vec<Vec<u8>> = listed.lines().map(|line| { return line.as_bytes().to_vec(); }).collect();
    git_reset(scene.repo.as_path(), scene.new.as_str(), names.as_slice());
    assert!(landing_reset(&git_context(), copy.as_path(), scene.new.as_str(), IndexPaths::Derived { old: scene.old.as_str() }));
    assert!(index_bytes(scene.repo.as_path()) == index_bytes(copy.as_path()), "derived paths differ\n{}\n{}", listing(scene.repo.as_path()), listing(copy.as_path()));
    finish(scene);
}

/// A branch that did not exist (the all-zero old value) resets every path the commit holds.
#[test]
fn a_root_commit_resets_every_path_it_holds() {
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }];
    let scene: Scene = scene("index-reset-root", &Plan { files: &BASE, edits: edits.as_slice(), ..Plan::default() });
    let copy: PathBuf = copy_of(&scene);
    let listed: String = git_text(scene.repo.as_path(), &["ls-tree", "-r", "--name-only", scene.new.as_str()]);
    let names: Vec<Vec<u8>> = listed.lines().map(|line| { return line.as_bytes().to_vec(); }).collect();
    git_reset(scene.repo.as_path(), scene.new.as_str(), names.as_slice());
    let zero: String = "0".repeat(scene.new.len());
    assert!(landing_reset(&git_context(), copy.as_path(), scene.new.as_str(), IndexPaths::Derived { old: zero.as_str() }));
    assert!(index_bytes(scene.repo.as_path()) == index_bytes(copy.as_path()), "root commit differs\n{}\n{}", listing(scene.repo.as_path()), listing(copy.as_path()));
    finish(scene);
}

/// Directories whose names differ in length keep the cache-tree in Git's order (shorter name first), which byte order is not.
#[test]
fn directories_of_different_name_lengths_equal_git() {
    let files: [(&str, &[u8], bool); 7] = [("b/x", b"1", false), ("aa/x", b"1", false), ("ab/x", b"1", false), ("c/y", b"1", false), ("longer-name/z", b"1", false), ("longer-name/inner/q", b"1", false), ("top", b"1", false)];
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "aa/x", mode: "100644", content: b"2" }, Edit::Delete("c/y"), Edit::Put { path: "longer-name/inner/q", mode: "100755", content: b"1" }];
    let scene: Scene = scene("index-reset-name-lengths", &Plan { files: &files, edits: edits.as_slice(), ..Plan::default() });
    let before: Vec<u8> = index_bytes(scene.repo.as_path());
    let tree = crate::index_reset_support_probe::extension_of(before.as_slice(), b"TREE", 20).expect("the commit wrote a cache-tree");
    assert!(crate::index_cache_tree::CacheTree::read(tree.as_slice(), 20).is_some(), "the reader takes Git's own order");
    same_as_git(&scene, &named(&["aa/x", "c/y", "longer-name/inner/q"]), "name lengths");
    finish(scene);
}

/// An intent-to-add entry whose mode and object name equal the tree's (an empty file removed from the index and added again
/// with `-N`) is no difference to `git reset`: the review expected Git to replace it, and Git 2.56.0 keeps the entry with its
/// flag, so the update keeps it too and the two index files are equal.
#[test]
fn an_intent_to_add_entry_equal_to_the_tree_is_left_as_git_leaves_it() {
    let files: [(&str, &[u8], bool); 3] = [("e", b"", false), ("a", b"a one\n", false), ("z", b"z one\n", false)];
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }];
    let scene: Scene = scene("index-reset-ita-empty", &Plan { files: &files, edits: edits.as_slice(), ..Plan::default() });
    git(scene.repo.as_path(), &["rm", "--quiet", "--cached", "e"]);
    git(scene.repo.as_path(), &["add", "--intent-to-add", "e"]);
    assert_eq!(&index_bytes(scene.repo.as_path())[4..8], &[0, 0, 0, 3], "the entry carries the intent-to-add flag");
    same_as_git(&scene, &named(&["e", "a"]), "empty intent-to-add");
    assert_eq!(&index_bytes(scene.repo.as_path())[4..8], &[0, 0, 0, 3], "Git's own result still carries the flag");
    finish(scene);
}

/// The index Git writes through its lock file takes the mode `core.sharedRepository` gives it (`adjust_shared_perm`); the
/// published file has the same mode, under a umask that would otherwise differ.
#[test]
fn the_file_mode_with_a_shared_repository_equals_git() {
    use std::os::unix::fs::PermissionsExt as _;
    // Each test runs in a process of its own under nextest, so the umask change reaches no other test (and the Git started
    //  below inherits it).
    // SAFETY: `umask` only sets this process's file creation mask.
    unsafe { libc::umask(0o077) };
    for (value, expected) in [("group", 0o660), ("all", 0o664), ("0640", 0o640), ("umask", 0o600)] {
        let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }];
        let scene: Scene = scene(format!("index-reset-mode-{value}").as_str(), &Plan { files: &BASE, edits: edits.as_slice(), ..Plan::default() });
        git(scene.repo.as_path(), &["config", "core.sharedRepository", value]);
        same_as_git(&scene, &named(&["a"]), format!("shared repository {value}").as_str());
        let copy: PathBuf = scene.root.join("copy");
        let mode = |repo: &Path| -> u32 { return std::fs::metadata(repo.join(".git").join("index")).expect("index").permissions().mode() & 0o7777; };
        assert_eq!(mode(scene.repo.as_path()), expected, "Git's mode for {value}");
        assert_eq!(mode(copy.as_path()), expected, "the mode of the published file for {value}");
        finish(scene);
    }
}
