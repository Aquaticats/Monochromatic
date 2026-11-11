//! Differential controls:
//! the in-process reset and `git reset --no-refresh --pathspec-from-file` run on two copies of one
//! repository,
//! and the two index files must be equal byte for byte.
//!
//! `--no-refresh` is the comparison because the in-process update never refreshes:
//! a refresh rewrites the stat data of the
//! entries a copy's changed inode numbers made stale,
//! which the landing does not want and the fallback command does not do.

use super::support::{Edit, Plan, Scene, copy_of, finish, git_reset, git_with, index_bytes, landing_reset, listing, named, scene};
use super::*;
use crate::test_support::{git, git_context, git_text};
use std::path::PathBuf;

/// The two index files after the same reset by Git and in process,
/// equal;
/// `label` names the case.
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

/// The version an index file's header names.
fn version_of(bytes: &[u8]) -> u32 {
    return u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
}

/// Whether the file ends with the zeros `index.skipHash` writes instead of a checksum.
fn trailer_is_zero(bytes: &[u8]) -> bool {
    return bytes[bytes.len() - 20..].iter().all(|byte| {
        return *byte == 0;
    });
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

/// What:
///  `bytes` as the index holds it,
/// with the stat data of every entry set to zero.
/// Why:
///  The landing records the worktree's stat data where a path's file provably holds the tree's bytes
/// (`index_reset_worktree_stat.rs`),
/// where `git reset --no-refresh` writes the all-zero value;
/// everything else
///  must still be Git's,
/// so a comparison that ignores exactly that one difference keeps its value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withoutStat = (bytes: Uint8Array) => { const index = parseFullIndex(bytes, 20); for (const entry of index.entries) entry.stat = zero; return index; };
/// ```
fn without_stat_data(bytes: &[u8]) -> crate::index_full::FullIndex {
    let mut index: crate::index_full::FullIndex = crate::index_full::parse_full_index(bytes, 20).expect("an index this reader takes");
    for entry in index.entries.iter_mut() {
        entry.stat = crate::index_stat::StatData::default();
    }
    return index;
}

/// The two index files after the same reset by Git and in process,
/// equal in everything but the stat data of the entries the landing records,
/// which also must be Git's own values where they are recorded,
/// `label`
///  names the case.
fn same_as_git_but_for_the_recorded_stat_data(scene: &Scene, paths: IndexPaths<'_>, names: &[Vec<u8>], label: &str) {
    let copy: PathBuf = copy_of(scene);
    let before: Vec<u8> = index_bytes(scene.repo.as_path());
    git_reset(scene.repo.as_path(), scene.new.as_str(), names);
    let done: bool = landing_reset(&git_context(), copy.as_path(), scene.new.as_str(), paths);
    assert!(done, "{label}: the in-process update declined");
    let (git_bytes, our_bytes) = (index_bytes(scene.repo.as_path()), index_bytes(copy.as_path()));
    assert!(git_bytes != before, "{label}: the case changes nothing, so it proves nothing");
    assert_eq!(without_stat_data(git_bytes.as_slice()), without_stat_data(our_bytes.as_slice()), "{label}: the index files differ beyond the stat data\nGit:\n{}\nin process:\n{}", listing(scene.repo.as_path()), listing(copy.as_path()));
}

/// Added,
/// changed,
/// removed and renamed paths,
/// and a named path that did not change.
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

/// A directory name selects everything below it,
/// in the tree and in the index.
#[test]
fn a_directory_pathspec_selects_what_is_below_it() {
    let edits: Vec<Edit<'_>> = vec![Edit::Delete("d/x"), Edit::Put { path: "d/y", mode: "100644", content: b"y two\n" }, Edit::Put { path: "d/n/m", mode: "100644", content: b"m\n" }];
    let scene: Scene = scene("index-reset-directory", &Plan { files: &BASE, edits: edits.as_slice(), ..Plan::default() });
    same_as_git(&scene, &named(&["d"]), "directory");
    finish(scene);
}

/// Mode changes in both directions,
/// content unchanged and changed.
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

/// A new symbolic link,
/// a file that became one,
/// and one that became a file.
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

/// Submodule entries:
/// one added,
/// one moved to another commit,
/// one removed,
/// one replaced by a file.
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

/// Unmerged entries of a path that the pathspec does not name stay as they are,
/// with the extensions Git keeps.
#[test]
fn an_unmerged_path_outside_the_pathspec_is_left_alone() {
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }];
    let scene: Scene = scene("index-reset-unmerged-outside", &Plan { files: &BASE, edits: edits.as_slice(), ..Plan::default() });
    make_unmerged(scene.repo.as_path(), "u");
    same_as_git(&scene, &named(&["a"]), "unmerged outside");
    finish(scene);
}

/// Unmerged entries of a named path are replaced by the tree's entry,
/// or removed when the tree has none,
/// and resolve-undo
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

/// Extended flags:
/// intent-to-add and skip-worktree entries (version 3) and the assume-unchanged bit.
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
    // The worktree file of `added-later` holds the bytes of the entry the commit names, so the landing records its stat data
    // where `git reset --no-refresh` writes zeros; everything else is Git's.
    same_as_git_but_for_the_recorded_stat_data(&scene, IndexPaths::Named(&named(&["a", "b", "added-later", "z"])), &named(&["a", "b", "added-later", "z"]), "flags");
    finish(scene);
}

/// The cache-tree and the untracked cache are kept and invalidated as Git does,
/// and an fsmonitor extension is dropped because
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

/// Parse an index,
/// give it an fsmonitor extension that lists its first entry as dirty,
/// and write it back.
pub(crate) fn add_monitor_extension(repo: &Path) {
    let path: PathBuf = repo.join(".git").join("index");
    let bytes: Vec<u8> = std::fs::read(&path).expect("index");
    let mut index = crate::index_full::parse_full_index(bytes.as_slice(), 20).expect("parsed");
    for (position, entry) in index.entries.iter_mut().enumerate() {
        entry.monitor_valid = position != 0;
    }
    index.fsmonitor = Some(crate::index_fsmonitor::Fsmonitor { token: b"builtin:fixture-token".to_vec(), dirty: vec![0] });
    let written = crate::index_full_write::write_full_index(&index, &|data: &[u8]| { return crate::subvolume_sha::sha1(data).to_vec(); }, crate::index_full_write::Trailer::Hashed).expect("written");
    std::fs::write(path, written).expect("index written");
}

/// `core.untrackedCache=false` makes Git drop the extension while reading;
/// so does the update.
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

/// A SHA-256 repository:
/// names,
/// checksum and extensions use the longer hash.
#[test]
fn a_sha256_repository_equals_git() {
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }, Edit::Delete("b")];
    let scene: Scene = scene("index-reset-sha256", &Plan { files: &BASE, edits: edits.as_slice(), sha256: true, ..Plan::default() });
    same_as_git(&scene, &named(&["a", "b"]), "sha256");
    finish(scene);
}

/// The follow-up's third way to name paths,
/// the ones that differ between two commits,
/// resets the same entries Git's reset of
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
    let listed: String = git_text(scene.repo.as_path(), &["ls-tree", "-r", "--name-only", scene.new.as_str()]);
    let names: Vec<Vec<u8>> = listed.lines().map(|line| { return line.as_bytes().to_vec(); }).collect();
    let zero: String = "0".repeat(scene.new.len());
    // The worktree still holds the bytes of every path of this tree, so the landing records the worktree's stat data where
    //  `git reset --no-refresh` writes zeros; everything else is Git's.
    same_as_git_but_for_the_recorded_stat_data(&scene, IndexPaths::Derived { old: zero.as_str() }, names.as_slice(), "root commit");
    finish(scene);
}

/// Directories whose names differ in length keep the cache-tree in Git's order (shorter name first),
/// which byte order is not.
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
/// with `-N`) is no difference to `git reset`:
/// the review expected Git to replace it,
/// and Git 2.56.0 keeps the entry with its
/// flag,
/// so the update keeps it too and the two index files are equal.
#[test]
fn an_intent_to_add_entry_equal_to_the_tree_is_left_as_git_leaves_it() {
    let files: [(&str, &[u8], bool); 3] = [("e", b"", false), ("a", b"a one\n", false), ("z", b"z one\n", false)];
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }];
    let scene: Scene = scene("index-reset-ita-empty", &Plan { files: &files, edits: edits.as_slice(), ..Plan::default() });
    git(scene.repo.as_path(), &["rm", "--quiet", "--cached", "e"]);
    git(scene.repo.as_path(), &["add", "--intent-to-add", "e"]);
    let entry_e = |repo: &Path| -> crate::index_full::FullEntry {
        let index = crate::index_full::parse_full_index(index_bytes(repo).as_slice(), 20).expect("parsed");
        return index.entries.into_iter().find(|entry| { return entry.path == b"e"; }).expect("the entry e");
    };
    let before = entry_e(scene.repo.as_path());
    assert!(before.extended & crate::index_full::INTENT_TO_ADD != 0, "the entry carries the intent-to-add flag");
    same_as_git(&scene, &named(&["e", "a"]), "empty intent-to-add");
    let after = entry_e(scene.repo.as_path());
    assert!(after.extended & crate::index_full::INTENT_TO_ADD != 0, "Git's own result still carries the flag");
    assert_eq!((after.stat, after.mode, after.id), (before.stat, before.mode, before.id), "and the entry is the one it was");
    finish(scene);
}

/// An intent-to-add entry at a path the commit does not name is left in the index by a landing that resets only the
/// commit's paths,
/// as `git reset --no-refresh -- <paths>` leaves it:
/// the person marked a new file with `git add -N` and a commit of another path keeps the mark,
/// so a later `git add` of that file stages it as the person expects.
#[test]
fn an_intent_to_add_entry_a_landing_does_not_name_is_left_as_it_is() {
    let files: [(&str, &[u8], bool); 2] = [("a", b"a one\n", false), ("z", b"z one\n", false)];
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }];
    let scene: Scene = scene("index-reset-ita-unnamed", &Plan { files: &files, edits: edits.as_slice(), ..Plan::default() });
    std::fs::write(scene.repo.join("later"), b"later\n").expect("the file the person marks");
    git(scene.repo.as_path(), &["add", "--intent-to-add", "later"]);
    let marked = |repo: &Path| -> bool {
        let index = crate::index_full::parse_full_index(index_bytes(repo).as_slice(), 20).expect("parsed");
        return index.entries.into_iter().any(|entry| { return entry.path == b"later" && entry.extended & crate::index_full::INTENT_TO_ADD != 0; });
    };
    assert!(marked(scene.repo.as_path()), "the fixture holds the mark the case is about");
    same_as_git(&scene, &named(&["a"]), "an intent-to-add entry the reset does not name");
    assert!(marked(scene.repo.as_path()), "Git's own result still carries the mark");
    finish(scene);
}

/// The modes of the index file after a reset in `repo` with `core.sharedRepository` set to each value,
/// in Git and in process,
/// under the mask this process runs with (the callers run it in a child process that starts with the mask).
fn check_shared_modes(label: &str, expectations: &[(&str, u32)]) {
    use std::os::unix::fs::PermissionsExt as _;
    for (value, expected) in expectations {
        let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }];
        let scene: Scene = scene(format!("index-reset-mode-{label}-{value}").as_str(), &Plan { files: &BASE, edits: edits.as_slice(), ..Plan::default() });
        git(scene.repo.as_path(), &["config", "core.sharedRepository", value]);
        same_as_git(&scene, &named(&["a"]), format!("shared repository {value}").as_str());
        let copy: PathBuf = scene.root.join("copy");
        let mode = |repo: &Path| -> u32 { return std::fs::metadata(repo.join(".git").join("index")).expect("index").permissions().mode() & 0o7777; };
        assert_eq!(mode(scene.repo.as_path()), *expected, "Git's mode for {value} under {label}");
        assert_eq!(mode(copy.as_path()), *expected, "the mode of the published file for {value} under {label}");
        finish(scene);
    }
}

/// The index Git writes through its lock file takes the mode `core.sharedRepository` gives it (`adjust_shared_perm`);
/// the
/// published file has the same mode,
/// under a mask of 077 that narrows what the value widens.
#[test]
fn the_file_mode_with_a_shared_repository_under_a_narrow_mask_equals_git() {
    if crate::umask_child_test_support::ran_in_child(module_path!(), "the_file_mode_with_a_shared_repository_under_a_narrow_mask_equals_git", 0o077) {
        return;
    }
    check_shared_modes("narrow", &[("group", 0o660), ("all", 0o664), ("0640", 0o640), ("umask", 0o600)]);
}

/// The same under the usual mask of 022,
/// where a file mode value is narrower than the mask gives:
/// the file is never readable by
/// the world,
/// and the bytes are never written before its mode is set.
#[test]
fn the_file_mode_with_a_shared_repository_under_the_usual_mask_equals_git() {
    if crate::umask_child_test_support::ran_in_child(module_path!(), "the_file_mode_with_a_shared_repository_under_the_usual_mask_equals_git", 0o022) {
        return;
    }
    check_shared_modes("usual", &[("group", 0o664), ("all", 0o664), ("0640", 0o640), ("umask", 0o644)]);
}

/// What `git status --porcelain=v1` and `git diff` print in `repo`.
fn status_and_diff(repo: &Path) -> (String, String) {
    let status: String = String::from_utf8(git_with(repo, &[], &["status", "--porcelain=v1"], None).stdout).expect("UTF-8");
    let diff: String = String::from_utf8(git_with(repo, &[], &["diff"], None).stdout).expect("UTF-8");
    return (status, diff);
}

/// After the update without a refresh,
/// entries made from the tree have no stat data,
/// so Git compares the files' bytes:
/// an edit
/// of a selected or an unselected path,
/// a same-size edit with the old modification time,
/// and an empty file are all seen,
/// and
/// what a path that matches shows as clean is clean.
/// The output equals the output after real Git's `reset --no-refresh`.
#[test]
fn later_status_and_diff_are_right_after_a_reset_without_a_refresh() {
    let files: [(&str, &[u8], bool); 5] = [("a", b"a one\n", false), ("b", b"b one\n", false), ("keep", b"keep\n", false), ("empty", b"", false), ("z", b"z one\n", false)];
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }, Edit::Put { path: "empty", mode: "100644", content: b"now not empty\n" }, Edit::Delete("b")];
    let scene: Scene = scene("index-reset-status-diff", &Plan { files: &files, edits: edits.as_slice(), ..Plan::default() });
    let copy: PathBuf = copy_of(&scene);
    git_reset(scene.repo.as_path(), scene.new.as_str(), &named(&["a", "empty", "b"]));
    assert!(landing_reset(&git_context(), copy.as_path(), scene.new.as_str(), IndexPaths::Named(&named(&["a", "empty", "b"]))));
    assert!(index_bytes(scene.repo.as_path()) == index_bytes(copy.as_path()));
    for repo in [scene.repo.as_path(), copy.as_path()] {
        // The worktree holds what the commit holds for the selected paths.
        std::fs::write(repo.join("a"), b"a two\n").expect("a");
        std::fs::write(repo.join("empty"), b"now not empty\n").expect("empty");
        std::fs::remove_file(repo.join("b")).expect("b");
    }
    let clean: (String, String) = status_and_diff(scene.repo.as_path());
    assert_eq!(clean.0, "", "a worktree that matches the commit is clean");
    assert_eq!(clean, status_and_diff(copy.as_path()));
    for repo in [scene.repo.as_path(), copy.as_path()] {
        // A selected path edited to the same size, with its modification time put back: only its bytes differ.
        let kept = std::fs::metadata(repo.join("a")).expect("metadata").modified().expect("mtime");
        std::fs::write(repo.join("a"), b"a TWO\n").expect("same-size edit");
        std::fs::OpenOptions::new().write(true).open(repo.join("a")).expect("open").set_modified(kept).expect("restore the time");
        // An unselected path edited, and the empty path emptied again.
        std::fs::write(repo.join("z"), b"z edited\n").expect("z");
        std::fs::write(repo.join("empty"), b"").expect("empty");
    }
    let (status, diff) = status_and_diff(scene.repo.as_path());
    assert_eq!(status.lines().collect::<Vec<&str>>(), vec![" M a", " M empty", " M z"]);
    assert!(diff.contains("-a two") && diff.contains("+a TWO") && diff.contains("+z edited") && diff.contains("-now not empty"));
    assert_eq!((status, diff), status_and_diff(copy.as_path()));
    finish(scene);
}

/// Sets the default ACL `u::rwx,g::---,o::---` (with the mask `setfacl` would add) on a directory,
/// the way `setfacl --default
/// --modify` does,
/// through the extended attribute the kernel keeps it in,
/// so the gate's image needs no ACL tool.
fn set_default_acl(directory: &Path) -> std::io::Result<()> {
    use std::os::unix::ffi::OsStrExt as _;
    const ACL_VERSION: u32 = 2;
    const USER_OBJ: u16 = 0x01;
    const GROUP_OBJ: u16 = 0x04;
    const MASK: u16 = 0x10;
    const OTHER: u16 = 0x20;
    const NO_ID: u32 = u32::MAX;
    const ALL: u16 = 0o7;
    const NONE: u16 = 0;
    let entries: [(u16, u16); 4] = [(USER_OBJ, ALL), (GROUP_OBJ, NONE), (MASK, NONE), (OTHER, NONE)];
    let mut value: Vec<u8> = ACL_VERSION.to_le_bytes().to_vec();
    for (tag, permissions) in entries {
        value.extend_from_slice(tag.to_le_bytes().as_slice());
        value.extend_from_slice(permissions.to_le_bytes().as_slice());
        value.extend_from_slice(NO_ID.to_le_bytes().as_slice());
    }
    let path = std::ffi::CString::new(directory.as_os_str().as_bytes()).expect("a path without a NUL");
    let name = std::ffi::CString::new("system.posix_acl_default").expect("a name without a NUL");
    // SAFETY: both strings are NUL-terminated and outlive the call, and the value pointer and length describe `value`.
    let status: libc::c_int = unsafe { libc::setxattr(path.as_ptr(), name.as_ptr(), value.as_ptr().cast(), value.len(), 0) };
    return if status == 0 { Ok(()) } else { Err(std::io::Error::last_os_error()) };
}

/// With a default ACL on the Git directory that takes the group's and the world's bits away,
/// Linux gives the file Git creates
/// the mode 0600 whatever the mask says,
/// and `core.sharedRepository = group` makes it 0660.
/// The published file has the same mode
/// (a mask-only calculation would give 0664 and let the world read it).
#[test]
fn the_file_mode_under_a_default_acl_equals_git() {
    if crate::umask_child_test_support::ran_in_child(module_path!(), "the_file_mode_under_a_default_acl_equals_git", 0o022) {
        return;
    }
    use std::os::unix::fs::PermissionsExt as _;
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }];
    let scene: Scene = scene("index-reset-acl", &Plan { files: &BASE, edits: edits.as_slice(), ..Plan::default() });
    git(scene.repo.as_path(), &["config", "core.sharedRepository", "group"]);
    let copy: PathBuf = copy_of(&scene);
    let refusal: Option<std::io::Error> = [scene.repo.join(".git"), copy.join(".git")].iter().find_map(|git_dir| { return set_default_acl(git_dir.as_path()).err(); });
    if let Some(refused) = refusal {
        eprintln!("skipped: this file system does not take a default ACL: {refused}");
        finish(scene);
        return;
    }
    git_reset(scene.repo.as_path(), scene.new.as_str(), &named(&["a"]));
    assert!(landing_reset(&git_context(), copy.as_path(), scene.new.as_str(), IndexPaths::Named(&named(&["a"]))));
    let mode = |repo: &Path| -> u32 { return std::fs::metadata(repo.join(".git").join("index")).expect("index").permissions().mode() & 0o7777; };
    assert_eq!(mode(scene.repo.as_path()), 0o660, "Git's mode");
    assert_eq!(mode(copy.as_path()), 0o660, "the published file's mode");
    assert!(index_bytes(scene.repo.as_path()) == index_bytes(copy.as_path()));
    finish(scene);
}

/// What:
///  A repository carrying Git's own setting for a large checkout,
///  `feature.manyFiles`:
///  a version 4 index file,
///  an untracked cache and no trailing hash.
/// Why:
///   The update must write the file Git writes,
///  so a repository that adopts the setting keeps the in-process index update instead of a Git process per landing,
///  and the bytes are compared with `git reset --no-refresh`'s.
#[test]
fn a_repository_with_feature_manyfiles_equals_git() {
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }, Edit::Delete("b"), Edit::Put { path: "e", mode: "100644", content: b"e new\n" }];
    let config: Vec<(&str, &str)> = vec![("feature.manyFiles", "true")];
    let scene: Scene = scene("index-reset-manyfiles", &Plan { files: &BASE, edits: edits.as_slice(), config: config.as_slice(), untracked_cache: true, ..Plan::default() });
    same_as_git(&scene, &named(&["a", "b", "e"]), "feature.manyFiles");
    let bytes: Vec<u8> = index_bytes(scene.repo.as_path());
    assert_eq!(version_of(bytes.as_slice()), 4, "feature.manyFiles writes a version 4 index file");
    assert!(trailer_is_zero(bytes.as_slice()), "feature.manyFiles writes no trailing hash");
    finish(scene);
}

/// What:
///  `index.skipHash` on its own,
///  without the untracked cache `feature.manyFiles` brings.
/// Why:
///   The setting only moves the trailer,
///  so the update must still run and end the file the way Git does.
#[test]
fn a_repository_with_a_skipped_hash_equals_git() {
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }];
    let config: Vec<(&str, &str)> = vec![("index.skipHash", "true")];
    let scene: Scene = scene("index-reset-skiphash", &Plan { files: &BASE, edits: edits.as_slice(), config: config.as_slice(), ..Plan::default() });
    same_as_git(&scene, &named(&["a"]), "index.skipHash");
    assert!(trailer_is_zero(index_bytes(scene.repo.as_path()).as_slice()), "the update writes the zeros Git writes");
    finish(scene);
}
