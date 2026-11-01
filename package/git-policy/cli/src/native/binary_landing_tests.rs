//! What:
//!  The landing attempt against real repositories:
//!  a landed commit and what it leaves behind,
//!  the other process's staging that survives it,
//!  and every outcome that lands nothing.
//! Why:
//!  A landing is the only step that moves a branch and replaces the real index,
//!  so each exit has to leave the lock released or handed to recovery,
//!  and the journal records exactly what recovery needs
//!  (`src/policy-engine/commit-landing.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await landTransaction({ gitPath, cwd, capture, workspace, mode, payload, ... });
//! expect(outcome).toEqual({ kind: 'landed', oid });
//! ```

/// The world the controls land in, and the shared fixtures.
use super::landing_support::{Opened, World, git_text, land};
use super::support::{Fixture, TRANSACTION_ID, fixture, git, remove};
use git_policy_cli::landing_attempt::{LandingError, LandingOutcome, LandingPayload};
use git_policy_cli::landing_findings::{LANDING_FINDING_EXIT_CODE, landing_finding};
use git_policy_cli::policy_events::render_policy_event;
use git_policy_cli::real_index_lock::real_lock_path;
use git_policy_cli::transaction_journal::{Base, Conclusion, LandingOperation, SymbolicHead};
use git_policy_cli::transaction_journal_encode::encode_ref_updated;
use git_policy_cli::transaction_journal_parse::{parse_index_lock, parse_landing};
use std::path::PathBuf;

/// The subjects of a ref's reflog, newest first.
fn reflog(fixture: &Fixture, world: &World, reference: &str) -> String {
    return git_text(fixture, world.repo.as_path(), &["reflog", "show", "--format=%gs", reference]);
}

/// How many reflog subjects of a ref carry this transaction's nonce.
fn nonce_count(fixture: &Fixture, world: &World, reference: &str) -> usize {
    return reflog(fixture, world, reference).matches(format!("commit (cli-git {TRANSACTION_ID}):").as_str()).count();
}

/// The staged entries of the real index.
fn staged(fixture: &Fixture, world: &World) -> String {
    return git_text(fixture, world.repo.as_path(), &["ls-files", "--stage"]);
}

/// A commit lands:
///  the branch holds the prepared commit, the real index holds its tree,
///  the nonce is in both reflogs once,
///  and no lock, PID file or keep is left.
#[test]
fn a_commit_lands_and_leaves_the_repository_consistent() {
    let fixture: Fixture = fixture("landing-commit");
    let world: World = World::build(&fixture);
    let mut opened: Opened = world.open();
    let outcome = land(&world, &mut opened, &world.commit_payload()).expect("the landing runs");
    assert_eq!(outcome, LandingOutcome::Landed { oid: world.prepared.clone() });
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-parse", "refs/heads/main"]), world.prepared);
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["write-tree"]), world.prepared_tree);
    assert_eq!(nonce_count(&fixture, &world, "refs/heads/main"), 1);
    assert_eq!(nonce_count(&fixture, &world, "HEAD"), 1);
    assert!(!world.index_lock().exists());
    assert!(!world.pid_file().exists());
    assert!(world.keeps().is_empty(), "{:?}", world.keeps());
    assert!(opened.workspace.preserved, "recovery's record stays until the caller completes the transaction");
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["status", "--porcelain=v1"]), "");
    git(&fixture, world.repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
    remove(&fixture);
}

/// The journal of a landed commit holds every record in the incumbent's bytes:
///  the lock identity, the landing record, `ref-updated.json` and the installed marker.
#[test]
fn a_landed_commit_leaves_the_journal_recovery_reads() {
    let fixture: Fixture = fixture("landing-journal");
    let world: World = World::build(&fixture);
    let mut opened: Opened = world.open();
    land(&world, &mut opened, &world.commit_payload()).expect("the landing runs");
    let lock_record = parse_index_lock(&std::fs::read(world.directory.join("index-lock-1.json")).expect("lock record"))
        .expect("a lock record");
    assert_eq!(lock_record.attempt, 1);
    let landing = parse_landing(&std::fs::read(world.directory.join("landing-1.json")).expect("landing record"))
        .expect("a landing record");
    assert_eq!(landing.operation, LandingOperation::Commit);
    assert_eq!(landing.attempt, 1);
    assert_eq!(landing.expected_old, Base::Commit(world.base.clone()));
    assert_eq!(landing.new_oid.as_deref(), Some(world.prepared.as_str()));
    assert_eq!(landing.landed_tree_oid, world.prepared_tree);
    assert_eq!(landing.lock, lock_record.lock, "the landing names the lock that was journaled first");
    let pack = landing.pack_name.expect("a commit migrates a pack");
    assert_eq!(pack.len(), 40);
    assert!(world.git_dir.join("objects/pack").join(format!("pack-{pack}.pack")).exists());
    assert_eq!(
        std::fs::read_to_string(world.directory.join("ref-updated.json")).expect("ref-updated record"),
        encode_ref_updated(world.prepared.as_str())
    );
    assert!(world.directory.join("index-installed").exists());
    assert_eq!(std::fs::read(world.directory.join("index-installed")).expect("marker"), b"");
    assert_eq!(
        std::fs::read(world.directory.join("post-1.index")).expect("post index"),
        std::fs::read(world.index()).expect("installed index"),
        "the installed index is the recorded post-index"
    );
    assert!(world.directory.join("pre-landing-1.index").exists());
    remove(&fixture);
}

/// A path another process staged after the invocation survives the landing,
///  next to the landed content.
#[test]
fn a_second_process_staged_path_survives_the_landing() {
    let fixture: Fixture = fixture("landing-second-staging");
    let world: World = World::build(&fixture);
    let mut opened: Opened = world.open();
    std::fs::write(world.repo.join("other.txt"), b"other\n").expect("other.txt");
    git(&fixture, world.repo.as_path(), &["add", "--", "other.txt"]);
    let outcome = land(&world, &mut opened, &world.commit_payload()).expect("the landing runs");
    assert_eq!(outcome, LandingOutcome::Landed { oid: world.prepared.clone() });
    let entries: String = staged(&fixture, &world);
    assert!(entries.contains("\tother.txt"), "{entries}");
    assert!(entries.contains("\ta.txt"), "{entries}");
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["status", "--porcelain=v1"]), "A  other.txt");
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-parse", "refs/heads/main"]), world.prepared);
    remove(&fixture);
}

/// A commit lands on an unborn branch: the all-zero expected value creates it.
#[test]
fn a_commit_lands_on_an_unborn_branch() {
    let fixture: Fixture = fixture("landing-unborn");
    let world: World = World::build(&fixture);
    git(&fixture, world.repo.as_path(), &["checkout", "--quiet", "--orphan", "fresh"]);
    let mut opened: Opened = world.open();
    assert_eq!(opened.capture.base, Base::Unborn);
    let payload: LandingPayload = LandingPayload::Commit {
        new_oid: world.root_commit.clone(),
        landed_tree_oid: world.prepared_tree.clone(),
        expected_old: Base::Unborn,
        landed_index: world.directory.join("commit.index"),
        exact_private_index: true,
        guarded_paths: Vec::new(),
    };
    let outcome = land(&world, &mut opened, &payload).expect("the landing runs");
    assert_eq!(outcome, LandingOutcome::Landed { oid: world.root_commit.clone() });
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-parse", "refs/heads/fresh"]), world.root_commit);
    assert_eq!(nonce_count(&fixture, &world, "refs/heads/fresh"), 1);
    remove(&fixture);
}

/// A detached `HEAD` lands by moving `HEAD` itself, and the nonce reaches its reflog once.
#[test]
fn a_detached_head_lands_by_moving_head() {
    let fixture: Fixture = fixture("landing-detached");
    let world: World = World::build(&fixture);
    git(&fixture, world.repo.as_path(), &["checkout", "--quiet", "--detach"]);
    let mut opened: Opened = world.open();
    assert_eq!(opened.capture.layout.symbolic_head, SymbolicHead::Detached);
    let outcome = land(&world, &mut opened, &world.commit_payload()).expect("the landing runs");
    assert_eq!(outcome, LandingOutcome::Landed { oid: world.prepared.clone() });
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-parse", "HEAD"]), world.prepared);
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-parse", "refs/heads/main"]), world.base);
    assert_eq!(nonce_count(&fixture, &world, "HEAD"), 1);
    remove(&fixture);
}

/// Conclusion cleanup runs after a commit installs its index:
///  a conclusion file that native Git removed in the shadow is removed from the worktree's Git directory.
#[test]
fn a_landed_commit_reproduces_conclusion_cleanup() {
    let fixture: Fixture = fixture("landing-conclusion");
    let world: World = World::build(&fixture);
    let mut opened: Opened = world.open();
    let copies: PathBuf = world.directory.join("conclusion");
    std::fs::create_dir(&copies).expect("copies");
    std::fs::write(copies.join("store-held.json"), b"{}").expect("store record");
    std::fs::write(copies.join("MERGE_MSG"), b"merge message\n").expect("copied message");
    std::fs::write(world.git_dir.join("MERGE_MSG"), b"merge message\n").expect("real message");
    land(&world, &mut opened, &world.commit_payload()).expect("the landing runs");
    assert!(!world.git_dir.join("MERGE_MSG").exists(), "native Git removed it in the shadow, so landing removes it");
    remove(&fixture);
}

/// A branch switch between preparation and landing lands nothing, releases the lock,
///  and renders `concurrent-commit/branch-switched` for an exit status of 1.
#[test]
fn a_branch_switch_lands_nothing_and_renders_the_finding() {
    let fixture: Fixture = fixture("landing-switched");
    let world: World = World::build(&fixture);
    let mut opened: Opened = world.open();
    git(&fixture, world.repo.as_path(), &["checkout", "--quiet", "-b", "other"]);
    let outcome = land(&world, &mut opened, &world.commit_payload()).expect("the landing runs");
    assert_eq!(outcome, LandingOutcome::BranchSwitched { current: SymbolicHead::Branch(String::from("refs/heads/other")) });
    assert!(!world.index_lock().exists());
    assert!(!world.pid_file().exists());
    assert!(!opened.workspace.preserved);
    assert!(!world.directory.join("landing-1.json").exists());
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-parse", "refs/heads/main"]), world.base);
    assert_eq!(nonce_count(&fixture, &world, "HEAD"), 0);
    let event = landing_finding(&outcome, &opened.capture, Some(world.prepared.as_str()), false).expect("a finding");
    let line: String = render_policy_event(0, &event);
    assert!(line.contains("\"code\":\"concurrent-commit/branch-switched\""), "{line}");
    assert!(line.contains("HEAD now names refs/heads/other instead of refs/heads/main"), "{line}");
    assert_eq!(LANDING_FINDING_EXIT_CODE, 1);
    remove(&fixture);
}

/// A detached `HEAD` where a branch was captured is a switch too.
#[test]
fn a_detached_head_where_a_branch_was_captured_is_a_switch() {
    let fixture: Fixture = fixture("landing-switched-detached");
    let world: World = World::build(&fixture);
    let mut opened: Opened = world.open();
    git(&fixture, world.repo.as_path(), &["checkout", "--quiet", "--detach"]);
    let outcome = land(&world, &mut opened, &world.commit_payload()).expect("the landing runs");
    assert_eq!(outcome, LandingOutcome::BranchSwitched { current: SymbolicHead::Detached });
    assert!(!world.index_lock().exists());
    remove(&fixture);
}

/// Move `main` to a new commit, as another landing would.
fn move_main(fixture: &Fixture, world: &World) -> String {
    let moved: String = git_text(
        fixture,
        world.repo.as_path(),
        &["commit-tree", world.base_tree.as_str(), "-p", world.base.as_str(), "-m", "winner"],
    );
    git(fixture, world.repo.as_path(), &["update-ref", "refs/heads/main", moved.as_str(), world.base.as_str()]);
    return moved;
}

/// A target that moved since preparation lands nothing and reports the winner,
///  replayable for an ordinary commit.
#[test]
fn a_moved_target_reports_the_winner_and_is_replayable() {
    let fixture: Fixture = fixture("landing-moved");
    let world: World = World::build(&fixture);
    let mut opened: Opened = world.open();
    let winner: String = move_main(&fixture, &world);
    let outcome = land(&world, &mut opened, &world.commit_payload()).expect("the landing runs");
    assert_eq!(outcome, LandingOutcome::HeadMoved { current: Base::Commit(winner.clone()), replayable: true });
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-parse", "refs/heads/main"]), winner);
    assert!(!world.index_lock().exists());
    assert!(!world.pid_file().exists());
    assert!(!opened.workspace.preserved);
    assert!(world.keeps().is_empty());
    assert!(!world.directory.join("landing-1.json").exists(), "the check precedes the record");
    remove(&fixture);
}

/// An amend or a conclusion is never replayed, whatever moved the target.
#[test]
fn a_moved_target_is_not_replayable_for_an_amend_or_a_conclusion() {
    for conclusion in [Conclusion::Amend, Conclusion::Merge, Conclusion::CherryPick, Conclusion::Revert] {
        let fixture: Fixture = fixture("landing-moved-conclusion");
        let world: World = World::build(&fixture);
        let mut opened: Opened = world.open();
        opened.capture.layout.conclusion = conclusion;
        let winner: String = move_main(&fixture, &world);
        let outcome = land(&world, &mut opened, &world.commit_payload()).expect("the landing runs");
        assert_eq!(outcome, LandingOutcome::HeadMoved { current: Base::Commit(winner), replayable: false }, "{conclusion:?}");
        remove(&fixture);
    }
}

/// A deleted target leaves nothing to replay onto.
#[test]
fn a_deleted_target_is_not_replayable() {
    let fixture: Fixture = fixture("landing-deleted");
    let world: World = World::build(&fixture);
    let mut opened: Opened = world.open();
    git(&fixture, world.repo.as_path(), &["update-ref", "-d", "refs/heads/main"]);
    let outcome = land(&world, &mut opened, &world.commit_payload()).expect("the landing runs");
    assert_eq!(outcome, LandingOutcome::HeadMoved { current: Base::Unborn, replayable: false });
    remove(&fixture);
}

/// A normalization is never replayed: it carries no commit to replay.
#[test]
fn a_moved_target_is_not_replayable_for_a_normalization() {
    let fixture: Fixture = fixture("landing-moved-normalization");
    let world: World = World::build(&fixture);
    let mut opened: Opened = world.open();
    let winner: String = move_main(&fixture, &world);
    let payload: LandingPayload = LandingPayload::NormalizeOnly { landed_tree_oid: world.base_tree.clone() };
    let outcome = land(&world, &mut opened, &payload).expect("the landing runs");
    assert_eq!(outcome, LandingOutcome::HeadMoved { current: Base::Commit(winner), replayable: false });
    remove(&fixture);
}

/// A normalization installs the settled index without moving the target:
///  no commit, no pack, no reflog entry, no `ref-updated.json`, and no conclusion cleanup.
#[test]
fn a_normalization_installs_the_index_without_moving_the_target() {
    let fixture: Fixture = fixture("landing-normalize");
    let world: World = World::build(&fixture);
    let mut opened: Opened = world.open();
    // The settled index is the captured one: every blob it names is in the real store.
    std::fs::copy(world.directory.join("captured.index"), world.directory.join("commit.index")).expect("settled index");
    let copies: PathBuf = world.directory.join("conclusion");
    std::fs::create_dir(&copies).expect("copies");
    std::fs::write(copies.join("store-held.json"), b"{}").expect("store record");
    std::fs::write(copies.join("MERGE_MSG"), b"merge message\n").expect("copied message");
    std::fs::write(world.git_dir.join("MERGE_MSG"), b"merge message\n").expect("real message");
    let payload: LandingPayload = LandingPayload::NormalizeOnly { landed_tree_oid: world.base_tree.clone() };
    let outcome = land(&world, &mut opened, &payload).expect("the landing runs");
    assert_eq!(outcome, LandingOutcome::Landed { oid: world.base.clone() });
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-parse", "refs/heads/main"]), world.base);
    assert_eq!(nonce_count(&fixture, &world, "refs/heads/main"), 0);
    assert!(!world.directory.join("ref-updated.json").exists());
    assert!(world.directory.join("index-installed").exists());
    assert!(world.git_dir.join("MERGE_MSG").exists(), "a normalization has no conclusion to clean up");
    let landing = parse_landing(&std::fs::read(world.directory.join("landing-1.json")).expect("landing record"))
        .expect("a landing record");
    assert_eq!(landing.operation, LandingOperation::NormalizeOnly);
    assert_eq!(landing.new_oid, None);
    assert_eq!(landing.pack_name, None);
    assert_eq!(landing.expected_old, Base::Commit(world.base.clone()));
    assert!(!world.index_lock().exists());
    assert!(!world.pid_file().exists());
    assert!(world.keeps().is_empty());
    remove(&fixture);
}

/// A normalization on an unborn branch lands the empty base.
#[test]
fn a_normalization_on_an_unborn_branch_lands_the_empty_base() {
    let fixture: Fixture = fixture("landing-normalize-unborn");
    let world: World = World::build(&fixture);
    git(&fixture, world.repo.as_path(), &["checkout", "--quiet", "--orphan", "fresh"]);
    let mut opened: Opened = world.open();
    std::fs::copy(world.directory.join("captured.index"), world.directory.join("commit.index")).expect("settled index");
    let payload: LandingPayload = LandingPayload::NormalizeOnly { landed_tree_oid: world.base_tree.clone() };
    let outcome = land(&world, &mut opened, &payload).expect("the landing runs");
    assert_eq!(outcome, LandingOutcome::Landed { oid: String::new() });
    remove(&fixture);
}

/// A failed object migration releases the lock and leaves the directory to the caller.
#[test]
fn a_failed_migration_releases_the_lock() {
    let fixture: Fixture = fixture("landing-migration-fails");
    let world: World = World::build(&fixture);
    let mut opened: Opened = world.open();
    std::fs::remove_dir_all(world.shadow.join("objects")).expect("break the shadow");
    let failure = land(&world, &mut opened, &world.commit_payload()).expect_err("the migration fails");
    assert!(matches!(failure, LandingError::Git(_)), "{failure}");
    assert!(!world.index_lock().exists());
    assert!(!world.pid_file().exists());
    assert!(!opened.workspace.preserved);
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-parse", "refs/heads/main"]), world.base);
    remove(&fixture);
}

/// A landing record that cannot be written fails the landing, releases the lock,
///  and marks the directory for recovery, which the incumbent marks before writing.
#[test]
fn a_failed_landing_record_releases_the_lock() {
    let fixture: Fixture = fixture("landing-record-fails");
    let world: World = World::build(&fixture);
    let mut opened: Opened = world.open();
    std::fs::write(world.directory.join("landing-1.json"), b"an earlier record").expect("blocking record");
    let failure = land(&world, &mut opened, &world.commit_payload()).expect_err("the record cannot be written twice");
    assert!(matches!(&failure, LandingError::Journal(text) if text.contains("landing-1.json")), "{failure}");
    assert!(!world.index_lock().exists());
    assert!(!world.pid_file().exists());
    assert!(opened.workspace.preserved);
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-parse", "refs/heads/main"]), world.base);
    assert!(world.keeps().len() == 1, "the pack stays kept until recovery removes the keep: {:?}", world.keeps());
    remove(&fixture);
}

/// A foreign lock of a dead owner outlasts the budget: nothing is journaled and the foreign lock stays.
#[test]
fn a_foreign_lock_of_a_dead_owner_stops_the_landing() {
    let fixture: Fixture = fixture("landing-foreign-lock");
    let world: World = World::build(&fixture);
    let mut opened: Opened = world.open();
    std::fs::write(real_lock_path(world.index().as_path()), b"").expect("foreign lock");
    std::fs::write(world.pid_file(), "pid 2147483646\n").expect("dead owner's PID file");
    let failure = land(&world, &mut opened, &world.commit_payload()).expect_err("the lock is not overtaken");
    assert!(matches!(failure, LandingError::Lock(_)), "{failure}");
    assert!(world.index_lock().exists());
    assert!(!world.directory.join("index-lock-1.json").exists());
    assert!(!opened.workspace.preserved);
    remove(&fixture);
}
