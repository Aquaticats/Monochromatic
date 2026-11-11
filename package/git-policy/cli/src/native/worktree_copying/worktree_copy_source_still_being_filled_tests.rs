//! Controls for the readiness of a copy's source:
//! a journal or an add's claim whose owner still runs holds the copy back,
//! a dead owner's item and an item of another destination do not,
//! and the line the person reads names both worktrees and the commands that copy later.

use super::*;
use crate::fs_beneath::ObjectId;
use crate::test_support::{dead_copy_owner, fixture, git, git_context, remove, repository};
use crate::worktree_add_binding::Binding;
use crate::worktree_copy_journal::{create_journal_for, ensure_journal_root};
use crate::worktree_copy_journal_encoding_and_validation::PendingJournal;
use crate::worktree_copy_owner::{CopyOwner, current_copy_owner};
use crate::worktree_copy_selection::{CopySelection, STAGE_PREFIX};
use crate::worktree_copy_stage::{new_stage_container, stage_ignored_snapshot};
use crate::worktree_copy_stage_claim::{StageClaim, claim_stage, claim_stage_for_add_as};
use crate::worktree_copy_unfilled_record::remember_unfilled;

/// A repository with an ignored `cache` directory and one linked worktree `source`,
/// returned with the canonical source root,
/// the canonical main checkout and the canonical common directory.
fn repository_with_a_source(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let main: PathBuf = repository(root, "main");
    std::fs::write(main.join(".gitignore"), b"cache/\n").expect("gitignore");
    std::fs::create_dir_all(main.join("cache")).expect("cache");
    std::fs::write(main.join("cache/data"), b"payload").expect("data");
    let source: PathBuf = root.join("source");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "source", source.to_str().expect("utf8")]);
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("common");
    return (
        std::fs::canonicalize(source).expect("canonical source"),
        std::fs::canonicalize(main).expect("canonical main"),
        common,
    );
}

/// A staged journal that names `destination` and `owner`,
/// as a copy that stopped after staging leaves it.
fn journal_of(common: &Path, main: &Path, destination: &Path, owner: &CopyOwner) -> PendingJournal {
    let snapshot = stage_ignored_snapshot(
        &git_context(),
        main,
        new_stage_container(destination).expect("container").as_path(),
        &[],
        &CopySelection::MainCheckout,
    )
    .expect("staged");
    return create_journal_for(common, destination, &snapshot, owner).expect("journal");
}

/// An add's claim on `destination`,
/// which reserves a stage name and records the destination beside it.
fn add_claim_of(common: &Path, destination: &Path, owner: &CopyOwner) -> StageClaim {
    return claim_stage_for_add_as(common, destination, owner).expect("claim");
}

/// The one claim file a repository holds.
fn only_claim(common: &Path) -> PathBuf {
    let root: PathBuf = journal_root(common);
    let names: Vec<String> = claim_names(root.as_path()).expect("claims");
    assert_eq!(names.len(), 1, "{names:?}");
    return root.join(names.first().expect("one claim").as_str());
}

/// A live journal of the source itself is a copy in progress:
/// the source is still being filled.
#[cfg(unix)]
#[test]
fn a_journals_live_owner_makes_the_source_busy() {
    let root: PathBuf = fixture("source-readiness-live-journal");
    let (source, main, common) = repository_with_a_source(root.as_path());
    let journal: PendingJournal = journal_of(common.as_path(), main.as_path(), source.as_path(), &current_copy_owner().expect("owner"));
    assert_eq!(
        source_is_still_being_filled(common.as_path(), source.as_path()).expect("judged"),
        Some(UnfinishedCopy::Running)
    );
    assert!(journal.path.exists(), "the running copy's journal is left whole");
    remove(root.as_path());
}

/// An add's claim on the source is the add that is filling it:
/// the source is still being filled.
#[cfg(unix)]
#[test]
fn an_adds_claim_of_a_live_owner_makes_the_source_busy() {
    let root: PathBuf = fixture("source-readiness-live-claim");
    let (source, _main, common) = repository_with_a_source(root.as_path());
    let claim: StageClaim = add_claim_of(common.as_path(), source.as_path(), &current_copy_owner().expect("owner"));
    assert_eq!(
        source_is_still_being_filled(common.as_path(), source.as_path()).expect("judged"),
        Some(UnfinishedCopy::Running)
    );
    assert!(claim.container().file_name().expect("name").to_string_lossy().starts_with(STAGE_PREFIX));
    remove(root.as_path());
}

/// A dead owner's journal shows its install did not finish,
/// so the source it names is still being filled,
/// and copying from it would carry a half-filled set.
#[cfg(unix)]
#[test]
fn a_journals_dead_owner_makes_the_source_busy() {
    let root: PathBuf = fixture("source-readiness-dead-journal");
    let (source, main, common) = repository_with_a_source(root.as_path());
    journal_of(common.as_path(), main.as_path(), source.as_path(), &dead_copy_owner());
    assert_eq!(
        source_is_still_being_filled(common.as_path(), source.as_path()).expect("judged"),
        Some(UnfinishedCopy::Interrupted)
    );
    remove(root.as_path());
}

/// A dead owner's claim shows the add that would have filled the source did not finish,
/// so the source is still being filled.
#[cfg(unix)]
#[test]
fn an_adds_claim_of_a_dead_owner_makes_the_source_busy() {
    let root: PathBuf = fixture("source-readiness-dead-claim");
    let (source, _main, common) = repository_with_a_source(root.as_path());
    add_claim_of(common.as_path(), source.as_path(), &dead_copy_owner());
    assert_eq!(
        source_is_still_being_filled(common.as_path(), source.as_path()).expect("judged"),
        Some(UnfinishedCopy::Interrupted)
    );
    remove(root.as_path());
}

/// A copy hands its stage over from its claim to its journal while a reader looks:
/// the reader still sees the source being filled,
/// because it reads the claims before the journals,
/// and the writer publishes the journal before it releases the claim.
#[cfg(unix)]
#[test]
fn a_copy_that_hands_over_between_the_two_scans_is_still_seen() {
    let root: PathBuf = fixture("source-readiness-handoff");
    let (source, main, common) = repository_with_a_source(root.as_path());
    let mut held: Option<StageClaim> = Some(add_claim_of(common.as_path(), source.as_path(), &current_copy_owner().expect("owner")));
    let mut handed_over: bool = false;
    let judged = source_is_still_being_filled_around(common.as_path(), source.as_path(), &mut || {
        // The writer's own order: the journal names the stage before the claim goes. The reader listed the claim names
        // before this runs, so the claim it is about to read is gone and only the journal can be seen.
        let _published: PendingJournal = journal_of(common.as_path(), main.as_path(), source.as_path(), &current_copy_owner().expect("owner"));
        held.take().expect("the claim is still this reader's").release().expect("released");
        handed_over = true;
    })
    .expect("judged");
    assert!(handed_over, "the handoff ran between the listing of the claims and the reading of them");
    assert_eq!(judged, Some(UnfinishedCopy::Running), "the handoff leaves the source busy");
    assert_eq!(claim_names(journal_root(common.as_path()).as_path()).expect("claims"), Vec::<String>::new(), "the claim is gone");
    assert_eq!(read_pending_journals(common.as_path()).expect("journals").len(), 1, "the journal is the record that is left");
    remove(root.as_path());
}

/// Only the source's own copy makes it busy:
/// a live copy into another worktree of the same repository says nothing about it.
#[cfg(unix)]
#[test]
fn a_copy_into_another_worktree_does_not_make_this_source_busy() {
    let root: PathBuf = fixture("source-readiness-other-destination");
    let (source, main, common) = repository_with_a_source(root.as_path());
    let other: PathBuf = root.join("other");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "other", other.to_str().expect("utf8")]);
    let other_canonical: PathBuf = std::fs::canonicalize(other).expect("canonical other");
    journal_of(common.as_path(), main.as_path(), other_canonical.as_path(), &current_copy_owner().expect("owner"));
    add_claim_of(common.as_path(), other_canonical.as_path(), &current_copy_owner().expect("owner"));
    assert_eq!(source_is_still_being_filled(common.as_path(), source.as_path()).expect("judged"), None);
    assert_eq!(
        source_is_still_being_filled(common.as_path(), other_canonical.as_path()).expect("judged"),
        Some(UnfinishedCopy::Running)
    );
    remove(root.as_path());
}

/// A copy's own claim records no destination,
/// so a copy that is staging into some worktree never holds another copy back.
#[cfg(unix)]
#[test]
fn a_copys_own_claim_never_makes_a_source_busy() {
    let root: PathBuf = fixture("source-readiness-copy-claim");
    let (source, _main, common) = repository_with_a_source(root.as_path());
    claim_stage(common.as_path(), source.as_path()).expect("claim");
    assert_eq!(source_is_still_being_filled(common.as_path(), source.as_path()).expect("judged"), None);
    remove(root.as_path());
}

/// The record an add writes once Git registered it names the same destination as the planned record does.
#[cfg(unix)]
#[test]
fn the_bound_record_names_the_destination_too() {
    let root: PathBuf = fixture("source-readiness-bound-record");
    let (source, _main, common) = repository_with_a_source(root.as_path());
    let claim: StageClaim = add_claim_of(common.as_path(), source.as_path(), &dead_copy_owner());
    claim
        .record_binding(&Binding {
            destination: source.clone(),
            destination_id: ObjectId { device: 1, inode: 2, birth: None },
            admin: common.join("worktrees/source"),
            admin_id: ObjectId { device: 3, inode: 4, birth: None },
            pointer: b"gitdir: elsewhere\n".to_vec(),
        })
        .expect("bound record");
    assert_eq!(claim_destination(only_claim(common.as_path()).as_path()), Some(source));
    remove(root.as_path());
}

/// A copy that ended before it was whole leaves its record naming the worktree it did not fill,
/// and no journal and no claim stand beside it,
/// so the record is the only sign that this source holds less than its own source ignores.
/// A copy staged from it would carry that state on with a success line and no word,
/// so the record holds the copy back.
#[cfg(unix)]
#[test]
fn a_source_a_copy_ended_unfilled_is_not_staged_from() {
    let root: PathBuf = fixture("source-readiness-unfilled");
    let (source, _main, common) = repository_with_a_source(root.as_path());
    let journal: PathBuf = ensure_journal_root(common.as_path()).expect("journal root").join("0123456789abcdef.json");
    let identity: ObjectId = crate::worktree_add_binding::path_identity(source.as_path()).expect("the source's identity");
    let owner: String = dead_copy_owner().token();
    remember_unfilled(journal.as_path(), source.as_path(), identity, Path::new("/elsewhere/main"), Some(owner.as_str())).expect("remembered");
    assert_eq!(
        source_is_still_being_filled(common.as_path(), source.as_path()).expect("judged"),
        Some(UnfinishedCopy::LeftUnfilled),
        "the record of the copy that ended is the only sign the source is not whole"
    );
    remove(root.as_path());
}

/// A record that names a worktree a copy ended in, standing where that worktree is not,
/// says nothing about the worktree at that path now:
/// the worktree it described is gone or was replaced,
/// so it holds no copy back and its own command settles it.
#[cfg(unix)]
#[test]
fn a_record_of_a_replaced_worktree_does_not_make_the_source_busy() {
    let root: PathBuf = fixture("source-readiness-unfilled-replaced");
    let (source, _main, common) = repository_with_a_source(root.as_path());
    let journal: PathBuf = ensure_journal_root(common.as_path()).expect("journal root").join("0123456789abcdef.json");
    let owner: String = dead_copy_owner().token();
    let replaced: ObjectId = ObjectId { device: 1, inode: 2, birth: None };
    remember_unfilled(journal.as_path(), source.as_path(), replaced, Path::new("/elsewhere/main"), Some(owner.as_str())).expect("remembered");
    assert_eq!(source_is_still_being_filled(common.as_path(), source.as_path()).expect("judged"), None);
    remove(root.as_path());
}

/// The line names the worktree that was made,
/// the source that is still being filled,
/// and the commands that have the copy made later.
#[test]
fn the_notice_names_both_worktrees_and_how_to_copy_later() {
    assert_eq!(
        source_still_being_filled_notice(Path::new("/repo/topic"), Path::new("/repo/source"), UnfinishedCopy::Running),
        String::from(
            "monochromatic-git-policy-cli: \"/repo/topic\" was made, but it received no ignored-state copy, because \"/repo/source\" is itself being filled by another `git worktree add`, so its ignored files are only partly there. Once that add has finished, `git worktree remove \"/repo/topic\"` and run this `git worktree add` line again to have the ignored state copied; `git worktree remove` leaves a worktree that holds changes alone, so save them first.\n"
        )
    );
}

/// A source a copy ended in is not one the next command finishes:
/// the person removes that worktree and makes it again,
/// so the line names that route and not a command that will never come.
#[test]
fn the_notice_of_a_source_a_copy_ended_in_names_the_route_that_settles_it() {
    assert_eq!(
        source_still_being_filled_notice(Path::new("/repo/topic"), Path::new("/repo/source"), UnfinishedCopy::LeftUnfilled),
        String::from(
            "monochromatic-git-policy-cli: \"/repo/topic\" was made, but it received no ignored-state copy, because the ignored-state copy of \"/repo/source\" ended before it was whole, and \"/repo/source\" holds less than its own source ignores until the record of that copy is settled. Once that worktree is whole, `git worktree remove \"/repo/topic\"` and run this `git worktree add` line again to have the ignored state copied; `git worktree remove` leaves a worktree that holds changes alone, so save them first.\n"
        )
    );
}
