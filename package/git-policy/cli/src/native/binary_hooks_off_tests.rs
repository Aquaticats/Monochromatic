//! Binary-level controls for traditional hooks never running through the wrapper, and for what `git cli-git doctor`
//!  says about the shims `git lfs` writes.
//! A hook file in the hooks directory writes a marker when it runs. Forwarded commands, the commit path, the
//!  auto-push and the push-race integration start the real Git with `-c core.hooksPath=` pointing at a directory that
//!  never exists, so no marker appears; hooks defined in the configuration still run, and a `-c core.hooksPath` the
//!  caller writes brings a directory back. Real Git started directly is the positive control for every marker.

use super::commit::{remove_settled, run_commit, wrapped_commit};
use super::support::{Fixture, Observed, direct, executable, fixture, git, observe, repository, stderr_of, wrapper_first_path};
use git_policy_cli::doctor::lfs_shim_text;
use std::ffi::OsStr;
use std::path::PathBuf;

/// A repository with a bare origin, hook files that write markers, and a stand-in `git-lfs` that records its runs.
struct World {
    /// The working repository, on `main`.
    repo: PathBuf,
    /// The bare origin.
    origin: PathBuf,
    /// The markers the hook files write.
    markers: PathBuf,
    /// The runs of the stand-in `git-lfs`.
    lfs_runs: PathBuf,
}

/// Write an executable hook file `name` into the repository's hooks directory that appends `name` to the markers
///  and exits with `status`.
fn marker_hook(world: &World, name: &str, status: i32) {
    executable(
        world.repo.join(".git/hooks").join(name).as_path(),
        format!("#!/bin/sh\necho {name} >> '{}'\nexit {status}\n", world.markers.display()).as_bytes(),
    );
}

/// The hooks that ran, one name per line.
fn markers(world: &World) -> String {
    return std::fs::read_to_string(&world.markers).unwrap_or_default();
}

/// How many times the stand-in `git-lfs` ran.
fn lfs_runs(world: &World) -> usize {
    return std::fs::read_to_string(&world.lfs_runs).unwrap_or_default().lines().count();
}

/// Create the world; `lfs` adds a Git LFS key to the repository configuration.
fn world(fixture: &Fixture, lfs: bool) -> World {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    let origin: PathBuf = fixture.root.join("origin.git");
    git(fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "origin.git"]);
    git(fixture, repo.as_path(), &["remote", "add", "origin", origin.to_str().expect("UTF-8")]);
    git(fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    if lfs {
        git(fixture, repo.as_path(), &["config", "lfs.url", "http://127.0.0.1:9/never-contacted"]);
    }
    let lfs_runs: PathBuf = fixture.root.join("lfs-runs.txt");
    executable(
        fixture.root.join("bin/git-lfs").as_path(),
        format!("#!/bin/sh\ncat > /dev/null\necho \"$*\" >> '{}'\nexit 0\n", lfs_runs.display()).as_bytes(),
    );
    return World { repo, origin, markers: fixture.root.join("markers.txt"), lfs_runs };
}

/// Run a wrapped command in the repository.
fn wrapped(fixture: &Fixture, world: &World, arguments: &[&str]) -> Observed {
    return observe(&mut wrapped_commit(fixture, world.repo.as_path(), arguments), b"");
}

/// A hook file does not run for a forwarded command, and runs for real Git started directly.
#[test]
fn a_forwarded_command_does_not_run_a_hook_file() {
    let fixture: Fixture = fixture("hooks-off-forwarded");
    let world: World = world(&fixture, false);
    marker_hook(&world, "post-checkout", 0);
    let switched: Observed = wrapped(&fixture, &world, &["checkout", "--quiet", "main"]);
    assert_eq!(switched.code, Some(0), "{}", stderr_of(&switched));
    assert_eq!(markers(&world), "", "the hook file must not run through the wrapper");
    // Positive control: the same command straight to real Git runs the hook file.
    let control = direct(&fixture).current_dir(&world.repo).args(["checkout", "--quiet", "main"]).output().expect("git starts");
    assert!(control.status.success());
    assert_eq!(markers(&world), "post-checkout\n", "real Git started directly runs the hook file");
    remove_settled(&fixture);
}

/// A hook defined in the configuration still runs through the wrapper.
#[test]
fn a_configured_hook_still_runs_for_a_forwarded_command() {
    let fixture: Fixture = fixture("hooks-off-configured");
    let world: World = world(&fixture, false);
    marker_hook(&world, "post-checkout", 1);
    let script: PathBuf = fixture.root.join("configured-hook");
    executable(script.as_path(), format!("#!/bin/sh\necho configured >> '{}'\n", world.markers.display()).as_bytes());
    git(&fixture, world.repo.as_path(), &["config", "hook.control.event", "post-checkout"]);
    git(&fixture, world.repo.as_path(), &["config", "hook.control.command", script.to_str().expect("UTF-8")]);
    let switched: Observed = wrapped(&fixture, &world, &["checkout", "--quiet", "main"]);
    assert_eq!(switched.code, Some(0), "{}", stderr_of(&switched));
    assert_eq!(markers(&world), "configured\n");
    remove_settled(&fixture);
}

/// A `-c core.hooksPath` the caller writes comes after the wrapper's option and wins.
#[test]
fn a_hooks_path_of_the_caller_brings_the_hook_file_back() {
    let fixture: Fixture = fixture("hooks-off-caller");
    let world: World = world(&fixture, false);
    marker_hook(&world, "post-checkout", 0);
    let hooks: String = world.repo.join(".git/hooks").display().to_string();
    let option: String = format!("core.hooksPath={hooks}");
    let switched: Observed = wrapped(&fixture, &world, &["-c", option.as_str(), "checkout", "--quiet", "main"]);
    assert_eq!(switched.code, Some(0), "{}", stderr_of(&switched));
    assert_eq!(markers(&world), "post-checkout\n");
    remove_settled(&fixture);
}

/// The repository's own `core.hooksPath` directory does not run either, and a configuration query still reports it.
#[test]
fn a_configured_hooks_directory_is_not_used_and_is_still_reported() {
    let fixture: Fixture = fixture("hooks-off-configured-directory");
    let world: World = world(&fixture, false);
    let hooks: PathBuf = fixture.root.join("other-hooks");
    std::fs::create_dir(&hooks).expect("hooks");
    executable(hooks.join("post-checkout").as_path(), format!("#!/bin/sh\necho other >> '{}'\n", world.markers.display()).as_bytes());
    git(&fixture, world.repo.as_path(), &["config", "core.hooksPath", hooks.to_str().expect("UTF-8")]);
    let switched: Observed = wrapped(&fixture, &world, &["checkout", "--quiet", "main"]);
    assert_eq!(switched.code, Some(0), "{}", stderr_of(&switched));
    assert_eq!(markers(&world), "");
    let reported: Observed = wrapped(&fixture, &world, &["config", "--get", "core.hooksPath"]);
    assert_eq!(String::from_utf8_lossy(&reported.stdout).trim(), hooks.to_str().expect("UTF-8"));
    let listed: Observed = wrapped(&fixture, &world, &["config", "--list"]);
    assert!(!String::from_utf8_lossy(&listed.stdout).contains("cli-git-no-hooks"));
    remove_settled(&fixture);
}

/// A failing `pre-push` hook file blocks neither a typed push nor the auto-push after a commit.
#[test]
fn a_failing_pre_push_hook_file_blocks_neither_a_push_nor_an_auto_push() {
    let fixture: Fixture = fixture("hooks-off-pre-push");
    let world: World = world(&fixture, false);
    marker_hook(&world, "pre-push", 1);
    let pushed: Observed = wrapped(&fixture, &world, &["push", "origin", "main"]);
    assert_eq!(pushed.code, Some(0), "{}", stderr_of(&pushed));
    std::fs::write(world.repo.join("a.txt"), b"one\n").expect("a.txt");
    git(&fixture, world.repo.as_path(), &["add", "--", "a.txt"]);
    let landed: Observed = run_commit(&fixture, world.repo.as_path(), &["commit", "--quiet", "--message=one", "--", "a.txt"]);
    let stderr: String = stderr_of(&landed);
    assert_eq!(landed.code, Some(0), "{stderr}");
    assert!(!stderr.contains("auto-push to origin failed"), "{stderr}");
    assert_eq!(markers(&world), "", "no hook file ran");
    let head = direct(&fixture).current_dir(&world.repo).args(["rev-parse", "HEAD"]).output().expect("git starts");
    let published = direct(&fixture).current_dir(&world.origin).args(["rev-parse", "refs/heads/main"]).output().expect("git starts");
    assert_eq!(head.stdout, published.stdout, "the auto-push published the commit");
    remove_settled(&fixture);
}

/// A local-path remote runs its own server-side hooks: the setting does not follow the push into the remote.
#[test]
fn a_local_remotes_server_side_hook_still_runs() {
    let fixture: Fixture = fixture("hooks-off-remote-hook");
    let world: World = world(&fixture, false);
    executable(
        world.origin.join("hooks/post-receive").as_path(),
        format!("#!/bin/sh\necho post-receive >> '{}'\n", world.markers.display()).as_bytes(),
    );
    let pushed: Observed = wrapped(&fixture, &world, &["push", "origin", "main"]);
    assert_eq!(pushed.code, Some(0), "{}", stderr_of(&pushed));
    assert_eq!(markers(&world), "post-receive\n");
    remove_settled(&fixture);
}

/// A regenerated `pre-push` shim does not upload a second time: only the wrapper's own step runs `git-lfs`.
#[test]
fn a_regenerated_pre_push_shim_does_not_double_the_upload() {
    let fixture: Fixture = fixture("hooks-off-shim-double");
    let world: World = world(&fixture, true);
    executable(world.repo.join(".git/hooks/pre-push").as_path(), lfs_shim_text("pre-push").as_bytes());
    let pushed: Observed = wrapped(&fixture, &world, &["push", "origin", "main"]);
    assert_eq!(pushed.code, Some(0), "{}", stderr_of(&pushed));
    assert_eq!(lfs_runs(&world), 1, "one upload step, not two");
    // Positive control: real Git started directly (with the wrapper first on PATH for the shim's own `git`) runs the
    // shim, and so uploads through it.
    commit_directly(&fixture, &world, "b.txt");
    let control = direct(&fixture)
        .current_dir(&world.repo)
        .env("PATH", wrapper_first_path(&fixture))
        .args(["push", "origin", "main"])
        .output()
        .expect("git starts");
    assert!(control.status.success(), "{}", String::from_utf8_lossy(&control.stderr));
    assert_eq!(lfs_runs(&world), 2, "the shim ran when real Git was started directly");
    remove_settled(&fixture);
}

/// Commit a file with real Git, so no auto-push runs.
fn commit_directly(fixture: &Fixture, world: &World, name: &str) {
    std::fs::write(world.repo.join(name), b"x\n").expect("fixture file");
    git(fixture, world.repo.as_path(), &["add", "--", name]);
    git(fixture, world.repo.as_path(), &["commit", "--quiet", &format!("--message={name}"), "--", name]);
}

/// Run `git cli-git doctor` through the wrapper.
fn doctor(fixture: &Fixture, world: &World) -> Observed {
    return wrapped(fixture, world, &["cli-git", "doctor"]);
}

/// Only the shims `git lfs` writes: status 0, nothing on standard output, the explanation on standard error.
#[test]
fn doctor_explains_the_git_lfs_shims_and_exits_zero() {
    let fixture: Fixture = fixture("hooks-off-doctor-shims");
    let world: World = world(&fixture, true);
    for name in ["pre-push", "post-checkout", "post-commit", "post-merge"] {
        executable(world.repo.join(".git/hooks").join(name).as_path(), lfs_shim_text(name).as_bytes());
    }
    let report: Observed = doctor(&fixture, &world);
    let stderr: String = stderr_of(&report);
    assert_eq!(report.code, Some(0), "{stderr}");
    assert!(report.stdout.is_empty(), "{}", String::from_utf8_lossy(&report.stdout));
    assert!(stderr.contains("the Git LFS shims, byte for byte, and nothing needs doing"), "{stderr}");
    assert!(stderr.contains("this is normal"), "{stderr}");
    assert!(stderr.contains(world.repo.join(".git/hooks/pre-push").to_str().expect("UTF-8")), "{stderr}");
    remove_settled(&fixture);
}

/// Any other hook beside the shims keeps status 1 and the unsupported report.
#[test]
fn doctor_keeps_reporting_other_hooks_beside_the_shims() {
    let fixture: Fixture = fixture("hooks-off-doctor-foreign");
    let world: World = world(&fixture, true);
    executable(world.repo.join(".git/hooks/pre-push").as_path(), lfs_shim_text("pre-push").as_bytes());
    marker_hook(&world, "pre-commit", 0);
    let report: Observed = doctor(&fixture, &world);
    assert_eq!(report.code, Some(1), "{}", stderr_of(&report));
    assert_eq!(
        String::from_utf8_lossy(&report.stdout),
        format!("{}\n", world.repo.join(".git/hooks/pre-commit").display())
    );
    assert!(stderr_of(&report).contains("unsupported"), "{}", stderr_of(&report));
    remove_settled(&fixture);
}

/// Outside a repository the command still fails with status 2.
#[test]
fn doctor_outside_a_repository_still_exits_two() {
    let fixture: Fixture = fixture("hooks-off-doctor-outside");
    let _world: World = world(&fixture, false);
    let report: Observed = observe(&mut wrapped_commit(&fixture, fixture.root.as_path(), &["cli-git", "doctor"]), b"");
    assert_eq!(report.code, Some(2), "{}", stderr_of(&report));
    remove_settled(&fixture);
}

/// The text a person sees is the text git-lfs itself wrote: this compares the recognized bytes with a real file.
#[test]
fn the_recognized_shim_bytes_are_the_ones_the_real_git_lfs_writes() {
    let Some(program) = std::env::var_os("PATH").and_then(|path| {
        return std::env::split_paths(&path).map(|directory| { return directory.join("git-lfs"); }).find(|candidate| { return candidate.is_file(); });
    }) else {
        eprintln!("skipped: git-lfs is not on PATH, so its shim bytes were not compared");
        return;
    };
    let fixture: Fixture = fixture("hooks-off-real-shim");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::os::unix::fs::symlink(program, fixture.root.join("bin/git-lfs")).expect("git-lfs link");
    for (key, value) in [
        ("filter.lfs.clean", "git-lfs clean -- %f"),
        ("filter.lfs.smudge", "git-lfs smudge -- %f"),
        ("filter.lfs.process", "git-lfs filter-process"),
        ("filter.lfs.required", "true"),
    ] {
        git(&fixture, repo.as_path(), &[ "config", key, value]);
    }
    std::fs::write(repo.join(".gitattributes"), b"*.bin filter=lfs diff=lfs merge=lfs -text\n").expect("attributes");
    std::fs::write(repo.join("x.bin"), b"hello\n").expect("x.bin");
    let hooks: PathBuf = repo.join(".git/hooks");
    let written = |name: &str| -> Option<Vec<u8>> { return std::fs::read(hooks.join(name)).ok(); };
    // Through the wrapper the clean filter inherits the hooks-off option, so git-lfs finds no hooks directory it can
    // write to and leaves the repository without shims.
    let added: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), &["add", "--", "x.bin"]), b"");
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    for name in ["pre-push", "post-checkout", "post-commit", "post-merge"] {
        assert_eq!(written(name), None, "{name} must not be written through the wrapper");
    }
    // Positive control: the same add by real Git started directly writes the four shims, with the recognized bytes.
    std::fs::write(repo.join("y.bin"), b"world\n").expect("y.bin");
    let control = direct(&fixture)
        .current_dir(&repo)
        .env("PATH", wrapper_first_path(&fixture))
        .args(["add", "--", "y.bin"])
        .output()
        .expect("git starts");
    assert!(control.status.success(), "{}", String::from_utf8_lossy(&control.stderr));
    for name in ["pre-push", "post-checkout", "post-commit", "post-merge"] {
        assert_eq!(written(name), Some(lfs_shim_text(name).into_bytes()), "{name}");
    }
    remove_settled(&fixture);
}
