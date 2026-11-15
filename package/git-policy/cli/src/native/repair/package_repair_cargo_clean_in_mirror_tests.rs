//! Controls of one workspace resolved and cleaned inside its mirror:
//! the person's `Cargo.lock` is read and never written,
//! and the clean removes the same artifacts a clean in the workspace removes.

use super::{MirrorClean, clean_in_a_mirror};
use crate::package_repair_cargo::CargoTool;
use crate::package_repair_command::{ToolEnd, ToolLimits, ToolRun};
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};

/// What:
///  A clean in a mirror removes the workspace's own artifacts from the directories it is given,
///  and the workspace's `Cargo.lock` keeps its bytes, its inode and its file time.
/// Why:
///   This is the whole of the design:
/// the repair needs Cargo to resolve and to delete the members' own artifacts,
/// and it gets both from a mirror of the workspace,
/// measured on this host on 2026-11-14 to delete exactly what the workspace's own clean deletes
/// (`experiment-mirror-shape`).
/// A person's lock file is never opened for writing, never renamed and never exchanged,
/// so no step can lose or mislabel their bytes and no kill leaves them anywhere else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const run = await cleanInAMirror({ ...belongings, locked: true }); expect(run.end).toBe('succeeded'); expect(readFileSync(lock)).toEqual(persons); expect(artifactsNamed('own')).toBe(0)
/// ```
#[test]
fn a_clean_in_a_mirror_removes_the_artifacts_and_leaves_the_persons_lock_alone() {
    let tool: PathBuf = cargo_on_the_search_path();
    if !tool.is_file() {
        panic!("no cargo on the search path, so this control cannot show what it exists to show");
    }
    let root: PathBuf = fixture("clean-in-a-mirror");
    let worktree: PathBuf = root.join("worktree");
    let workspace: PathBuf = worktree.join("package/own");
    std::fs::create_dir_all(workspace.join("src")).expect("the workspace");
    std::fs::write(workspace.join("Cargo.toml"), b"[workspace]\nmembers = [\".\"]\n\n[package]\nname = \"own\"\nversion = \"0.1.0\"\nedition = \"2021\"\n").expect("the manifest");
    std::fs::write(workspace.join("src/lib.rs"), b"pub fn own() {}\n").expect("the source");
    let build: PathBuf = root.join("build");
    let target: PathBuf = root.join("target");
    // A build of the workspace into the directories the clean will be given, so there is state to delete.
    let built: std::process::Output = std::process::Command::new(tool.as_path())
        .args(["build", "--offline", "--quiet"])
        .current_dir(workspace.as_path())
        .env("CARGO_BUILD_BUILD_DIR", build.as_os_str())
        .env("CARGO_TARGET_DIR", target.as_os_str())
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .expect("cargo runs");
    assert!(built.status.success(), "the workspace builds: {}", String::from_utf8_lossy(built.stderr.as_slice()));
    // The person's own lock, which no clean accepts: the mirror is where the resolution is made.
    let persons: &[u8] = b"# the person's own lock, which the resolution would have replaced\n";
    let lock: PathBuf = workspace.join(crate::package_repair_cargo_lock::LOCK_FILE);
    std::fs::write(lock.as_path(), persons).expect("the person's lock");
    let before_time: std::time::SystemTime = std::fs::metadata(lock.as_path()).expect("the lock").modified().expect("a file time");
    assert!(!entries_named(build.as_path(), "own").is_empty(), "the build left the member's artifacts to delete");
    let git_dir: PathBuf = root.join("git-dir");
    std::fs::create_dir_all(git_dir.as_path()).expect("the Git directory");
    let run: ToolRun = crate::lazy_runtime::run_on_runtime(clean_in_a_mirror(MirrorClean {
        tool: CargoTool { program: tool, limits: ToolLimits { window: std::time::Duration::from_secs(120), ceiling: std::time::Duration::from_secs(300) } },
        worktree: worktree.clone(),
        workspace: workspace.clone(),
        members: vec![String::from("own")],
        build_directory: build.clone(),
        target_directory: target.clone(),
        locked: true,
        kept_in: Some(git_dir.clone()),
        job: String::from("a-controls-job"),
    }))
    .expect("the runtime")
    .expect("the clean ran in the mirror");
    assert_eq!(run.end, ToolEnd::Succeeded, "the clean ran in the mirror");
    assert!(entries_named(build.as_path(), "own").is_empty(), "the members' own artifacts are gone from the build directory: {:?}", entries_named(build.as_path(), "own"));
    assert_eq!(std::fs::read(lock.as_path()).expect("the person's lock"), persons, "the person's lock keeps its bytes");
    assert_eq!(std::fs::metadata(lock.as_path()).expect("the lock").modified().expect("a file time"), before_time, "and its file time: nothing wrote it");
    let displaced: PathBuf = git_dir.join("monochromatic-git-policy-cli-displaced-locks");
    assert!(!displaced.exists(), "and no copy of any lock is left beside it: {}", displaced.display());
    // The mirror itself goes when the step ends.
    let mirrors: PathBuf = git_dir.join("monochromatic-git-policy-cli-lock-resolutions");
    let mut left: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![mirrors];
    while let Some(directory) = pending.pop() {
        let Ok(listing) = std::fs::read_dir(directory.as_path()) else { continue };
        for found in listing.flatten() {
            let path: PathBuf = found.path();
            if path.is_dir() { pending.push(path); } else { left.push(path); }
        }
    }
    assert!(left.is_empty(), "no file of the mirror is left behind: {left:?}");
    remove(root.as_path());
}

/// What:
///  The mirror writes a lock of its own,
///  so a workspace whose lock a `--locked` clean refuses still cleans.
/// Why:
///   The resolution is the write the whole design moved:
/// it happens in the mirror, and this control shows the mirror's lock is a file of its own by
/// writing to it from outside (a stand-in Cargo) and reading the person's file afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standIn = cargoThatWrites('.mirror-was-cleaned'); await cleanInAMirror({ ...belongings, tool: standIn }); expect(readFileSync(lock)).toEqual(persons)
/// ```
#[test]
fn the_lock_the_mirror_writes_is_not_the_persons_file() {
    let root: PathBuf = fixture("clean-in-a-mirror-writes-its-own-lock");
    let worktree: PathBuf = root.join("worktree");
    let workspace: PathBuf = worktree.join("package/own");
    std::fs::create_dir_all(workspace.join("src")).expect("the workspace");
    std::fs::write(workspace.join("Cargo.toml"), b"[package]\nname = \"own\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n").expect("the manifest");
    std::fs::write(workspace.join("src/lib.rs"), b"pub fn own() {}\n").expect("the source");
    let persons: &[u8] = b"# the person's lock\n";
    let lock: PathBuf = workspace.join(crate::package_repair_cargo_lock::LOCK_FILE);
    std::fs::write(lock.as_path(), persons).expect("the person's lock");
    let git_dir: PathBuf = root.join("git-dir");
    std::fs::create_dir_all(git_dir.as_path()).expect("the Git directory");
    // A stand-in Cargo that writes its lock where it is run, which is the mirror, and says it cleaned.
    let marker: PathBuf = root.join("the-mirror-was-cleaned");
    let stand_in: PathBuf = root.join("cargo-that-writes-its-lock");
    crate::test_support::executable(
        stand_in.as_path(),
        format!(
            "#!/bin/sh\nif [ \"$1\" = metadata ]; then\n  printf '%s' '# the resolution wrote this in the mirror\n' > Cargo.lock\n  exit 0\nfi\nif [ \"$1\" = clean ]; then\n  printf '%s' \"$(pwd -P)\" > '{marker}'\n  exit 0\nfi\nexit 0\n",
            marker = marker.display()
        )
        .as_bytes(),
    );
    let run: ToolRun = crate::lazy_runtime::run_on_runtime(clean_in_a_mirror(MirrorClean {
        tool: CargoTool { program: stand_in, limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: std::time::Duration::from_secs(10) } },
        worktree: worktree.clone(),
        workspace: workspace.clone(),
        members: vec![String::from("own")],
        build_directory: root.join("build"),
        target_directory: root.join("target"),
        locked: true,
        kept_in: Some(git_dir.clone()),
        job: String::from("a-controls-job"),
    }))
    .expect("the runtime")
    .expect("the stand-in clean ran");
    assert_eq!(run.end, ToolEnd::Succeeded, "the stand-in clean ran");
    assert_eq!(std::fs::read(lock.as_path()).expect("the person's lock"), persons, "the person's lock is exactly as they wrote it");
    let cleaned_in: String = std::fs::read_to_string(marker.as_path()).expect("the clean recorded where it ran");
    assert!(cleaned_in.contains("lock-resolutions"), "the clean ran in the mirror and not in the worktree: {cleaned_in}");
    assert!(!cleaned_in.contains(worktree.to_str().expect("text")), "and never in the worktree: {cleaned_in}");
    remove(root.as_path());
}

/// Every path under `directory` whose name holds `needle`.
fn entries_named(directory: &Path, needle: &str) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![directory.to_path_buf()];
    while let Some(at) = pending.pop() {
        let Ok(listing) = std::fs::read_dir(at.as_path()) else { continue };
        for entry in listing.flatten() {
            let path: PathBuf = entry.path();
            if entry.file_name().to_string_lossy().contains(needle) {
                found.push(path.clone());
            }
            if entry.file_type().is_ok_and(|kind| return kind.is_dir()) {
                pending.push(path);
            }
        }
    }
    found.sort();
    return found;
}

/// The Cargo the fixtures build with,
/// which is the workspace's own link first and the search path after it.
fn cargo_on_the_search_path() -> PathBuf {
    if let Some(found) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(found.as_os_str()) {
            let candidate: PathBuf = directory.join("cargo");
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    return PathBuf::from("/nonexistent/cargo");
}
