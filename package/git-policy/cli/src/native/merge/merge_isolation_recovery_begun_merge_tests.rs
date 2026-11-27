//! What:
//!  Whether a saved merge counts as begun:
//!  an independent commit of the same content,
//!  a note a clean filter hides,
//!  a directory holding an untracked file,
//!  a mode changed before the judgement or a mode a repository ignores,
//!  a merge that settled completely without reading the worktree,
//!  and one that installed nothing while a command on another branch goes on.
//! Why:
//!  Reading the whole worktree to answer this would stop commands that only look,
//!  so the answer comes from the index and the paths the merge names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(installed).toBe(false); expect(commandExited).toBe(0);
//! ```

use super::super::*;
use super::fixtures::{
    A_CONTROLS_MERGE,
    World,
    aside_names,
    dead_owner,
    deferred_record_of,
    directory_to_file_world,
    one_path_merge_world,
    place,
    record_of,
    world,
};
use super::readings::{
    judged_changes,
    snapshot,
    the_action_for,
};
use crate::merge_isolation_recovery_records::recover_interrupted_merges;
/// The install of a merge's index entries and files,
/// which a placement of the merge's own bytes makes.
use crate::merge_isolation_recovery_steps::install_entries;
use crate::test_support::{
    git,
    git_text,
    remove,
};
use std::path::PathBuf;

/// What:
///  An independent commit of the same content does not make a saved merge look installed,
///  so the command goes on.
/// Why:
///  One index entry or file matching the merge's side was taken as evidence that the merge began,
/// so a lock holder who
///  independently committed the same content for one path made every later command stop on a saved merge that had
///  installed nothing
///  (a cross-vendor review of the seventh round,
/// its sixth finding).
/// The merge changes two other paths that hold nothing of it,
/// so only every path matching is an install.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = deferredRecord(); write('f.txt', mergeContent); git add f.txt; commit('independent'); const outcomes = await recover(...); expect(outcomes[0].action).toBe('saved-branch-moved'); expect(exists(record)).toBe(false);
/// ```
#[test]
fn an_unrelated_commit_of_the_same_content_does_not_make_a_saved_merge_began() {
    let world: World = world("merge-recovery-same-content-elsewhere");
    let record: PathBuf = deferred_record_of(
        &world,
        "37373737-0000-4000-8000-000000000037",
        "refs/cli-git/kept/37",
    );
    // The holder of the index independently commits the content this merge would give `f.txt`, and nothing else of it.
    std::fs::write(world.repo.join("f.txt"), b"f.txt feature\n").expect("the same content");
    git(world.repo.as_path(), &["add", "--", "f.txt"]);
    git(
        world.repo.as_path(),
        &["commit", "--quiet", "--message=independent", "--", "f.txt"],
    );
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the command goes on");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::SavedBranchMoved,
        "nothing of the merge was installed"
    );
    assert!(!record.exists(), "the record only goes");
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        moved,
        "the branch is the independent commit"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "refs/cli-git/kept/37"]),
        world.new_oid,
        "the saved merge is kept"
    );
    remove(world.parent.as_path());
}

/// What:
///  A file that carries bytes a clean filter drops is not replaced by the merge,
///  and its bytes stay where they are.
/// Why:
///  Recovery judged such a file by its filtered hash,
/// which equals the branch's object,
/// so the note the person saved was
///  accepted as the branch's own content and replaced by the merge,
/// and the aside that held its only copy was
///  removed
///  (a cross-vendor review of the eighth round,
/// its second finding).
/// The aside exists to keep the person's bytes,
/// so it compares them raw.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cleanFilter('strip', 'sed /^NOTE:/d'); write('f.txt', 'f.txt base\nNOTE: mine\n'); const (changes, judged) = judge(world); await expect(installEntries(...)).rejects.toMatch(/f.txt/); expect(read('f.txt')).toEqual('f.txt base\nNOTE: mine\n');
/// ```
#[test]
fn a_note_a_filter_hides_is_not_replaced_and_stays_where_it_is() {
    let world: World = world("merge-recovery-filtered-note-kept");
    std::fs::write(world.repo.join(".gitattributes"), b"f.txt filter=strip\n")
        .expect("the attribute file");
    git(
        world.repo.as_path(),
        &["config", "filter.strip.clean", "sed /^NOTE:/d"],
    );
    // The person's note is in the file before recovery judges it, and the filter hides it.
    let note: &[u8] = b"f.txt base\nNOTE: locally saved data\n";
    std::fs::write(world.repo.join("f.txt"), note).expect("the person's note");
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let judged: &Judged = &judgements[changes
        .iter()
        .position(|change| {
            return change.path == b"f.txt";
        })
        .expect("the path")];
    assert_eq!(
        judged.standing,
        Standing::Old,
        "the filtered content is the branch's, which is why the note is invisible to it"
    );
    let mut said: Vec<u8> = Vec::new();
    let refused = install_entries(
        &place(&world),
        world.repo.as_path(),
        changes.as_slice(),
        judgements.as_slice(),
        false,
        A_CONTROLS_MERGE,
        &mut said,
    )
    .expect_err("a file whose raw bytes are not the branch's is not the merge's to replace");
    assert!(
        refused.error.0.contains("f.txt"),
        "the path is named: {}",
        refused.error.0
    );
    assert_eq!(
        std::fs::read(world.repo.join("f.txt")).expect("the file"),
        note,
        "the note stays where it is"
    );
    assert_eq!(
        aside_names(world.repo.as_path()),
        Vec::<String>::new(),
        "nothing is left at a private name"
    );
    remove(world.parent.as_path());
}

/// What:
///  A merge that is installed and then touched by another command still counts as installed,
///  so the command stops.
/// Why:
///  The installed state was read as every path holding the merge,
/// so one path another command edited and staged made a
///  half-installed merge look uninstalled,
/// the refusal was deferred,
/// and a later commit could record the other half
///  as ordinary staged work with no merge parent
///  (a cross-vendor review of the eighth round,
/// its third finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged, {updateWorktree: true}); deferredRecord(); write('f.txt', edit); git add f.txt; await expect(recover(...)).rejects.toMatch(/f.txt|partly installed/);
/// ```
#[test]
fn an_installed_merge_another_command_touched_still_stops_the_command() {
    let world: World = world("merge-recovery-installed-then-touched");
    git(
        world.repo.as_path(),
        &[
            "read-tree",
            "-m",
            "-u",
            world.old_oid.as_str(),
            world.new_oid.as_str(),
        ],
    );
    let record: PathBuf = deferred_record_of(
        &world,
        "38383838-0000-4000-8000-000000000038",
        "refs/cli-git/kept/38",
    );
    // Another command edits and stages `f.txt`; `dir/new.txt` and `gone.txt` still hold the merge.
    std::fs::write(world.repo.join("f.txt"), b"another command's edit\n").expect("the edit");
    git(world.repo.as_path(), &["add", "--", "f.txt"]);
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect_err("an installed merge stops the command");
    assert!(
        refused.0.contains("f.txt") || refused.0.contains("partly installed"),
        "the state is named: {}",
        refused.0
    );
    assert!(record.exists(), "the record is kept");
    remove(world.parent.as_path());
}

/// What:
///  A one-path merge whose content the branch's holder committed independently is not read as installed,
///  so the command goes on instead of being refused for ever.
/// Why:
///  With one changed path,
/// every entry and every file matching the merge is exactly what that independent
///  commit leaves,
/// so the branch being unchanged is what tells an install from it
///  (a cross-vendor review of the eighth round,
/// its fourth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = onePathMergeWorld(); const record = deferredRecord(); write('f.txt', mergeContent); git add f.txt; commit('independent'); const outcomes = await recover(...); expect(outcomes[0].action).toBe('saved-branch-moved');
/// ```
#[test]
fn an_independent_commit_of_a_one_path_merge_does_not_block_the_command() {
    let world: World = one_path_merge_world("merge-recovery-one-path-independent");
    let record: PathBuf = deferred_record_of(
        &world,
        "39393939-0000-4000-8000-000000000039",
        "refs/cli-git/kept/39",
    );
    std::fs::write(world.repo.join("f.txt"), b"f.txt feature\n")
        .expect("the merge's content, committed independently");
    git(world.repo.as_path(), &["add", "--", "f.txt"]);
    git(
        world.repo.as_path(),
        &["commit", "--quiet", "--message=independent", "--", "f.txt"],
    );
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the command goes on");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::SavedBranchMoved,
        "nothing of the merge was installed"
    );
    assert!(!record.exists(), "the record only goes");
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        moved,
        "the branch is the independent commit"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "refs/cli-git/kept/39"]),
        world.new_oid,
        "the saved merge is kept"
    );
    remove(world.parent.as_path());
}

/// What:
///  A directory that holds a file the merge does not delete stops the merge before anything is written.
/// Why:
///  The directory rule read a directory that had a tracked deletion beneath it as the branch's own,
/// without looking at
///  what else was in it,
/// so an untracked file there was met after the merge's index entries were written and its child was
///  taken,
/// and every later command failed trying to replace the same nonempty directory
///  (a cross-vendor review of the eighth round,
/// its fifth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = directoryToFileWorld(); write('dir/notes.txt', mine); const record = deferredRecord(); const before = snapshot(world); await expect(recover(...)).rejects.toMatch(/dir/); expect(snapshot(world)).toEqual(before);
/// ```
#[test]
fn a_directory_holding_an_untracked_file_refuses_before_anything_is_written() {
    let world: World = directory_to_file_world("merge-recovery-directory-extra-file");
    std::fs::write(world.repo.join("dir/notes.txt"), b"my own notes\n")
        .expect("the person's untracked file");
    let record: PathBuf = deferred_record_of(
        &world,
        "40404040-0000-4000-8000-000000000040",
        "refs/cli-git/kept/40",
    );
    let before = snapshot(&world);
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("a saved merge that wrote nothing lets the command go on");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::DeferredRefused,
        "it is refused, not installed"
    );
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(text.contains("dir"), "the path is named: {text}");
    assert_eq!(
        snapshot(&world),
        before,
        "nothing was written and nothing was taken"
    );
    assert_eq!(
        std::fs::read(world.repo.join("dir/notes.txt")).expect("the person's file"),
        b"my own notes\n"
    );
    assert!(
        world.repo.join("dir/child.txt").is_file(),
        "the child is where it was"
    );
    assert!(record.exists(), "the record is kept");
    remove(world.parent.as_path());
}

/// What:
///  A merge that already landed is settled without reading the worktree,
///  so a file that cannot be read does not keep its leftover record alive.
/// Why:
///  The worktree was read before the branch was resolved,
/// so a merge whose branch already named the merge commit
///  and whose changed path had become unreadable stopped every command,
/// and nothing could clear the record
///  because the failure came before the check that would have removed it
///  (a cross-vendor review of the eighth round,
/// its seventh finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged, {updateWorktree: true}); await updateRef('main', merged, old); const record = recordOf(deadOwner); gitConfig('filter.broken.required', 'true'); const outcomes = await recover(...); expect(outcomes[0].action).toBe('already-landed'); expect(exists(record)).toBe(false);
/// ```
#[test]
fn a_completed_merge_is_settled_without_reading_the_worktree() {
    let world: World = world("merge-recovery-landed-unreadable");
    // The merge landed: the checkout, the index and the branch all hold it, and only the record was left behind.
    git(
        world.repo.as_path(),
        &[
            "read-tree",
            "-m",
            "-u",
            world.old_oid.as_str(),
            world.new_oid.as_str(),
        ],
    );
    git(
        world.repo.as_path(),
        &[
            "update-ref",
            "refs/heads/main",
            world.new_oid.as_str(),
            world.old_oid.as_str(),
        ],
    );
    let record: PathBuf = record_of(
        &world,
        dead_owner("41414141-0000-4000-8000-000000000041"),
        Some("refs/heads/main"),
    );
    // A clean filter the path requires now fails, so any read of the worktree would stop the command.
    std::fs::write(world.repo.join(".gitattributes"), b"f.txt filter=broken\n")
        .expect("the attribute file");
    git(
        world.repo.as_path(),
        &["config", "filter.broken.clean", "false"],
    );
    git(
        world.repo.as_path(),
        &["config", "filter.broken.required", "true"],
    );
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("a landed merge needs no worktree read");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::AlreadyLanded);
    assert!(!record.exists(), "the leftover record goes");
    remove(world.parent.as_path());
}

/// What:
///  A merge that was saved on a side ref,
/// and whose first record names no side ref,
///  does not stop a command on another
///  branch when nothing of the merge is in the checkout.
/// Why:
///  The branch check refused the record without a side ref,
/// and only a saved record's refusal was let past,
/// so the first
///  record of a merge that installed nothing stopped every command in a worktree another command had switched
/// to a branch naming the same commit,
/// and no later command could clear it
///  (a cross-vendor review of the ninth round,
/// its fifth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const original = recordOf(deadOwner, { sideRef: undefined }); const saved = deferredRecord(); git switch -c other; const outcomes = await recover(...); expect(outcomes.find(o => o.record === original).action).toBe('discarded'); expect(exists(original)).toBe(false); expect(exists(saved)).toBe(true);
/// ```
#[test]
fn a_merge_that_installed_nothing_does_not_stop_a_command_on_another_branch() {
    let world: World = world("merge-recovery-other-branch");
    // The merge met a lock it could not clear: its first record names no side ref and a second one names the kept commit.
    let original: PathBuf = record_of(
        &world,
        dead_owner("56565656-0000-4000-8000-000000000056"),
        Some("refs/heads/main"),
    );
    let saved: PathBuf = deferred_record_of(
        &world,
        "57575757-0000-4000-8000-000000000057",
        "refs/cli-git/kept/57",
    );
    let before = snapshot(&world);
    // Another command switches the worktree to a branch that names the same commit and installs nothing.
    git(
        world.repo.as_path(),
        &["switch", "--quiet", "--create", "other"],
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "other"]),
        world.old_oid,
        "the other branch names the old commit"
    );
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("nothing of the merge is in the checkout, so the command goes on");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(
        the_action_for(outcomes.as_slice(), original.as_path()),
        Some(MergeRecoveryAction::Discarded),
        "the record that names no side ref only goes: {text}"
    );
    assert!(!original.exists(), "the record nothing can finish is gone");
    assert!(
        saved.exists(),
        "the saved merge's record is kept for the branch it is for"
    );
    assert_eq!(
        the_action_for(outcomes.as_slice(), saved.as_path()),
        Some(MergeRecoveryAction::DeferredRefused),
        "the saved merge is named and the command goes on: {text}"
    );
    assert!(
        text.contains("refs/heads/other"),
        "the branch the worktree has is named: {text}"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.old_oid,
        "the merged branch did not move"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["symbolic-ref", "HEAD"]),
        "refs/heads/other",
        "the worktree keeps the other command's branch"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "refs/cli-git/kept/57"]),
        world.new_oid,
        "the saved merge is kept"
    );
    assert_eq!(snapshot(&world), before, "nothing of the checkout changed");
    remove(world.parent.as_path());
}

/// What:
///  An executable bit a person set before recovery judged the file keeps the saved merge from replacing it.
/// Why:
///  The judgement compared content alone,
/// so a file whose bytes were the branch's and whose mode was the person's was
///  read as untouched,
/// the take's identity check passed because it read the mode the person had already set,
/// and
///  the merge's file replaced it with its own mode:
/// the bit was erased without a word
///  (a cross-vendor review of the ninth round,
/// its seventh finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = world(); const record = deferredRecord(); chmod('f.txt', 0o755); const outcomes = await recover(...); expect(outcomes[0].action).toBe('deferred-refused'); expect(mode('f.txt')).toEqual(0o755); expect(read('f.txt')).toEqual(base);
/// ```
#[cfg(unix)]
#[test]
fn a_file_whose_mode_changed_before_the_judgement_is_not_replaced() {
    use std::os::unix::fs::PermissionsExt;
    let world: World = world("merge-recovery-mode-before-judging");
    let record: PathBuf = deferred_record_of(
        &world,
        "58585858-0000-4000-8000-000000000058",
        "refs/cli-git/kept/58",
    );
    // The person makes the file executable while the merge waits for the index lock.
    let location: PathBuf = world.repo.join("f.txt");
    std::fs::set_permissions(location.as_path(), std::fs::Permissions::from_mode(0o755))
        .expect("the person's executable bit");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("a saved merge that wrote nothing lets the command go on");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::DeferredRefused,
        "the merge is refused, not installed: {text}"
    );
    assert!(text.contains("f.txt"), "the path is named: {text}");
    assert_eq!(
        std::fs::read(location.as_path()).expect("the file"),
        b"f.txt base\n",
        "the person's file is where it was"
    );
    let mode: u32 = std::fs::metadata(location.as_path())
        .expect("the file")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o755, "the person's executable bit is kept");
    assert!(record.exists(), "the record is kept");
    assert_eq!(
        git_text(world.repo.as_path(), &["status", "--porcelain=v1"]),
        " M f.txt",
        "the checkout is exactly as the person left it"
    );
    remove(world.parent.as_path());
}

/// What:
///  A mode difference in a repository that ignores file modes does not stop the merge.
/// Why:
///  `core.fileMode` is false where the filesystem cannot keep the executable bit,
/// and Git then reads no mode
///  difference at all;
/// a judgement that refused one there would stop every recovery of a merge that touches an
///  executable path on such a filesystem,
/// which is the class of failure this route exists to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = world(); const record = deferredRecord(); gitConfig('core.filemode', 'false'); chmod('f.txt', 0o755); const outcomes = await recover(...); expect(outcomes[0].action).toBe('completed'); expect(read('f.txt')).toEqual(mergeContent);
/// ```
#[cfg(unix)]
#[test]
fn a_mode_a_repository_ignores_does_not_stop_the_merge() {
    use std::os::unix::fs::PermissionsExt;
    let world: World = world("merge-recovery-mode-ignored");
    let record: PathBuf = deferred_record_of(
        &world,
        "59595959-0000-4000-8000-000000000059",
        "refs/cli-git/kept/59",
    );
    git(world.repo.as_path(), &["config", "core.filemode", "false"]);
    std::fs::set_permissions(
        world.repo.join("f.txt").as_path(),
        std::fs::Permissions::from_mode(0o755),
    )
    .expect("a mode the repository ignores");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the command goes on");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::Completed,
        "the merge lands, as Git would land it: {text}"
    );
    assert!(
        !record.exists(),
        "the record only goes once the merge landed"
    );
    assert_eq!(
        std::fs::read(world.repo.join("f.txt")).expect("the merge's file"),
        b"f.txt feature\n",
        "the merge's content is in place"
    );
    remove(world.parent.as_path());
}
