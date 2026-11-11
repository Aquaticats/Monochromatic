//! Controls for the restart decision about an open batch when a Git question about the branch fails:
//! a failure is no answer,
//! so
//!  the batch stays open (the owner's rule is "never discard user data"),
//! and only an answer that the branch never moved,
//! or
//!  that the reference is gone,
//! lets the intent go.

use super::head_tests::blind_context;
use super::tests::{World, land_with_failed_index, request, world};
use super::*;
use crate::service_instance::instance_of;
use crate::service_journal::{OPEN_EXTENSION, batch_path, parse_intent, write_intent};
use crate::test_support::{git, git_text, remove};

/// The context of the world's journal with a Git that fails every question with status 128.
fn blind_land_context(world: &World) -> LandContext {
    return LandContext { git: blind_context(world.root.as_path()), journal: world.context.journal.clone(), on_objects_written: None, lock_ceiling: world.context.lock_ceiling };
}

/// The intent of batch `seq`,
/// as the journal holds it.
fn open_intent(world: &World, seq: u64) -> Intent {
    let text: String = std::fs::read_to_string(batch_path(world.context.journal.as_path(), seq, OPEN_EXTENSION)).expect("the open intent");
    return parse_intent(text.as_str()).expect("a readable intent");
}

/// The decisions a recovery made,
/// by batch.
fn decisions(reports: &[RecoveryReport]) -> Vec<(u64, Recovery)> {
    return reports.iter().map(|report| { return (report.seq, report.decision); }).collect();
}

/// A batch that landed stays open when the Git question about its branch fails,
/// for the service's own recovery and for the
///  recovery of one instance;
/// once Git answers again the batch is finished and the index shows the landing.
#[test]
fn a_git_that_fails_does_not_make_a_landed_batch_look_unmoved() {
    for name in ["all", "instance"] {
        let world: World = world(format!("finish-git-failure-{name}").as_str());
        let landed: CommitRequest = request(&world, "kept", "kept.txt", "kept\n");
        land_with_failed_index(&world, 1, &[landed]);
        let blind: LandContext = blind_land_context(&world);
        let instance: String = instance_of(world.repo.join(".git").as_path());
        let reports: Vec<RecoveryReport> = if name == "all" { recover(&blind) } else { recover_instance(&blind, instance.as_str()) };
        assert!(batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION).exists(), "{name}: the intent of a landed batch is not dropped on a Git failure");
        assert!(reports.is_empty(), "{name}: no decision is made without an answer: {reports:?}");
        let later: Vec<RecoveryReport> = recover(&world.context);
        assert_eq!(decisions(&later), vec![(1, Recovery::Finish)], "{name}");
        assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "", "{name}: the index shows the landing");
        remove(world.root.as_path());
    }
}

/// A reference file Git cannot read exits like a missing reference but says so on its standard error;
/// it is no proof that the
///  branch never moved.
#[test]
fn a_reference_git_warns_about_is_not_a_missing_reference() {
    let world: World = world("finish-git-broken-reference");
    let landed: CommitRequest = request(&world, "broken", "broken.txt", "broken\n");
    land_with_failed_index(&world, 1, &[landed]);
    let new: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let reference_file: PathBuf = world.repo.join(".git/refs/heads/main");
    std::fs::write(&reference_file, "not a commit name\n").expect("break the reference");
    let reports: Vec<RecoveryReport> = recover(&world.context);
    assert!(batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION).exists(), "the intent stays: {reports:?}");
    std::fs::write(&reference_file, format!("{new}\n")).expect("restore the reference");
    assert_eq!(decisions(&recover(&world.context)), vec![(1, Recovery::Finish)]);
    remove(world.root.as_path());
}

/// Control:
/// a reference that is gone for certain still lets the intent of a batch that cannot land go.
#[test]
fn a_reference_that_is_missing_still_drops_the_intent() {
    let world: World = world("finish-git-missing-reference");
    let landed: CommitRequest = request(&world, "gone", "gone.txt", "gone\n");
    land_with_failed_index(&world, 1, &[landed]);
    let mut intent: Intent = open_intent(&world, 1);
    intent.reference = crate::ref_name::RefName::from_text("refs/heads/no-such-branch");
    write_intent(world.context.journal.as_path(), &intent).expect("rewritten intent");
    assert_eq!(decisions(&recover(&world.context)), vec![(1, Recovery::Drop)]);
    assert!(!batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION).exists());
    remove(world.root.as_path());
}

/// When the branch is neither at the old value nor at the new one,
/// the question whether the new commit is below it decides.
///  A question that fails keeps the batch open;
/// a clean "no" (a commit that is not below the branch) drops it.
#[test]
fn a_failed_ancestry_question_keeps_the_batch_and_a_clean_no_drops_it() {
    let world: World = world("finish-git-ancestry");
    let landed: CommitRequest = request(&world, "ancestry", "ancestry.txt", "ancestry\n");
    land_with_failed_index(&world, 1, &[landed]);
    let first: Intent = open_intent(&world, 1);
    git(world.repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=moved on"]);
    // The branch is now past the new commit: the batch landed.
    assert_eq!(decisions(&recover(&world.context)), vec![(1, Recovery::Finish)]);
    let unknown: Intent = Intent { seq: 2, new: "e".repeat(40), ..first.clone() };
    write_intent(world.context.journal.as_path(), &unknown).expect("intent of a commit Git does not have");
    let reports: Vec<RecoveryReport> = recover(&world.context);
    assert!(batch_path(world.context.journal.as_path(), 2, OPEN_EXTENSION).exists(), "an object Git cannot find is no proof: {reports:?}");
    git(world.repo.as_path(), &["checkout", "--quiet", "--detach", first.old.as_str()]);
    git(world.repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=side"]);
    let side: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    git(world.repo.as_path(), &["checkout", "--quiet", "main"]);
    let elsewhere: Intent = Intent { seq: 3, new: side, ..first };
    write_intent(world.context.journal.as_path(), &elsewhere).expect("intent of a commit off the branch");
    let after: Vec<(u64, Recovery)> = decisions(&recover(&world.context));
    assert!(after.contains(&(3, Recovery::Drop)), "{after:?}");
    assert!(!batch_path(world.context.journal.as_path(), 3, OPEN_EXTENSION).exists());
    remove(world.root.as_path());
}
