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

/// What:
///  A worktree of this repository, with its own administrative directory registered under the common one,
///  so the crate's registration proof accepts it as one of this repository's worktrees.
/// Why:
///   The notice target is chosen from what a command in that worktree would read,
///   and only a real registration names the same directory a command there reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const registered = (root, name) => { mkdir(common/worktrees/name); writeFile(worktree/.git, `gitdir: ${admin}`); writeFile(admin/gitdir, worktree/.git); writeFile(admin/commondir, '../..'); return { common, worktree, admin }; }
/// ```
fn registered_worktree(root: &Path, name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let common: PathBuf = root.join("common");
    let worktree: PathBuf = root.join(name);
    let admin: PathBuf = common.join("worktrees").join(name);
    std::fs::create_dir_all(admin.as_path()).expect("the registration");
    std::fs::create_dir_all(worktree.as_path()).expect("the worktree");
    std::fs::write(admin.join("gitdir"), format!("{}\n", worktree.join(".git").display())).expect("the back-link");
    std::fs::write(admin.join("commondir"), "../..\n").expect("the common directory");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", admin.display())).expect("the pointer");
    return (common, worktree, admin);
}

/// A record of one relink whose moment is `queued_at`,
/// so a control can put it before or after a worktree was registered.
fn record_queued_at(common: &Path, worktree: &Path, queued_at: &str) -> WorkItem {
    let entry: RelinkEntry = entry_rooted_at(worktree);
    let mut item: WorkItem = item_of(common, worktree, &entry).expect("the record");
    item.created = String::from(queued_at);
    return item;
}

/// What:
///  A `.git` pointer past the crate's bound for one is refused rather than read,
///  and nothing waits for a name that is not a plain file.
/// Why:
///  The give-up pass reads that pointer to find where the person's words go,
///  and a pointer is one line:
///  a file of any size at that name is read whole by a bare read into this process's memory,
///  and every other pointer read in the crate takes the type from the open handle and refuses a file past
///  `GIT_FILE_LIMIT` for exactly that reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFile(join(worktree, '.git'), 'gitdir: /w\n' + 'x'.repeat(1 << 20)); assert(gitDirOf(worktree) === undefined);
/// ```
#[test]
fn a_pointer_file_past_the_git_file_limit_is_refused_instead_of_read() {
    let root: PathBuf = fixture("relink-notice-pointer-limit");
    let worktree: PathBuf = root.join("the worktree");
    std::fs::create_dir_all(worktree.as_path()).expect("the worktree");
    let mut text: String = format!("gitdir: {}\n", root.join("admin").display());
    text.push_str("x".repeat(1 << 20).as_str());
    std::fs::write(worktree.join(".git"), text).expect("the pointer");
    let resolved: Option<PathBuf> = git_dir_of_a_worktree(worktree.as_path());
    let size: usize = resolved.as_ref().map_or(0, |path| return path.as_os_str().len());
    assert!(resolved.is_none(), "a 1 MiB pointer file was read whole: a path of {size} bytes came back");
    remove(root.as_path());
}

/// What:
///  A worktree whose registration is gone is told in the directory the record was drained from,
///  and the gone administrative directory is never made again.
/// Why:
///   An interrupted removal leaves a checkout whose `.git` names an administrative directory that is gone;
///   making it again writes the person's only word into a directory Git refuses
///   (`fatal: gitfile does not point to a valid repository`),
///   so no command in that checkout reads it.
///   The drain directory is one a command in the worktree whose add wrote the record does read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { common, worktree, admin } = registered(root, 'W'); rmSync(admin, { recursive: true });
/// assert(tellAboutGivenUp(common, [record])); assert(!existsSync(admin)); assert(takeNotices(common).length === 1);
/// ```
#[test]
fn a_worktree_whose_registration_is_gone_is_told_where_a_command_reads_it() {
    let root: PathBuf = fixture("relink-notice-registration-gone");
    let (common, worktree, admin): (PathBuf, PathBuf, PathBuf) = registered_worktree(root.as_path(), "W");
    let item: WorkItem = record_queued_at(common.as_path(), worktree.as_path(), "2030-01-01T00:00:00.000Z");
    std::fs::remove_dir_all(admin.as_path()).expect("the registration is gone");
    assert!(
        crate::service_work_handlers_replanning_and_acceptance::tell_about_given_up(common.as_path(), std::slice::from_ref(&item)),
        "the person was not told at all"
    );
    assert!(!admin.exists(), "the notice made the gone administrative directory again");
    let told: Vec<String> = take_notices(common.as_path());
    assert_eq!(told.len(), 1, "the notice does not wait in the directory the record was drained from: {told:?}");
    assert!(told[0].contains("was not relinked"), "{told:?}");
    remove(root.as_path());
}

/// What:
///  A worktree whose own Git directory refuses the words is told in the drain directory,
///  and the record settles as told.
/// Why:
///   The recovery settles nothing while the person cannot be told (`work_item_recovery.rs`),
///   so words that no directory takes hold the record in `recovering` for ever.
///   The drain directory is writable whenever the recovery can move records at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { common, worktree, admin } = registered(root, 'W'); chmod(admin, 0o555);
/// assert(tellAboutGivenUp(common, [record])); assert(takeNotices(common).length === 1);
/// ```
#[test]
fn a_worktree_whose_own_git_directory_refuses_the_words_is_told_in_the_drain_directory() {
    use std::os::unix::fs::PermissionsExt as _;
    let root: PathBuf = fixture("relink-notice-unwritable-worktree");
    let (common, worktree, admin): (PathBuf, PathBuf, PathBuf) = registered_worktree(root.as_path(), "W");
    let item: WorkItem = record_queued_at(common.as_path(), worktree.as_path(), "2030-01-01T00:00:00.000Z");
    std::fs::set_permissions(admin.as_path(), std::fs::Permissions::from_mode(0o555)).expect("the registration refuses writes");
    let told: bool = crate::service_work_handlers_replanning_and_acceptance::tell_about_given_up(common.as_path(), std::slice::from_ref(&item));
    std::fs::set_permissions(admin.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the registration takes writes again");
    assert!(told, "the record is not settled because the worktree's own Git directory refused the words");
    let written: Vec<String> = take_notices(common.as_path());
    assert_eq!(written.len(), 1, "no notice waits in the directory the record was drained from: {written:?}");
    assert!(take_notices(admin.as_path()).is_empty(), "a notice is in a directory that refused the words");
    remove(root.as_path());
}

/// What:
///  A notice about an earlier worktree is not left in a later worktree that took the same path.
/// Why:
///   A registration at a path can be another worktree than the one the record is about:
///   the worktree was removed and another was added at its path,
///   and the administrative directory's name is the path's.
///   The record carries the moment its add queued it,
///   and a registration made after that moment is not the worktree the words are about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const first = registered(root, 'W'); const item = recordQueuedAt(common, worktree, '2020-01-01T00:00:00.000Z');
/// rmSync(first.admin, { recursive: true }); rmSync(first.worktree, { recursive: true }); const later = registered(root, 'W');
/// assert(tellAboutGivenUp(common, [item])); assert(takeNotices(later.admin).length === 0);
/// ```
#[test]
fn a_later_worktree_at_the_same_path_does_not_answer_for_an_earlier_records_notice() {
    let root: PathBuf = fixture("relink-notice-later-worktree");
    let (common, worktree, admin): (PathBuf, PathBuf, PathBuf) = registered_worktree(root.as_path(), "W");
    let item: WorkItem = record_queued_at(common.as_path(), worktree.as_path(), "2020-01-01T00:00:00.000Z");
    std::fs::remove_dir_all(admin.as_path()).expect("the first registration is gone");
    std::fs::remove_dir_all(worktree.as_path()).expect("the first worktree is gone");
    let (later_common, later_worktree, later_admin): (PathBuf, PathBuf, PathBuf) = registered_worktree(root.as_path(), "W");
    assert_eq!(later_common, common, "the fixture registered the later worktree under the same repository");
    assert_eq!(later_worktree, worktree, "and at the same path");
    assert!(
        crate::service_work_handlers_replanning_and_acceptance::tell_about_given_up(common.as_path(), std::slice::from_ref(&item)),
        "the person was not told at all"
    );
    let later: Vec<String> = take_notices(later_admin.as_path());
    assert!(later.is_empty(), "a notice about the earlier worktree waits in the later one: {later:?}");
    let drain: Vec<String> = take_notices(common.as_path());
    assert_eq!(drain.len(), 1, "the notice does not wait where a command of the adding worktree reads it: {drain:?}");
    remove(root.as_path());
}
