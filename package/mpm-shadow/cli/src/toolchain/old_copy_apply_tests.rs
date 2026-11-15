//! Unit tests of the last step of the old-copy cleanup:
//! the `current` link is settled before any version it names goes,
//! and a link that could not be settled keeps every copy.

/// Imports `fs` from `std`.
use std::fs;
/// Imports what these tests use from `std::path`.
use std::path::PathBuf;

/// Imports the context and the fakes it needs.
use super::context::{Context, Host};
use super::fakes::Rig;
use super::http::NoNetwork;
/// Imports the plan the cleanup works from.
use super::managed_losers::LoserCopy;
/// Imports the module under test.
use super::old_copy_apply::{OldCopyOutcome, apply_loser_for_a_test};
use super::old_copies::OldCopyPlans;
use super::runner::ProcessRunner;

/// What:
/// A link this run could not settle keeps the version it names:
/// the copy is not deleted while `current` still points into it,
/// and the failure says which link was not settled.
/// Why:
/// The comment before the link step says the link is settled before any version goes,
/// so that no launcher or record that follows it ever sees it dangle;
/// the review of this change showed a `Forward` that failed and still had its version deleted,
/// which left `current` naming a directory that was gone.
#[test]
fn a_link_that_could_not_be_settled_keeps_the_version_it_names() {
    let rig = Rig::new();
    let root: PathBuf = rig._directory.path().join("loser-root");
    let tool: PathBuf = root.join("cargo").join("fd-find");
    let old: PathBuf = tool.join("1.0.0");
    let current: PathBuf = tool.join("current");
    let winner_root: PathBuf = rig._directory.path().join("winner-root");
    fs::create_dir_all(&old).expect("make the old version");
    fs::write(old.join("payload"), "old").expect("write the old version's payload");
    std::os::unix::fs::symlink("1.0.0", &current).expect("current names the old version");
    // A directory where the atomic link's temporary goes: the link cannot be moved aside or written.
    let temporary: PathBuf = tool.join(format!(".current.{}.tmp", std::process::id()));
    fs::create_dir_all(&temporary).expect("occupy the atomic link's temporary path");

    let plan = super::managed_losers::LoserPlan {
        tool_id: "cargo:fd-find".to_string(),
        root: root.clone(),
        winner_root: winner_root.clone(),
        winner_version: "2.0.0".to_string(),
        copies: vec![LoserCopy { version: "1.0.0".to_string(), directory: old.clone(), bins: Vec::new() }],
        kept_copies: Vec::new(),
        repoints: Vec::new(),
        kept: None,
        forward: Some((current.clone(), winner_root.join("cargo").join("fd-find").join("current"))),
        archive: false,
    };
    let mut outcome = OldCopyOutcome::default();
    let context = Context {
        scope: &rig.scope,
        env: &rig.env,
        runner: &ProcessRunner,
        http: &NoNetwork,
        host: &Host::current(),
        tools: &rig.lock.tools,
    };

    apply_loser_for_a_test(&context, &OldCopyPlans::default(), &plan, &mut outcome);

    assert_eq!(outcome.failed.len(), 1, "the link was not settled, and the outcome says so: {outcome:?}");
    assert!(outcome.failed[0].1.contains("cannot forward"), "{outcome:?}");
    assert!(old.join("payload").is_file(), "the version the link names was deleted after the link was not settled");
    assert_eq!(
        fs::read_link(&current).expect("current is still a link"),
        PathBuf::from("1.0.0"),
        "the link was left naming the version it named"
    );
}

/// What:
/// A loser root whose `current` link is absent is still forwarded to the winner before its version goes.
/// Why:
/// The review of this change showed the link step returning "leave it" because it could not read a link that was not
/// there, before it looked at the plan's forward: the root was left with neither a finished version nor `current`,
/// and every launcher and record that follows that root stopped working.
#[test]
fn a_loser_root_without_a_current_is_forwarded() {
    let rig = Rig::new();
    let root: PathBuf = rig._directory.path().join("loser-root");
    let old: PathBuf = root.join("cargo").join("fd-find").join("1.0.0");
    super::fakes_cargo::finish_install(&old, "fd-find", "1.0.0", &["fd"]);
    let current: PathBuf = root.join("cargo").join("fd-find").join("current");
    let winner_root: PathBuf = rig._directory.path().join("winner-root");
    let winner_current: PathBuf = winner_root.join("cargo").join("fd-find").join("current");
    fs::create_dir_all(winner_current.parent().expect("the winner's tool directory")).expect("make the winner's tool");
    std::os::unix::fs::symlink("2.0.0", &winner_current).expect("make the winner's current");
    let plan = super::managed_losers::LoserPlan {
        tool_id: "cargo:fd-find".to_string(),
        root: root.clone(),
        winner_root,
        winner_version: "2.0.0".to_string(),
        copies: vec![LoserCopy { version: "1.0.0".to_string(), directory: old.clone(), bins: Vec::new() }],
        kept_copies: Vec::new(),
        repoints: Vec::new(),
        kept: None,
        forward: Some((current.clone(), winner_current.clone())),
        archive: false,
    };
    let mut outcome = OldCopyOutcome::default();
    let context = Context {
        scope: &rig.scope,
        env: &rig.env,
        runner: &ProcessRunner,
        http: &NoNetwork,
        host: &Host::current(),
        tools: &rig.lock.tools,
    };

    apply_loser_for_a_test(&context, &OldCopyPlans::default(), &plan, &mut outcome);

    assert!(outcome.failed.is_empty(), "{outcome:?}");
    assert!(!old.exists(), "the planned older copy was not removed: {outcome:?}");
    assert_eq!(
        fs::read_link(&current).expect("the emptied root needs its forwarding current"),
        winner_current,
        "the root that kept no version points at the winner's current"
    );
}
