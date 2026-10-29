//! The confined launch's exact command lines,
//!  settings overrides,
//!  state naming,
//!  and refusals.

use super::recipe::{confined_settings, recipe};
use super::{confine_with, private_state, state_directory};
use crate::language::launch::{LaunchRequest, scratch_name};
use serde_json::json;
use std::{collections::HashMap, ffi::OsStr, path::Path, path::PathBuf};

fn request(server: &str, environment: &[(&str, &str)]) -> LaunchRequest {
    let mut variables = HashMap::new();
    for (name, value) in environment {
        variables.insert(name.to_string(), value.to_string());
    }
    return LaunchRequest {
        server: server.to_string(),
        executable: PathBuf::from("/opt/servers/bin/server"),
        args: vec!["--stdio".to_string()],
        environment: variables,
        settings: Some(json!({ "kept": true })),
        project_root: PathBuf::from("/work/project"),
        project_spellings: Vec::new(),
        read_only_binds: Vec::new(),
        state_root: Some(PathBuf::from("/state")),
    };
}

fn strings(parts: &[&str]) -> Vec<String> {
    let mut owned: Vec<String> = Vec::new();
    for part in parts {
        owned.push(part.to_string());
    }
    return owned;
}

#[test]
fn rust_analyzer_gets_the_adopted_recipe_with_cargo_redirects() {
    let inherited = vec![
        ("HOME".to_string(), "/home/someone".to_string()),
        ("PATH".to_string(), "/usr/bin".to_string()),
    ];
    let state = Path::new("/state/project-0/rust-analyzer");
    let launch = recipe(
        &request("rust-analyzer", &[]),
        state,
        "/usr/bin/bwrap",
        &inherited,
    )
    .expect("recipe");
    assert_eq!(launch.command, "/usr/bin/bwrap");
    // The private temporary directory is named for the process, so the line is built from the same name.
    let scratch = format!("/state/project-0/rust-analyzer/{}", scratch_name());
    let expected = strings(&[
        "--die-with-parent",
        "--new-session",
        "--unshare-user",
        "--unshare-pid",
        "--unshare-ipc",
        "--unshare-uts",
        "--unshare-cgroup",
        "--unshare-net",
        "--ro-bind",
        "/",
        "/",
        "--dev",
        "/dev",
        "--proc",
        "/proc",
        "--tmpfs",
        "/run",
        "--bind",
        &scratch,
        "/tmp",
        "--ro-bind",
        "/work/project",
        "/work/project",
        "--bind",
        "/state/project-0/rust-analyzer",
        "/state/project-0/rust-analyzer",
        "--clearenv",
        "--setenv",
        "CARGO_BUILD_BUILD_DIR",
        "/state/project-0/rust-analyzer/build",
        "--setenv",
        "CARGO_TARGET_DIR",
        "/state/project-0/rust-analyzer/target",
        "--setenv",
        "HOME",
        "/home/someone",
        "--setenv",
        "PATH",
        "/usr/bin",
        "--setenv",
        "XDG_CACHE_HOME",
        "/state/project-0/rust-analyzer/cache",
        "--setenv",
        "npm_config_cache",
        "/state/project-0/rust-analyzer/npm-cache",
        "--",
        "/opt/servers/bin/server",
        "--stdio",
    ]);
    assert_eq!(
        launch.args, expected,
        "the rust-analyzer command line differs from the adopted shape"
    );
    assert!(
        launch.environment.is_empty(),
        "variables must travel through --setenv after --clearenv"
    );
    assert_eq!(launch.settings, Some(json!({ "kept": true })));
    assert_eq!(
        launch.directories,
        vec![
            state.to_path_buf(),
            state.join("cache"),
            state.join("npm-cache"),
            state.join("target"),
            state.join("build"),
        ]
    );
    assert_eq!(launch.scratch, vec![state.join(scratch_name())]);
}

#[test]
fn typescript_server_runs_without_a_pid_namespace_and_without_type_acquisition() {
    let state = Path::new("/state/project-0/typescript-native");
    let launch = recipe(
        &request("typescript-native", &[]),
        state,
        "/usr/bin/bwrap",
        &[],
    )
    .expect("recipe");
    assert!(
        !launch.args.contains(&"--unshare-pid".to_string()),
        "the TypeScript server would exit inside a process-id namespace"
    );
    assert!(launch.args.contains(&"--unshare-net".to_string()));
    assert!(!launch.args.contains(&"CARGO_TARGET_DIR".to_string()));
    assert_eq!(
        launch.settings,
        Some(json!({
            "kept": true,
            "typescript": { "tsserver": { "automaticTypeAcquisition": { "enabled": false } } },
            "js/ts": { "tsserver": { "automaticTypeAcquisition": { "enabled": false } } },
        }))
    );
    assert_eq!(launch.directories.len(), 3);
}

#[test]
fn node_servers_that_watch_the_client_run_without_a_pid_namespace_but_without_network() {
    let watchers = [
        "yaml-language-server",
        "actions-language-server",
        "ansible-language-server",
        "vscode-json-language-server",
        "vscode-css-language-server",
        "vscode-html-language-server",
        "bash-language-server",
        "docker-langserver",
        "docker-compose-langserver",
    ];
    for server in watchers {
        let state = PathBuf::from(format!("/state/project-0/{server}"));
        let launch = recipe(&request(server, &[]), &state, "/usr/bin/bwrap", &[]).expect("recipe");
        assert!(
            !launch.args.contains(&"--unshare-pid".to_string()),
            "{server} exits about 3 s after start when the client's process id is invisible"
        );
        assert!(
            launch.args.contains(&"--unshare-net".to_string()),
            "{server} kept network access"
        );
    }
    let other = recipe(
        &request("taplo", &[]),
        Path::new("/state/project-0/taplo"),
        "/usr/bin/bwrap",
        &[],
    )
    .expect("recipe");
    assert!(
        other.args.contains(&"--unshare-pid".to_string()),
        "a server that does not watch the client lost its process-id namespace"
    );
}

#[test]
fn definition_variables_pass_through_and_redirects_win() {
    let inherited = vec![("HOME".to_string(), "/home/someone".to_string())];
    let wanted = request(
        "scripted-ls",
        &[
            ("IDE_SCRIPTED_AUDIT", "1"),
            ("XDG_CACHE_HOME", "/elsewhere"),
            ("HOME", "/override"),
        ],
    );
    let launch = recipe(
        &wanted,
        Path::new("/s/p/scripted-ls"),
        "/usr/bin/bwrap",
        &inherited,
    )
    .expect("recipe");
    let text = launch.args.join(" ");
    assert!(text.contains("--setenv IDE_SCRIPTED_AUDIT 1"), "{text}");
    assert!(text.contains("--setenv HOME /override"), "{text}");
    assert!(
        text.contains("--setenv XDG_CACHE_HOME /s/p/scripted-ls/cache"),
        "{text}"
    );
    assert!(
        !text.contains("/elsewhere"),
        "a definition redirected the cache out of private state"
    );
    assert!(text.contains("--unshare-pid"));
}

#[test]
fn typescript_language_server_disables_typing_acquisition() {
    assert_eq!(
        confined_settings("typescript-language-server", None),
        Some(json!({ "disableAutomaticTypingAcquisition": true }))
    );
    assert_eq!(confined_settings("rust-analyzer", None), None);
}

#[test]
fn state_directory_is_per_project_and_per_server() {
    let root = Path::new("/state");
    let first = state_directory(root, Path::new("/a/project"), "rust-analyzer");
    let second = state_directory(root, Path::new("/b/project"), "rust-analyzer");
    assert_ne!(
        first, second,
        "two projects with one name shared private state"
    );
    assert_eq!(
        first.file_name().and_then(OsStr::to_str),
        Some("rust-analyzer")
    );
    assert_eq!(
        first,
        state_directory(root, Path::new("/a/project"), "rust-analyzer")
    );
    let parent = first.parent().expect("project component");
    let file_name = parent.file_name().expect("text");
    let name = file_name.to_str().expect("text");
    assert!(
        name.starts_with("project-") && name.len() == "project-".len() + 16,
        "{name}"
    );
    let odd = state_directory(root, Path::new("/x/we ird:..name"), "../escape");
    assert!(odd.starts_with(root));
    assert_eq!(odd.file_name().and_then(OsStr::to_str), Some(".._escape"));
    assert_eq!(
        odd.components().count(),
        4,
        "a server name escaped its state directory: {odd:?}"
    );
}

#[test]
fn missing_bubblewrap_refuses_with_the_remedy() {
    let reason = confine_with(&request("rust-analyzer", &[]), "/nonexistent/bwrap")
        .expect_err("a launch without bubblewrap was accepted");
    assert!(
        reason.contains("/nonexistent/bwrap is not installed"),
        "{reason}"
    );
    assert!(reason.contains("Install bubblewrap"), "{reason}");
}

#[test]
fn only_paths_below_proc_refuse() {
    let mut under_tmp = request("rust-analyzer", &[]);
    under_tmp.project_root = PathBuf::from("/tmp/project");
    under_tmp.project_spellings = vec![PathBuf::from("/run/media/someone/project")];
    let missing =
        confine_with(&under_tmp, "/nonexistent/bwrap").expect_err("bubblewrap is missing");
    assert!(
        missing.contains("is not installed"),
        "a project below /tmp or /run was refused for its location: {missing}"
    );
    let mut in_proc = request("rust-analyzer", &[]);
    in_proc.project_root = PathBuf::from("/proc/1/cwd");
    let reason = confine_with(&in_proc, "/nonexistent/bwrap").expect_err("/proc was accepted");
    assert!(reason.contains("is below /proc"), "{reason}");
    assert!(reason.contains("Open the project from"), "{reason}");
    let mut spelled = request("rust-analyzer", &[]);
    spelled.project_spellings = vec![PathBuf::from("/proc/self/cwd")];
    let spelling = confine_with(&spelled, "/nonexistent/bwrap").expect_err("/proc was accepted");
    assert!(spelling.contains("project spelling"), "{spelling}");
    let mut state_in_proc = request("rust-analyzer", &[]);
    state_in_proc.state_root = Some(PathBuf::from("/proc/self/cache"));
    let refused = private_state(&state_in_proc).expect_err("state below /proc was accepted");
    assert!(refused.contains("is below /proc"), "{refused}");
    assert!(refused.contains("restart the application"), "{refused}");
}

#[test]
fn state_root_is_resolved_before_it_is_checked_and_bound() {
    let project = crate::test_scratch::scratch().expect("project");
    let elsewhere = crate::test_scratch::scratch().expect("elsewhere");
    let links = crate::test_scratch::scratch().expect("links");
    let project_root = project.path().canonicalize().expect("canonical project");
    let target = elsewhere.path().canonicalize().expect("canonical target");
    std::os::unix::fs::symlink(&target, links.path().join("away")).expect("link away");
    std::os::unix::fs::symlink(&project_root, links.path().join("inside")).expect("link inside");
    let mut linked = request("rust-analyzer", &[]);
    linked.project_root = project_root.clone();
    linked.state_root = Some(links.path().join("away").join("cache"));
    let state = private_state(&linked).expect("a linked state root outside the project");
    assert!(
        state.starts_with(target.join("cache")),
        "the state directory kept the link spelling: {state:?}"
    );
    // State inside the project (the home folder opened as the project) is allowed and is bound at its
    // real location, which the sandbox mounts writable after the read-only project.
    linked.state_root = Some(links.path().join("inside").join("cache"));
    let inside = private_state(&linked).expect("state inside the project through a link");
    assert!(
        inside.starts_with(project_root.join("cache")),
        "the state directory kept the link spelling: {inside:?}"
    );
    // The project itself, reached through a link, would make all of it writable.
    linked.state_root = Some(links.path().join("inside"));
    let reason = private_state(&linked).expect_err("the project itself through a link");
    assert!(reason.contains("contains the project"), "{reason}");
    assert!(reason.contains("restart the application"), "{reason}");
    linked.state_root = Some(links.path().join("away"));
    linked.project_root = target.join("nested");
    let ancestor = private_state(&linked).expect_err("state containing the project");
    assert!(ancestor.contains("contains the project"), "{ancestor}");
}

#[test]
fn missing_state_root_refuses_before_anything_runs() {
    let mut without_state = request("rust-analyzer", &[]);
    without_state.state_root = None;
    // `/bin/sh` exists everywhere; the state check comes before any bubblewrap run.
    let reason =
        confine_with(&without_state, "/bin/sh").expect_err("a launch without state was accepted");
    assert!(reason.contains("no private state directory"), "{reason}");
}
