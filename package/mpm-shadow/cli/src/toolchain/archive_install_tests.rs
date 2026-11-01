//! Unit tests of how an archive install becomes visible:
//! the staging directory,
//! the completion marker,
//! and what a failed install leaves behind.
//! Each runs against the fake runner and the fake network inside disposable homes;
//! the directories are the ones `tar` leaves when it is cut short.

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use super::archive_staging::{COMPLETE_MARKER, has_complete_marker, publish_staged, staging_dir_of, write_complete_marker};
use super::fakes::{Rig, write_executable};
use super::layout::{is_executable, tool_directory, tool_downloads_dir};
use super::lock::Lock;
use super::prune_cargo::prune_toolchain;
use super::published_archive::{
    is_finished, newest_installed_archive, point_current_at_newest_archive, remove_downloads, running_version_archive,
};
use super::published_archive_tests::{
    BIN, ID, context_of, finish_archive, lock_at, named_lock, release_body, release_url, serve_releases, tar_lines, unmarked_archive,
};
use super::runner::RunRequest;
use super::sync::sync_tools;

/// Makes the fake network serve the release archives of `versions`.
fn serve_bodies(rig: &Rig, versions: &[&str]) {
    for version in versions {
        rig.http.serve(&release_url(version), &release_body(version));
    }
}

/// The `--directory` a `tar --extract` request unpacks into.
fn tar_directory(request: &RunRequest) -> PathBuf {
    let position: usize = request.args.iter().position(|argument| return argument == "--directory").expect("a directory");
    return PathBuf::from(&request.args[position + 1]);
}

/// The names directly inside the tool's directory,
/// sorted.
fn names_in_tool_directory(rig: &Rig) -> Vec<String> {
    return names_of(&tool_directory(&rig.scope.toolchain_root, ID).expect("a plain name"));
}

/// A rig with 0.2.0 installed and finished by hand,
/// `current` pointing at it.
fn rig_with_working_older_release() -> Rig {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    finish_archive(&root, "0.2.0");
    point_current_at_newest_archive(&root, ID, BIN).expect("link current");
    assert_eq!(running_version_archive(&root, ID).as_deref(), Some("0.2.0"));
    return rig;
}

/// Asserts the working 0.2.0 is untouched and no 0.2.1 or hidden sibling of it exists.
fn assert_older_release_kept_and_nothing_left(rig: &Rig) {
    let root = &rig.scope.toolchain_root;
    assert_eq!(names_in_tool_directory(rig), vec!["0.2.0".to_string(), "current".to_string()]);
    assert!(is_finished(&tool_directory(root, ID).expect("a plain name").join("0.2.0"), BIN));
    assert_eq!(running_version_archive(root, ID).as_deref(), Some("0.2.0"), "current still names the working version");
}

#[test]
fn a_finished_install_carries_the_marker_and_leaves_no_staging_sibling() {
    let rig = Rig::new();
    serve_releases(&rig, &["0.2.1"]);
    let root = rig.scope.toolchain_root.clone();
    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert_eq!(report.installed, vec![ID.to_string()], "{report:?}");
    assert_eq!(names_in_tool_directory(&rig), vec!["0.2.1".to_string(), "current".to_string()]);
    let version_dir = tool_directory(&root, ID).expect("a plain name").join("0.2.1");
    assert!(has_complete_marker(&version_dir));
    assert!(is_finished(&version_dir, BIN));
    assert_eq!(newest_installed_archive(&root, ID, BIN).as_deref(), Some("0.2.1"));
}

#[test]
fn an_unpacking_cut_short_leaves_the_working_older_version_and_no_directory_behind() {
    let rig = rig_with_working_older_release();
    serve_releases(&rig, &["0.2.1"]);
    // GNU tar creates the file with its final mode, writes part of it, then fails with "Unexpected EOF".
    rig.runner.fail("tar --extract", "tar: Unexpected EOF in archive");
    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert!(report.installed.is_empty(), "{report:?}");
    assert_eq!(report.failed.len(), 1, "{report:?}");
    assert!(report.failed[0].1.contains("Unexpected EOF"), "{report:?}");
    assert_older_release_kept_and_nothing_left(&rig);
    assert_eq!(tar_lines(&rig).len(), 1, "the unpacking was tried");
}

#[test]
fn a_release_without_its_program_is_refused_and_so_is_one_whose_program_cannot_run() {
    for broken in ["another file", "a dangling link"] {
        let rig = rig_with_working_older_release();
        serve_bodies(&rig, &["0.2.1"]);
        let behaviour = broken.to_string();
        rig.runner.on(
            "tar --extract",
            Box::new(move |request: &RunRequest| {
                let folder = tar_directory(request).join("release-folder");
                fs::create_dir_all(&folder).expect("make the folder");
                if behaviour == "another file" {
                    write_executable(&folder.join("something-else"));
                } else {
                    symlink("does-not-exist", folder.join(BIN)).expect("link");
                }
            }),
        );
        let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
        assert!(report.installed.is_empty(), "{broken}: {report:?}");
        assert!(report.failed[0].1.contains(&format!("holds no executable file named {BIN}")), "{broken}: {report:?}");
        assert_older_release_kept_and_nothing_left(&rig);
    }
}

#[test]
fn a_directory_with_an_executable_of_the_right_name_but_no_marker_is_never_finished() {
    let rig = rig_with_working_older_release();
    let root = rig.scope.toolchain_root.clone();
    // What `tar` left of an 8 MB program cut off at 3 MB: mode 755, the right name, the wrong bytes.
    let garbage = unmarked_archive(&root, "0.2.1");
    fs::write(&garbage, "cut off").expect("write");
    assert!(is_executable(&garbage));
    assert_eq!(newest_installed_archive(&root, ID, BIN).as_deref(), Some("0.2.0"), "the unmarked 0.2.1 is not the newest");

    fs::write(rig.scope.root.join("toolchain.lock.json"), named_lock("0.2.1")).expect("write the lock");
    rig.runner.answer("git -C", &format!("worktree {}\nHEAD a\n\n", rig.scope.root.display()), 0);
    let first_pass = prune_toolchain(&rig.scope, &rig.runner, &[], false).expect("pruned");
    assert!(first_pass.is_empty(), "nothing is superseded by a copy that never finished: {first_pass:?}");
    assert!(tool_directory(&root, ID).expect("a plain name").join("0.2.0").is_dir());
    assert_eq!(running_version_archive(&root, ID).as_deref(), Some("0.2.0"), "current does not move to the garbage");

    // Positive control: the same layout with the marker on 0.2.1 does delete 0.2.0, so the test can fail.
    write_complete_marker(&tool_directory(&root, ID).expect("a plain name").join("0.2.1"), ID, "0.2.1").expect("mark");
    let second_pass = prune_toolchain(&rig.scope, &rig.runner, &[], false).expect("pruned");
    assert_eq!(second_pass, vec![tool_directory(&root, ID).expect("a plain name").join("0.2.0")]);
}

#[test]
fn a_marked_directory_whose_program_is_gone_or_cannot_run_is_not_finished() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let program = finish_archive(&root, "0.2.0");
    let version_dir = tool_directory(&root, ID).expect("a plain name").join("0.2.0");
    assert!(is_finished(&version_dir, BIN));
    assert!(!is_finished(&version_dir, "another-program"), "the program named is not in the directory");
    fs::remove_file(&program).expect("remove");
    assert!(!is_finished(&version_dir, BIN), "the marker without the program is not finished");
    fs::write(&program, "not executable").expect("write");
    assert!(!is_finished(&version_dir, BIN), "a program without an execute bit is not finished");
    assert!(is_finished(&version_dir, ""), "a tool that declares no program finishes on the marker alone");
}

#[test]
fn a_directory_an_earlier_release_unpacked_in_place_is_replaced_by_the_next_install_of_that_version() {
    let rig = Rig::new();
    serve_releases(&rig, &["0.2.1"]);
    let root = rig.scope.toolchain_root.clone();
    let legacy = unmarked_archive(&root, "0.2.1");
    fs::write(&legacy, "unpacked in place by 0.2.0").expect("write");
    fs::write(legacy.with_file_name("left-over"), "x").expect("write");
    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert_eq!(report.installed, vec![ID.to_string()], "an unmarked directory does not count as installed: {report:?}");
    assert!(is_finished(&tool_directory(&root, ID).expect("a plain name").join("0.2.1"), BIN));
    assert_ne!(fs::read_to_string(&legacy).expect("read"), "unpacked in place by 0.2.0", "the program is the freshly unpacked one");
    assert!(!legacy.with_file_name("left-over").exists(), "nothing of the old copy is kept");
    assert_eq!(names_in_tool_directory(&rig), vec!["0.2.1".to_string(), "current".to_string()], "no hidden sibling is left");
}

#[test]
fn a_leftover_staging_directory_of_a_run_with_this_process_id_is_cleared_before_unpacking() {
    let rig = Rig::new();
    serve_releases(&rig, &["0.2.1"]);
    let version_dir = tool_directory(&rig.scope.toolchain_root, ID).expect("a plain name").join("0.2.1");
    let leftover = staging_dir_of(&version_dir).join("stale-file");
    fs::create_dir_all(leftover.parent().expect("a parent")).expect("make");
    fs::write(&leftover, "from a killed run").expect("write");
    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert_eq!(report.installed, vec![ID.to_string()], "{report:?}");
    assert!(!version_dir.join("stale-file").exists());
    assert_eq!(names_in_tool_directory(&rig), vec!["0.2.1".to_string(), "current".to_string()]);
}

#[test]
fn a_published_tool_that_declares_no_program_is_installed_once_not_on_every_sync() {
    let rig = Rig::new();
    serve_releases(&rig, &["0.2.1"]);
    let mut lock: Lock = lock_at(&rig, "0.2.1");
    lock.tools[0].declaration.bins = Vec::new();
    let first = sync_tools(&context_of(&rig, &rig.scope, &lock), None);
    assert_eq!(first.installed, vec![ID.to_string()], "{first:?}");
    let second = sync_tools(&context_of(&rig, &rig.scope, &lock), None);
    assert!(second.installed.is_empty(), "{second:?}");
    assert_eq!(second.unchanged, vec![ID.to_string()]);
    assert_eq!(tar_lines(&rig).len(), 1, "the archive was not downloaded and unpacked again");
}

#[test]
fn a_single_binary_that_fails_to_decompress_leaves_no_directory() {
    let rig = Rig::new();
    let body = b"this is not gzip data";
    let url = "https://x.example/releases/tool.gz";
    rig.http.serve(url, body);
    let mut tool = rig.tool("node");
    tool.declaration.bins = vec!["tool".to_string()];
    tool.artifacts = vec![(rig.host.platform.clone(), super::model::Artifact { url: url.to_string(), sha256: super::hash::sha256_hex(body) })];
    let error = super::install::install_tool(&rig.context(), &tool).expect_err("not gzip data");
    assert!(error.message.contains("cannot decompress"), "{error}");
    let tool_dir = tool_directory(&rig.scope.toolchain_root, "node").expect("a plain name");
    let left: Vec<String> = fs::read_dir(&tool_dir)
        .map(|entries| return entries.flatten().map(|entry| return entry.file_name().to_string_lossy().to_string()).collect())
        .unwrap_or_default();
    assert!(left.is_empty(), "no version directory and no staging directory: {left:?}");
}

#[test]
fn publishing_keeps_a_complete_copy_another_run_installed_and_puts_back_a_copy_when_the_rename_fails() {
    let rig = Rig::new();
    let tool_dir = tool_directory(&rig.scope.toolchain_root, ID).expect("a plain name");
    let final_dir = tool_dir.join("0.2.1");

    // A complete copy is already there: the staged one is discarded and the copy is kept.
    fs::create_dir_all(&final_dir).expect("make");
    fs::write(final_dir.join("program"), "the first run's").expect("write");
    write_complete_marker(&final_dir, ID, "0.2.1").expect("mark");
    let staging = staging_dir_of(&final_dir);
    fs::create_dir_all(&staging).expect("make");
    fs::write(staging.join("program"), "the second run's").expect("write");
    publish_staged(&staging, &final_dir).expect("kept");
    assert_eq!(fs::read_to_string(final_dir.join("program")).expect("read"), "the first run's");
    assert!(!staging.exists());

    // A copy without the marker is replaced.
    fs::remove_file(final_dir.join(COMPLETE_MARKER)).expect("unmark");
    fs::create_dir_all(&staging).expect("make");
    fs::write(staging.join("program"), "the replacement").expect("write");
    publish_staged(&staging, &final_dir).expect("replaced");
    assert_eq!(fs::read_to_string(final_dir.join("program")).expect("read"), "the replacement");
    assert_eq!(names_of(&tool_dir), vec!["0.2.1".to_string()], "the moved-aside copy is removed");

    // The staged directory is missing, so the rename fails and the copy that was there is put back.
    let missing: &Path = &tool_dir.join(".0.2.1.staging.missing");
    let error = publish_staged(missing, &final_dir).expect_err("nothing to publish");
    assert!(error.message.contains("cannot move the finished install"), "{error}");
    assert_eq!(fs::read_to_string(final_dir.join("program")).expect("read"), "the replacement", "the copy is back");
    assert_eq!(names_of(&tool_dir), vec!["0.2.1".to_string()]);
}

#[test]
fn a_link_named_like_the_marker_does_not_mark_a_directory() {
    let rig = Rig::new();
    let directory = rig.scope.toolchain_root.join("probe");
    fs::create_dir_all(&directory).expect("make");
    let real = rig.scope.toolchain_root.join("real-file");
    fs::write(&real, "x").expect("write");
    symlink(&real, directory.join(COMPLETE_MARKER)).expect("link");
    assert!(!has_complete_marker(&directory));
    fs::remove_file(directory.join(COMPLETE_MARKER)).expect("remove");
    write_complete_marker(&directory, ID, "1").expect("mark");
    assert!(has_complete_marker(&directory));
}

/// The names directly inside `directory`,
/// sorted.
fn names_of(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(directory)
        .expect("list")
        .flatten()
        .map(|entry| return entry.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    return names;
}

#[test]
fn removing_the_downloads_of_one_version_never_touches_another_tools_with_a_similar_name() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    // Under the earlier flat names `<id>-<version>-<file>`, `a` at 1-b and `a-1` at b made the same prefix `a-1-b-`.
    for (id, version) in [("a", "1-b"), ("a-1", "b"), ("a", "2")] {
        let directory = tool_downloads_dir(&root, id, version).expect("a plain name");
        fs::create_dir_all(&directory).expect("make");
        fs::write(directory.join("file"), "x").expect("write");
    }
    remove_downloads(&root, "a", "1-b").expect("removed");
    assert!(!tool_downloads_dir(&root, "a", "1-b").expect("a plain name").exists());
    assert!(tool_downloads_dir(&root, "a-1", "b").expect("a plain name").join("file").is_file(), "the other tool's download stays");
    assert!(tool_downloads_dir(&root, "a", "2").expect("a plain name").join("file").is_file(), "so does another version of the same tool");
    remove_downloads(&root, "a", "2").expect("removed");
    assert!(!root.join("downloads").join("a").exists(), "the tool's downloads directory goes with its last version");
    remove_downloads(&root, "never-installed", "1.0.0").expect("nothing to remove is not an error");
    assert!(remove_downloads(&root, "..", "1.0.0").is_err(), "a crafted id is refused");
}
