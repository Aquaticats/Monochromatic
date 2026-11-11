//! Tests of the stage claim and the cleanup of stages an interrupted copy left behind.

use super::*;
use crate::json_record::quote;
use crate::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
use crate::worktree_copy_stage_claim_reaping::*;
use crate::private_storage::{create_private_directory, take_directory_syncs, take_file_syncs};
use crate::test_support::{dead_copy_owner, fixture, git_context, remove, repository};
use crate::worktree_add_binding::{Recorded, bind_registration, decode_record};
use crate::worktree_copy_journal::{create_journal, ensure_journal_root, journal_root};
use crate::worktree_copy_journal_encoding_and_validation::JOURNAL_PARENT_NAME;
use crate::worktree_copy_owner::current_copy_owner;
use crate::worktree_copy_selection::CopySelection;
use std::ffi::OsString;
use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};
use crate::worktree_copy_stage::stage_ignored_snapshot;
use std::os::unix::fs::PermissionsExt;

/// `settle_interrupted_add` with the two lookups as values,
/// so that a lookup that fails other than by absence can be driven.
fn settle_from(record: &Path, marker: std::io::Result<bool>, read: &mut dyn FnMut() -> std::io::Result<String>) -> Settlement {
    if let Some(settled) = settle_marker(record, marker) {
        return settled;
    }
    return settle_text(record, read());
}

/// A common directory and a destination parent beside it,
/// as a copy into `worktrees/topic` would use.
fn scene(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join("common");
    let worktrees: PathBuf = root.join("worktrees");
    std::fs::create_dir_all(common.as_path()).expect("common");
    std::fs::create_dir_all(worktrees.as_path()).expect("worktrees");
    return (root, common, worktrees.join("topic"));
}

/// A stage container with a payload and a read-only directory,
/// the shape an interrupted copy leaves.
fn abandoned_stage(container: &Path) {
    create_private_directory(container).expect("container");
    let payload: PathBuf = container.join("payload");
    create_private_directory(payload.as_path()).expect("payload");
    let locked: PathBuf = payload.join("locked");
    create_private_directory(locked.as_path()).expect("locked");
    std::fs::write(locked.join("data"), b"staged").expect("data");
    std::fs::set_permissions(locked.as_path(), std::fs::Permissions::from_mode(0o500)).expect("read-only");
}

/// A claim names a container beside the destination,
/// writes it under the journal root,
/// and creates nothing else.
#[test]
fn a_claim_names_the_container_and_creates_no_container() {
    let (root, common, destination) = scene("claim-names");
    let claim: StageClaim = claim_stage(common.as_path(), destination.as_path()).expect("claim");
    assert_eq!(claim.container().parent(), destination.parent());
    assert!(claim.container().file_name().expect("name").to_string_lossy().starts_with(STAGE_PREFIX));
    assert!(!claim.container().exists(), "the claim precedes the container");
    let recorded: String = std::fs::read_to_string(claim.path.as_path()).expect("claim file");
    let mut lines = recorded.lines();
    assert_eq!(lines.next(), Some(claim.container().to_string_lossy().as_ref()), "a text path stays where the earlier release reads it");
    assert_eq!(lines.next(), Some(format!("{CONTAINER_HEX_PREFIX}{}", hex_of_path(claim.container())).as_str()), "the claim carries the container's bytes");
    assert_eq!(lines.next(), Some(format!("owner={}", current_copy_owner().expect("owner").token()).as_str()), "the claim names its owner");
    assert_eq!(claim.path.parent(), Some(journal_root(common.as_path()).as_path()));
    assert!(claim.path.to_string_lossy().ends_with(".stage"));
    assert_eq!(
        std::fs::metadata(claim.path.as_path()).expect("metadata").permissions().mode() & 0o777,
        0o600
    );
    remove(root.as_path());
}

/// Releasing a claim removes its file and nothing else;
/// two claims have different containers.
#[test]
fn releasing_a_claim_removes_only_its_file() {
    let (root, common, destination) = scene("claim-release");
    let first: StageClaim = claim_stage_for(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("first");
    let second: StageClaim = claim_stage_for(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("second");
    assert_ne!(first.container(), second.container());
    let first_path: PathBuf = first.path.clone();
    let second_path: PathBuf = second.path.clone();
    first.release().expect("release");
    assert!(!first_path.exists());
    assert!(second_path.exists());
    second.release().expect("release");
    remove(root.as_path());
}

/// A claim failure names the claim and leaves no temporary file when the journal root cannot be created.
#[test]
fn a_claim_without_a_common_directory_fails() {
    let (root, common, destination) = scene("claim-no-common");
    let failure: WorktreeCopyError =
        claim_stage(common.join("absent").as_path(), destination.as_path()).expect_err("no common directory");
    assert!(!failure.0.is_empty());
    remove(root.as_path());
}

/// A claim exists for the pending check until it is released.
#[test]
fn claims_exist_until_released() {
    let (root, common, destination) = scene("claim-exist");
    assert!(!stage_claims_exist(common.as_path()).expect("no root"));
    ensure_journal_root(common.as_path()).expect("root");
    assert!(!stage_claims_exist(common.as_path()).expect("empty root"));
    let claim: StageClaim = claim_stage_for(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    assert!(stage_claims_exist(common.as_path()).expect("claimed"));
    claim.release().expect("release");
    assert!(!stage_claims_exist(common.as_path()).expect("released"));
    remove(root.as_path());
}

/// Only claim files count:
/// journals and unrelated files do not make the repository pending.
#[test]
fn other_files_are_not_claims() {
    let (root, common, _destination) = scene("claim-others");
    let journals: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    std::fs::write(journals.join("id.json"), b"{}").expect("journal-shaped file");
    std::fs::write(journals.join("notes.txt"), b"x").expect("note");
    assert!(!stage_claims_exist(common.as_path()).expect("others"));
    remove(root.as_path());
}

/// An abandoned stage is removed with its read-only directories,
/// its claim goes,
/// and one notice names the stage.
#[test]
fn an_abandoned_stage_is_removed_with_a_notice() {
    let (root, common, destination) = scene("claim-reap");
    let claim: StageClaim = claim_stage_for(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    abandoned_stage(claim.container());
    let container: PathBuf = claim.container().to_path_buf();
    let claim_path: PathBuf = claim.path.clone();
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reaped");
    assert!(!container.exists());
    assert!(!claim_path.exists());
    assert_eq!(
        notices,
        vec![format!(
            "monochromatic-git-policy-cli: removed the stage {} that an interrupted ignored-state copy left behind.",
            quote(container.to_string_lossy().as_ref())
        )]
    );
    remove(root.as_path());
}

/// A claim whose container was never created is removed quietly:
/// nothing was staged.
#[test]
fn a_claim_without_a_container_is_removed_without_a_notice() {
    let (root, common, destination) = scene("claim-no-container");
    let claim: StageClaim = claim_stage_for(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    let claim_path: PathBuf = claim.path.clone();
    assert!(reap_abandoned_stages(common.as_path()).expect("reaped").is_empty());
    assert!(!claim_path.exists());
    remove(root.as_path());
}

/// A stage that a pending journal names is kept for recovery;
/// only its claim is removed.
#[test]
fn a_journaled_stage_is_kept() {
    let (root, common, destination) = scene("claim-journaled");
    let source: PathBuf = repository(root.as_path(), "source");
    std::fs::write(source.join(".gitignore"), b"cache/\n").expect("gitignore");
    std::fs::create_dir_all(source.join("cache")).expect("cache");
    std::fs::write(source.join("cache/data"), b"payload").expect("data");
    let claim: StageClaim = claim_stage_for(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    let snapshot = stage_ignored_snapshot(
        &git_context(),
        source.as_path(),
        claim.container(),
        &[],
        &CopySelection::Ignored,
    )
    .expect("staged");
    let pending = create_journal(common.as_path(), destination.as_path(), &snapshot).expect("journal");
    let claim_path: PathBuf = claim.path.clone();
    assert!(reap_abandoned_stages(common.as_path()).expect("reaped").is_empty());
    assert!(snapshot.stage_container.exists(), "the journal owns the stage");
    assert!(pending.path.exists());
    assert!(!claim_path.exists());
    remove(root.as_path());
}

/// Every abandoned stage of a repository is removed in one pass,
/// each with its own notice.
#[test]
fn every_abandoned_stage_is_removed() {
    let (root, common, destination) = scene("claim-reap-many");
    let first: StageClaim = claim_stage_for(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("first");
    let second: StageClaim = claim_stage_for(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("second");
    abandoned_stage(first.container());
    abandoned_stage(second.container());
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reaped");
    assert_eq!(notices.len(), 2);
    assert!(!first.container().exists());
    assert!(!second.container().exists());
    assert!(!stage_claims_exist(common.as_path()).expect("claims"));
    remove(root.as_path());
}

/// A claim that names a path outside the stage container naming is refused,
/// and the named path survives.
#[test]
fn a_claim_naming_another_directory_is_refused() {
    let (root, common, destination) = scene("claim-foreign-name");
    let victim: PathBuf = destination.parent().expect("parent").join("precious");
    std::fs::create_dir_all(victim.as_path()).expect("victim");
    let claim: StageClaim = claim_stage_for(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    std::fs::write(claim.path.as_path(), victim.to_string_lossy().as_bytes()).expect("rewrite claim");
    let failure: WorktreeCopyError = reap_abandoned_stages(common.as_path()).expect_err("refused");
    assert!(failure.0.contains("does not name a stage container"), "{}", failure.0);
    assert!(victim.exists());
    assert!(claim.path.exists(), "an unproven claim stays for inspection");
    remove(root.as_path());
}

/// An earlier release wrote a container path verbatim,
/// so a destination parent that holds line breaks puts lines of its own inside the claim.
/// A line of bytes inside such a path is not where this release writes one,
/// so the claim is refused and the private directory those bytes name survives.
#[test]
fn a_claim_that_names_other_bytes_inside_a_path_is_refused_and_what_it_names_survives() {
    let (root, common, destination) = scene("claim-path-lines");
    let target: PathBuf = destination.parent().expect("parent").join(format!("{STAGE_PREFIX}keep"));
    create_private_directory(target.as_path()).expect("a private directory with the stage naming");
    std::fs::write(target.join("keep"), b"user data").expect("user data");
    let journals: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    let claim: PathBuf = journals.join("injected.stage");
    // The record the earlier release wrote for the destination parent `/w/p\ncontainerHex=<bytes>\nq`.
    let text: String = format!(
        "/w/p\n{CONTAINER_HEX_PREFIX}{}\nq\n{OWNER_LINE_PREFIX}{}\n",
        hex_of_path(target.as_path()),
        dead_copy_owner().token()
    );
    crate::private_storage::write_private_file(claim.as_path(), text.as_bytes()).expect("claim");
    reap_abandoned_stages(common.as_path()).expect_err("a claim of a shape this release does not write is refused");
    assert!(target.join("keep").exists(), "the directory those bytes name is not this claim's to remove");
    assert!(claim.exists(), "a record that cannot be trusted is kept");
    remove(root.as_path());
}

/// A claim this release writes names one path on its line of text and on its line of bytes.
/// Two lines that name different paths are refused,
/// and the directory the bytes name survives.
#[test]
fn a_claim_whose_text_and_bytes_name_different_paths_is_refused() {
    let (root, common, destination) = scene("claim-text-bytes-disagree");
    let target: PathBuf = destination.parent().expect("parent").join(format!("{STAGE_PREFIX}keep"));
    create_private_directory(target.as_path()).expect("a private directory with the stage naming");
    std::fs::write(target.join("keep"), b"user data").expect("user data");
    let journals: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    let claim: PathBuf = journals.join("disagreeing.stage");
    let named_elsewhere: PathBuf = destination.parent().expect("parent").join(format!("{STAGE_PREFIX}other"));
    let text: String = format!(
        "{}\n{CONTAINER_HEX_PREFIX}{}\n{OWNER_LINE_PREFIX}{}\n",
        named_elsewhere.to_string_lossy(),
        hex_of_path(target.as_path()),
        dead_copy_owner().token()
    );
    crate::private_storage::write_private_file(claim.as_path(), text.as_bytes()).expect("claim");
    reap_abandoned_stages(common.as_path()).expect_err("a claim whose two lines name two paths is refused");
    assert!(target.join("keep").exists(), "the directory those bytes name is not this claim's to remove");
    assert!(claim.exists(), "a record that cannot be trusted is kept");
    remove(root.as_path());
}

/// A claim that names a relative path is refused even when its last component looks like a stage.
#[test]
fn a_claim_naming_a_relative_path_is_refused() {
    let (root, common, destination) = scene("claim-relative");
    let claim: StageClaim = claim_stage_for(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    std::fs::write(claim.path.as_path(), b".cli-git-worktree-copy-relative").expect("rewrite claim");
    let failure: WorktreeCopyError = reap_abandoned_stages(common.as_path()).expect_err("refused");
    assert!(failure.0.contains("does not name a stage container"), "{}", failure.0);
    remove(root.as_path());
}

/// A claim that names a stage others can read is refused:
/// the directory is not provably monochromatic-git-policy-cli's own.
#[test]
fn a_shared_stage_is_refused_and_kept() {
    let (root, common, destination) = scene("claim-shared");
    let claim: StageClaim = claim_stage_for(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    std::fs::create_dir(claim.container()).expect("container");
    std::fs::set_permissions(claim.container(), std::fs::Permissions::from_mode(0o755)).expect("shared");
    let failure: WorktreeCopyError = reap_abandoned_stages(common.as_path()).expect_err("refused");
    assert!(failure.0.contains("private stage is unsafe"), "{}", failure.0);
    assert!(claim.container().exists());
    remove(root.as_path());
}

/// A symbolic link in the container's place is refused,
/// so a planted link never redirects the removal.
#[test]
fn a_linked_container_is_refused_and_its_target_kept() {
    let (root, common, destination) = scene("claim-linked");
    let target: PathBuf = root.join("target");
    std::fs::create_dir(target.as_path()).expect("target");
    std::fs::write(target.join("keep"), b"keep").expect("keep");
    let claim: StageClaim = claim_stage_for(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    std::os::unix::fs::symlink(target.as_path(), claim.container()).expect("link");
    assert!(reap_abandoned_stages(common.as_path()).is_err());
    assert!(target.join("keep").exists());
    remove(root.as_path());
}

/// A claim file others can read is refused before it is believed.
#[test]
fn a_shared_claim_file_is_refused() {
    let (root, common, destination) = scene("claim-file-shared");
    let claim: StageClaim = claim_stage_for(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    std::fs::set_permissions(claim.path.as_path(), std::fs::Permissions::from_mode(0o644)).expect("shared");
    let failure: WorktreeCopyError = reap_abandoned_stages(common.as_path()).expect_err("refused");
    assert!(failure.0.contains("journal file is unsafe"), "{}", failure.0);
    remove(root.as_path());
}

/// A write cut off before its rename leaves a temporary that names its dead writer;
/// the next pass removes it without a notice,
///  and leaves the temporary of a writer that still runs,
/// which is about to rename it.
#[test]
fn an_unfinished_write_of_a_dead_writer_is_removed_and_a_live_writers_is_kept() {
    let (root, common, _destination) = scene("claim-temporary");
    let journals: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    let dead: String = dead_copy_owner().token();
    let live: String = current_copy_owner().expect("owner").token();
    let leftover: PathBuf = journals.join(format!("abc.stage.{dead}.def.tmp"));
    let journal_leftover: PathBuf = journals.join(format!("abc.{dead}.json.{dead}.def.tmp"));
    let unnamed: PathBuf = journals.join("abc.stage.def.tmp");
    let running: PathBuf = journals.join(format!("abc.stage.{live}.def.tmp"));
    for file in [&leftover, &journal_leftover, &unnamed, &running] {
        std::fs::write(file.as_path(), b"cut off").expect("temporary");
    }
    assert!(reap_abandoned_stages(common.as_path()).expect("reaped").is_empty());
    assert!(!leftover.exists());
    assert!(!journal_leftover.exists());
    assert!(!unnamed.exists(), "a temporary no writer is named in belongs to nobody");
    assert!(running.exists(), "a live writer is about to rename its temporary");
    remove(root.as_path());
}

/// A claim whose owner still runs is an add or a copy in progress:
/// its stage,
/// its records and its claim are left whole.
#[test]
fn a_live_owners_claim_is_left_whole() {
    let (root, common, destination) = scene("claim-live-owner");
    let claim: StageClaim = claim_stage_for_add(common.as_path(), destination.as_path()).expect("claim");
    create_private_directory(claim.container()).expect("container");
    claim.mark_complete().expect("marker");
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(notices.is_empty(), "{notices:?}");
    assert!(claim.container().exists());
    assert!(claim.path.exists());
    assert!(claim.path.with_extension("add").exists());
    assert!(claim.path.with_extension("done").exists());
    claim.release().expect("release");
    remove(root.as_path());
}

/// Without a journal root there is nothing to reap and nothing is created.
#[test]
fn a_repository_without_a_journal_root_reaps_nothing() {
    let (root, common, _destination) = scene("claim-no-root");
    assert!(reap_abandoned_stages(common.as_path()).expect("nothing").is_empty());
    assert!(!journal_root(common.as_path()).exists());
    remove(root.as_path());
}

/// A journal root that cannot be listed because a file stands where a directory belongs is a failure,
///  not an empty root:
/// the claims and the unfinished writes under it were not looked at.
#[test]
fn an_unlistable_journal_root_is_a_failure_and_not_an_empty_one() {
    let (root, common, _destination) = scene("claim-root-not-a-directory");
    std::fs::write(common.join(JOURNAL_PARENT_NAME), b"a file where the directory belongs").expect("file");
    let unlistable: PathBuf = journal_root(common.as_path());
    claim_names(unlistable.as_path()).expect_err("the root cannot be listed");
    judge_stage_owners(common.as_path()).expect_err("the root cannot be listed");
    remove(root.as_path());
}

/// Making a directory durable is fine when it succeeded or the directory was never created,
/// and a fault otherwise.
#[test]
fn only_an_absent_directory_may_skip_its_sync() {
    absent_is_fine(Ok(())).expect("done");
    absent_is_fine(Err(std::io::Error::from(std::io::ErrorKind::NotFound))).expect("absent");
    absent_is_fine(Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied)))
        .expect_err("refused");
}

/// A claim whose container cannot be examined (a file stands in the way) fails and keeps its claim,
///  instead of being taken for a claim whose container was never created.
#[test]
fn a_container_that_cannot_be_examined_keeps_its_claim() {
    let (root, common, _destination) = scene("claim-container-blocked");
    let journals: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    let blocker: PathBuf = root.join("blocker");
    std::fs::write(blocker.as_path(), b"a file").expect("blocker");
    let claim: PathBuf = journals.join(format!("blocked{CLAIM_SUFFIX}"));
    let container: PathBuf = blocker.join(format!("{STAGE_PREFIX}blocked"));
    crate::private_storage::write_private_file(claim.as_path(), container.to_string_lossy().as_bytes())
        .expect("claim");
    reap_abandoned_stages(common.as_path()).expect_err("the container cannot be examined");
    assert!(claim.exists(), "the claim stays for inspection");
    remove(root.as_path());
}

/// Releasing a claim made for an add removes the destination record with it.
#[test]
fn releasing_an_add_claim_removes_the_destination_record() {
    let (root, common, destination) = scene("claim-add-release");
    let claim: StageClaim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    let record: PathBuf = claim.path.with_extension("add");
    assert_eq!(decode_record(std::fs::read_to_string(record.as_path()).expect("record").as_str()), Some(Recorded::Planned(destination.clone())));
    claim.release().expect("release");
    assert!(!record.exists());
    assert!(!journal_root(common.as_path()).read_dir().expect("listing").flatten().any(|entry| { return entry.file_name().to_string_lossy().ends_with(".add"); }));
    remove(root.as_path());
}

/// A process killed between Git and the swap,
/// before the add was bound,
/// leaves a claim,
/// a stage and a destination nothing
///  proves to be the add's:
/// recovery changes nothing,
/// tells the person about the stage and how to see and remove the destination,
/// and keeps
///  the stage's claim while the stage stands so the next command names it again.
#[test]
fn an_unbound_add_deletes_nothing_and_is_reported() {
    let (root, common, destination) = scene("claim-add-interrupted");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    std::fs::write(destination.join(".git"), b"gitdir: elsewhere\n").expect("pointer");
    let claim: StageClaim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    abandoned_stage(claim.container());
    let container: PathBuf = claim.container().to_path_buf();
    // The process dies here: the claim is neither released nor dropped cleanly.
    std::mem::forget(claim);
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(container.exists(), "the stage stays");
    assert_eq!(notices.len(), 2, "{notices:?}");
    std::fs::set_permissions(container.join("payload/locked"), std::fs::Permissions::from_mode(0o700)).expect("writable");
    let shown: String = crate::shell_quote::shell_quote(destination.as_os_str().as_bytes());
    assert!(notices[1].contains("nothing proves it is the add's") && notices[1].contains(format!("git worktree remove --force {shown}").as_str()), "{}", notices[1]);
    assert!(destination.join(".git").exists(), "nothing was deleted");
    let again: Vec<String> = reap_abandoned_stages(common.as_path()).expect("again");
    assert!(again.iter().any(|notice: &String| { return notice.contains("was left by an interrupted"); }), "the stage is named again while it stands: {again:?}");
    remove(root.as_path());
}

/// A real repository with a worktree registered without a checkout,
/// and the add's claim,
/// abandoned as a kill leaves it:
/// bound
///  and recorded when `bound`,
/// as planned only otherwise.
fn killed_add(name: &str, bound: bool, marked: bool) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "repo")).expect("canonical");
    std::fs::write(repo.join("file.txt"), b"content\n").expect("file");
    crate::test_support::git(repo.as_path(), &["add", "file.txt"]);
    crate::test_support::git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    let claim_dir: PathBuf = repo.join(".git");
    let destination_wanted: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical").join("topic");
    let claim: StageClaim = claim_stage_for_add_as(claim_dir.as_path(), destination_wanted.as_path(), &dead_copy_owner()).expect("claim");
    crate::test_support::git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    if bound {
        let found = bind_registration(claim_dir.as_path(), destination.as_path()).expect("bound");
        claim.record_binding(&found.binding).expect("recorded");
    }
    if marked {
        claim.mark_complete().expect("marker");
    }
    // The process dies here: the claim is neither released nor dropped cleanly.
    std::mem::forget(claim);
    return (root, repo.clone(), destination, repo.join(".git/worktrees/topic"));
}

/// What:
///  An add killed before its binding was recorded deletes nothing,
/// and its record stays while the worktree it
///  created is not proven whole.
///  Once the checkout is whole,
/// the copy that never finished is named once and the record goes.
/// Why:
///   Real Git's `--no-checkout` registers the destination before the wrapper writes the binding,
/// so the kill
///  between them leaves a planned record and a pointer,
/// and the pointer is what names the registration the
///  proof reads.
/// The stage is gone and the record alone names the worktree,
/// so one report and a forgetting would leave
///  a registered,
/// `.git`-only worktree that nothing ever mentions again.
/// A checkout that is whole proves only Git's own work:
/// the record exists because the source's ignored state was owed,
/// and nothing about the worktree makes that copy appear later,
/// so it is reported once and its record goes rather than naming a worktree that needs remaking on every command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const first = reap(common); expect(first.length).toBe(1); expect(reap(common)).toEqual(first);
/// git(destination).resetHard(); const finished = reap(common); expect(finished.length).toBe(1);
/// expect(reap(common)).toEqual([]); expect(claimsExist(common)).toBe(false);
/// ```
#[test]
fn an_add_killed_before_its_binding_is_recorded_is_named_until_it_is_finished() {
    let (root, repo, destination, admin) = killed_add("claim-add-pre-binding", false, false);
    let first: Vec<String> = reap_abandoned_stages(repo.join(".git").as_path()).expect("reap");
    assert!(destination.join(".git").exists() && admin.exists(), "nothing of the worktree was changed");
    assert_eq!(first.len(), 1, "{first:?}");
    let shown: String = crate::shell_quote::shell_quote(destination.as_os_str().as_bytes());
    // The two commands are the ones a report of an interrupted add names everywhere, and they are the ones
    //  `worktree_finish_commands.rs` owns, so the expected words come from there: the finish command carries the
    //  worktree guard's hatch, which a checkout that never ran needs.
    let (finish, remove_it): (String, String) = crate::worktree_finish_commands::finish_and_remove(destination.as_path());
    assert!(first[0].contains(finish.as_str()) && first[0].contains(remove_it.as_str()), "the report names what finishes or removes it: {}", first[0]);
    assert!(stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the record stays while the worktree is not whole");
    assert_eq!(reap_abandoned_stages(repo.join(".git").as_path()).expect("again"), first, "the next command says the same");
    // The person finishes the checkout the report names.
    crate::test_support::git(destination.as_path(), &["reset", "--hard"]);
    let finished: Vec<String> = reap_abandoned_stages(repo.join(".git").as_path()).expect("after finishing");
    assert_eq!(finished.len(), 1, "{finished:?}");
    assert!(
        finished[0].contains("the copy of its source's ignored state never finished") && finished[0].contains(shown.as_str()),
        "the worktree that is whole is named for the copy it never received: {}",
        finished[0]
    );
    assert_eq!(reap_abandoned_stages(repo.join(".git").as_path()).expect("the command after that"), Vec::<String>::new(), "it is said once, since nothing on the worktree can make that copy appear");
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "and its record goes");
    remove(root.as_path());
}

/// An add killed after it was bound and before it completed leaves a registered worktree:
/// recovery changes nothing,
/// says
///  what is incomplete and the commands that finish or remove it,
/// and keeps its records until that worktree is whole or
///  gone,
/// so the next command says the same.
#[test]
fn an_add_killed_after_binding_is_reported_and_nothing_is_changed() {
    let (root, repo, destination, admin) = killed_add("claim-add-killed", true, false);
    std::fs::write(destination.join("mine.txt"), b"work started in the incomplete worktree").expect("their work");
    let notices: Vec<String> = reap_abandoned_stages(repo.join(".git").as_path()).expect("reap");
    assert!(destination.join(".git").exists() && destination.join("mine.txt").exists() && admin.exists(), "{notices:?}");
    assert_eq!(notices.len(), 1, "{notices:?}");
    let (finish, remove_it): (String, String) = crate::worktree_finish_commands::finish_and_remove(destination.as_path());
    assert!(notices[0].contains(finish.as_str()) && notices[0].contains(remove_it.as_str()), "{}", notices[0]);
    assert!(stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the records stay while the worktree is not whole");
    assert_eq!(reap_abandoned_stages(repo.join(".git").as_path()).expect("again"), notices, "the next command says the same");
    remove(root.as_path());
}

/// A bound destination that another process replaced with a worktree of its own is left whole with its registration;
/// the
///  report names the path,
/// and nothing is deleted or unregistered.
#[test]
fn a_replaced_destination_is_left_whole_and_reported() {
    let (root, repo, destination, _admin) = killed_add("claim-add-replaced", true, false);
    crate::test_support::git(repo.as_path(), &["worktree", "move", "../topic", "../moved"]);
    crate::test_support::git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic-again", "../topic"]);
    std::fs::write(destination.join("mine.txt"), b"work in the new worktree").expect("their work");
    let notices: Vec<String> = reap_abandoned_stages(repo.join(".git").as_path()).expect("reap");
    assert!(destination.join("mine.txt").exists() && destination.join("file.txt").exists(), "{notices:?}");
    let listed: String = crate::test_support::git_text(repo.as_path(), &["worktree", "list", "--porcelain"]);
    assert_eq!(listed.lines().filter(|line: &&str| { return line.starts_with("worktree "); }).count(), 3, "all three worktrees stay registered: {listed}");
    assert!(root.join("moved").exists(), "the moved worktree stays");
    remove(root.as_path());
}

/// What:
///  An interrupted add that is not proven whole is named by every command that runs the recovery,
///  and its
///  records stay,
/// until the worktree is finished or removed.
/// Why:
///   A person's `git status`,
/// `git commit` or `git log` in the main checkout never reaches the recovery,
/// and an
///  agent reads only its own command's output,
/// so one line about one command's standard error is lost work:
///  the
///  worktree looks ordinary in `git worktree list` and holds only `.git`,
/// and the stage beside it stays.
///   A worktree that is whole,
/// or gone,
/// settles silently and takes the records with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(reap(common)).toEqual(reap(common)); // the same line twice
/// git(destination).resetHard(); expect(reap(common)).toEqual([]);
/// ```
#[test]
fn an_interrupted_add_is_named_by_every_recovery_until_it_is_finished_or_removed() {
    let (root, repo, destination, _admin) = killed_add("claim-add-repeated", true, false);
    let first: Vec<String> = reap_abandoned_stages(repo.join(".git").as_path()).expect("reap");
    assert_eq!(first.len(), 1, "{first:?}");
    let second: Vec<String> = reap_abandoned_stages(repo.join(".git").as_path()).expect("the next command");
    assert_eq!(second, first, "the next command says the same about the same worktree");
    // The person finishes the checkout the way the line tells them to; the copy that never recorded a completion is
    //  named once, and then the worktree is not reported again.
    crate::test_support::git(destination.as_path(), &["reset", "--hard"]);
    let finished: Vec<String> = reap_abandoned_stages(repo.join(".git").as_path()).expect("after finishing");
    assert_eq!(finished.len(), 1, "{finished:?}");
    assert!(finished[0].contains("the copy of its source's ignored state never finished"), "{}", finished[0]);
    assert_eq!(reap_abandoned_stages(repo.join(".git").as_path()).expect("the command after finishing"), Vec::<String>::new(), "a worktree that is whole is not reported again");
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the records go once the worktree is whole");
    remove(root.as_path());
}

/// An interrupted add whose worktree the person removes stops being reported,
/// and its records go.
#[test]
fn an_interrupted_add_that_is_removed_stops_being_reported() {
    let (root, repo, destination, _admin) = killed_add("claim-add-removed", true, false);
    assert_eq!(reap_abandoned_stages(repo.join(".git").as_path()).expect("reap").len(), 1);
    crate::test_support::git(repo.as_path(), &["worktree", "remove", "--force", destination.to_string_lossy().as_ref()]);
    assert_eq!(reap_abandoned_stages(repo.join(".git").as_path()).expect("after removing"), Vec::<String>::new(), "a worktree that is gone is not reported");
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "nothing is left to report");
    remove(root.as_path());
}

/// What:
///  A kill inside the index install leaves the new worktree's `index.lock`,
/// and the report names it with the one
///  step that makes the rest work.
/// Why:
///   `git -C <destination> reset --no-enforce-worktree --hard --no-recurse-submodules` cannot start while that lock stands (Git answers
/// `fatal: Unable to create '.../index.lock': File exists`),
/// so a report that offers only that command offers a
///  remedy that cannot run;
/// and `git status` in the worktree still works,
/// so the worktree looks usable while every
///  write to it fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFileSync(join(admin, 'index.lock'), '')
/// const line = reap(common).find((text) => text.includes('index.lock'))
/// expect(git(destination).resetHard().code).toBe(128)
/// rmSync(join(admin, 'index.lock')); expect(git(destination).resetHard().code).toBe(0)
/// expect(reap(common)).toEqual([])
/// ```
#[test]
fn a_leftover_index_lock_is_named_and_the_remedy_works() {
    let (root, repo, destination, admin) = killed_add("claim-add-index-lock", true, false);
    let lock: PathBuf = admin.join("index.lock");
    std::fs::write(lock.as_path(), b"").expect("the lock a killed Git leaves");
    let notices: Vec<String> = reap_abandoned_stages(repo.join(".git").as_path()).expect("reap");
    let Some(line) = notices.iter().find(|notice: &&String| { return notice.contains("index.lock"); }) else {
        panic!("the report names the lock the remedy has to get past: {notices:?}");
    };
    assert!(line.contains(lock.to_string_lossy().as_ref()), "the exact file is named: {line}");
    // The premise: the command the report offered cannot run while the lock stands.
    let blocked: std::process::Output = crate::test_support::git_output(destination.as_path(), &["reset", "--hard"]);
    assert_eq!(blocked.status.code(), Some(128), "{}", String::from_utf8_lossy(&blocked.stderr));
    assert!(String::from_utf8_lossy(&blocked.stderr).contains("index.lock"), "{}", String::from_utf8_lossy(&blocked.stderr));
    // The remedy the report prints: the named file goes once no Git runs there, and then the checkout finishes.
    std::fs::remove_file(lock.as_path()).expect("the named lock");
    crate::test_support::git(destination.as_path(), &["reset", "--hard"]);
    let remedy: Vec<String> = reap_abandoned_stages(repo.join(".git").as_path()).expect("after the remedy");
    // The checkout the remedy finishes is one the ignored-state copy never reached, and that is said once.
    assert_eq!(remedy.len(), 1, "the remedy finishes the add: {remedy:?}");
    assert!(remedy[0].contains("the copy of its source's ignored state never finished"), "{}", remedy[0]);
    assert_eq!(reap_abandoned_stages(repo.join(".git").as_path()).expect("the command after the remedy"), Vec::<String>::new(), "and nothing is said again");
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "and the records go");
    remove(root.as_path());
}

/// An add that was marked complete before the kill is a whole worktree:
/// recovery keeps it and says nothing.
#[test]
fn an_add_marked_complete_is_kept_by_recovery() {
    let (root, repo, destination, admin) = killed_add("claim-add-marked", true, true);
    std::fs::write(destination.join("file.txt"), b"content\n").expect("checked out");
    let notices: Vec<String> = reap_abandoned_stages(repo.join(".git").as_path()).expect("reap");
    assert!(destination.join("file.txt").exists() && admin.exists(), "{notices:?}");
    assert_eq!(notices, Vec::<String>::new());
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the records are gone");
    remove(root.as_path());
}

/// A lookup that fails other than by absence is not "complete" and not "incomplete":
/// the record stays,
/// the report says that
///  the state is unknown,
/// and nothing is changed.
#[test]
fn a_lookup_that_fails_keeps_the_record_and_says_the_state_is_unknown() {
    let record: PathBuf = PathBuf::from("/records/one.add");
    let denied = || -> std::io::Error { return std::io::Error::from(std::io::ErrorKind::PermissionDenied); };
    let marker_unknown: Settlement = settle_from(record.as_path(), Err(denied()), &mut || { return Ok(String::new()); });
    assert!(marker_unknown.keep && marker_unknown.notice.as_deref().is_some_and(|text: &str| { return text.contains("cannot be checked"); }));
    let record_unknown: Settlement = settle_from(record.as_path(), Ok(false), &mut || { return Err(denied()); });
    assert!(record_unknown.keep && record_unknown.notice.as_deref().is_some_and(|text: &str| { return text.contains("cannot be read"); }));
    let absent: Settlement = settle_from(record.as_path(), Ok(false), &mut || { return Err(std::io::Error::from(std::io::ErrorKind::NotFound)); });
    assert!(!absent.keep && absent.notice.is_none(), "a record that is gone is nothing to report");
    let complete: Settlement = settle_from(record.as_path(), Ok(true), &mut || { return Err(denied()); });
    assert!(!complete.keep && complete.notice.is_none(), "a complete add needs no look at its record");
}

/// A stage that the claim of a worktree add names is reported and never removed while no record proves the object
/// its creation made,
/// whatever stands at the name,
/// and
///  its claim stays so the next command reports it again.
#[test]
fn a_stage_named_by_an_add_is_reported_and_never_removed() {
    let (root, common, destination) = scene("claim-add-stage");
    let claim: StageClaim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    abandoned_stage(claim.container());
    std::fs::write(claim.container().join("payload/foreign"), b"somebody else's directory").expect("foreign content");
    let container: PathBuf = claim.container().to_path_buf();
    std::mem::forget(claim);
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(container.join("payload/foreign").exists(), "the stage stays");
    assert!(notices.iter().any(|notice: &String| { return notice.contains("was not changed") && notice.contains("was left by an interrupted"); }), "{notices:?}");
    assert!(stage_claims_exist(common.as_path()).expect("claims"), "the claim stays while the stage stands");
    assert_eq!(reap_abandoned_stages(common.as_path()).expect("again"), notices, "the next command names the stage again");
    std::fs::set_permissions(container.join("payload/locked"), std::fs::Permissions::from_mode(0o700)).expect("writable");
    remove(root.as_path());
}

/// A release that stops after its first step has removed the record and kept the marker,
/// never the other way round:
/// a
///  marker standing in for one that cannot be removed makes the second step fail.
#[test]
fn releasing_an_add_removes_the_record_before_the_marker() {
    let (root, common, destination) = scene("claim-add-release-order");
    let claim: StageClaim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    let record: PathBuf = claim.path.with_extension("add");
    let marker: PathBuf = claim.path.with_extension("done");
    std::fs::create_dir(marker.as_path()).expect("a marker that cannot be removed as a file");
    assert!(record.exists());
    claim.release().expect_err("the marker cannot be removed");
    assert!(!record.exists(), "the record went first");
    assert!(marker.exists(), "the marker stayed");
    remove(root.as_path());
}

/// The temporaries of an interrupted record or marker write are removed with the unfinished claims.
#[test]
fn unfinished_record_and_marker_writes_are_removed() {
    let (root, common, _destination) = scene("claim-add-temporaries");
    let journals: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    for name in ["a.stage.1.tmp", "a.add.2.tmp", "a.done.3.tmp"] {
        std::fs::write(journals.join(name), b"half").expect("temporary");
    }
    reap_abandoned_stages(common.as_path()).expect("reap");
    assert_eq!(journals.read_dir().expect("listing").count(), 0);
    remove(root.as_path());
}

/// A claim that is gone by the time it is read (its owner released it,
/// or another command reaped it) is skipped,
/// while a
/// claim that is there and names no stage container is still refused.
#[test]
fn a_claim_that_vanished_is_skipped_and_a_malformed_one_is_refused() {
    let (root, common, _destination) = scene("claim-vanished");
    let journals: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    assert_eq!(read_claim(journals.join("gone.stage").as_path()).expect("skipped"), None);
    let malformed: PathBuf = journals.join("malformed.stage");
    crate::private_storage::write_private_file(malformed.as_path(), b"relative/path\n").expect("claim");
    read_claim(malformed.as_path()).expect_err("a claim that names no stage container is refused");
    let owned: PathBuf = journals.join("owned.stage");
    let container: String = format!("/parent/{STAGE_PREFIX}owned");
    crate::private_storage::write_private_file(owned.as_path(), format!("{container}\nowner=7~linux:1~\n").as_bytes()).expect("claim");
    assert_eq!(read_claim(owned.as_path()).expect("read"), Some((PathBuf::from(container), Some(String::from("7~linux:1~")))));
    remove(root.as_path());
}

/// A marker published after a subvolume was swapped into place is atomic and flushes nothing,
/// where the durable marker flushes
///  its file and the directory that holds it.
#[test]
fn an_unsynced_marker_is_published_whole_and_flushes_nothing() {
    let (root, common, destination) = scene("claim-unsynced-marker");
    let claim: StageClaim = claim_stage_for_add(common.as_path(), destination.as_path()).expect("claim");
    take_file_syncs();
    take_directory_syncs();
    claim.mark_complete().expect("durable marker");
    assert!(!take_file_syncs().is_empty() && !take_directory_syncs().is_empty(), "the durable marker flushes (the control that the recorders see it)");
    std::fs::remove_file(claim.path.with_extension("done")).expect("clear");
    claim.mark_complete_unsynced().expect("unsynced marker");
    assert_eq!(std::fs::read(claim.path.with_extension("done")).expect("marker"), b"complete");
    assert_eq!(take_file_syncs(), Vec::<PathBuf>::new());
    assert_eq!(take_directory_syncs(), Vec::<PathBuf>::new());
    let leftovers: Vec<String> = std::fs::read_dir(claim.path.parent().expect("journal root")).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).filter(|name: &String| { return name.contains(".tmp.") || name.ends_with(".partial"); }).collect();
    assert_eq!(leftovers, Vec::<String>::new(), "no temporary stays");
    claim.release().expect("release");
    remove(root.as_path());
}

/// A release after a swap removes the records and flushes nothing.
#[test]
fn an_unsynced_release_removes_the_records_and_flushes_nothing() {
    let (root, common, destination) = scene("claim-unsynced-release");
    let claim: StageClaim = claim_stage_for_add(common.as_path(), destination.as_path()).expect("claim");
    claim.mark_complete_unsynced().expect("marker");
    let (claim_file, record, marker): (PathBuf, PathBuf, PathBuf) = (claim.path.clone(), claim.path.with_extension("add"), claim.path.with_extension("done"));
    assert!(claim_file.exists() && record.exists() && marker.exists());
    take_file_syncs();
    take_directory_syncs();
    claim.release_unsynced().expect("release");
    assert!(!claim_file.exists() && !record.exists() && !marker.exists());
    assert_eq!(take_directory_syncs(), Vec::<PathBuf>::new());
    remove(root.as_path());
}

/// An owner of another PID namespace,
/// and the age that makes its item expired.
fn foreign_owner() -> crate::worktree_copy_owner::CopyOwner {
    return crate::worktree_copy_owner::CopyOwner { pid: 31337, birth_identity: String::from("linux:1"), pid_namespace: Some(String::from("pid:[1]")) };
}

/// Makes a file older than the patience of a foreign owner.
fn age_past_the_patience(path: &Path) {
    std::fs::File::options()
        .write(true)
        .open(path)
        .expect("open")
        .set_modified(std::time::SystemTime::now() - crate::pid_namespace::FOREIGN_OWNER_PATIENCE - std::time::Duration::from_secs(300))
        .expect("age");
}

/// A stage claim file with a chosen name,
/// so the order of the sweep is the order of the names.
fn claim_file(common: &Path, name: &str, container: &Path, owner: &crate::worktree_copy_owner::CopyOwner) -> PathBuf {
    let root: PathBuf = ensure_journal_root(common).expect("root");
    let path: PathBuf = root.join(format!("{name}{CLAIM_SUFFIX}"));
    std::fs::write(path.as_path(), format!("{}\n{OWNER_LINE_PREFIX}{}\n", container.to_string_lossy(), owner.token())).expect("claim");
    std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o600)).expect("private");
    return path;
}

/// A dead owner's claim comes first and an expired foreign claim after it:
/// the sweep refuses before it removes anything (sol-1 round 3,
/// P1 C:
/// the refusal said "nothing was changed" after the first claim's stage was gone).
#[test]
fn an_expired_foreign_claim_stops_the_sweep_before_any_stage_is_removed() {
    let (root, common, destination) = scene("claim-foreign-after-dead");
    let first: PathBuf = destination.parent().expect("parent").join(format!("{STAGE_PREFIX}first"));
    abandoned_stage(first.as_path());
    let dead_claim: PathBuf = claim_file(common.as_path(), "0-dead", first.as_path(), &dead_copy_owner());
    let second: PathBuf = destination.parent().expect("parent").join(format!("{STAGE_PREFIX}second"));
    abandoned_stage(second.as_path());
    let foreign_claim: PathBuf = claim_file(common.as_path(), "1-foreign", second.as_path(), &foreign_owner());
    age_past_the_patience(foreign_claim.as_path());
    let refused: WorktreeCopyError = reap_abandoned_stages(common.as_path()).expect_err("an expired foreign claim stops the sweep");
    assert!(refused.0.contains("31337") && refused.0.contains("rm -- "), "{}", refused.0);
    assert!(first.exists() && dead_claim.exists(), "the dead owner's stage and claim are untouched");
    assert!(second.exists() && foreign_claim.exists());
    for stage in [&first, &second] {
        std::fs::set_permissions(stage.join("payload/locked"), std::fs::Permissions::from_mode(0o700)).expect("make the fixture removable");
    }
    remove(root.as_path());
}

/// The same for the temporaries:
/// a dead writer's file first,
/// an expired foreign writer's after it.
#[test]
fn an_expired_foreign_temporary_stops_the_sweep_before_any_temporary_is_removed() {
    let (root, common, _destination) = scene("temporary-foreign-after-dead");
    let journals: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    let dead_name: String = format!("0-item.{}.1.tmp", dead_copy_owner().token());
    let foreign_name: String = format!("1-item.{}.1.tmp", foreign_owner().token());
    std::fs::write(journals.join(dead_name.as_str()), b"x").expect("dead writer's temporary");
    std::fs::write(journals.join(foreign_name.as_str()), b"x").expect("foreign writer's temporary");
    age_past_the_patience(journals.join(foreign_name.as_str()).as_path());
    let refused: WorktreeCopyError = reap_abandoned_stages(common.as_path()).expect_err("an expired foreign temporary stops the sweep");
    assert!(refused.0.contains("31337"), "{}", refused.0);
    assert!(journals.join(dead_name.as_str()).exists(), "nothing was removed before the refusal");
    remove(root.as_path());
}

/// What:
///  A worktree whose commit has an empty tree is whole once its checkout wrote the empty index,
/// so the report of
///  the unfinished checkout stops,
/// the copy that never recorded its completion is named once,
/// and the records go.
/// Why:
///   Nothing beside Git's pointer is written by a checkout of an empty tree,
/// so a proof that asked for a file
///  beside the pointer could never be satisfied:
/// the report would name the same worktree in every command for ever,
///  and the remedy it prints could not end it.
/// The index's entry count is what tells a tree with nothing to write
///  from a checkout that never ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claim = claimForAdd(common, destination); await git(repo, 'worktree add --no-checkout ...');
/// expect(reap(common).length).toBe(1);
/// git(destination).resetHard(); const said = reap(common); expect(said.length).toBe(1); expect(reap(common)).toEqual([]);
/// ```
#[test]
fn an_add_interrupted_in_a_worktree_of_an_empty_tree_settles_once_its_checkout_is_whole() {
    let root: PathBuf = fixture("claim-add-empty-tree");
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "repo")).expect("canonical");
    crate::test_support::git(repo.as_path(), &["commit", "--allow-empty", "--quiet", "--message=empty tree"]);
    let destination_wanted: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical").join("topic");
    let claim: StageClaim = claim_stage_for_add_as(repo.join(".git").as_path(), destination_wanted.as_path(), &dead_copy_owner()).expect("claim");
    crate::test_support::git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let bound = bind_registration(repo.join(".git").as_path(), destination.as_path()).expect("bound");
    claim.record_binding(&bound.binding).expect("recorded");
    // The process dies here: the claim is neither released nor dropped cleanly.
    std::mem::forget(claim);
    let first: Vec<String> = reap_abandoned_stages(repo.join(".git").as_path()).expect("reap");
    assert_eq!(first.len(), 1, "an add whose checkout never ran is named: {first:?}");
    // The person finishes the checkout the report names: an empty tree writes nothing beside the pointer.
    crate::test_support::git(destination.as_path(), &["reset", "--hard"]);
    assert_eq!(std::fs::read_dir(destination.as_path()).expect("listing").count(), 1, "the worktree holds only Git's pointer");
    let remedy: Vec<String> = reap_abandoned_stages(repo.join(".git").as_path()).expect("after the remedy");
    assert_eq!(remedy.len(), 1, "the remedy ends the report of the unfinished checkout: {remedy:?}");
    assert!(remedy[0].contains("the copy of its source's ignored state never finished"), "{}", remedy[0]);
    assert_eq!(reap_abandoned_stages(repo.join(".git").as_path()).expect("the command after the remedy"), Vec::<String>::new(), "and nothing is said again");
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "and the records go");
    remove(root.as_path());
}

/// What:
///  The awkward path shapes a command printed for a person has to survive:
/// a shell expansion,
/// a substitution,
/// both quotes,
/// a newline,
/// a control character,
/// a leading dash and bytes that are not UTF-8.
/// Why:
///   The instruction is copied into a shell,
/// so a name is read by the shell and not only by the wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = ['$bucket', '`id`', "it's", 'say "hi"', 'a\nb', '\x1b[31mred', '-leading', <0xff>];
/// ```
fn awkward_components() -> Vec<OsString> {
    return vec![
        OsString::from("$bucket"),
        OsString::from("`id`"),
        OsString::from("it's"),
        OsString::from("say \"hi\""),
        OsString::from("a\nb"),
        OsString::from("escape\u{1b}[31mred"),
        OsString::from("-leading-dash"),
        OsString::from_vec(vec![b'n', b'o', b't', 0xff, b'.', b'd']),
    ];
}

/// What:
///  The commands a report of an interrupted add prints,
/// as the shell would read them,
/// naming exactly the
///  destination and the lock file.
/// Why:
///   `git -C <path> reset --no-enforce-worktree --hard --no-recurse-submodules` and `rm <lock>` act on whatever path the shell read,
/// so the text has
///  to name one place under a shell,
/// whatever the directory is called.
/// A report that prints the path the way JSON does leaves
///  `$` and a backquote to the shell.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const path of awkward) { const text = finishInstructions(path); for (const word of words(text)) expect(shellReads(word)).toEqual(word) }
/// ```
#[test]
fn a_report_names_its_path_as_the_shell_reads_it_back() {
    let root: PathBuf = fixture("claim-report-shell-quoting");
    for component in awkward_components() {
        let repository: PathBuf = root.join(&component).join("repo");
        let admin: PathBuf = repository.join(".git/worktrees/topic");
        std::fs::create_dir_all(admin.as_path()).expect("administrative directory");
        std::fs::write(admin.join(INDEX_LOCK), b"").expect("the lock a killed Git leaves");
        let destination: PathBuf = repository.join("topic");
        let text: String = finish_instructions(destination.as_path(), admin.as_path());
        // The finish command names the path before its own flags, so that word ends where the rest of the command begins;
        //  the flags are the ones `worktree_finish_commands.rs` prints.
        let finish_tail: String = format!(" reset {WORKTREE_ENFORCEMENT_ESCAPE_HATCH} --hard");
        // A word that ends the command ends at the closing backtick and the first word the sentence after it writes: a
        //  backtick alone cannot end a word, because one of the names below holds a backtick of its own.
        let expected: Vec<(&str, &str, PathBuf)> = vec![
            ("git -C ", finish_tail.as_str(), destination.clone()),
            ("git worktree remove --force ", "` removes it with everything in it", destination.clone()),
            ("rm ", "` removes it,", admin.join(INDEX_LOCK)),
        ];
        for (marker, end, path) in expected {
            let word: &str = word_between(text.as_str(), marker, end);
            assert_eq!(shell_reads(word), path.as_os_str().as_bytes(), "{marker}{word}{end} in {text}");
        }
        remove(repository.parent().expect("the component's directory"));
    }
    remove(root.as_path());
}

/// What:
///  The one word a report writes between `marker` and `end`,
/// which is the word the person would copy.
/// Why:
///   The check reads what was printed,
/// not what the code meant to print.
fn word_between<'a>(text: &'a str, marker: &str, end: &str) -> &'a str {
    let at: usize = match text.find(marker) {
        Some(found) => found,
        None => panic!("the report prints {marker}: {text}"),
    };
    let after: &str = &text[at + marker.len()..];
    return &after[..match after.find(end) {
        Some(found) => found,
        None => panic!("the report ends the word with {end}: {text}"),
    }];
}

/// What:
///  What `bash` reads a word as,
/// byte for byte.
/// Why:
///   A quoted word is right when a real shell hands the shell's own reader the same bytes;
/// nothing
///  else proves it.
fn shell_reads(word: &str) -> Vec<u8> {
    let printed: std::process::Output = std::process::Command::new("bash").arg("-c").arg(format!("printf %s {word}")).output().expect("start bash");
    return printed.stdout;
}

/// What:
///  The lock-removal instruction removes the lock it names and no other repository's lock at the path a
///  shell would expand the name to.
/// Why:
///   The lock path is printed for the person to run,
/// and a repository may live at a path holding `$bucket`:
///  with the name quoted the way JSON quotes it,
/// the shell expands it and `rm` clears another repository's lock while the
///  named one stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = finishInstructions('/work/$bucket/repo/topic'); execSync(text.match(/`(rm [^`]*)`/)[1], { env: { bucket: 'other' } })
/// ```
#[test]
fn the_lock_removal_instruction_clears_only_the_lock_it_names() {
    let root: PathBuf = fixture("claim-lock-shell-quoting");
    let literal: PathBuf = root.join("$bucket").join("repo");
    let decoy: PathBuf = root.join("other").join("repo");
    for repository in [&literal, &decoy] {
        let admin: PathBuf = repository.join(".git/worktrees/topic");
        std::fs::create_dir_all(admin.as_path()).expect("administrative directory");
        std::fs::write(admin.join(INDEX_LOCK), b"").expect("a lock");
    }
    let admin: PathBuf = literal.join(".git/worktrees/topic");
    let destination: PathBuf = literal.join("topic");
    let text: String = finish_instructions(destination.as_path(), admin.as_path());
    let command: &str = word_between(text.as_str(), "`rm ", "` removes it,");
    let ran: std::process::Output = std::process::Command::new("bash").arg("-c").arg(format!("bucket=other; rm {command}")).output().expect("start bash");
    assert!(ran.status.success(), "{text}: {}", String::from_utf8_lossy(&ran.stderr));
    assert!(!admin.join(INDEX_LOCK).exists(), "the lock the report names is gone: {text}");
    assert!(decoy.join(".git/worktrees/topic/index.lock").exists(), "another repository's lock at the path the shell expands the name to is untouched: {text}");
    remove(root.as_path());
}

/// What:
///  The registration a destination's pointer names is read the way Git reads it,
/// on bytes,
/// so a
///  path holding a newline is whole and a remedy never names another repository.
/// Why:
///   Git reads the rest of the file after `gitdir: `,
/// removing only the newlines at its end
///  (`read_gitfile_gently`,
/// `setup.c`).
/// A reader that takes the first line truncates such a path,
/// and the truncated name
///  can be a directory that exists,
/// another repository's,
/// so the lock-removal step printed for the person would clear
///  the wrong lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(registrationDirectory(destination)).toEqual(root.join('other/.git\nowner/repo/.git/worktrees/topic'));
/// ```
#[test]
fn a_registration_named_across_a_newline_is_read_as_git_reads_it() {
    let root: PathBuf = fixture("claim-pointer-newline");
    let admin: PathBuf = root.join("other/.git\nowner/repo/.git/worktrees/topic");
    std::fs::create_dir_all(admin.as_path()).expect("the registration the pointer names");
    // The truncated name is a directory too, which is what makes a wrong reader look right.
    std::fs::create_dir_all(root.join("other/.git").as_path()).expect("the directory the truncated name reaches");
    let destination: PathBuf = root.join("topic");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    std::fs::write(destination.join(".git"), format!("gitdir: {}\n", admin.display())).expect("pointer");
    assert_eq!(registration_directory(destination.as_path()), Some(admin), "the whole path the pointer names, as Git reads it");
    remove(root.as_path());
}

/// What:
///  A checkout that another process restored only partly is settled as whole,
/// and this is the limit named in `SPEC.md`
///  rather than a mechanism.
/// Why:
///   The proof is what the two directories say:
/// an index,
/// no `index.lock`,
/// and a file beside Git's
///  pointer.
/// A separate process that runs `git read-tree HEAD` in the destination installs the index without
///  writing the files,
/// and one that then copies `a.txt` and stops leaves the destination looking whole although
///  `b.txt` was never written.
/// Closing that window needs to know another process's progress,
/// which no reading of the
///  destination can tell,
/// so the sequence is written down where the proof is described and the record goes:
/// the
///  person who ran the partial restoration is the one who knows it was partial.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git(destination, 'read-tree HEAD'); writeFileSync(join(destination, 'a.txt'), 'a');
/// const said = reap(common); expect(said.length).toBe(1); expect(said[0]).toMatch(/ignored state never finished/);
/// expect(existsSync(join(admin, 'index'))).toBe(true); expect(existsSync(join(destination, 'b.txt'))).toBe(false);
/// ```
#[test]
fn a_checkout_another_process_restored_partly_is_settled_as_whole() {
    let root: PathBuf = fixture("claim-add-partly-restored");
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "repo")).expect("canonical");
    for name in ["a.txt", "b.txt"] {
        std::fs::write(repo.join(name), format!("{name}\n").as_bytes()).expect("a tracked file");
    }
    crate::test_support::git(repo.as_path(), &["add", "a.txt", "b.txt"]);
    crate::test_support::git(repo.as_path(), &["commit", "--quiet", "--message=two files"]);
    let destination_wanted: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical").join("topic");
    let claim: StageClaim = claim_stage_for_add_as(repo.join(".git").as_path(), destination_wanted.as_path(), &dead_copy_owner()).expect("claim");
    crate::test_support::git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let bound = bind_registration(repo.join(".git").as_path(), destination.as_path()).expect("bound");
    claim.record_binding(&bound.binding).expect("recorded");
    let admin: PathBuf = bound.binding.admin.clone();
    // The process dies here: the claim is neither released nor dropped cleanly.
    std::mem::forget(claim);
    // Another process installs the index without the files, then copies one of the two.
    crate::test_support::git(destination.as_path(), &["read-tree", "HEAD"]);
    std::fs::write(destination.join("a.txt"), b"a.txt\n").expect("one file only");
    let notices: Vec<String> = reap_abandoned_stages(repo.join(".git").as_path()).expect("reap");
    // The proof reads an index and a file and settles the checkout; the copy the record was made for never recorded a
    //  completion, so the worktree is named for it once and its records go.
    assert_eq!(notices.len(), 1, "the proof reads an index and a file and settles: {notices:?}");
    assert!(notices[0].contains("the copy of its source's ignored state never finished"), "{}", notices[0]);
    assert!(!destination.join("b.txt").exists(), "the file the other process never copied is the one this limit is about");
    assert!(admin.join(INDEX).exists() && !admin.join(INDEX_LOCK).exists(), "an index with no lock is what the proof reads");
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "and the record goes with the settlement");
    remove(root.as_path());
}
/// A scene whose destination sits under a parent named with bytes that are not text,
/// so the stage container beside it holds those bytes too.
fn scene_with_bytes(name: &str, parent_bytes: &[u8]) -> (PathBuf, PathBuf, PathBuf) {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let (root, common, _destination) = scene(name);
    let parent: PathBuf = root.join(OsStr::from_bytes(parent_bytes));
    std::fs::create_dir_all(parent.as_path()).expect("a parent that is not text");
    return (root, common, parent.join("topic"));
}

/// The claim reader of the release installed before the line of hexadecimal bytes,
/// copied from `read_claim` in `worktree_copy_stage_claim_reaping.rs` at commit 9462675cd8.
/// Why:
///  A record this build writes must still be read by that release when the path is text,
///  and must make it refuse rather than act on the replacement character when it is not.
fn earlier_release_claim_container(text: &str) -> Option<PathBuf> {
    let mut lines = text.lines();
    let container: PathBuf = PathBuf::from(lines.next().unwrap_or_default());
    let stage_named: bool = match container.file_name() {
        Some(name) => name.to_string_lossy().starts_with(STAGE_PREFIX),
        None => false,
    };
    if !(container.is_absolute() && stage_named) {
        return None;
    }
    return Some(container);
}

/// A claim keeps the bytes of a container whose parent is not text,
/// so the reader names the container the claim was made for.
#[test]
fn a_claim_keeps_the_bytes_of_a_container_path_that_is_not_text() {
    let (root, common, destination) = scene_with_bytes("claim-bytes", b"par\xffnt");
    let owner: crate::worktree_copy_owner::CopyOwner = dead_copy_owner();
    let claim: StageClaim = claim_stage_for(common.as_path(), destination.as_path(), &owner).expect("claim");
    assert_eq!(claim.container().parent(), destination.parent(), "the container stands beside the destination");
    let read: (PathBuf, Option<String>) = read_claim(claim.path.as_path()).expect("read").expect("present");
    assert_eq!(read.0, claim.container().to_path_buf(), "the claim names the container it was made for");
    assert_eq!(read.1, Some(owner.token()), "the claim names its owner");
    claim.release().expect("release");
    remove(root.as_path());
}

/// Two destinations that differ only in a byte that is not text keep two claims,
/// and releasing one leaves the other's record.
#[test]
fn two_claims_whose_paths_differ_only_in_bytes_are_two_records() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let (root, common, _destination) = scene("claim-two-byte-names");
    let mut claims: Vec<StageClaim> = Vec::new();
    for bytes in [b"one\xff".as_slice(), b"one\xfe".as_slice()] {
        let parent: PathBuf = root.join(OsStr::from_bytes(bytes));
        std::fs::create_dir_all(parent.as_path()).expect("parent");
        claims.push(claim_stage_for(common.as_path(), parent.join("topic").as_path(), &dead_copy_owner()).expect("claim"));
    }
    let first: StageClaim = claims.remove(0);
    let second: StageClaim = claims.remove(0);
    assert_ne!(first.path, second.path, "two records, two files");
    assert_eq!(read_claim(first.path.as_path()).expect("read").expect("present").0, first.container().to_path_buf());
    assert_eq!(read_claim(second.path.as_path()).expect("read").expect("present").0, second.container().to_path_buf());
    let first_path: PathBuf = first.path.clone();
    let second_path: PathBuf = second.path.clone();
    first.release().expect("release");
    assert!(!first_path.exists(), "its own record goes");
    assert!(second_path.exists(), "the other record stays");
    second.release().expect("release");
    remove(root.as_path());
}

/// An abandoned stage whose path is not text is removed,
/// and a directory standing at the replacement spelling is not touched.
#[test]
fn an_abandoned_stage_that_is_not_text_is_removed_and_no_other_directory_is() {
    let (root, common, destination) = scene_with_bytes("claim-reap-bytes", b"par\xffnt");
    let claim: StageClaim = claim_stage_for(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    abandoned_stage(claim.container());
    let container: PathBuf = claim.container().to_path_buf();
    let replacement: PathBuf = PathBuf::from(container.to_string_lossy().into_owned());
    std::fs::create_dir_all(replacement.as_path()).expect("a directory at the replacement spelling");
    std::fs::set_permissions(replacement.as_path(), std::fs::Permissions::from_mode(0o700)).expect("private decoy");
    std::fs::write(replacement.join("keep"), b"not the stage").expect("marker");
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reaped");
    assert!(!container.exists(), "the stage the claim named is removed");
    assert!(replacement.join("keep").exists(), "the replacement spelling names another directory and is not touched");
    assert!(!claim.path.exists(), "the claim goes with its stage");
    assert_eq!(
        notices,
        vec![format!(
            "monochromatic-git-policy-cli: removed the stage {} that an interrupted ignored-state copy left behind.",
            quote(container.to_string_lossy().as_ref())
        )]
    );
    remove(root.as_path());
}

/// The release now installed reads a claim this build writes for a text path,
/// and refuses one that names other bytes instead of acting on the replacement spelling.
#[test]
fn an_earlier_release_reads_a_text_claim_and_refuses_one_that_names_other_bytes() {
    let (root, common, destination) = scene("claim-earlier-text");
    let text_claim: StageClaim = claim_stage_for(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    let text: String = std::fs::read_to_string(text_claim.path.as_path()).expect("claim file");
    assert_eq!(
        earlier_release_claim_container(text.as_str()),
        Some(text_claim.container().to_path_buf()),
        "a path that is text stays readable by the installed release"
    );
    text_claim.release().expect("release");
    remove(root.as_path());
    let (bytes_root, bytes_common, bytes_destination) = scene_with_bytes("claim-earlier-bytes", b"par\xffnt");
    let bytes_claim: StageClaim = claim_stage_for(bytes_common.as_path(), bytes_destination.as_path(), &dead_copy_owner()).expect("claim");
    let bytes_text: String = std::fs::read_to_string(bytes_claim.path.as_path()).expect("claim file");
    assert_eq!(
        earlier_release_claim_container(bytes_text.as_str()),
        None,
        "the installed release refuses rather than naming the replacement character"
    );
    bytes_claim.release().expect("release");
    remove(bytes_root.as_path());
}

use crate::worktree_copy_interrupted_add_settlement::UNNAMED_CHILD_GRACE;

/// What:
///  A record whose destination is not there yet,
/// naming a Git the add started that still runs,
/// is kept until that Git is gone.
/// Why:
///   The record is written before Git runs,
/// and the command that starts Git names it in the record at once;
///  a command killed while its Git is still making the destination leaves a record whose destination is absent,
/// and settling it there would drop the only sign that the worktree Git then finishes was owed a copy.
///  This is the reading the fault-trace sweeps of `git worktree add` need:
/// the kill stops the wrapper and leaves its Git
///  running.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// claim.rememberRunningChild(destination, gitPid); expect(reap(common)).toEqual([]); expect(claimsExist(common)).toBe(true);
/// kill(git); git.waitForExit(); expect(reap(common)).toEqual([]); expect(claimsExist(common)).toBe(false);
/// ```
#[test]
fn a_record_whose_named_git_still_runs_is_kept_while_its_destination_is_absent() {
    use std::process::{Command, Stdio};
    let (root, common, destination) = scene("claim-child-running");
    assert!(!destination.exists(), "the destination Git has not made yet");
    let claim: StageClaim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    // The Git of a real `git worktree add` that has made nothing yet: `hash-object --stdin` needs no repository, which
    // the gate's container does not have around the test, and stays up until its input ends.
    let mut git: std::process::Child = Command::new(crate::test_support::REAL_GIT)
        .arg("hash-object")
        .arg("--stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("a real Git starts");
    claim.remember_running_child(destination.as_path(), git.id());
    let kept: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(kept.is_empty(), "nothing is reported while the add's Git may still make the destination: {kept:?}");
    assert!(!destination.exists(), "the reaping made nothing");
    assert!(stage_claims_exist(common.as_path()).expect("claims"), "the record stays while its Git still runs");
    // The Git that made nothing ends, and the record settles with nothing to report.
    git.kill().expect("the Git is killed");
    let _reaped = git.wait().expect("the Git is collected");
    let after: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap after the Git ended");
    assert!(after.is_empty(), "a Git that made nothing leaves nothing to finish: {after:?}");
    assert!(!stage_claims_exist(common.as_path()).expect("claims"), "and its record goes");
    remove(root.as_path());
}

/// What:
///  A record that names no Git is kept only while it is young,
/// and settles once it is older than that.
/// Why:
///   The name is written a few calls after the spawn,
/// so a command killed in between leaves a record that names no process
///  while its Git may still be at work;
/// nothing proves that Git gone,
/// so the record is kept for a while and then
///  settles,
/// because a record nothing will ever settle must not stay for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(reap(common)).toEqual([]); expect(claimsExist(common)).toBe(true);
/// ageTheRecord(grace * 2); expect(reap(common)).toEqual([]); expect(claimsExist(common)).toBe(false);
/// ```
#[test]
fn a_record_that_names_no_git_is_kept_while_young_and_settles_after_the_grace() {
    let (root, common, destination) = scene("claim-child-unnamed");
    let claim: StageClaim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    let record: PathBuf = claim.path.with_extension("add");
    let young: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(young.is_empty(), "nothing is reported for a record whose Git may still be starting: {young:?}");
    assert!(stage_claims_exist(common.as_path()).expect("claims"), "a young record that names no Git stays");
    // The record is older than the grace: nothing of it can still be starting.
    let held: std::fs::File = std::fs::OpenOptions::new().write(true).open(record.as_path()).expect("the record");
    held.set_modified(std::time::SystemTime::now() - UNNAMED_CHILD_GRACE * 2).expect("aged");
    let old: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap an old record");
    assert!(old.is_empty(), "an old record that names no Git settles without a word: {old:?}");
    assert!(!stage_claims_exist(common.as_path()).expect("claims"), "and it goes");
    remove(root.as_path());
}

/// What:
///  A destination whose pointer cannot be examined keeps the record and says the state is unknown,
/// instead of
///  reading the refusal as "no pointer" and dropping what names a registered checkout.
/// Why:
///   Recovery decides from `destination/.git` whether a planned add left anything,
/// and the lookup fails for two
///  different reasons:
/// the pointer is not there,
/// which is an answer,
/// or the directory cannot be searched,
/// which is
///  no answer at all.
/// `Path::exists()` maps both to false,
/// so a permission the person or another process holds for a
///  moment deleted the record and the claim of a worktree that is still registered,
/// and no later command named it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chmodSync(destination, 0o000); const settled = settleInterruptedAdd(record); chmodSync(destination, 0o755);
/// expect(settled.keep).toBe(true); expect(settled.notice).toContain('cannot be examined');
/// ```
#[test]
fn a_pointer_that_cannot_be_examined_keeps_the_record() {
    let (root, common, destination) = scene("claim-pointer-unreadable");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    std::fs::write(destination.join(".git"), b"gitdir: /nowhere\n").expect("pointer");
    let claim: StageClaim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    let record: PathBuf = claim.path.with_extension("add");
    std::fs::set_permissions(destination.as_path(), std::fs::Permissions::from_mode(0o000)).expect("no search permission");
    let settled: Settlement = settle_interrupted_add(record.as_path());
    std::fs::set_permissions(destination.as_path(), std::fs::Permissions::from_mode(0o755)).expect("restored");
    assert!(settled.keep, "an answer that could not be read is not an answer of absence");
    assert!(settled.notice.as_deref().is_some_and(|text: &str| { return text.contains("cannot be examined"); }), "{:?}", settled.notice);
    assert!(record.exists(), "the record stays while the state is unknown");
    remove(root.as_path());
}

use crate::worktree_copy_stage_made_object::{MadeObject, made_record_beside_claim, read_made, record_creating_at, record_made, record_made_at};

/// What:
///  Give the read-only directory of an abandoned stage its owner's write permission back,
/// so the fixture teardown removes the fixture.
/// Why:
///   `abandoned_stage` leaves one directory at mode `0500`,
/// the shape an interrupted installation leaves,
/// so a control that keeps the stage must clear that before its own fixture can be removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chmodSync(join(container, 'payload/locked'), 0o700);
/// ```
fn make_removable(container: &Path) {
    let locked: PathBuf = container.join("payload/locked");
    if locked.exists() {
        std::fs::set_permissions(locked.as_path(), std::fs::Permissions::from_mode(0o700)).expect("writable");
    }
}

/// What:
///  The record of the object a stage's creation made,
/// written here by hand,
/// so a control of the reaping fails while nothing writes one.
/// Why:
///   The reaping reads a record,
/// never the process that made the folder;
/// writing the record here keeps the control about the
///  reaping and the shape of the record,
/// and the writer is controlled on its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFile(claim.replace(/\.stage$/, '.made'), `stage=${hex(container)}\nobject=${object}\nwritten=${seconds}.${nanos}\n`);
/// ```
fn made_record_by_hand(claim: &Path, container: &Path, object: crate::fs_beneath::ObjectId, written: std::time::SystemTime) -> PathBuf {
    let path: PathBuf = made_record_beside_claim(claim);
    let since: std::time::Duration = written.duration_since(std::time::SystemTime::UNIX_EPOCH).expect("after the epoch");
    let text: String = format!(
        "stage={}\nobject={}\nwritten={}.{}\n",
        crate::path_bytes_in_records::hex_of_path(container),
        object.encode(),
        since.as_secs(),
        since.subsec_nanos()
    );
    std::fs::write(path.as_path(), text.as_bytes()).expect("the record");
    return path;
}

/// What:
///  A stage a killed add left beside a destination,
/// with the record of the object its creation made,
/// and the claim never released.
/// Why:
///   This is the shape `s14_snapshot_add_kill.ts` found:
/// a claim,
/// a destination record and a stage holding a
///  copy of the checkout's state,
/// with the process that made it gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claim = await claimStageForAdd(common, destination); await abandonStage(claim.container); await recordMade(...); forgetClaim();
/// ```
fn killed_add_with_its_object_recorded(name: &str) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let (root, common, destination) = scene(name);
    let claim: StageClaim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    abandoned_stage(claim.container());
    let container: PathBuf = claim.container().to_path_buf();
    let object = crate::worktree_add_binding::path_identity(container.as_path()).expect("the object's identity");
    let record: PathBuf = made_record_by_hand(claim.path.as_path(), container.as_path(), object, std::time::SystemTime::now());
    // The add's own record is aged past the grace a record that names no Git is kept for, so a control does not wait a
    // minute for the settlement the next command makes.
    age_past_the_patience(claim.path.with_extension("add").as_path());
    // The process dies here: neither the claim nor its records are released.
    std::mem::forget(claim);
    return (root, common, container, record);
}

/// A stage an add left is removed when the record of the object its creation made proves that object still stands there,
/// and the person reads what went.
#[test]
fn the_stage_of_an_add_is_removed_when_its_record_proves_the_object() {
    let (root, common, container, record) = killed_add_with_its_object_recorded("claim-add-proved");
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(!container.exists(), "the stage the record proves is removed");
    assert!(!record.exists(), "the record of the object goes with the stage's claim once the stage is gone");    assert_eq!(
        notices.iter().filter(|notice: &&String| { return notice.contains("removed the stage"); }).count(),
        1,
        "one line says what was removed: {notices:?}"
    );
    assert!(
        notices[0].contains("removed the stage") && notices[0].contains("an interrupted `git worktree add` left behind"),
        "the line names the stage and what left it: {}",
        notices[0]
    );
    assert!(!stage_claims_exist(common.as_path()).expect("claims"), "the claim goes once the add settles");
    remove(root.as_path());
}

/// A stage the record of the object does not prove is reported and never removed,
/// and its claim stays so the next command reports it again.
#[test]
fn a_stage_the_record_of_its_object_does_not_prove_is_reported_and_kept() {
    let (root, common, destination) = scene("claim-add-unproved");
    let claim: StageClaim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    abandoned_stage(claim.container());
    let container: PathBuf = claim.container().to_path_buf();
    std::mem::forget(claim);
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(container.join("payload/locked/data").exists(), "nothing in the stage is removed");
    assert!(
        notices.iter().any(|notice: &String| {
            return notice.contains("was not changed") && notice.contains("no record of the object its creation made");
        }),
        "{notices:?}"
    );
    assert!(stage_claims_exist(common.as_path()).expect("claims"), "the claim stays while the stage stands");
    make_removable(container.as_path());
    remove(root.as_path());
}

/// A record that does not read as this release writes it proves nothing,
/// so the folder it names is kept.
#[test]
fn a_record_of_an_object_that_does_not_read_keeps_the_stage() {
    let (root, common, container, record) = killed_add_with_its_object_recorded("claim-add-record-damaged");
    std::fs::write(record.as_path(), b"object=not a record of this release\n").expect("a damaged record");
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(container.exists(), "the stage stays");
    assert!(
        notices.iter().any(|notice: &String| { return notice.contains("was not changed") && notice.contains("does not read"); }),
        "{notices:?}"
    );
    make_removable(container.as_path());
    remove(root.as_path());
}

/// Another object at the container's name is not the one the record proves,
/// so it is left alone.
#[test]
fn a_stage_another_object_took_the_name_of_is_left_alone() {
    let (root, common, container, _record) = killed_add_with_its_object_recorded("claim-add-replaced");
    let elsewhere: PathBuf = container.parent().expect("parent").join("moved-away");
    std::fs::rename(container.as_path(), elsewhere.as_path()).expect("the wrapper's object moved away");
    crate::private_storage::create_private_directory(container.as_path()).expect("another directory at the name");
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(container.exists() && elsewhere.exists(), "neither directory is removed");
    assert!(
        notices.iter().any(|notice: &String| {
            return notice.contains("was not changed") && notice.contains("not the one the record of its creation made");
        }),
        "{notices:?}"
    );
    make_removable(container.as_path());
    make_removable(elsewhere.as_path());
    remove(root.as_path());
}

/// A file a person adds to the folder after the kill is not something the record accounts for,
/// so the whole folder is kept and the file is named.
#[test]
fn a_stage_holding_something_the_record_cannot_account_for_is_kept() {
    let (root, common, container, record) = killed_add_with_its_object_recorded("claim-add-person");
    // A person works in the folder after the kill: the entry is born after the recorded instant.
    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::write(container.join("mine.txt"), b"a person's only copy").expect("a person's file");
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(container.join("mine.txt").exists(), "nothing of the person's is removed");
    assert!(record.exists(), "the record stays with the folder it cannot account for");
    assert!(
        notices.iter().any(|notice: &String| { return notice.contains("was not changed") && notice.contains("mine.txt"); }),
        "the line names the entry: {notices:?}"
    );
    make_removable(container.as_path());
    remove(root.as_path());
}

/// A person's file below the document folder of the source keeps the folder too:
/// only the trash below it is the wrapper's.
#[test]
fn a_stage_holding_a_persons_file_below_the_document_folder_is_kept() {
    let (root, common, container, _record) = killed_add_with_its_object_recorded("claim-add-person-below-claude");
    std::thread::sleep(std::time::Duration::from_millis(20));
    let notes: PathBuf = container.join(".claude/notes");
    crate::private_storage::create_private_directory_all(notes.as_path()).expect("a person's directory beside the trash");
    std::fs::write(notes.join("mine.txt"), b"a person's only copy").expect("a person's file");
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(notes.join("mine.txt").exists(), "nothing of the person's is removed");
    assert!(
        notices.iter().any(|notice: &String| { return notice.contains("was not changed") && notice.contains(".claude/notes"); }),
        "the line names the entry: {notices:?}"
    );
    make_removable(container.as_path());
    remove(root.as_path());
}

/// The trash a snapshot sets its left-out directories aside in,
/// and Git's own files,
/// are the wrapper's own even though the wrapper makes them after the record is written.
#[test]
fn a_stage_the_wrapper_wrote_later_is_still_accounted_for() {
    let (root, common, container, _record) = killed_add_with_its_object_recorded("claim-add-wrappers-own");
    std::thread::sleep(std::time::Duration::from_millis(20));
    let trash: PathBuf = container.join(".claude/cli-git-trash").join("0".repeat(40));
    crate::private_storage::create_private_directory_all(trash.as_path()).expect("the trash the leave-out makes");
    std::fs::write(trash.join("set-aside"), b"a directory the leave-out moved").expect("a set-aside entry");
    std::fs::write(container.join(".git"), b"gitdir: elsewhere\n").expect("the pointer the ladder writes");
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(!container.exists(), "the stage holds only what the wrapper wrote and is removed");
    assert!(notices.iter().any(|notice: &String| { return notice.contains("removed the stage"); }), "{notices:?}");
    remove(root.as_path());
}

/// Two commands doing the same sweep at the same moment remove one stage once and neither fails.
#[test]
fn two_sweeps_of_one_dead_stage_remove_it_once_and_neither_fails() {
    let (root, common, container, record) = killed_add_with_its_object_recorded("claim-add-two-sweeps");
    let sweeps = std::thread::scope(|scope| {
        let one = scope.spawn(|| { return reap_abandoned_stages(common.as_path()); });
        let other = scope.spawn(|| { return reap_abandoned_stages(common.as_path()); });
        return (one.join().expect("no panic"), other.join().expect("no panic"));
    });
    let removals: Vec<String> = sweeps.0.expect("the first sweep finishes");
    let other_removals: Vec<String> = sweeps.1.expect("the second sweep finishes");
    assert!(!container.exists(), "the stage is gone");
    assert!(!record.exists(), "its record is gone");
    assert!(!stage_claims_exist(common.as_path()).expect("claims"), "its claim is gone");
    for notice in removals.iter().chain(other_removals.iter()) {
        assert!(
            !notice.contains("cannot") && !notice.contains("could not"),
            "neither sweep reports a failure: {notice}"
        );
    }
    assert!(
        removals.iter().chain(other_removals.iter()).any(|notice: &String| { return notice.contains("removed the stage"); }),
        "the stage was removed, so one of them says so: {removals:?} {other_removals:?}"
    );
    remove(root.as_path());
}

/// The record of an object whose claim nothing keeps is swept,
/// so a kill between the two removals leaves nothing behind for ever.
#[test]
fn the_record_of_an_object_whose_claim_is_gone_is_swept() {
    let (root, common, container, record) = killed_add_with_its_object_recorded("claim-add-orphan");
    let claim: PathBuf = journal_root(common.as_path()).join(format!(
        "{}.stage",
        container.file_name().expect("name").to_string_lossy().trim_start_matches(STAGE_PREFIX)
    ));
    std::fs::remove_file(claim.as_path()).expect("a claim another command reaped");
    reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(!record.exists(), "the record names no folder once its claim is gone");
    make_removable(container.as_path());
    remove(root.as_path());
}

/// The record of the object a stage's creation made names the container,
/// that object's identity and the instant after which nothing of the wrapper's is in the folder,
/// and a release takes it with the claim.
#[test]
fn recording_the_object_writes_the_container_the_identity_and_the_instant() {
    let (root, common, destination) = scene("claim-made-record");
    let claim: StageClaim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    abandoned_stage(claim.container());
    let container: PathBuf = claim.container().to_path_buf();
    let object = crate::worktree_add_binding::path_identity(container.as_path()).expect("identity");
    let written: std::time::SystemTime = std::time::SystemTime::now();
    record_made_at(common.as_path(), container.as_path(), object, written).expect("record");
    let record: PathBuf = made_record_beside_claim(claim.path.as_path());
    assert!(record.exists(), "the record stands beside the claim");
    assert_eq!(
        read_made(claim.path.as_path()),
        Some(crate::worktree_copy_stage_made_object::MadeObject { container: container.clone(), object: Some(object), written }),
        "the record reads back whole"
    );
    record_made_at(common.as_path(), container.as_path(), object, written + std::time::Duration::from_secs(1)).expect("a raised cut-off");
    assert_eq!(
        read_made(claim.path.as_path()).map(|made| { return made.object; }),
        Some(Some(object)),
        "a raised cut-off keeps the object the record proves"
    );
    claim.release().expect("release");
    assert!(!record.exists(), "the release takes the record with the claim");
    make_removable(container.as_path());
    remove(root.as_path());
}

/// A container name no claim reserves gets no record,
/// so nothing is written into the journal root for a stage this wrapper never claimed.
#[test]
fn a_stage_name_no_claim_reserves_gets_no_record() {
    let (root, common, _destination) = scene("claim-made-unreserved");
    let stage: PathBuf = common.parent().expect("parent").join("stage");
    std::fs::create_dir(stage.as_path()).expect("a stage of another shape");
    let object = crate::worktree_add_binding::path_identity(stage.as_path()).expect("identity");
    record_made(common.as_path(), stage.as_path(), object).expect("nothing to write");
    assert!(!journal_root(common.as_path()).exists(), "no record is written for it");
    remove(root.as_path());
}

/// What:
///  A real repository whose main checkout is a btrfs subvolume,
/// a real `git worktree add` that registered a destination without checking it out,
/// and the snapshot the ladder takes there,
/// with the process that made it gone.
/// Why:
///   This is the shape a kill inside `git worktree add` leaves:
/// the stage is a whole snapshot of the checkout,
///  credentials and `.git` included,
/// and the claim and the record of its object are the only things that name it.
///  `s14_snapshot_add_kill.ts` of the worktree-lifecycle audit found two of ten kills leaving it,
/// holding a hundred
///  megabytes of credentials and a copy of `.git`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { root, repo, claim } = await killedSnapshotAdd('claim-add-snapshot'); expect(isSubvolume(claim.container)).toBe(true);
/// ```
fn killed_snapshot_add(name: &str) -> Option<(PathBuf, PathBuf, PathBuf)> {
    let root: PathBuf = crate::fs_btrfs_test_support::btrfs_fixture(name)?;
    let repo: PathBuf = crate::worktree_ladder_test_support::subvolume_repository(root.as_path());
    let common: PathBuf = repo.join(".git");
    let destination: PathBuf = root.join("topic");
    let plan = crate::worktree_ladder_test_support::plan_to(destination.clone());
    let claim: StageClaim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    crate::test_support::git(repo.as_path(), &["worktree", "add", "--no-checkout", "-b", "topic", "../topic"]);
    let bound = crate::worktree_add_binding::bind_registration(common.as_path(), destination.as_path()).expect("bound");
    let source = crate::worktree_snapshot_source::source_or_dirty(&git_context(), &crate::worktree_ladder_test_support::main_identity(repo.as_path())).expect("source");
    let container: PathBuf = claim.container().to_path_buf();
    crate::worktree_snapshot::take_snapshot(
        &source,
        &plan,
        container.as_path(),
        &bound.admin,
        bound.binding.pointer.as_slice(),
        &crate::fixture_content_decision::content_environment(),
        &crate::worktree_snapshot_ahead::AheadService::nothing(),
    )
    .expect("the snapshot");
    assert!(crate::fs_kind::is_subvolume_root(container.as_path()).expect("probe"), "the stage is a subvolume");
    assert!(container.join("ignored/pkg/index.js").is_file(), "the stage holds the snapshot of the ignored state");
    assert!(container.join(".git").is_file(), "the stage holds the pointer the ladder writes in place of the source's .git");
    // The process dies here: neither the claim nor its records are released.
    std::mem::forget(claim);
    return Some((root, common, container));
}

/// A stage a real killed snapshot add left is removed once the record of the object its creation made proves it,
/// and the ignored state it held goes with it.
#[test]
fn btrfs_the_stage_a_killed_snapshot_add_left_is_removed() {
    let Some((root, common, container)) = killed_snapshot_add("claim-add-snapshot-killed") else { return };
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(!container.exists(), "the whole snapshot is gone, credentials and `.git` with it");
    assert!(
        notices.iter().any(|notice: &String| { return notice.contains("removed the stage") && notice.contains("`git worktree add`"); }),
        "{notices:?}"
    );
    assert!(root.join("topic/.git").exists(), "the destination Git registered is not touched");
    // The add's binding is a registration with no checkout, which nothing proves whole, so its records keep naming the
    // worktree for the person and nothing names the stage again.
    let again: Vec<String> = reap_abandoned_stages(common.as_path()).expect("a second sweep");
    assert!(again.iter().all(|notice: &String| { return !notice.contains("removed the stage"); }), "nothing is removed twice: {again:?}");
    assert!(!container.exists(), "the stage stays gone");
    remove(root.as_path());
}

/// The record a command writes before it creates the container names the container and the instant and no identity,
/// and a release reads it back as that shape.
#[test]
fn a_record_written_before_the_container_exists_names_no_object() {
    let (root, common, destination) = scene("claim-made-creation-record");
    let claim: StageClaim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    abandoned_stage(claim.container());
    let container: PathBuf = claim.container().to_path_buf();
    let written: std::time::SystemTime = std::time::SystemTime::now();
    record_creating_at(common.as_path(), container.as_path(), written).expect("the record of the creation");
    assert_eq!(
        read_made(claim.path.as_path()),
        Some(MadeObject { container: container.clone(), object: None, written }),
        "the record names the container and the instant, and no object"
    );
    let record: PathBuf = made_record_beside_claim(claim.path.as_path());
    claim.release().expect("release");
    assert!(!record.exists(), "the release takes the record with the claim");
    make_removable(container.as_path());
    remove(root.as_path());
}

/// A record whose lines are not the shape this release writes proves nothing,
/// so a folder it names is kept.
#[test]
fn a_record_with_a_repeated_or_unknown_line_proves_nothing() {
    let (root, common, destination) = scene("claim-made-damaged");
    let claim: StageClaim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    abandoned_stage(claim.container());
    let container: PathBuf = claim.container().to_path_buf();
    let record: PathBuf = made_record_beside_claim(claim.path.as_path());
    let stage_line: String = format!("stage={}\n", crate::path_bytes_in_records::hex_of_path(container.as_path()));
    for text in [
        format!("{stage_line}{stage_line}written=1.0\n"),
        format!("{stage_line}written=1.0\nunknown=1\n"),
        format!("{stage_line}object=not-an-object\nwritten=1.0\n"),
        format!("{stage_line}"),
    ] {
        std::fs::write(record.as_path(), text.as_bytes()).expect("a damaged record");
        assert_eq!(read_made(claim.path.as_path()), None, "a record this release does not write proves nothing: {text}");
    }
    std::mem::forget(claim);
    make_removable(container.as_path());
    remove(root.as_path());
}

/// What:
///  A stage a kill left in the instant between the wrapper's creation of it and the record of the object it made:
///  a claim,
/// the record of the destination,
/// a whole snapshot at the stage name,
/// and the record of the creation with no object in it.
/// Why:
///   This is the state the kill matrix found at point 13
/// (`snapshot_stage_window.sh`),
/// and nothing but the shape of
///  the folder and the instant the record holds can prove the wrapper made it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { root, common, repo, claim } = await killedSnapshotAddBeforeItsRecord(name); expect(isSubvolume(claim.container)).toBe(true);
/// ```
fn killed_snapshot_add_before_its_record(name: &str) -> Option<(PathBuf, PathBuf, PathBuf, PathBuf)> {
    let root: PathBuf = crate::fs_btrfs_test_support::btrfs_fixture(name)?;
    let repo: PathBuf = crate::worktree_ladder_test_support::subvolume_repository(root.as_path());
    let common: PathBuf = repo.join(".git");
    let destination: PathBuf = root.join("topic");
    let source = crate::worktree_snapshot_source::source_or_dirty(&git_context(), &crate::worktree_ladder_test_support::main_identity(repo.as_path())).expect("source");
    let claim: StageClaim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    crate::test_support::git(repo.as_path(), &["worktree", "add", "--no-checkout", "-b", "topic", "../topic"]);
    let container: PathBuf = claim.container().to_path_buf();
    // The record comes first, and then the creation: the kill follows the ioctl and comes before the identity is read.
    record_creating_at(common.as_path(), container.as_path(), std::time::SystemTime::now()).expect("the record of the creation");
    crate::fs_btrfs_ioctl::snapshot_create_at(source.root.as_path(), container.as_path()).expect("the snapshot the killed command made");
    assert!(crate::fs_kind::is_subvolume_root(container.as_path()).expect("probe"), "the stage is a snapshot");
    assert!(container.join("ignored/pkg/index.js").is_file(), "the stage holds the source's ignored state");
    // The process dies here: neither the claim nor its records are released, and no object was ever recorded.
    std::mem::forget(claim);
    return Some((root, common, container, repo));
}

/// The snapshot a kill left between the wrapper's creation of the stage and the record of its object is settled by
/// the next command,
/// not kept for ever.
#[test]
fn btrfs_the_snapshot_created_before_its_record_is_removed() {
    let Some((root, common, container, _repo)) = killed_snapshot_add_before_its_record("claim-add-window") else { return };
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(!container.exists(), "the whole snapshot is gone, and no command waits for a person's rm");
    assert!(
        notices.iter().any(|notice: &String| { return notice.contains("removed the stage") && notice.contains("`git worktree add`"); }),
        "{notices:?}"
    );
    let again: Vec<String> = reap_abandoned_stages(common.as_path()).expect("a second sweep");
    assert!(again.iter().all(|notice: &String| { return !notice.contains("removed the stage"); }), "nothing is removed twice: {again:?}");
    remove(root.as_path());
}

/// A plain directory at the stage name of a record that names no object is kept and named:
/// the shape is what proves the wrapper made the folder.
#[test]
fn a_plain_directory_is_not_taken_for_the_snapshot_a_kill_left() {
    let Some((root, common, container, _repo)) = killed_snapshot_add_before_its_record("claim-add-window-plain") else { return };
    // Another party's own directory at the name the record holds.
    crate::fs_btrfs_ioctl::subvolume_destroy_at(container.as_path()).expect("the snapshot goes");
    crate::private_storage::create_private_directory(container.as_path()).expect("a person's directory");
    std::fs::write(container.join("notes.txt"), b"a person's only copy").expect("a person's file");
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(container.join("notes.txt").exists(), "nothing of the person's is removed");
    assert!(
        notices.iter().any(|notice: &String| { return notice.contains("was not changed") && notice.contains("is not a snapshot"); }),
        "{notices:?}"
    );
    std::fs::remove_dir_all(container.as_path()).expect("the control's directory");
    remove(root.as_path());
}

/// A snapshot that holds an entry born after the instant the record names is kept and named:
/// what a person wrote there after the wrapper died exists nowhere else.
#[test]
fn a_snapshot_holding_a_newer_entry_is_kept() {
    let Some((root, common, container, _repo)) = killed_snapshot_add_before_its_record("claim-add-window-newer") else { return };
    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::write(container.join("mine.txt"), b"a person's only copy").expect("a person's file");
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(container.join("mine.txt").exists(), "nothing of the person's is removed");
    assert!(
        notices.iter().any(|notice: &String| { return notice.contains("was not changed") && notice.contains("mine.txt"); }),
        "{notices:?}"
    );
    std::fs::remove_file(container.join("mine.txt")).expect("the control's file");
    let settled: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap again");
    assert!(!container.exists(), "the snapshot is settled once the person's entry is gone");
    assert!(settled.iter().any(|notice: &String| { return notice.contains("removed the stage"); }), "{settled:?}");
    remove(root.as_path());
}

/// A snapshot a person made before the record is not the stage that record's creation made:
/// every entry of it predates the instant the record holds,
/// and its shape is a btrfs snapshot and nothing more,
/// so the identity the record never gained is the only thing that would have told the two apart.
/// Why the control reads this way:
///  a reaping that takes the shape for the identity deletes the person's snapshot and the only copy of a file in it.
#[test]
fn a_different_snapshot_at_the_stage_name_is_not_taken_for_the_one_a_kill_left() {
    let Some(root) = crate::fs_btrfs_test_support::btrfs_fixture("claim-add-window-replacement") else { return };
    let repo: PathBuf = crate::worktree_ladder_test_support::subvolume_repository(root.as_path());
    std::fs::write(repo.join("mine.txt"), b"the only copy of this file").expect("the file a person keeps");
    let backup: PathBuf = root.join("a-persons-snapshot");
    crate::fs_btrfs_ioctl::snapshot_create_at(repo.as_path(), backup.as_path()).expect("the person's own snapshot");
    std::fs::remove_file(repo.join("mine.txt")).expect("the backup now holds the only copy");
    // Every entry of the backup predates the instant the record below names.
    std::thread::sleep(std::time::Duration::from_millis(20));
    let common: PathBuf = repo.join(".git");
    let destination: PathBuf = root.join("topic");
    let claim: StageClaim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_copy_owner()).expect("claim");
    crate::test_support::git(repo.as_path(), &["worktree", "add", "--no-checkout", "-b", "topic", "../topic"]);
    let container: PathBuf = claim.container().to_path_buf();
    let source = crate::worktree_snapshot_source::source_or_dirty(&git_context(), &crate::worktree_ladder_test_support::main_identity(repo.as_path())).expect("source");
    record_creating_at(common.as_path(), container.as_path(), std::time::SystemTime::now()).expect("the record of the creation");
    crate::fs_btrfs_ioctl::snapshot_create_at(source.root.as_path(), container.as_path()).expect("the snapshot the killed add made");
    // The kill: the claim stays, and the record never gained an identity.
    std::mem::forget(claim);
    let interrupted: PathBuf = root.join("the-add-stage-moved-aside");
    std::fs::rename(container.as_path(), interrupted.as_path()).expect("the add's own stage is moved aside");
    std::fs::rename(backup.as_path(), container.as_path()).expect("the person's snapshot is put at the stage name");
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(container.join("mine.txt").exists(), "the person's snapshot and the only copy of the file survive: {notices:?}");
    assert!(interrupted.exists(), "the stage the add made and another command moved aside is not this claim's to remove");
    assert!(
        notices.iter().any(|notice: &String| { return notice.contains("was not changed") && notice.contains("names no object"); }),
        "the folder is kept and the reason is said: {notices:?}"
    );
    crate::fs_btrfs_ioctl::subvolume_destroy_at(interrupted.as_path()).expect("the add's snapshot");
    crate::fs_btrfs_ioctl::subvolume_destroy_at(container.as_path()).expect("the control's snapshot");
    remove(root.as_path());
}
