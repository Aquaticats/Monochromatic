//! What:
//!  Controls for the notice a relink given up on leaves,
//!  and for the lines a later command in that worktree shows of it.
//! Why:
//!  The notice is the only word the person gets about a worktree whose package links still name the checkout it was copied
//!  from,
//!  so every control here names the words,
//!  the commands in them,
//!  and that one read takes the notice away.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! recordNotice(gitDir, id, wordsFor(worktree, entry, 3)); assert(takeNotices(gitDir) === [expected]);
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
///  the tries,
///  and both commands that finish the job.
/// Why:
///  A person who reads only this line must be able to finish the relinking without a journal or a record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(words.includes(worktree) && words.includes('3 tries') && words.includes('pnpm --dir '));
/// ```
#[test]
fn the_words_name_the_worktree_the_tries_and_every_command() {
    let root: PathBuf = fixture("relink-notice-words");
    let worktree: PathBuf = root.join("the worktree");
    let words: String = words_for(worktree.as_path(), &entry_rooted_at(worktree.as_path()), 3);
    assert!(words.contains(worktree.to_string_lossy().as_ref()), "the words do not name the worktree: {words}");
    assert!(words.contains("3 tries"), "the words do not say how many times the service tried: {words}");
    assert!(words.contains("pnpm --dir "), "the words do not name the pnpm command: {words}");
    assert!(words.contains(MANUAL_COMMAND), "the words do not name the toolchain command: {words}");
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
    assert!(!notice.contains(MANUAL_COMMAND), "the notice names a tool the record did not ask for: {notice}");    let both: WorkItem = record_of(worktree.as_path(), root.as_path());
    assert!(notice_for_given_up(&both, 3).expect("the notice").contains(MANUAL_COMMAND));
    remove(root.as_path());
}
