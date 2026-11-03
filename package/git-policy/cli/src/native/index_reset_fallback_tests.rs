//! Controls for the forms the in-process reset leaves to Git: it returns `false`, leaves the index file as it was, and the
//! landing's follow-up then runs `git reset --no-refresh` on the same file.

use super::support::{Edit, Plan, Scene, finish, git_with, index_bytes, named, scene};
use super::*;
use crate::recording_git::{recorded_starts, recording_context};
use crate::repo_read_hook::SWITCH_VARIABLE;
use crate::test_support::{age, git, git_context, git_output};
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
    let index: PathBuf = scene.repo.join(".git").join("index");
    return reset_index_file(&git_context(), scene.repo.as_path(), scene.new.as_str(), IndexPaths::Named(&named(&["a"])), index.as_path());
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
    let built: PathBuf = crate::service_land_follow::build_index(&context, scene.repo.as_path(), scene.new.as_str(), IndexPaths::Named(&paths), index.as_path(), 7).expect("built");
    let starts: Vec<String> = recorded_starts(log.as_path());
    assert!(starts.iter().any(|start| { return start.starts_with("reset --no-refresh "); }), "{label}: no fallback reset in {starts:?}");
    assert!(!starts.iter().any(|start| { return start.starts_with("reset -q"); }), "{label}: the fallback still asks for -q or a refresh: {starts:?}");
    std::fs::rename(built, index).expect("installed");
    assert!(git_output(scene.repo.as_path(), &["diff", "--cached", "--quiet", scene.new.as_str(), "--", "a"]).status.success(), "{label}: the path is not at the commit");
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

/// A required extension this reader does not know: Git would refuse the file, so Git is asked.
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

/// Git runs `post-index-change` after it writes an index; the update would not.
#[test]
fn a_post_index_change_hook_is_left_to_git() {
    let scene: Scene = changed_a("index-fallback-hook");
    crate::test_support::executable(scene.repo.join(".git/hooks/post-index-change").as_path(), b"#!/bin/sh\nexit 0\n");
    declines(&scene, "hook");
    finish(scene);
}

/// An entry recorded in the second the index file was written is racily clean; Git hashes it from the worktree.
#[test]
fn a_racily_clean_entry_is_left_to_git() {
    let edits: Vec<Edit<'_>> = vec![Edit::Put { path: "a", mode: "100644", content: b"a two\n" }];
    let scene: Scene = scene("index-fallback-racy", &Plan { files: &FILES, edits: edits.as_slice(), fresh: true, ..Plan::default() });
    age(scene.repo.join(".git").join("index").as_path());
    declines(&scene, "racy");
    follow_up_falls_back(&scene, "racy");
    finish(scene);
}

/// Configuration the update does not reproduce: sparse checkout, index threads, and an untracked cache Git would add.
#[test]
fn configuration_the_update_does_not_reproduce_is_left_to_git() {
    for (key, value) in [("core.sparseCheckout", "true"), ("index.threads", "2"), ("core.untrackedCache", "true"), ("index.skipHash", "true"), ("core.hooksPath", "hooks-elsewhere")] {
        let scene: Scene = changed_a(format!("index-fallback-{}", key.to_lowercase().replace('.', "-")).as_str());
        git(scene.repo.as_path(), &["config", key, value]);
        declines(&scene, format!("{key}={value}").as_str());
        finish(scene);
    }
}

/// A file in the commit where the index has a directory of files is a conflict Git settles.
#[test]
fn a_file_where_the_index_has_a_directory_is_left_to_git() {
    let edits: Vec<Edit<'_>> = vec![Edit::Delete("d/x"), Edit::Put { path: "d", mode: "100644", content: b"now a file\n" }];
    let scene: Scene = scene("index-fallback-dir-file", &Plan { files: &FILES, edits: edits.as_slice(), ..Plan::default() });
    let before: Vec<u8> = index_bytes(scene.repo.as_path());
    let index: PathBuf = scene.repo.join(".git").join("index");
    assert!(!reset_index_file(&git_context(), scene.repo.as_path(), scene.new.as_str(), IndexPaths::Named(&named(&["d"])), index.as_path()));
    assert!(before == index_bytes(scene.repo.as_path()));
    finish(scene);
}

/// A damaged index (wrong checksum), a missing one, and the switch that turns the in-process readers off.
#[test]
fn a_damaged_or_missing_index_and_the_off_switch_are_left_to_git() {
    let scene: Scene = changed_a("index-fallback-damaged");
    let index: PathBuf = scene.repo.join(".git").join("index");
    let good: Vec<u8> = index_bytes(scene.repo.as_path());
    let mut off = git_context();
    off.overlay.push((std::ffi::OsString::from(SWITCH_VARIABLE), std::ffi::OsString::from("off")));
    assert!(!reset_index_file(&off, scene.repo.as_path(), scene.new.as_str(), IndexPaths::Named(&named(&["a"])), index.as_path()), "the off switch");
    let mut damaged: Vec<u8> = good.clone();
    damaged[20] ^= 0xff;
    std::fs::write(&index, damaged).expect("write");
    declines(&scene, "checksum");
    std::fs::remove_file(&index).expect("remove");
    assert!(!try_reset(&scene), "missing index");
    assert!(git_with(scene.repo.as_path(), &[], &["status", "--short"], None).status.success());
    finish(scene);
}
