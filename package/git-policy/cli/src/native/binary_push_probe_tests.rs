//! Binary-level controls for the manual-push probe through the whole wrapper: the policies see a push however the caller
//! spells it (`-q`, `--quiet`), and the options that set an upstream publish as they do in plain Git.

use super::commit::remove_settled;
use super::lfs_pre_push::{World, commit_directly, ref_of, wrapped, world};
use super::support::{Fixture, Observed, fixture, git, stderr_of};
use std::path::Path;

/// The finding the policy gives for a file without a final newline.
const FINDING: &str = "noncanonical-final-newline";

/// A repository whose policy file makes a missing final newline an error and whose `main` holds such a file.
fn blocked_world(fixture: &Fixture) -> World {
    let world: World = world(fixture, false, None);
    std::fs::write(
        world.repo.join("cli-git.config.jsonc"),
        "{ \"policies\": { \"final-newline\": \"error\" } }\n",
    )
    .expect("policy file");
    commit_directly(fixture, world.repo.as_path(), "cli-git.config.jsonc", "{ \"policies\": { \"final-newline\": \"error\" } }\n");
    std::fs::write(world.repo.join("a.txt"), "no final newline").expect("a.txt");
    git(fixture, world.repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, world.repo.as_path(), &["commit", "--quiet", "--no-verify", "--message=a", "--", "a.txt"]);
    return world;
}

/// The policy blocks the push with and without `-q`, `--quiet` and `--no-quiet` in any position, and publishes nothing.
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

/// `-u`, `--set-upstream` and `push.autoSetupRemote` publish and set the upstream, with the policies satisfied.
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
