//! What:
//!  Controls for the notice a relink given up on leaves,
//!  and for the lines a later command in that worktree shows of it.
//! Why:
//!  The notice is the only word the person gets about a worktree whose package links still name the checkout it was copied
//!  from,
//!  so every control here names the words,
//!  the reason they carry,
//!  the commands in them,
//!  the Git directory they are written in,
//!  and that one read takes the notice away.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! recordNotice(worktreeGitDir, id, wordsFor(worktree, entry, 3, 'pnpm: exit status 1')); assert(takeNotices(worktreeGitDir) === [expected]);
//! ```

use super::*;
use crate::package_repair_relink::{RELINK_REFERENCE, SearchPathOfTheAdd, item_of, object_name_of};
use crate::test_support::{fixture, remove};
use crate::work_item::{WorkItem, WorkKind};

/// An entry naming both tools of this release,
/// rooted at the worktree the notice is about.
fn entry_rooted_at(worktree: &Path) -> RelinkEntry {
    return RelinkEntry { root: worktree.to_path_buf(), search_path: SearchPathOfTheAdd::Unset, tools: vec![RelinkTool::Pnpm, RelinkTool::Toolchain] };
}

/// A record of one relink,
/// as the add writes it.
fn record_of(worktree: &Path, common_dir: &Path) -> WorkItem {
    let entry: RelinkEntry = entry_rooted_at(worktree);
    return item_of(common_dir, worktree, &entry).expect("the record");
}

/// What:
///  The words name the worktree,
///  say a tool answered and with what,
///  and name both commands the record asked for.
/// Why:
///  A person who reads only this line must be able to tell what failed and what finishes the job,
///  without a journal or a record,
///  and a count of tries alone names nothing they can change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(words.includes(worktree) && words.includes('pnpm: exit status 1') && words.includes('pnpm --dir '));
/// ```
#[test]
fn the_words_name_the_worktree_what_failed_and_every_command() {
    let root: PathBuf = fixture("relink-notice-words");
    let worktree: PathBuf = root.join("the worktree");
    let words: String = words_for(worktree.as_path(), &entry_rooted_at(worktree.as_path()), 3, Some("pnpm: exit status 1: ERR_PNPM_META_FETCH_FAIL"));
    assert!(words.contains(worktree.to_string_lossy().as_ref()), "the words do not name the worktree: {words}");
    assert!(words.contains("The last try failed with: pnpm: exit status 1: ERR_PNPM_META_FETCH_FAIL."), "the words do not say what failed: {words}");
    assert!(words.contains("ERR_PNPM_META_FETCH_FAIL. Running the same command again"), "the tool's own words run into the next sentence: {words}");
    assert!(words.contains("3 times"), "the words do not say how many times the service tried: {words}");
    assert!(words.contains("pnpm --dir "), "the words do not name the pnpm command: {words}");
    assert!(words.contains(MANUAL_COMMAND), "the words do not name the toolchain command: {words}");
    remove(root.as_path());
}

/// What:
///  A record with no kept answer says so and points at the service's journal,
///  and claims neither that a tool failed nor that no tool ever ran.
/// Why:
///   A record killed on every try holds no answer of any tool,
///  and so does one whose reason could not be written;
///  the two cannot be told apart from the files, so the words may only say what is true of both,
///  and the journal is where the rest is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = wordsFor(worktree, entry, 3, undefined); assert(words.includes("No tool's answer was kept"));
/// ```
#[test]
fn the_words_of_a_record_with_no_kept_answer_point_at_the_journal() {
    let root: PathBuf = fixture("relink-notice-no-answer");
    let worktree: PathBuf = root.join("the worktree");
    let words: String = words_for(worktree.as_path(), &entry_rooted_at(worktree.as_path()), 3, None);
    assert!(words.contains("No tool's answer was kept for this record, so what stopped it is only in the commit service's journal."), "the words do not say what is known: {words}");
    assert!(!words.contains("The last try failed with"), "the words name a failure no tool gave: {words}");
    assert!(!words.contains("no tool's answer was ever recorded"), "the words claim something no file can show: {words}");
    remove(root.as_path());
}

/// What:
///  A worktree's own Git directory is the one its pointer names,
///  and a directory holding `.git` itself is that directory.
/// Why:
///   The notice is written where a command run in that worktree reads it,
///  and that is the worktree's own Git directory,
///  never the one of the worktree whose command added it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(gitDirOf(worktree) === adminDir); assert(gitDirOf(mainWorktree) === join(mainWorktree, '.git'));
/// ```
#[test]
fn a_worktree_git_directory_is_the_one_its_pointer_names() {
    let root: PathBuf = fixture("relink-notice-git-dir");
    let worktree: PathBuf = root.join("the worktree");
    let admin: PathBuf = root.join("common/worktrees/the-worktree");
    std::fs::create_dir_all(worktree.as_path()).expect("worktree");
    std::fs::create_dir_all(admin.as_path()).expect("registration");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", admin.display())).expect("pointer");
    assert_eq!(git_dir_of_a_worktree(worktree.as_path()), Some(admin.clone()));
    std::fs::remove_file(worktree.join(".git")).expect("pointer");
    std::fs::create_dir_all(worktree.join(".git")).expect("git directory");
    assert_eq!(git_dir_of_a_worktree(worktree.as_path()), Some(worktree.join(".git")));
    remove(root.as_path());
}

/// What:
///  A worktree whose pointer is gone names no Git directory,
///  so nothing is written where no command will ever read it.
/// Why:
///   A worktree removed before its record was drained takes its Git directory with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(gitDirOf(join(root, 'removed')) === undefined);
/// ```
#[test]
fn a_removed_worktree_names_no_git_directory() {
    let root: PathBuf = fixture("relink-notice-git-dir-gone");
    assert_eq!(git_dir_of_a_worktree(root.join("removed").as_path()), None);
    remove(root.as_path());
}

/// What:
///  A notice is shown once and removed by the showing,
///  and a second read finds nothing.
/// Why:
///   The person is told once:
///   a notice that stays would be read at every later command in that worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// record(gitDir, 'relink-1', 'the words'); assert(take(gitDir) === ['the words']); assert(take(gitDir) === []);
/// ```
#[test]
fn a_notice_is_shown_once_and_taken_away() {
    let root: PathBuf = fixture("relink-notice-once");
    let git_dir: PathBuf = root.join(".git");
    std::fs::create_dir_all(git_dir.as_path()).expect("git directory");
    record_notice(git_dir.as_path(), "relink-1", "the words").expect("recorded");
    assert_eq!(take_notices(git_dir.as_path()), vec![String::from("the words")]);
    assert!(take_notices(git_dir.as_path()).is_empty(), "the notice was shown twice");
    remove(root.as_path());
}

/// What:
///  A record of another kind leaves no relink notice.
/// Why:
///   Every kind of record is given up on through one step,
///   and only a relink has the package state to name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(noticeForGivenUp(trashItem, 3) === undefined);
/// ```
#[test]
fn a_record_of_another_kind_leaves_no_notice() {
    let root: PathBuf = fixture("relink-notice-kind");
    let worktree: PathBuf = root.join("the worktree");
    let trash: WorkItem = WorkItem {
        kind: WorkKind::Trash,
        repository: worktree.clone(),
        git_dir: root.clone(),
        branch_ref: String::from(RELINK_REFERENCE),
        oid: object_name_of(worktree.as_path(), "2026-11-12T04:00:00Z"),
        parent: String::from("{\"root\":\"/w\",\"tools\":[\"pnpm\"]}"),
        created: String::from("2026-11-12T04:00:00Z"),
        tries: 0,
    };
    assert_eq!(notice_for_given_up(&trash, 3), None);
    remove(root.as_path());
}

/// What:
///  A relink record's own payload gives the notice,
///  and the commands in it are the tools the record asked for.
/// Why:
///   The record is the only place that says which tools the worktree needs,
///   so the notice may not name a tool the record did not ask for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notice = noticeForGivenUp(recordOf(worktree), 3); assert(notice.includes('pnpm --dir ') && !notice.includes('toolchain:install'));
/// ```
#[test]
fn a_relink_record_names_the_tools_it_asked_for() {
    let root: PathBuf = fixture("relink-notice-tools");
    let worktree: PathBuf = root.join("the worktree");
    let entry: RelinkEntry = RelinkEntry { root: worktree.clone(), search_path: SearchPathOfTheAdd::Unset, tools: vec![RelinkTool::Pnpm] };
    let item: WorkItem = item_of(root.as_path(), worktree.as_path(), &entry).expect("the record");
    let notice: String = notice_for_given_up(&item, 3).expect("the notice");
    assert!(notice.contains("pnpm --dir "), "{notice}");
    assert!(!notice.contains(MANUAL_COMMAND), "the notice names a tool the record did not ask for: {notice}");
    let both: WorkItem = record_of(worktree.as_path(), root.as_path());
    assert!(notice_for_given_up(&both, 3).expect("the notice").contains(MANUAL_COMMAND));
    remove(root.as_path());
}

/// What:
///  The notice quotes the reason the last try gave,
///  kept beside the record by the round that ran the tool.
/// Why:
///   The recovery gives a record up and never saw the tool's answer,
///   so the reason has to travel through the file that round wrote,
///   or the person reads a count of tries where the reason is owed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// remember(item.gitDir, item.id(), 'toolchain: 1 launcher records still name another lock file');
/// assert(noticeForGivenUp(item, 3).includes('The last try failed with: toolchain: 1 launcher records still name another lock file.'));
/// ```
#[test]
fn the_notice_quotes_the_reason_the_last_try_gave() {
    let root: PathBuf = fixture("relink-notice-reason");
    let worktree: PathBuf = root.join("the worktree");
    let item: WorkItem = record_of(worktree.as_path(), root.as_path());
    crate::package_repair_relink_failure::remember(item.git_dir.as_path(), item.id().as_str(), "pnpm: exit status 1: ERR_PNPM_OUTDATED_LOCKFILE").expect("kept");
    let notice: String = notice_for_given_up(&item, 3).expect("the notice");
    assert!(notice.contains("The last try failed with: pnpm: exit status 1: ERR_PNPM_OUTDATED_LOCKFILE."), "{notice}");
    remove(root.as_path());
}
