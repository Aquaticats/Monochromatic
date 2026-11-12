//! Binary-level controls for the Git LFS upload before a push.
//! The built executable runs `git-lfs push <remote> <refs>` as a child of its own before every push it starts,
//! in a
//!  repository whose configuration holds LFS keys:
//! before any ref moves,
//! with a failure stopping the push before it
//!  starts.
//! The push itself carries no hook.
//! A stand-in `git-lfs` on the fixture's PATH records what it was given and what the origin's `main` was at that
//!  moment;
//! the real `git-lfs` against an LFS server written inline is `binary_lfs_server_tests.rs`.

use super::commit::{git_text, keep_maintenance_in_the_foreground, remove_settled, run_commit, wrapped_commit};
use super::support::{Fixture, Observed, REAL_GIT, decline_in_process_lfs_scan, direct, executable, fixture, git, observe, repository, stderr_of};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// A repository,
/// its bare origin,
/// and the file the stand-in program appends to.
pub(super) struct World {
    /// The working repository,
    /// on `main`.
    pub(super) repo: PathBuf,
    /// The bare origin.
    pub(super) origin: PathBuf,
    /// The record of the stand-in `git-lfs`.
    pub(super) record: PathBuf,
}

/// One run of the stand-in program.
#[derive(Debug)]
pub(super) struct Run {
    /// Its arguments,
    /// joined by spaces.
    arguments: String,
    /// Its standard input lines.
    input: Vec<String>,
    /// The origin's `refs/heads/main` when the program ran,
    /// or `absent`.
    origin_main: String,
}

/// The stand-in `git-lfs`:
/// record the arguments,
/// the input and the origin's `main`,
/// then exit with `status` from the
///  run numbered `from_run` (counting from zero) on,
/// and with `0` before it.
fn stand_in_source(world_origin: &Path, record: &Path, status: i32, from_run: usize) -> String {
    return format!(
        "#!/bin/sh\n\
         count=$(grep -c '^run ' '{record}' 2>/dev/null || true)\n\
         {{\n\
         printf 'run arguments=%s\\n' \"$*\"\n\
         sed 's/^/input /'\n\
         printf 'origin-main=%s\\n' \"$({REAL_GIT} --git-dir='{origin}' rev-parse --verify --quiet refs/heads/main || echo absent)\"\n\
         }} >> '{record}'\n\
         if [ \"${{count:-0}}\" -ge {from_run} ]; then exit {status}; fi\n\
         exit 0\n",
        origin = world_origin.display(),
        record = record.display(),
    );
}

/// Create the repository and its origin,
/// and a stand-in program that fails every run.
pub(super) fn world(fixture: &Fixture, lfs: bool, program: Option<i32>) -> World {
    return world_failing_from(fixture, lfs, program, 0);
}

/// Create the repository and its origin.
///  `lfs` adds a Git LFS key to the repository configuration;
/// `program` installs the stand-in with that exit status,
///  which it returns from run number `from_run` on.
fn world_failing_from(fixture: &Fixture, lfs: bool, program: Option<i32>, from_run: usize) -> World {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    let origin: PathBuf = fixture.root.join("origin.git");
    git(fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "origin.git"]);
    git(fixture, repo.as_path(), &["remote", "add", "origin", origin.to_str().expect("UTF-8")]);
    if lfs {
        git(fixture, repo.as_path(), &["config", "lfs.url", "http://127.0.0.1:9/never-contacted"]);
    }
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    let record: PathBuf = fixture.root.join("lfs-record.txt");
    if lfs {
        decline_in_process_lfs_scan(fixture, repo.as_path());
    }
    if let Some(status) = program {
        executable(
            fixture.root.join("bin/git-lfs").as_path(),
            stand_in_source(origin.as_path(), record.as_path(), status, from_run).as_bytes(),
        );
    }
    return World { repo, origin, record };
}

/// Every run the stand-in recorded,
/// in order.
pub(super) fn runs(world: &World) -> Vec<Run> {
    let text: String = std::fs::read_to_string(&world.record).unwrap_or_default();
    let mut found: Vec<Run> = Vec::new();
    for line in text.lines() {
        if let Some(arguments) = line.strip_prefix("run arguments=") {
            found.push(Run { arguments: arguments.to_owned(), input: Vec::new(), origin_main: String::new() });
        } else if let Some(input) = line.strip_prefix("input ") {
            found.last_mut().expect("a run precedes its input").input.push(input.to_owned());
        } else if let Some(main) = line.strip_prefix("origin-main=") {
            found.last_mut().expect("a run precedes its record").origin_main = main.to_owned();
        }
    }
    return found;
}

/// The object a reference names in a repository,
/// or `absent`.
pub(super) fn ref_of(fixture: &Fixture, repo: &Path, name: &str) -> String {
    let output = direct(fixture)
        .current_dir(repo)
        .args(["rev-parse", "--verify", "--quiet", name])
        .output()
        .expect("rev-parse starts");
    let text: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    return if text.is_empty() { String::from("absent") } else { text };
}

/// Commit a file with real Git,
/// so no auto-push runs.
pub(super) fn commit_directly(fixture: &Fixture, repo: &Path, name: &str, text: &str) {
    std::fs::write(repo.join(name), text).expect("fixture file");
    git(fixture, repo, &["add", "--", name]);
    git(fixture, repo, &["commit", "--quiet", &format!("--message={name}"), "--", name]);
}

/// Run one wrapped command in the repository.
pub(super) fn wrapped(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> Observed {
    return observe(&mut wrapped_commit(fixture, directory, arguments), b"");
}

/// A plain push uploads once,
/// naming its remote and refs,
/// before the origin's `main` exists.
#[test]
fn a_push_runs_the_upload_before_any_ref_moves() {
    let fixture: Fixture = fixture("lfs-push");
    let world: World = world(&fixture, true, Some(0));
    let pushed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "origin", "main"]);
    assert_eq!(pushed.code, Some(0), "{}", stderr_of(&pushed));
    let head: String = ref_of(&fixture, world.repo.as_path(), "refs/heads/main");
    let recorded: Vec<Run> = runs(&world);
    assert_eq!(recorded.len(), 1, "one upload for one push (the scope probe must not upload): {recorded:?}");
    assert_eq!(recorded[0].arguments, "push origin main", "the upload names the remote and the refs the push names");
    assert!(recorded[0].input.is_empty(), "git lfs push reads no update lines: {recorded:?}");
    assert_eq!(recorded[0].origin_main, "absent", "the upload runs before the ref moves");
    assert_eq!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), head);
    remove_settled(&fixture);
}

/// A failing upload stops the push before anything is published,
/// and the message names the cause and the ways on.
#[test]
fn a_failing_upload_blocks_the_push_and_names_every_way_forward() {
    let fixture: Fixture = fixture("lfs-push-fails");
    let world: World = world(&fixture, true, Some(3));
    let pushed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "origin", "main"]);
    let stderr: String = stderr_of(&pushed);
    assert_ne!(pushed.code, Some(0), "{stderr}");
    assert_eq!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent", "{stderr}");
    assert!(stderr.contains("`git-lfs push` exited with status 3"), "{stderr}");
    assert!(stderr.contains("nothing was published"), "{stderr}");
    assert!(stderr.contains("git push --no-verify"), "{stderr}");
    assert_eq!(runs(&world).len(), 1);
    remove_settled(&fixture);
}

/// A push that cannot take the stacked route (a forced push) carries the upload too.
#[test]
fn a_forced_push_runs_the_upload_before_any_ref_moves() {
    let fixture: Fixture = fixture("lfs-push-forced");
    let world: World = world(&fixture, true, Some(0));
    let pushed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "--force", "origin", "main"]);
    assert_eq!(pushed.code, Some(0), "{}", stderr_of(&pushed));
    let recorded: Vec<Run> = runs(&world);
    assert_eq!(recorded.len(), 1, "{recorded:?}");
    assert_eq!(recorded[0].origin_main, "absent");
    remove_settled(&fixture);
}

/// A failing upload blocks a forced push as well.
#[test]
fn a_failing_upload_blocks_a_forced_push() {
    let fixture: Fixture = fixture("lfs-push-forced-fails");
    let world: World = world(&fixture, true, Some(5));
    let pushed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "--force", "origin", "main"]);
    assert_ne!(pushed.code, Some(0), "{}", stderr_of(&pushed));
    assert_eq!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent");
    remove_settled(&fixture);
}

/// A dry run publishes nothing,
/// so it uploads nothing.
#[test]
fn a_dry_run_push_uploads_nothing() {
    let fixture: Fixture = fixture("lfs-push-dry-run");
    let world: World = world(&fixture, true, Some(0));
    let pushed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "--dry-run", "origin", "main"]);
    assert_eq!(pushed.code, Some(0), "{}", stderr_of(&pushed));
    assert!(runs(&world).is_empty(), "{:?}", runs(&world));
    remove_settled(&fixture);
}

/// Git's own `--no-verify` skips the upload,
/// as it skipped the shim.
#[test]
fn no_verify_skips_the_upload() {
    let fixture: Fixture = fixture("lfs-push-no-verify");
    let world: World = world(&fixture, true, Some(9));
    let pushed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "--no-verify", "origin", "main"]);
    assert_eq!(pushed.code, Some(0), "{}", stderr_of(&pushed));
    assert!(runs(&world).is_empty());
    assert_ne!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent");
    remove_settled(&fixture);
}

/// The caller's switch `-c hook.cli-git-lfs-pre-push.enabled=false` skips the upload for one push.
#[test]
fn the_caller_can_switch_the_upload_off_for_one_push() {
    let fixture: Fixture = fixture("lfs-push-disabled");
    let world: World = world(&fixture, true, Some(9));
    let pushed: Observed = wrapped(
        &fixture,
        world.repo.as_path(),
        &["-c", "hook.cli-git-lfs-pre-push.enabled=false", "push", "origin", "main"],
    );
    assert_eq!(pushed.code, Some(0), "{}", stderr_of(&pushed));
    assert!(runs(&world).is_empty(), "{:?}", runs(&world));
    assert_ne!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent");
    remove_settled(&fixture);
}

/// What:
///  The caller's switch given through the environment disables the upload for one push,
/// exactly as the `-c` spelling does.
/// Why:
///   Both spellings set the same Git configuration,
/// and a wrapper that read only `-c` ran the upload the caller had
///  disabled:
/// a failing upload then stopped a push real Git would have published.
#[test]
fn the_caller_can_switch_the_upload_off_through_a_configuration_the_environment_names() {
    let fixture: Fixture = fixture("lfs-push-disabled-config-env");
    let world: World = world(&fixture, true, Some(9));
    let mut command = wrapped_commit(
        &fixture,
        world.repo.as_path(),
        &["--config-env=hook.cli-git-lfs-pre-push.enabled=SWITCH_VALUE", "push", "origin", "main"],
    );
    command.env("SWITCH_VALUE", "false");
    let pushed: Observed = observe(&mut command, b"");
    assert_eq!(pushed.code, Some(0), "{}", stderr_of(&pushed));
    assert!(runs(&world).is_empty(), "{:?}", runs(&world));
    assert_ne!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent");
    remove_settled(&fixture);
}

/// What:
///  A newer command-line setting wins over the same key in an inherited `GIT_CONFIG_PARAMETERS`,
/// in both spellings,
/// as it does in real Git.
/// Why:
///   The upload child inherits the parameters and is given the caller's pairs beside them;
/// Git reads the parameters last,
/// so an
///  inherited entry decided which server the object went to while the ref went to the one the caller selected.
#[test]
fn an_inherited_command_scope_does_not_override_the_callers_newer_setting() {
    const NEW_URL: &str = "https://new.invalid/lfs";
    const OLD_URL: &str = "https://old.invalid/lfs";
    for (case, selection, variable) in [
        ("dash-c", vec![String::from("-c"), format!("lfs.url={NEW_URL}")], None),
        ("config-env", vec![String::from("--config-env=lfs.url=PUSH_URL")], Some(("PUSH_URL", NEW_URL))),
    ] {
        let fixture: Fixture = fixture(format!("lfs-push-inherited-{case}").as_str());
        let world: World = world(&fixture, true, None);
        executable(fixture.root.join("bin/git-lfs").as_path(), url_recording_source(world.record.as_path()).as_bytes());
        let mut arguments: Vec<&str> = selection.iter().map(String::as_str).collect();
        arguments.extend(["push", "origin", "main"]);
        let mut command = wrapped_commit(&fixture, world.repo.as_path(), arguments.as_slice());
        command.env("GIT_CONFIG_PARAMETERS", format!("'lfs.url'='{OLD_URL}'"));
        if let Some((name, value)) = variable {
            command.env(name, value);
        }
        let pushed: Observed = observe(&mut command, b"");
        let stderr: String = stderr_of(&pushed);
        assert_eq!(pushed.code, Some(0), "{case}: {stderr}");
        let recorded: String = std::fs::read_to_string(world.record.as_path()).unwrap_or_default();
        assert!(recorded.contains(&format!("lfs-url={NEW_URL}")), "{case}: the upload child reads the caller's newer setting: {recorded}");
        assert_ne!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent", "{case}: {stderr}");
        remove_settled(&fixture);
    }
}

/// A stand-in `git-lfs` that records its arguments and the `lfs.url` real Git answers for the environment it was given,
/// then succeeds.
/// Why:
///  The setting is what decides which server the object travels to,
/// and the child's own answer is the only evidence of it.
fn url_recording_source(record: &Path) -> String {
    return format!(
        "#!/bin/sh\n{{ printf 'arguments=%s\\n' \"$*\"; printf 'lfs-url=%s\\n' \"$({REAL_GIT} config --get lfs.url)\"; }} >> '{record}'\nexit 0\n",
        record = record.display()
    );
}

/// `git lfs install` in an account writes the filter to the global file;
/// that alone must not turn every repository
///  into an LFS repository,
/// or a push would need `git-lfs` everywhere.
#[test]
fn a_global_lfs_filter_alone_runs_no_upload() {
    let fixture: Fixture = fixture("lfs-push-global-only");
    let world: World = world(&fixture, false, None);
    let global: PathBuf = fixture.root.join("global-gitconfig");
    std::fs::write(&global, "[filter \"lfs\"]\n\tprocess = git-lfs filter-process\n\trequired = true\n").expect("global file");
    let mut command = wrapped_commit(&fixture, world.repo.as_path(), &["push", "origin", "main"]);
    command.env("GIT_CONFIG_GLOBAL", &global);
    let pushed: Observed = observe(&mut command, b"");
    assert_eq!(pushed.code, Some(0), "no git-lfs is installed here, so any upload attempt would stop the push: {}", stderr_of(&pushed));
    assert_ne!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent");
    remove_settled(&fixture);
}

/// A repository without LFS keys pushes as before:
/// no upload step,
/// and no need for `git-lfs` at all.
#[test]
fn a_repository_without_lfs_keys_runs_no_upload() {
    let fixture: Fixture = fixture("lfs-push-none");
    let world: World = world(&fixture, false, Some(9));
    let pushed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "origin", "main"]);
    assert_eq!(pushed.code, Some(0), "{}", stderr_of(&pushed));
    assert!(runs(&world).is_empty(), "{:?}", runs(&world));
    remove_settled(&fixture);
}

/// Without `git-lfs` on PATH the push stops with the install-or-remove explanation and publishes nothing.
#[test]
fn a_missing_git_lfs_blocks_the_push_and_names_the_remedies() {
    let fixture: Fixture = fixture("lfs-push-missing");
    let world: World = world(&fixture, true, None);
    let pushed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "origin", "main"]);
    let stderr: String = stderr_of(&pushed);
    assert_ne!(pushed.code, Some(0), "{stderr}");
    assert_eq!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent", "{stderr}");
    assert!(stderr.contains("`git-lfs` was not found in any directory of PATH"), "{stderr}");
    assert!(stderr.contains("install Git LFS"), "{stderr}");
    assert!(stderr.contains("git lfs uninstall"), "{stderr}");
    remove_settled(&fixture);
}

/// The auto-push after a landed commit uploads first,
/// before the origin's branch exists.
#[test]
fn the_auto_push_after_a_commit_runs_the_upload_before_any_ref_moves() {
    let fixture: Fixture = fixture("lfs-auto-push");
    let world: World = world(&fixture, true, Some(0));
    std::fs::write(world.repo.join("a.txt"), b"one\n").expect("a.txt");
    git(&fixture, world.repo.as_path(), &["add", "--", "a.txt"]);
    let landed: Observed = run_commit(&fixture, world.repo.as_path(), &["commit", "--quiet", "--message=one", "--", "a.txt"]);
    let stderr: String = stderr_of(&landed);
    assert_eq!(landed.code, Some(0), "{stderr}");
    assert!(!stderr.contains("auto-push to origin failed"), "{stderr}");
    // The push is the service's work after the command returned; the control reads the origin once that work settled.
    super::support::settle_service_work(world.repo.as_path());
    let head: String = ref_of(&fixture, world.repo.as_path(), "refs/heads/main");
    let recorded: Vec<Run> = runs(&world);
    assert_eq!(recorded.len(), 1, "{recorded:?}");
    assert_eq!(recorded[0].arguments, "push origin refs/heads/main", "the auto-push names its remote and the branch it pushes, by name");
    assert_eq!(recorded[0].origin_main, "absent");
    assert_eq!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), head);
    remove_settled(&fixture);
}

/// A failing upload keeps the commit local,
/// leaves the origin untouched and says so.
#[test]
fn a_failing_upload_keeps_the_auto_pushed_commit_local() {
    let fixture: Fixture = fixture("lfs-auto-push-fails");
    let world: World = world(&fixture, true, Some(4));
    std::fs::write(world.repo.join("a.txt"), b"one\n").expect("a.txt");
    git(&fixture, world.repo.as_path(), &["add", "--", "a.txt"]);
    let landed: Observed = run_commit(&fixture, world.repo.as_path(), &["commit", "--quiet", "--message=one", "--", "a.txt"]);
    let stderr: String = stderr_of(&landed);
    assert_eq!(landed.code, Some(0), "the commit itself succeeds: {stderr}");
    // With the service the push ends after the command returned, and its failure is the notice the next commit shows.
    let report: String = super::support::push_report(world.repo.as_path(), &stderr);
    assert!(report.contains(super::support::push_unfinished_words()), "{report}");
    // The notice carries the wrapper's own explanation of the failed upload, which names the setting and never its value.
    assert!(report.contains("`git-lfs push` exited with status 4"), "{report}");
    assert_eq!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent", "{report}");
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(), "2");
    remove_settled(&fixture);
}

/// A commit in a repository without LFS keys auto-pushes without any upload step.
#[test]
fn an_auto_push_without_lfs_keys_runs_no_upload() {
    let fixture: Fixture = fixture("lfs-auto-push-none");
    let world: World = world(&fixture, false, Some(9));
    std::fs::write(world.repo.join("a.txt"), b"one\n").expect("a.txt");
    git(&fixture, world.repo.as_path(), &["add", "--", "a.txt"]);
    let landed: Observed = run_commit(&fixture, world.repo.as_path(), &["commit", "--quiet", "--message=one", "--", "a.txt"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert!(runs(&world).is_empty());
    assert_ne!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent");
    remove_settled(&fixture);
}

/// The origin,
/// a second clone that wins races,
/// and the pushing repository one commit ahead of a moved origin.
fn lost_race(fixture: &Fixture, world: &World) -> (PathBuf, String) {
    git(fixture, world.repo.as_path(), &["push", "--quiet", "--set-upstream", "origin", "main"]);
    let rival: PathBuf = fixture.root.join("rival");
    git(
        fixture,
        fixture.root.as_path(),
        &["clone", "--quiet", world.origin.to_str().expect("UTF-8"), "rival"],
    );
    commit_directly(fixture, rival.as_path(), "rival.txt", "rival\n");
    git(fixture, rival.as_path(), &["push", "--quiet", "origin", "HEAD:main"]);
    let rival_tip: String = ref_of(fixture, rival.as_path(), "HEAD");
    return (rival, rival_tip);
}

/// The merge a lost explicit push publishes through the executable's own push is uploaded for first.
#[test]
fn the_merge_push_after_a_lost_race_runs_the_upload_before_the_ref_moves() {
    let fixture: Fixture = fixture("lfs-lost-race");
    let world: World = world(&fixture, true, Some(0));
    let (_rival, rival_tip) = lost_race(&fixture, &world);
    commit_directly(&fixture, world.repo.as_path(), "local.txt", "local\n");
    let pushed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "origin", "main"]);
    let stderr: String = stderr_of(&pushed);
    assert_eq!(pushed.code, Some(0), "{stderr}");
    let tip: String = ref_of(&fixture, world.origin.as_path(), "refs/heads/main");
    assert_ne!(tip, rival_tip, "the merge was published: {stderr}");
    let recorded: Vec<Run> = runs(&world);
    let last: &Run = recorded.last().expect("an upload ran");
    assert_eq!(recorded.len(), 2, "the rejected push and the merge push each ran the hook once, the probe never: {recorded:?}");
    assert_eq!(last.arguments, format!("push origin {tip}"), "the merge is uploaded for by its object name: {recorded:?}");
    assert_eq!(last.origin_main, rival_tip, "the merge was uploaded for before the ref moved");
    remove_settled(&fixture);
}

/// A failing upload during the merge push leaves the origin where the rival put it.
#[test]
fn a_failing_upload_blocks_the_merge_push_after_a_lost_race() {
    let fixture: Fixture = fixture("lfs-lost-race-fails");
    let world: World = world_failing_from(&fixture, true, Some(6), 1);
    let (_rival, rival_tip) = lost_race(&fixture, &world);
    commit_directly(&fixture, world.repo.as_path(), "local.txt", "local\n");
    let pushed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "origin", "main"]);
    assert_ne!(pushed.code, Some(0), "{}", stderr_of(&pushed));
    assert_eq!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), rival_tip);
    remove_settled(&fixture);
}

/// The reconcile push of an auto-push that lost a race is uploaded for first,
/// with the merge as the local object.
#[test]
fn the_reconcile_push_of_an_auto_push_runs_the_upload_before_the_ref_moves() {
    let fixture: Fixture = fixture("lfs-reconcile");
    let world: World = world(&fixture, true, Some(0));
    commit_directly(&fixture, world.repo.as_path(), "a.txt", "one\n");
    let (_rival, rival_tip) = lost_race(&fixture, &world);
    std::fs::write(world.repo.join("a.txt"), b"one\ntwo\n").expect("edit");
    let landed: Observed = run_commit(&fixture, world.repo.as_path(), &["commit", "--quiet", "--message=two", "--", "a.txt"]);
    let stderr: String = stderr_of(&landed);
    assert_eq!(landed.code, Some(0), "{stderr}");
    // The reconcile is the service's work after the command returned; the control reads the origin once that work settled.
    super::support::settle_service_work(world.repo.as_path());
    let tip: String = ref_of(&fixture, world.origin.as_path(), "refs/heads/main");
    let report: String = super::support::push_report(world.repo.as_path(), &stderr);
    assert_ne!(tip, rival_tip, "the merge was published: {report}");
    let recorded: Vec<Run> = runs(&world);
    let last: &Run = recorded.last().expect("an upload ran");
    assert_eq!(recorded.len(), 2, "the rejected push and the reconcile push each ran the hook once: {recorded:?}");
    assert_eq!(last.arguments, format!("push origin {tip}"), "{recorded:?}");
    assert_eq!(last.origin_main, rival_tip);
    remove_settled(&fixture);
}

/// A failing upload during an auto-push reconcile publishes nothing.
#[test]
fn a_failing_upload_blocks_the_reconcile_push_of_an_auto_push() {
    let fixture: Fixture = fixture("lfs-reconcile-fails");
    let world: World = world_failing_from(&fixture, true, Some(8), 1);
    commit_directly(&fixture, world.repo.as_path(), "a.txt", "one\n");
    let (_rival, rival_tip) = lost_race(&fixture, &world);
    std::fs::write(world.repo.join("a.txt"), b"one\ntwo\n").expect("edit");
    let landed: Observed = run_commit(&fixture, world.repo.as_path(), &["commit", "--quiet", "--message=two", "--", "a.txt"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    super::support::settle_service_work(world.repo.as_path());
    assert_eq!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), rival_tip);
    remove_settled(&fixture);
}
