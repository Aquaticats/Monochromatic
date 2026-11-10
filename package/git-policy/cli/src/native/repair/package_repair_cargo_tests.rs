//! The Cargo repair:
//! metadata parsing,
//! one repair per workspace,
//! the plan for each layout,
//! and the clean arguments.

use super::*;
use crate::package_repair_cargo_workspaces::{MANIFEST, distinct_workspaces, manifest_directories_awaiting, parse_metadata, read_workspace};
use crate::transaction_git::GitContext;
use crate::package_repair_command::ToolLimits;
use crate::test_support::{fixture, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// The `cargo metadata --no-deps` text of a workspace,
/// built from the fields the repair reads (the paths are plain text).
fn metadata(root: &str, build: &str, target: &str, members: &[&str]) -> String {
    let packages: Vec<String> = members.iter().map(|name| { return format!("{{\"name\":\"{name}\",\"version\":\"0.1.0\",\"id\":\"path+file://{root}/{name}#0.1.0\"}}"); }).collect();
    return format!("{{\"packages\":[{}],\"workspace_root\":\"{root}\",\"build_directory\":\"{build}\",\"target_directory\":\"{target}\",\"workspace_members\":[]}}", packages.join(","));
}

/// A workspace whose state is where the layout says.
fn workspace(root: &str, build: &str, members: &[&str]) -> Workspace {
    return Workspace { root: PathBuf::from(root), build_directory: PathBuf::from(build), target_directory: PathBuf::from(format!("{root}/target")), members: members.iter().map(|name| { return (*name).to_owned(); }).collect() };
}

/// The four facts and the member names come out of the metadata text.
#[test]
fn parses_the_directories_and_the_member_names() {
    let found: Workspace = parse_metadata(metadata("/w/ws", "/cache/ab/cd", "/w/ws/target", &["app", "lib"]).as_str()).expect("parse");
    assert_eq!(found, Workspace { root: PathBuf::from("/w/ws"), build_directory: PathBuf::from("/cache/ab/cd"), target_directory: PathBuf::from("/w/ws/target"), members: vec![String::from("app"), String::from("lib")] });
}

/// A Cargo too old to name the build directory falls back to the target directory,
/// where it has always been.
#[test]
fn a_missing_build_directory_is_the_target_directory() {
    let text: &str = "{\"packages\":[],\"workspace_root\":\"/w/ws\",\"target_directory\":\"/w/ws/target\"}";
    assert_eq!(parse_metadata(text).expect("parse").build_directory, PathBuf::from("/w/ws/target"));
}

/// Text that is not metadata is an error that names the missing field.
#[test]
fn malformed_metadata_is_an_error() {
    assert!(parse_metadata("not json").is_err());
    let error: String = parse_metadata("{\"packages\":[]}").expect_err("no root");
    assert!(error.contains("workspace_root"), "{error}");
}

/// A workspace seen through several of its manifests is repaired once,
/// in root order.
#[test]
fn a_workspace_is_listed_once_in_root_order() {
    let found: Vec<Workspace> = vec![workspace("/w/b", "/c/b", &["b"]), workspace("/w/a", "/c/a", &["a1", "a2"]), workspace("/w/a", "/c/a", &["a1", "a2"])];
    let distinct: Vec<Workspace> = distinct_workspaces(found);
    assert_eq!(distinct.iter().map(|each| { return each.root.clone(); }).collect::<Vec<PathBuf>>(), vec![PathBuf::from("/w/a"), PathBuf::from("/w/b")]);
}

/// Facts for a source that has the counterpart workspace.
fn facts(source_build: &str) -> StateFacts {
    return StateFacts { source_build: Some(PathBuf::from(source_build)), source_build_exists: true, destination_build_exists: false, destination_target_exists: true };
}

/// An external build directory that the copy did not carry is filled by the commit service,
/// so the add does not clone it and the members are cleaned in that copy.
#[test]
fn an_external_build_directory_is_filled_by_the_service() {
    let destination: Workspace = workspace("/w/new/ws", "/cache/new", &["app"]);
    let plan: BuildStatePlan = plan_build_state(&destination, Path::new("/w/new"), &facts("/cache/old"));
    assert_eq!(plan, BuildStatePlan::FillFromTheSource { from: PathBuf::from("/cache/old"), to: PathBuf::from("/cache/new") });
}

/// Under the default layout the build directory is inside the worktree and arrived with `target/`:
/// only the clean remains.
#[test]
fn an_in_tree_build_directory_is_only_cleaned() {
    let destination: Workspace = workspace("/w/new/ws", "/w/new/ws/target", &["app"]);
    let plan: BuildStatePlan = plan_build_state(&destination, Path::new("/w/new"), &facts("/w/old/ws/target"));
    assert_eq!(plan, BuildStatePlan::CleanOnly);
}

/// An in-tree build directory that did not arrive leaves nothing to clean.
#[test]
fn an_absent_in_tree_state_needs_nothing() {
    let destination: Workspace = workspace("/w/new/ws", "/w/new/ws/target", &["app"]);
    let mut absent: StateFacts = facts("/w/old/ws/target");
    absent.destination_target_exists = false;
    assert!(matches!(plan_build_state(&destination, Path::new("/w/new"), &absent), BuildStatePlan::Nothing(_)));
}

/// A build directory shared by source and worktree is never cleaned:
/// that would remove state the source still uses.
#[test]
fn a_shared_build_directory_is_left_alone() {
    let destination: Workspace = workspace("/w/new/ws", "/cache/shared", &["app"]);
    assert!(matches!(plan_build_state(&destination, Path::new("/w/new"), &facts("/cache/shared")), BuildStatePlan::Nothing(_)));
}

/// A workspace the source does not have,
/// and a worktree build directory that already holds state,
/// are left alone.
#[test]
fn a_new_workspace_and_existing_state_are_left_alone() {
    let destination: Workspace = workspace("/w/new/ws", "/cache/new", &["app"]);
    let mut unknown: StateFacts = facts("/cache/old");
    unknown.source_build = None;
    assert!(matches!(plan_build_state(&destination, Path::new("/w/new"), &unknown), BuildStatePlan::Nothing(_)));
    let mut existing: StateFacts = facts("/cache/old");
    existing.destination_build_exists = true;
    assert!(matches!(plan_build_state(&destination, Path::new("/w/new"), &existing), BuildStatePlan::Nothing(_)));
}

/// A source with no build state clones nothing,
/// and a worktree with a copied target is still cleaned.
#[test]
fn an_empty_source_clones_nothing() {
    let destination: Workspace = workspace("/w/new/ws", "/cache/new", &["app"]);
    let mut empty: StateFacts = facts("/cache/old");
    empty.source_build_exists = false;
    assert_eq!(plan_build_state(&destination, Path::new("/w/new"), &empty), BuildStatePlan::CleanOnly);
    empty.destination_target_exists = false;
    assert!(matches!(plan_build_state(&destination, Path::new("/w/new"), &empty), BuildStatePlan::Nothing(_)));
}

/// Every member gets its own `--package` and the clean never uses the network;
/// `--locked` would make a stale lock fail it.
#[test]
fn the_clean_names_every_member_and_stays_offline() {
    let members: Vec<String> = vec![String::from("app"), String::from("lib")];
    let arguments: Vec<String> = clean_arguments(&members).iter().map(|each| { return each.to_string_lossy().into_owned(); }).collect();
    assert_eq!(arguments.first().map(String::as_str), Some("clean"));
    assert!(arguments.iter().any(|each| { return each == "--offline"; }), "--offline missing in {arguments:?}");
    assert!(!arguments.iter().any(|each| { return each == "--locked"; }), "--locked would fail on a stale lock: {arguments:?}");
    for member in &members {
        let position: usize = arguments.iter().position(|each| { return each == member; }).expect("member present");
        assert_eq!(arguments[position - 1], "--package", "{arguments:?}");
    }
}



/// Output that was cut is never read as the metadata (sol-1 round 4:
/// nothing exercised the refusal):
/// a cargo that succeeds with more
/// output than the snapshot keeps is an error that says the output could not be taken whole,
/// not a parse of its tail.
#[test]
fn metadata_that_was_cut_is_not_parsed() {
    let root: std::path::PathBuf = crate::test_support::fixture("cargo-metadata-cut");
    let cargo: std::path::PathBuf = root.join("cargo");
    let too_much: usize = crate::output_capture::STEP_OUTPUT_LIMIT + 1024;
    crate::test_support::executable(cargo.as_path(), format!("#!/bin/sh\nhead --bytes={too_much} /dev/zero | tr '\\0' 'x'\n").as_bytes());
    let limits: ToolLimits = ToolLimits { window: std::time::Duration::from_secs(60), ceiling: std::time::Duration::from_secs(60) };
    let refused: String = read_workspace(cargo.as_path(), root.as_path(), limits).expect_err("cut output is refused");
    assert!(refused.contains("could not be taken whole"), "{refused}");
    crate::test_support::remove(root.as_path());
}

/// The tracked manifests are named from the top of the worktree and found whatever pathspec modes the caller set:
/// a caller whose `-C` moved Git into a subdirectory must still be answered with `sub/Cargo.toml`,
/// because every name is joined to the destination root.
#[test]
fn the_manifest_listing_answers_from_the_top_under_the_callers_prefix_and_modes() {
    let root: PathBuf = crate::test_support::fixture("cargo-manifests-nested-c");
    let destination: PathBuf = crate::test_support::repository(root.as_path(), "destination");
    std::fs::create_dir(destination.join("sub")).expect("sub directory");
    std::fs::write(destination.join("sub/Cargo.toml"), b"[package]\nname = \"sub\"\n").expect("manifest");
    crate::test_support::git(destination.as_path(), &["add", "--", "sub/Cargo.toml"]);
    // The caller wrote `-C <destination>/sub`, and two glob modes that disagree in its environment.
    let mut context = crate::test_support::git_context();
    context.global_prefix = vec![
        std::ffi::OsString::from("-C"),
        destination.join("sub").into_os_string(),
    ];
    context.overlay.push((std::ffi::OsString::from("GIT_GLOB_PATHSPECS"), std::ffi::OsString::from("1")));
    context.overlay.push((std::ffi::OsString::from("GIT_NOGLOB_PATHSPECS"), std::ffi::OsString::from("1")));
    let environment: crate::package_repair_request::RepairEnvironment = crate::package_repair_request::RepairEnvironment {
        search_path: None,
        quiet_window: std::time::Duration::from_secs(60),
        ceiling: std::time::Duration::from_secs(600),
        workers: 1,
    };
    let request: crate::package_repair_request::RepairRequest<'_> = crate::package_repair_request::RepairRequest {
        git: &context,
        source: root.as_path(),
        destination: destination.as_path(),
        switch: crate::package_repair_request::RepairSwitch::default(),
        environment: &environment,
    };
    let _ = &request;
    let directories: Vec<PathBuf> = crate::lazy_runtime::run_on_runtime(crate::package_repair_cargo_workspaces::manifest_directories_awaiting(
        &context,
        destination.as_path(),
    ))
    .expect("this process's runtime starts")
    .expect("the tracked manifests are listed");
    assert_eq!(directories, vec![destination.join("sub")], "the manifest below the top is found by its repository-relative name");
    crate::test_support::remove(root.as_path());
}

/// What:
///  The tracked manifests are read as a job of the process's blocking pool,
///  never on the task that polls the three repairs.
/// Why:
///  Whether a listed path is a regular file is a storage question,
/// and this reading shares its task with the pnpm repair's quiet window and ceiling:
/// a filesystem that stops answering would then stop those limits with it
/// (`package_repair_cargo_workspaces.rs`,
/// `directories_of_the_listing`).
///  A control holds the process's only place,
/// so it proves the reading reached the bound by asking for it.
#[test]
fn the_manifest_directories_are_read_as_a_job_of_the_pool() {
    let root: PathBuf = fixture("cargo-manifest-place");
    let worktree: PathBuf = repository(root.as_path(), "worktree");
    std::fs::write(worktree.join(MANIFEST), b"").expect("manifest");
    std::fs::create_dir_all(worktree.join("member")).expect("member directory");
    std::fs::write(worktree.join("member").join(MANIFEST), b"").expect("member manifest");
    let _staged: String = git_text(worktree.as_path(), &["add", "--", MANIFEST, "member/Cargo.toml"]);
    let (_given_back, finished) = crate::parallel_test_support::hold_the_only_place();
    let git: GitContext = git_context();
    let destination: PathBuf = worktree.clone();
    let reading = crate::lazy_runtime::spawn_on_runtime(async move { return manifest_directories_awaiting(&git, destination.as_path()).await; });
    assert!(reading.is_some(), "this process's runtime has a worker of its own");
    let asked: bool = finished.recv_timeout(crate::parallel_test_support::PATIENCE).expect("the control that holds the only place answered");
    remove(&root);
    assert!(
        asked,
        "the tracked manifests were read without asking for a place of the process's bound on blocking jobs, so the reading ran on the task that polls the repairs"
    );
}
