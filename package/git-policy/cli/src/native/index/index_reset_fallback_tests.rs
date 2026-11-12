//! Controls for the forms the in-process reset leaves to Git:
//! it returns `false`,
//! leaves the index file as it was,
//! and the
//! landing's follow-up then runs `git reset --no-refresh` on the same file.

use super::support::{Edit, Plan, Scene, copy_of, finish, git_reset, git_with, index_bytes, landing_reset, landing_reset_copy, named, scene};
use super::*;
use crate::recording_git::{recorded_starts, recording_context};
use crate::repo_read_hook::SWITCH_VARIABLE;
use crate::test_support::{git, git_context, git_output};
use std::path::PathBuf;

/// The files of the first commit.
const FILES: [(&str, &[u8], bool); 3] = [("a", b"a one\n", false), ("d/x", b"x one\n", false), ("z", b"z one\n", false)];

/// A scene whose second commit changes `a`.
fn changed_a(name: &str) -> Scene {
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }];
    return scene(name, &Plan { files: &FILES, edits: edits.as_slice(), ..Plan::default() });
}

/// Run the in-process update on the scene's repository for the path `a`.
fn try_reset(scene: &Scene) -> bool {
    return landing_reset(&git_context(), scene.repo.as_path(), scene.new.as_str(), IndexPaths::Named(&named(&["a"])));
}

/// The in-process update declines and changes neither the index file nor the directory around it.
fn declines(scene: &Scene, label: &str) {
    let before: Vec<u8> = index_bytes(scene.repo.as_path());
    let files: usize = std::fs::read_dir(scene.repo.join(".git")).expect("directory").count();
    assert!(!try_reset(scene), "{label}: the in-process update did not decline");
    assert!(before == index_bytes(scene.repo.as_path()), "{label}: a declined update changed the index");
    assert_eq!(files, std::fs::read_dir(scene.repo.join(".git")).expect("directory").count(), "{label}: a declined update left a file behind");
}

/// The follow-up's own call falls back to `git reset --no-refresh` and ends with the index at the commit for the path.
fn follow_up_falls_back(scene: &Scene, label: &str) {
    let (mut context, log) = recording_context(scene.root.as_path());
    context.overlay.retain(|(name, _)| { return name != SWITCH_VARIABLE; });
    // A request that carries input makes its input file in the repository's common directory.
    context.common_dir = Some(scene.repo.join(".git"));
    let index: PathBuf = scene.repo.join(".git").join("index");
    let paths: Vec<Vec<u8>> = named(&["a"]);
    let held = crate::service_index_lock::try_acquire(index.as_path(), 7).expect("the lock is free");
    let built = crate::service_land_follow::build_index(&context, scene.repo.as_path(), scene.new.as_str(), IndexPaths::Named(&paths), &held, 7).expect("built");
    let starts: Vec<String> = recorded_starts(log.as_path());
    assert!(starts.iter().any(|start| { return start.starts_with("reset --no-refresh "); }), "{label}: no fallback reset in {starts:?}");
    assert!(!starts.iter().any(|start| { return start.starts_with("reset -q"); }), "{label}: the fallback still asks for -q or a refresh: {starts:?}");
    held.release_with(built.path()).expect("installed");
    assert!(git_output(scene.repo.as_path(), &["diff", "--cached", "--quiet", scene.new.as_str(), "--", "a"]).status.success(), "{label}: the path is not at the commit");
}

/// Whether an index file carries the untracked cache.
fn has_untracked(bytes: &[u8]) -> bool {
    return crate::index_reset_support_probe::extension_of(bytes, b"UNTR", 20).is_some();
}

/// A split index (`link` extension).
#[test]
fn a_split_index_is_left_to_git() {
    let scene: Scene = changed_a("index-fallback-split");
    git(scene.repo.as_path(), &["update-index", "--split-index"]);
    declines(&scene, "split index");
    follow_up_falls_back(&scene, "split index");
    finish(scene);
}

/// A sparse index and a sparse checkout.
#[test]
fn a_sparse_checkout_is_left_to_git() {
    let scene: Scene = changed_a("index-fallback-sparse");
    git(scene.repo.as_path(), &["sparse-checkout", "init", "--cone", "--sparse-index"]);
    declines(&scene, "sparse index");
    finish(scene);
}

/// A required extension this reader does not know:
/// Git would refuse the file,
/// so Git is asked.
#[test]
fn an_unknown_required_extension_is_left_to_git() {
    let scene: Scene = changed_a("index-fallback-required");
    let path: PathBuf = scene.repo.join(".git").join("index");
    let bytes: Vec<u8> = std::fs::read(&path).expect("index");
    let (body, _) = bytes.split_at(bytes.len() - 20);
    let mut patched: Vec<u8> = body.to_vec();
    patched.extend_from_slice(b"xtra\0\0\0\0");
    patched.extend_from_slice(crate::subvolume_sha::sha1(patched.as_slice()).as_slice());
    std::fs::write(path, patched).expect("write");
    declines(&scene, "required extension");
    finish(scene);
}

/// Git runs `post-index-change` after it writes an index;
/// the update would not.
#[test]
fn a_post_index_change_hook_is_left_to_git() {
    let scene: Scene = changed_a("index-fallback-hook");
    crate::test_support::executable(scene.repo.join(".git/hooks/post-index-change").as_path(), b"#!/bin/sh\nexit 0\n");
    declines(&scene, "hook");
    finish(scene);
}

/// An entry recorded in the second the index file was written is racily clean;
/// Git hashes it from the worktree.
#[test]
fn a_racily_clean_entry_is_left_to_git() {
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }];
    let scene: Scene = scene("index-fallback-racy", &Plan { files: &FILES, edits: edits.as_slice(), fresh: true, ..Plan::default() });
    // The update compares the entries' times with the private copy's, so the copy is the old file here.
    let before: Vec<u8> = index_bytes(scene.repo.as_path());
    assert!(!landing_reset_copy(&git_context(), scene.repo.as_path(), scene.new.as_str(), IndexPaths::Named(&named(&["a"])), true), "racy");
    assert!(before == index_bytes(scene.repo.as_path()));
    finish(scene);
}

/// Configuration the update does not reproduce:
/// a sparse checkout or a sparse index,
/// a split index,
/// index threads,
/// and an untracked cache Git would add.
/// The sparse and split cases matter twice over here:
/// the proof reads worktree files named by index paths,
/// and neither an index of sparse directories nor a split index's shared file is one it could trust.
#[test]
fn configuration_the_update_does_not_reproduce_is_left_to_git() {
    for (key, value) in [
        ("core.sparseCheckout", "true"),
        ("index.sparse", "true"),
        ("core.splitIndex", "true"),
        ("index.threads", "2"),
        ("core.untrackedCache", "true"),
        ("core.hooksPath", "hooks-elsewhere"),
    ] {
        let scene: Scene = changed_a(format!("index-fallback-{}", key.to_lowercase().replace('.', "-")).as_str());
        git(scene.repo.as_path(), &["config", key, value]);
        declines(&scene, format!("{key}={value}").as_str());
        finish(scene);
    }
}

/// What:
///  `feature.manyFiles` beside `core.untrackedCache=keep`:
///  real Git adds the untracked cache the index does not have,
///  so the landing leaves the file to `git reset --no-refresh`,
///  which ends with Git's own index.
/// Why:
///  Git sets the untracked-cache default from `feature.manyFiles` before it reads `core.untrackedCache`,
///  and `keep` is a value its boolean reader refuses,
///  so it leaves that default alone (`repo-settings.c` of Git 2.56.0).
///  A landing that read `keep` as "leave the file's extensions as they are" installed an index without the cache
///  Git adds,
///  so the person's index stopped being the file Git writes.
#[test]
fn feature_manyfiles_with_an_untracked_cache_kept_lets_git_add_the_cache() {
    let scene: Scene = changed_a("index-fallback-manyfiles-keep");
    for (key, value) in [("feature.manyFiles", "true"), ("core.untrackedCache", "keep")] {
        git(scene.repo.as_path(), &["config", key, value]);
    }
    assert!(!has_untracked(index_bytes(scene.repo.as_path()).as_slice()), "the scene starts without the cache, or the case proves nothing");
    // Real Git's own reset is the statement of what this configuration does; the control below depends on it.
    let control: PathBuf = copy_of(&scene);
    git_reset(control.as_path(), scene.new.as_str(), &named(&["a"]));
    assert!(has_untracked(index_bytes(control.as_path()).as_slice()), "real Git adds the untracked cache here");
    declines(&scene, "feature.manyFiles with core.untrackedCache=keep");
    follow_up_falls_back(&scene, "feature.manyFiles with core.untrackedCache=keep");
    assert!(has_untracked(index_bytes(scene.repo.as_path()).as_slice()), "the follow-up ends with the cache Git adds");
    finish(scene);
}

/// A file in the commit where the index has a directory of files is a conflict Git settles.
#[test]
fn a_file_where_the_index_has_a_directory_is_left_to_git() {
    let edits: Vec<Edit<'_>> = vec![Edit::Delete("d/x"), Edit::Put { path: "d", mode: "100644", content: b"now a file\n" }];
    let scene: Scene = scene("index-fallback-dir-file", &Plan { files: &FILES, edits: edits.as_slice(), ..Plan::default() });
    let before: Vec<u8> = index_bytes(scene.repo.as_path());
    assert!(!landing_reset(&git_context(), scene.repo.as_path(), scene.new.as_str(), IndexPaths::Named(&named(&["d"]))));
    assert!(before == index_bytes(scene.repo.as_path()));
    finish(scene);
}

/// A damaged index (wrong checksum),
/// a missing one,
/// and the switch that turns the in-process readers off.
#[test]
fn a_damaged_or_missing_index_and_the_off_switch_are_left_to_git() {
    let scene: Scene = changed_a("index-fallback-damaged");
    let index: PathBuf = scene.repo.join(".git").join("index");
    let good: Vec<u8> = index_bytes(scene.repo.as_path());
    let mut off = git_context();
    off.overlay.push((std::ffi::OsString::from(SWITCH_VARIABLE), std::ffi::OsString::from("off")));
    assert!(!landing_reset(&off, scene.repo.as_path(), scene.new.as_str(), IndexPaths::Named(&named(&["a"]))), "the off switch");
    let mut damaged: Vec<u8> = good.clone();
    damaged[20] ^= 0xff;
    std::fs::write(&index, damaged).expect("write");
    declines(&scene, "checksum");
    std::fs::remove_file(&index).expect("remove");
    assert!(!try_reset(&scene), "missing index");
    assert!(git_with(scene.repo.as_path(), &[], &["status", "--short"], None).status.success());
    finish(scene);
}

/// Another run's temporary file is not this run's to remove or publish:
/// a file already at the name an earlier version used for
/// every run stays untouched,
/// and the target still gets the complete result.
#[test]
fn another_runs_temporary_file_is_left_alone() {
    let scene: Scene = changed_a("index-fallback-decoy");
    let decoy: PathBuf = scene.repo.join(".git").join("index.cli-git-reset");
    std::fs::write(&decoy, b"another run's partial file").expect("decoy");
    assert!(try_reset(&scene));
    assert_eq!(std::fs::read(&decoy).expect("the decoy stays"), b"another run's partial file".to_vec());
    assert!(crate::index_full::parse_full_index(index_bytes(scene.repo.as_path()).as_slice(), 20).is_some());
    finish(scene);
}

/// The names the update declines for the protection settings are names real Git 2.56.0 refuses to put in an index with those
/// settings on (the default for NTFS),
/// so declining them leaves Git to say so.
#[test]
fn git_refuses_the_protected_names_the_update_declines() {
    let scene: Scene = changed_a("index-fallback-protected-names");
    let blob: String = String::from_utf8(git_with(scene.repo.as_path(), &[], &["hash-object", "-w", "--stdin"], Some(b"target")).stdout).expect("UTF-8").trim().to_string();
    for (name, mode) in [(".gitmodules::$DATA", "120000"), (".gitmodules ", "120000"), ("GITMOD~1", "120000"), (".git::$INDEX_ALLOCATION", "100644"), ("dir/.git ", "100644"), ("dir\\.git\\config", "100644"), ("GIT~1", "100644")] {
        let output = git_with(scene.repo.as_path(), &[], &["-c", "core.protectNTFS=true", "update-index", "--add", "--cacheinfo", format!("{mode},{blob},{name}").as_str()], None);
        assert!(!output.status.success(), "Git accepted {name:?}");
    }
    finish(scene);
}

/// Names Git refuses by their NTFS short forms,
/// among them the hashed short name of `.gitmodules` as a link,
/// are in no index
/// real `git reset --no-refresh` writes:
/// it fails and leaves the index file as it was,
/// and so does the update's decline;
/// an
/// ordinary link name in the same position is taken by both and gives equal files.
#[test]
fn ntfs_short_names_are_refused_by_git_and_declined_by_the_update() {
    use super::support::{commit_with_raw_entry, copy_of, git_reset_status};
    for name in ["GI7EBA~1", "gi7eba~2", "GI7EBA~1 .", "GITMOD~3", "GI7EBA~9"] {
        let scene: Scene = changed_a("index-fallback-short-name");
        let commit: String = commit_with_raw_entry(&scene, name, "120000", b"target");
        let before: Vec<u8> = index_bytes(scene.repo.as_path());
        let copy: PathBuf = copy_of(&scene);
        assert!(!git_reset_status(scene.repo.as_path(), commit.as_str(), &named(&[name])), "Git accepted {name:?}");
        assert!(before == index_bytes(scene.repo.as_path()), "Git changed the index for {name:?}");
        assert!(!landing_reset(&git_context(), copy.as_path(), commit.as_str(), IndexPaths::Named(&named(&[name]))), "the update accepted {name:?}");
        assert!(before == index_bytes(copy.as_path()), "the update changed the index for {name:?}");
        finish(scene);
    }
    let scene: Scene = changed_a("index-fallback-short-name-control");
    let commit: String = commit_with_raw_entry(&scene, "ordinary-link", "120000", b"target");
    let copy: PathBuf = copy_of(&scene);
    assert!(git_reset_status(scene.repo.as_path(), commit.as_str(), &named(&["ordinary-link"])));
    assert!(landing_reset(&git_context(), copy.as_path(), commit.as_str(), IndexPaths::Named(&named(&["ordinary-link"]))));
    assert!(index_bytes(scene.repo.as_path()) == index_bytes(copy.as_path()));
    finish(scene);
}
