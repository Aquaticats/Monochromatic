//! The launch seam's default policy,
//!  executable lookup,
//!  and private-directory containment.

use super::{
    LaunchRequest, ServerLaunch, check_state_directory, launch_directly, prepare,
    resolve_executable,
};
use std::{collections::HashMap, fs, os::unix::fs::PermissionsExt, path::Path, path::PathBuf};

fn canonical(directory: &tempfile::TempDir) -> PathBuf {
    return directory
        .path()
        .canonicalize()
        .expect("canonical directory");
}

fn write_mode(path: &Path, mode: u32) {
    fs::write(path, "#!/bin/sh\n").expect("fixture file");
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).expect("fixture mode");
}

fn launch(directories: Vec<PathBuf>, scratch: Vec<PathBuf>) -> ServerLaunch {
    return ServerLaunch {
        command: "/bin/true".to_string(),
        args: Vec::new(),
        environment: HashMap::new(),
        settings: None,
        directories,
        scratch,
    };
}

#[test]
fn default_policy_spawns_the_resolved_program_unchanged() {
    let request = LaunchRequest {
        server: "scripted".to_string(),
        executable: PathBuf::from("/opt/tools/scripted-ls"),
        args: vec!["--stdio".to_string()],
        environment: HashMap::from([("KEY".to_string(), "value".to_string())]),
        settings: Some(serde_json::json!({ "flag": true })),
        project_root: PathBuf::from("/project"),
        project_spellings: Vec::new(),
        read_only_binds: Vec::new(),
        state_root: None,
    };
    let launch = launch_directly(&request).expect("default policy never refuses a Unicode path");
    assert_eq!(launch.command, "/opt/tools/scripted-ls");
    assert_eq!(launch.args, request.args);
    assert_eq!(launch.environment, request.environment);
    assert_eq!(launch.settings, request.settings);
    assert!(launch.directories.is_empty());
    assert!(launch.scratch.is_empty());
}

#[test]
fn command_with_a_separator_resolves_against_the_project_root() {
    let project = crate::test_scratch::scratch().expect("project");
    let root = canonical(&project);
    fs::create_dir_all(root.join("node_modules/tool/bin")).expect("tool directory");
    write_mode(&root.join("node_modules/tool/bin/server"), 0o755);
    write_mode(&root.join("node_modules/tool/bin/data"), 0o644);
    assert_eq!(
        resolve_executable("node_modules/tool/bin/server", &root),
        Some(root.join("node_modules/tool/bin/server"))
    );
    assert_eq!(
        resolve_executable("node_modules/tool/bin/data", &root),
        None,
        "a file without execute permission was treated as a program"
    );
    assert_eq!(resolve_executable("node_modules/tool/bin", &root), None);
    assert_eq!(
        resolve_executable("node_modules/tool/bin/absent", &root),
        None
    );
    let absolute = root.join("node_modules/tool/bin/server");
    assert_eq!(
        resolve_executable(absolute.to_str().expect("Unicode path"), Path::new("/")),
        Some(absolute)
    );
}

#[test]
fn bare_name_is_searched_on_the_path_and_resolved_absolute() {
    let found = resolve_executable("sh", Path::new("/")).expect("a shell is on PATH");
    assert!(found.is_absolute());
    assert_eq!(
        resolve_executable("definitely-not-installed-language-server", Path::new("/")),
        None
    );
}

/// The home folder opened as the project holds the private cache;
///  the sandbox binds that state
/// writable after the read-only project,
///  so a state directory strictly inside the project is
/// accepted,
///  while the project itself as state would make all of it writable.
#[test]
fn state_directory_inside_the_project_is_accepted_but_the_project_itself_is_not() {
    let project = crate::test_scratch::scratch().expect("project");
    let root = canonical(&project);
    assert_eq!(
        check_state_directory(&root.join(".cache/monochromatic-ide/language"), &root),
        Ok(())
    );
    let same = check_state_directory(&root, &root).expect_err("the project itself was accepted");
    assert!(same.contains("contains the project"), "{same}");
}

#[test]
fn state_directory_containing_the_project_is_rejected() {
    let parent = crate::test_scratch::scratch().expect("parent");
    let root = canonical(&parent).join("project");
    fs::create_dir(&root).expect("project");
    let reason = check_state_directory(&canonical(&parent), &root)
        .expect_err("a state directory containing the project was accepted");
    assert!(reason.contains("contains the project"), "{reason}");
}

#[test]
fn state_directory_reached_through_a_link_to_the_project_itself_is_rejected() {
    let project = crate::test_scratch::scratch().expect("project");
    let elsewhere = crate::test_scratch::scratch().expect("elsewhere");
    let root = canonical(&project);
    let link = canonical(&elsewhere).join("link");
    std::os::unix::fs::symlink(&root, &link).expect("link");
    let reason = check_state_directory(&link, &root)
        .expect_err("a link to the project was accepted as state");
    assert!(reason.contains("contains the project"), "{reason}");
}

#[test]
fn sibling_state_directory_is_accepted_even_before_it_exists() {
    let parent = crate::test_scratch::scratch().expect("parent");
    let root = canonical(&parent).join("project");
    fs::create_dir(&root).expect("project");
    assert_eq!(
        check_state_directory(&canonical(&parent).join("project-state/server"), &root),
        Ok(())
    );
    assert!(check_state_directory(Path::new("relative/state"), &root).is_err());
}

#[test]
fn preparation_creates_directories_and_empties_scratch_below_the_state_root() {
    let parent = crate::test_scratch::scratch().expect("parent");
    let root = canonical(&parent).join("project");
    let state = canonical(&parent).join("state");
    fs::create_dir(&root).expect("project");
    fs::create_dir_all(state.join("server/tmp")).expect("old scratch");
    fs::write(state.join("server/tmp/leftover"), "old").expect("leftover");
    let wanted = launch(
        vec![state.join("server/cache")],
        vec![state.join("server/tmp")],
    );
    prepare(&wanted, &root, Some(&state)).expect("preparation below the state root");
    assert!(state.join("server/cache").is_dir());
    assert!(state.join("server/tmp").is_dir());
    assert!(
        !state.join("server/tmp/leftover").exists(),
        "the private temporary directory kept a file from an earlier launch"
    );
}

/// A sandbox binds the scratch directory by its path,
/// and another process on the same project prepares the same path.
/// The directory must stay the one that was there:
/// a directory removed and made again is missing for a moment,
/// and bubblewrap stops with `Can't bind mount ... No such file or directory` when it looks in that moment.
#[cfg(target_os = "linux")]
#[test]
fn preparation_empties_a_scratch_directory_without_replacing_it() {
    use std::os::unix::fs::MetadataExt;
    let parent = crate::test_scratch::scratch().expect("parent");
    let root = canonical(&parent).join("project");
    let state = canonical(&parent).join("state");
    fs::create_dir(&root).expect("project");
    let scratch = state.join("server/tmp");
    fs::create_dir_all(scratch.join("nested/deeper")).expect("old scratch");
    fs::write(scratch.join("nested/deeper/leftover"), "old").expect("leftover");
    fs::write(scratch.join("top"), "old").expect("leftover");
    // An open directory is what a running sandbox holds: it must stay linked, and stay the directory at the path.
    let held = fs::File::open(&scratch).expect("hold the scratch directory");
    let before = held.metadata().expect("metadata before").ino();
    prepare(
        &launch(Vec::new(), vec![scratch.clone()]),
        &root,
        Some(&state),
    )
    .expect("preparation");
    let after = held.metadata().expect("metadata after");
    assert!(
        after.nlink() > 0,
        "the scratch directory was removed: a sandbox that holds it keeps a deleted directory"
    );
    assert_eq!(
        fs::metadata(&scratch)
            .expect("the path still names a directory")
            .ino(),
        before,
        "the path names another directory than the one that was held"
    );
    assert_eq!(
        fs::read_dir(&scratch).expect("scratch").count(),
        0,
        "files of an earlier launch stayed"
    );
}

/// A link where the scratch directory belongs is never followed:
/// one that leads out of the state root is refused,
/// and one that leads inside it is replaced by a directory while what it led to stays.
#[test]
fn preparation_does_not_follow_a_link_where_the_scratch_directory_belongs() {
    let parent = crate::test_scratch::scratch().expect("parent");
    let root = canonical(&parent).join("project");
    let state = canonical(&parent).join("state");
    let other = canonical(&parent).join("other");
    fs::create_dir(&root).expect("project");
    fs::create_dir(&other).expect("other");
    fs::write(other.join("keep"), "keep").expect("unrelated file");
    fs::create_dir_all(state.join("server/real")).expect("a directory inside the state root");
    fs::write(state.join("server/real/keep"), "keep").expect("file behind the inner link");
    let scratch = state.join("server/tmp");
    std::os::unix::fs::symlink(&other, &scratch).expect("link out of the state root");
    let reason = prepare(
        &launch(Vec::new(), vec![scratch.clone()]),
        &root,
        Some(&state),
    )
    .expect_err("a link out of the state root was accepted");
    assert!(
        reason.contains("is not below the private state directory"),
        "{reason}"
    );
    assert!(
        other.join("keep").exists(),
        "the link was followed and the directory behind it emptied"
    );
    fs::remove_file(&scratch).expect("remove the outer link");
    std::os::unix::fs::symlink(state.join("server/real"), &scratch)
        .expect("link inside the state root");
    prepare(
        &launch(Vec::new(), vec![scratch.clone()]),
        &root,
        Some(&state),
    )
    .expect("preparation");
    assert!(
        state.join("server/real/keep").exists(),
        "the link was followed and what it led to emptied"
    );
    let found = fs::symlink_metadata(&scratch).expect("scratch");
    assert!(
        found.is_dir() && !found.file_type().is_symlink(),
        "the link stayed in place of the directory"
    );
}

#[test]
fn preparation_refuses_directories_outside_the_state_root_or_inside_the_project() {
    let parent = crate::test_scratch::scratch().expect("parent");
    let root = canonical(&parent).join("project");
    let state = canonical(&parent).join("state");
    let other = canonical(&parent).join("other");
    fs::create_dir(&root).expect("project");
    fs::create_dir(&other).expect("other");
    fs::write(other.join("keep"), "keep").expect("unrelated file");
    let outside = launch(Vec::new(), vec![other.clone()]);
    let reason = prepare(&outside, &root, Some(&state))
        .expect_err("a scratch directory outside the state root was emptied");
    assert!(
        reason.contains("is not below the private state directory"),
        "{reason}"
    );
    assert!(other.join("keep").exists());
    let whole_state = launch(Vec::new(), vec![state.clone()]);
    assert!(prepare(&whole_state, &root, Some(&state)).is_err());
    let no_state = launch(vec![state.join("server")], Vec::new());
    assert!(prepare(&no_state, &root, None).is_err());
    let project_as_state = launch(vec![root.join("server")], Vec::new());
    let whole = prepare(&project_as_state, &root, Some(&root))
        .expect_err("the project itself was accepted as the state root");
    assert!(whole.contains("contains the project"), "{whole}");
    assert!(
        !root.join("server").exists(),
        "a directory was created in the project refused as state"
    );
    // The home folder opened as the project: its private cache is created below the project.
    let state_in_project = launch(vec![root.join(".cache/state/server")], Vec::new());
    assert_eq!(
        prepare(&state_in_project, &root, Some(&root.join(".cache/state"))),
        Ok(())
    );
    assert!(root.join(".cache/state/server").is_dir());
    assert_eq!(
        prepare(&launch(Vec::new(), Vec::new()), &root, None),
        Ok(())
    );
}
