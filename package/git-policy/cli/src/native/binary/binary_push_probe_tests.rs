//! Binary-level controls for the manual-push probe through the whole wrapper:
//! the policies see a push however the caller
//! spells it (`-q`,
//! `--quiet`),
//! and the options that set an upstream publish as they do in plain Git.

use super::commit::remove_settled;
use super::lfs_pre_push::{World, commit_directly, ref_of, wrapped, world};
use super::support::{Fixture, Observed, REAL_GIT, executable, fixture, git, observe, stderr_of};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The finding the policy gives for a file without a final newline.
const FINDING: &str = "noncanonical-final-newline";

/// A repository whose policy file makes a missing final newline an error and whose `main` holds such a file.
fn blocked_world(fixture: &Fixture) -> World {
    let world: World = world(fixture, false, None);
    std::fs::write(
        world.repo.join("monochromatic-git-policy-cli.config.jsonc"),
        "{ \"policies\": { \"final-newline\": \"error\" } }\n",
    )
    .expect("policy file");
    commit_directly(fixture, world.repo.as_path(), "monochromatic-git-policy-cli.config.jsonc", "{ \"policies\": { \"final-newline\": \"error\" } }\n");
    std::fs::write(world.repo.join("a.txt"), "no final newline").expect("a.txt");
    git(fixture, world.repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, world.repo.as_path(), &["commit", "--quiet", "--no-verify", "--message=a", "--", "a.txt"]);
    return world;
}

/// The policy blocks the push with and without `-q`,
/// `--quiet` and `--no-quiet` in any position,
/// and publishes nothing.
#[test]
fn a_policy_blocks_a_manual_push_whatever_its_verbosity() {
    let fixture: Fixture = fixture("push-probe-quiet");
    let world: World = blocked_world(&fixture);
    for region in [
        vec!["push", "origin", "main"],
        vec!["push", "-q", "origin", "main"],
        vec!["push", "--quiet", "origin", "main"],
        vec!["push", "origin", "main", "-q"],
        vec!["push", "-q", "--no-quiet", "origin", "main"],
    ] {
        let pushed: Observed = wrapped(&fixture, world.repo.as_path(), region.as_slice());
        let stderr: String = stderr_of(&pushed);
        assert_ne!(pushed.code, Some(0), "{region:?}: {stderr}");
        assert!(stderr.contains(FINDING), "{region:?}: {stderr}");
        assert_eq!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent", "{region:?}: {stderr}");
    }
    remove_settled(&fixture);
}

/// `-u`,
/// `--set-upstream` and `push.autoSetupRemote` publish and set the upstream,
/// with the policies satisfied.
#[test]
fn the_ways_to_set_an_upstream_publish_and_set_it() {
    let fixture: Fixture = fixture("push-probe-upstream");
    let world: World = world(&fixture, false, None);
    git(&fixture, world.repo.as_path(), &["config", "push.autoSetupRemote", "false"]);
    let upstream_of = |repo: &Path, branch: &str| -> String {
        let output = super::support::direct(&fixture)
            .current_dir(repo)
            .args(["rev-parse", "--abbrev-ref", &format!("{branch}@{{upstream}}")])
            .output()
            .expect("rev-parse starts");
        return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    };
    for (index, (branch, region)) in [
        ("feature-a", vec!["push", "-u", "origin", "feature-a"]),
        ("feature-b", vec!["push", "--set-upstream", "origin", "feature-b"]),
        ("feature-c", vec!["-c", "push.autoSetupRemote=true", "push"]),
    ]
    .into_iter()
    .enumerate()
    {
        git(&fixture, world.repo.as_path(), &["checkout", "--quiet", "-b", branch]);
        commit_directly(&fixture, world.repo.as_path(), &format!("f{index}.txt"), "text\n");
        let pushed: Observed = wrapped(&fixture, world.repo.as_path(), region.as_slice());
        assert_eq!(pushed.code, Some(0), "{region:?}: {}", stderr_of(&pushed));
        let published: String = ref_of(&fixture, world.origin.as_path(), &format!("refs/heads/{branch}"));
        assert_eq!(published, ref_of(&fixture, world.repo.as_path(), "HEAD"), "{region:?}");
        assert_eq!(upstream_of(world.repo.as_path(), branch), format!("origin/{branch}"), "{region:?}");
    }
    remove_settled(&fixture);
}

/// What:
///  A directory of Git's own helpers whose `git-upload-pack` first publishes the branch from the fixture's repository,
/// then answers as Git's own does.
/// Why:
///   The one thing that can move a destination between the probe's dry run and its confirmation,
/// deterministically:
/// the commit's own auto-push publishing the same commit.
/// The refusal the report names ("Remote ref changed during manual-push discovery") needs that ordering,
/// and a sleep or a race decides it by luck;
/// a replaced transport helper decides it by construction.
/// The directory holds a link to every helper Git has,
/// so nothing else changes about the Git the wrapper runs,
/// and the helper publishes once,
/// so a second read of the same remote changes nothing.
fn publish_on_the_first_read(fixture: &Fixture, repo: &Path) -> PathBuf {
    let helpers: String = String::from_utf8_lossy(&git(fixture, fixture.root.as_path(), &["--exec-path"]).stdout)
        .trim()
        .to_owned();
    let directory: PathBuf = fixture.root.join("exec");
    std::fs::create_dir_all(&directory).expect("exec directory");
    for listed in std::fs::read_dir(&helpers).expect("Git's helpers") {
        let helper = listed.expect("helper entry");
        let name: OsString = helper.file_name();
        if name == "git-upload-pack" {
            continue;
        }
        std::os::unix::fs::symlink(helper.path(), directory.join(&name)).expect("helper link");
    }
    let marker: PathBuf = fixture.root.join("published-first.txt");
    executable(
        directory.join("git-upload-pack").as_path(),
        format!(
            "#!/bin/sh\n\
             if [ ! -e '{marker}' ]; then\n\
             : > '{marker}'\n\
             {REAL_GIT} -C '{repo}' push --quiet origin HEAD:refs/heads/main >/dev/null 2>&1\n\
             fi\n\
             exec '{real}' \"$@\"\n",
            marker = marker.display(),
            repo = repo.display(),
            real = Path::new(&helpers).join("git-upload-pack").display(),
        )
        .as_bytes(),
    );
    return directory;
}

/// A commit the origin does not have yet reaches the origin by the commit's own auto-push while the person's
/// `git push` is on its way,
/// and that push ends with success:
/// the origin holds what it sends,
/// so nothing is left to publish.
#[test]
fn a_push_the_auto_push_published_first_sends_nothing_and_succeeds() {
    let fixture: Fixture = fixture("push-probe-published-first");
    let world: World = world(&fixture, false, None);
    git(&fixture, world.repo.as_path(), &["push", "--quiet", "--set-upstream", "origin", "main"]);
    commit_directly(&fixture, world.repo.as_path(), "next.txt", "next\n");
    let pushed: String = ref_of(&fixture, world.repo.as_path(), "refs/heads/main");
    let helpers: PathBuf = publish_on_the_first_read(&fixture, world.repo.as_path());
    let observed: Observed = observe(
        super::support::wrapped(&fixture).current_dir(world.repo.as_path()).args(["push"]).env("GIT_EXEC_PATH", helpers.as_path()),
        b"",
    );
    let stderr: String = stderr_of(&observed);
    assert!(fixture.root.join("published-first.txt").exists(), "the stand-in never moved the origin: {stderr}");
    assert_eq!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), pushed, "the origin holds the commit");
    assert_eq!(observed.code, Some(0), "the origin holds what this push sends: {stderr}");
    assert!(stderr.contains("Everything up-to-date"), "Git's own report says the push published nothing: {stderr}");
    remove_settled(&fixture);
}
