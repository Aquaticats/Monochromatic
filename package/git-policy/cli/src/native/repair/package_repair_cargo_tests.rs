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
use std::path::{Path, PathBuf};

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

/// Facts for a source that has the counterpart workspace,
/// with both of the worktree's directories inside it once their links are followed.
fn facts(source_build: &str) -> StateFacts {
    return StateFacts {
        source_has_the_workspace: true,
        source_build: Some(PathBuf::from(source_build)),
        source_build_exists: true,
        destination_build_exists: false,
        destination_build_is_inside: true,
        destination_target_is_inside: true,
    };
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
/// the add moves that state aside and the service cleans it, so no build can trust the copy's artifacts.
#[test]
fn an_in_tree_build_directory_the_copy_carried_is_moved_aside() {
    let destination: Workspace = workspace("/w/new/ws", "/w/new/ws/target", &["app"]);
    let mut carried: StateFacts = facts("/w/old/ws/target");
    carried.destination_build_exists = true;
    let plan: BuildStatePlan = plan_build_state(&destination, Path::new("/w/new"), &carried);
    assert_eq!(plan, BuildStatePlan::MovedAside { to: PathBuf::from("/w/new/ws/target") });
}

/// An in-tree build directory that did not arrive leaves nothing to clean.
#[test]
fn an_absent_in_tree_state_needs_nothing() {
    let destination: Workspace = workspace("/w/new/ws", "/w/new/ws/target", &["app"]);
    let absent: StateFacts = facts("/w/old/ws/target");
    assert!(matches!(plan_build_state(&destination, Path::new("/w/new"), &absent), BuildStatePlan::Nothing(_)));
}

/// What:
///  A copied `target/` that is a link to a directory outside the worktree is left alone.
/// Why:
///   The copy of the ignored state copies the link,
/// so `cargo clean --package` run in the worktree would delete the files of the directory the link names,
///  which is the source checkout's build output when both checkouts share one.
/// A repair deletes only from directories the worktree it serves holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// planBuildState(destination, root, { ...facts, destinationBuildIsInside: false }) // Nothing
/// ```
#[test]
fn a_build_directory_a_link_carries_out_of_the_worktree_is_left_alone() {
    let destination: Workspace = workspace("/w/new/ws", "/w/new/ws/target", &["app"]);
    // Each of the two directories the clean deletes from, one at a time.
    for outside in ["build", "target"] {
        let mut linked: StateFacts = facts("/w/old/ws/target");
        linked.destination_build_exists = true;
        if outside == "build" {
            linked.destination_build_is_inside = false;
        } else {
            linked.destination_target_is_inside = false;
        }
        assert!(
            matches!(plan_build_state(&destination, Path::new("/w/new"), &linked), BuildStatePlan::Nothing(_)),
            "a {outside} directory that resolves outside the worktree is not cleaned"
        );
    }
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
    unknown.source_has_the_workspace = false;
    unknown.source_build = None;
    unknown.source_build_exists = false;
    assert!(matches!(plan_build_state(&destination, Path::new("/w/new"), &unknown), BuildStatePlan::Nothing(_)));
    let mut existing: StateFacts = facts("/cache/old");
    existing.destination_build_exists = true;
    assert!(matches!(plan_build_state(&destination, Path::new("/w/new"), &existing), BuildStatePlan::Nothing(_)));
}

/// A source with no build state fills nothing,
/// and a worktree with a copied target is still not cleaned.
///
/// What:
///  The worktree's build directory is absent, so Cargo has no fingerprint of its own crates to trust:
/// it rebuilds them,
/// and a clean would delete nothing a build could reuse.
/// Why:
///   This is the case every add on this host takes (34 of 35 workspaces in the records of 2026-11-12),
/// and the clean cost 32.8 to 47.7 s of work per add there while deleting nothing.
/// A copy of a built workspace whose `target/` holds the source's binary is rebuilt by Cargo by itself:
/// the copy's first build named the copy's own path
/// (`doc/planning/cli-git-worktree-add-package-repair-off-the-waiting-path.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// planBuildState(destination, root, { ...facts, sourceBuild: undefined, sourceBuildExists: false }) // Nothing
/// ```
#[test]
fn a_worktree_with_no_build_state_of_its_own_is_never_cleaned() {
    let destination: Workspace = workspace("/w/new/ws", "/cache/new", &["app"]);
    let mut empty: StateFacts = facts("/cache/old");
    empty.source_build_exists = false;
    assert!(
        matches!(plan_build_state(&destination, Path::new("/w/new"), &empty), BuildStatePlan::Nothing(_)),
        "a worktree whose build directory is absent holds nothing a clean would reach, so it is not cleaned: {:?}",
        plan_build_state(&destination, Path::new("/w/new"), &empty)
    );
    // The positive control: the same facts with the worktree's build directory present are cleaned.
    let mut carried: StateFacts = facts("/cache/old");
    carried.destination_build_exists = true;
    assert!(matches!(plan_build_state(&destination, Path::new("/w/new"), &carried), BuildStatePlan::Nothing(_)), "state of the worktree's own is kept, not filled");
    let mut in_tree: StateFacts = facts("/w/old/ws");
    in_tree.destination_build_exists = true;
    let in_tree_workspace: Workspace = workspace("/w/new/ws", "/w/new/ws/target", &["app"]);
    assert_eq!(plan_build_state(&in_tree_workspace, Path::new("/w/new"), &in_tree), BuildStatePlan::MovedAside { to: PathBuf::from("/w/new/ws/target") });
}

/// A stand-in `cargo` that records the arguments and the directory of every run it is given,
/// and answers `metadata` with one workspace.
fn recording_cargo(root: &Path, metadata: &str) -> (PathBuf, PathBuf) {
    let program: PathBuf = root.join("cargo");
    let calls: PathBuf = root.join("calls.log");
    crate::test_support::executable(
        program.as_path(),
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$* | $(pwd -P)\" >> '{}'\ncase \"$1\" in\nmetadata) printf '%s\\n' '{}' ;;\nesac\nexit 0\n",
            calls.display(),
            metadata
        )
        .as_bytes(),
    );
    return (program, calls);
}

/// What:
///  The metadata one workspace of a fixture answers with,
/// naming the source as its root and the two directories the plan reads.
/// Why:
///   The stand-in's answer is the only place those paths are spelled,
/// so a control names them once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const metadataOf = ({ root, build, target }) => JSON.stringify({ packages: [{ name: 'tiny' }], workspace_root: root, build_directory: build, target_directory: target });
/// ```
fn metadata_of(root: &Path, build: &Path, target: &Path) -> String {
    return format!(
        "{{\"packages\":[{{\"name\":\"tiny\"}}],\"workspace_root\":\"{}\",\"build_directory\":\"{}\",\"target_directory\":\"{}\"}}",
        root.display(),
        build.display(),
        target.display()
    );
}

/// What:
///  The add answers without waiting for a clean it would not need:
/// a worktree whose build directory is absent holds nothing a build could reuse,
/// so it starts no `cargo clean`.
/// Why:
///   Every add on this host took that clean for 34 of its 35 workspaces,
/// 32.8 to 47.7 s of workspace resolutions that deleted nothing
/// (`doc/planning/cli-git-worktree-add-package-repair-off-the-waiting-path.md`),
/// and a copied `target/` is what made the plan choose it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = await repairWorkspaceAwaiting(tool, workspace, worktree, source, undefined); expect(recorded).not.toContain('clean')
/// ```
#[test]
fn a_worktree_with_no_build_directory_of_its_own_runs_no_clean() {
    let root: PathBuf = crate::test_support::fixture("cargo-repair-no-clean");
    let worktree: PathBuf = root.join("worktree");
    let package: PathBuf = worktree.join("package/tiny");
    let source: PathBuf = root.join("source");
    std::fs::create_dir_all(package.as_path()).expect("the worktree's workspace");
    std::fs::create_dir_all(source.join("package/tiny")).expect("the source's counterpart workspace");
    std::fs::write(source.join("package/tiny/Cargo.toml"), b"[package]\nname = \"tiny\"\n").expect("the source's manifest");
    // The copy carried the workspace's `target/`, and the worktree has no build directory of its own.
    std::fs::create_dir_all(package.join("target/debug")).expect("the copied target directory");
    let external: PathBuf = root.join("build/aa/cccccccccccccc");
    // The source checkout's own build directory is a different one, which nothing has built: the case every add on this
    //  host takes, where the worktree holds no build state and the source holds none for it either.
    let unbuilt: PathBuf = root.join("build/aa/ssssssssssssss");
    let (cargo, calls): (PathBuf, PathBuf) = recording_cargo(
        root.as_path(),
        metadata_of(source.as_path(), unbuilt.as_path(), package.join("target").as_path()).as_str(),
    );
    let tool: CargoTool = CargoTool { program: cargo, limits: ToolLimits { window: std::time::Duration::from_secs(60), ceiling: std::time::Duration::from_secs(60) } };
    let workspace: Workspace = Workspace { root: package.clone(), build_directory: external.clone(), target_directory: package.join("target"), members: vec![String::from("tiny")] };
    let done: Done = crate::lazy_runtime::run_on_runtime(repair_workspace_awaiting(&tool, &workspace, worktree.as_path(), source.as_path(), None)).expect("this process's runtime starts");
    let recorded: String = std::fs::read_to_string(calls.as_path()).unwrap_or_default();
    // The stand-in records the argument line, so the verb is the first word of a line; a directory whose name holds "clean" is
    // not a clean.
    let cleaned: Vec<&str> = recorded.lines().filter(|line: &&str| line.starts_with("clean ")).collect();
    assert!(cleaned.is_empty(), "the add starts no clean where the worktree holds no build directory: {cleaned:?} of {recorded}");
    assert!(!matches!(done, Done::Cleaned(_)), "nothing of the worktree was cleaned");
    crate::test_support::remove(root.as_path());
}

/// What:
///  A build directory inside the worktree that the copy carried is moved aside before the add answers,
/// in one rename and with no Cargo run at all,
/// and the workspace is handed over as an entry whose own state is staged.
/// Why:
///   This is the positive control of the test above:
/// the same stand-in records a Cargo run for the worktree whose build directory is absent,
/// and records none at all where the copy carried state a build could trust.
/// Cargo would treat that state as fresh, so it must be out of the way before the add returns,
/// and the rename is what puts it there without a `cargo clean` the add would have to wait for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = await repairWorkspaceAwaiting(tool, workspace, worktree, source, undefined); expect(done.staged).toBe(true); expect(existsSync(staging)).toBe(true); expect(recorded).not.toContain('clean')
/// ```
#[test]
fn a_build_directory_the_copy_carried_is_moved_aside_before_the_add_answers() {
    let root: PathBuf = crate::test_support::fixture("cargo-repair-carried-state");
    let worktree: PathBuf = root.join("worktree");
    let package: PathBuf = worktree.join("package/tiny");
    let source: PathBuf = root.join("source");
    std::fs::create_dir_all(package.join("target/debug")).expect("the copied build state");
    std::fs::write(package.join("target/debug/copied"), b"the copy's state\n").expect("a file of the copy");
    std::fs::create_dir_all(source.join("package/tiny")).expect("the source's counterpart workspace");
    std::fs::write(source.join("package/tiny/Cargo.toml"), b"[package]\nname = \"tiny\"\n").expect("the source's manifest");
    let (cargo, calls): (PathBuf, PathBuf) = recording_cargo(
        root.as_path(),
        metadata_of(source.as_path(), package.join("target").as_path(), package.join("target").as_path()).as_str(),
    );
    let tool: CargoTool = CargoTool { program: cargo, limits: ToolLimits { window: std::time::Duration::from_secs(60), ceiling: std::time::Duration::from_secs(60) } };
    let workspace: Workspace = Workspace { root: package.clone(), build_directory: package.join("target"), target_directory: package.join("target"), members: vec![String::from("tiny")] };
    // The repository's Git directory of this fixture, whose exclude file names the staging patterns.
    let git_directory: PathBuf = root.join("git-directory");
    std::fs::create_dir_all(git_directory.as_path()).expect("the fixture's Git directory");
    let done: Done = crate::lazy_runtime::run_on_runtime(repair_workspace_awaiting(&tool, &workspace, worktree.as_path(), source.as_path(), Some(git_directory.as_path()))).expect("this process's runtime starts");
    let recorded: String = std::fs::read_to_string(calls.as_path()).unwrap_or_default();
    let cleaned: Vec<&str> = recorded.lines().filter(|line: &&str| line.starts_with("clean ")).collect();
    assert!(cleaned.is_empty(), "the add runs no clean for state it moved aside: {recorded}");
    let Done::HandedOver(entry) = done else {
        panic!("the workspace is handed over to the filling");
    };
    let staged: PathBuf = entry.staged_in.clone().expect("the entry says where the worktree's own state waits");
    assert_eq!(entry.to, package.join("target"));
    assert!(!entry.to.exists(), "the copied state is out of the worktree's build directory before the add answers");
    assert_eq!(staged, crate::package_repair_prewarm_job::staging_of(entry.to.as_path()), "the state waits beside the build directory it was taken from");
    let exclude: String = std::fs::read_to_string(git_directory.join("info/exclude")).expect("the repository's exclude file names the staging patterns");
    assert!(exclude.contains("*.monochromatic-prewarm/"), "the staging name is kept out of the person's `git status`: {exclude}");
    assert!(staged.join("debug/copied").is_file(), "the copy's state sits at the staging name: {}", staged.display());
    crate::test_support::remove(root.as_path());
}

/// What:
///  A copied `target` that is a link into somebody else's directory is never cleaned,
/// even where Cargo keeps its build directory outside the worktree.
/// Why:
///   The audit of 2026-11-11 (`kimi-audit-alpha-12-package-repair-on-add-and-commit`) found the external layout
/// skipped the linked-target guard:
/// a copied `target` link plus a source with no build directory chose `CleanOnly`,
/// and the clean deleted the artifact the link names (`README.md`, "the target is left alone").
/// The workspace holds no build directory of its own here,
/// so nothing a build could reuse is in the worktree and no clean runs at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// planBuildState(external, root, { ...facts, sourceBuildExists: false, destinationTargetIsInside: false }) // Nothing
/// ```
#[test]
fn a_target_link_that_leaves_the_worktree_is_never_cleaned_with_an_external_build_directory() {
    let destination: Workspace = workspace("/w/new/ws", "/cache/new", &["app"]);
    let mut found: StateFacts = facts("/cache/old");
    found.source_build_exists = false;
    found.destination_target_is_inside = false;
    assert!(
        matches!(plan_build_state(&destination, Path::new("/w/new"), &found), BuildStatePlan::Nothing(_)),
        "a clean must never reach outside the worktree it was started for: {:?}",
        plan_build_state(&destination, Path::new("/w/new"), &found)
    );
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
