//! What:
//!  Controls for the record of what a removal deleted,
//!  and for the two listings that record is made of.
//! Why:
//!   The record is the only thing left of those files,
//!  so the four questions it answers (what,
//!  from which worktree,
//!  when,
//!  by which command) are pinned here,
//!  with the bound that keeps one `node_modules` from becoming ten thousand lines,
//!  and with the two listings that tell an ignored file from a tracked one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(DeletedEntries.ignoredOf(git, worktree, admin).shown).toEqual(['build/', '.env']);
//! ```

use super::{
    DELETIONS_LOG, DeletedEntries, HiddenEntry, HiddenMarks, NestedWorktree, RemovalBegun, RemovalDeletions, SHOWN_ENTRIES, deletions_line,
    deleted_line, from_status_listing, record_removal_begins, record_removal_deleted, removal_words,
};
use crate::test_support::{age, fixture, git, git_context, repository};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// One listing of the shape Git writes,
/// from the names a control chose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listingOf(['build/', '.env']) // b'build/\0.env\0'
/// ```
fn listing_of(names: &[&str]) -> Vec<u8> {
    let mut listing: Vec<u8> = Vec::new();
    for name in names {
        listing.extend_from_slice(name.as_bytes());
        listing.push(0);
    }
    return listing;
}

/// A repository with one tracked file and one linked worktree,
/// with the ignore rules a control writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { main, worktree, admin } = aRepositoryWithAWorktree('deletions', 'build/\n');
/// ```
fn a_repository_with_a_worktree(name: &str, rules: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "main");
    let tracked: PathBuf = main.join("README.md");
    std::fs::write(tracked.as_path(), b"# fixture\n").expect("a tracked file");
    let ignored: PathBuf = main.join(".gitignore");
    std::fs::write(ignored.as_path(), rules.as_bytes()).expect("the ignore rules");
    age(tracked.as_path());
    age(ignored.as_path());
    git(main.as_path(), &["add", "--", "README.md", ".gitignore"]);
    git(main.as_path(), &["commit", "--quiet", "--message=a tracked file"]);
    let worktree: PathBuf = root.join("topic");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "feature", worktree.to_str().expect("a UTF-8 fixture path")]);
    let admin: PathBuf = std::fs::canonicalize(main.join(".git/worktrees/topic")).expect("the worktree's own Git directory");
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("the common Git directory");
    return (common, std::fs::canonicalize(worktree.as_path()).expect("the worktree's real path"), admin);
}

#[test]
fn a_whole_ignored_directory_is_one_entry_and_every_other_name_is_its_own() {
    let found: DeletedEntries = DeletedEntries::from_listing(listing_of(&["build/", ".env", "node_modules/"]).as_slice());
    assert_eq!(found.shown, vec![String::from("build/"), String::from(".env"), String::from("node_modules/")]);
    assert_eq!(found.total, 3, "three entries went");
    assert_eq!(found.more(), 0, "a set this small is shown whole");
    assert_eq!(found.unreadable, None, "a listing that was read needs no excuse");
}

#[test]
fn a_worktree_with_more_entries_than_the_bound_names_the_bound_and_counts_the_rest() {
    let names: Vec<String> = (0..SHOWN_ENTRIES + 5).map(|index| return format!("ignored-{index}.log")).collect();
    let borrowed: Vec<&str> = names.iter().map(String::as_str).collect();
    let found: DeletedEntries = DeletedEntries::from_listing(listing_of(borrowed.as_slice()).as_slice());
    assert_eq!(found.shown.len(), SHOWN_ENTRIES, "the record shows its bound and no more");
    assert_eq!(found.total, SHOWN_ENTRIES + 5, "everything Git listed is counted");
    assert_eq!(found.more(), 5, "the entries left out are counted");
}

#[test]
fn one_record_names_the_worktree_the_command_the_time_and_the_entries() {
    let worktree: PathBuf = PathBuf::from("/fixtures/outer");
    let command: Vec<OsString> = removal_words(worktree.as_path(), false);
    let deletions: RemovalDeletions = RemovalDeletions {
        ignored: DeletedEntries::from_listing(listing_of(&["build/", ".env"]).as_slice()),
        nested: Vec::new(),
    };
    let begun: RemovalBegun = RemovalBegun { id: String::from("0123abcd-1-2") };
    let line: String = deletions_line(&begun, worktree.as_path(), command.as_slice(), &deletions, 1_700_000_000_000);
    assert!(line.starts_with("{\"schemaVersion\":1,"), "{line}");
    assert!(line.contains("\"worktree\":\"/fixtures/outer\""), "the worktree is named: {line}");
    assert!(line.contains("\"worktreeHex\":\"2f66697874757265732f6f75746572\""), "its bytes are kept too: {line}");
    assert!(line.contains("\"command\":[\"worktree\",\"remove\",\"/fixtures/outer\"]"), "the command is named: {line}");
    assert!(line.contains("\"ignored\":[\"build/\",\".env\"]"), "the entries are named: {line}");
    assert!(line.contains("\"ignoredShown\":2,\"ignoredTotal\":2"), "the counts are written: {line}");
    assert!(line.contains("\"nested\":[]"), "a worktree with none nested says so: {line}");
    assert!(line.ends_with("}\n"), "one line: {line}");
    assert!(!line.contains("\"unreadable\""), "a listing that was read says nothing about being unreadable: {line}");
    assert!(line.contains("\"time\":\"2023-11-14T"), "the moment is written as a stamp: {line}");
}

#[test]
fn a_nested_worktree_is_named_with_its_own_files() {
    let worktree: PathBuf = PathBuf::from("/fixtures/outer");
    let deletions: RemovalDeletions = RemovalDeletions {
        ignored: DeletedEntries::from_listing(listing_of(&["nested/"]).as_slice()),
        nested: vec![NestedWorktree {
            worktree: PathBuf::from("/fixtures/outer/nested"),
            entries: DeletedEntries::from_listing(listing_of(&["personal.txt", "build/"]).as_slice()),
            changed: from_status_listing(b" M tracked.txt\0A  staged.txt\0"),
            hidden: HiddenMarks { shown: vec![HiddenEntry { path: String::from("hidden.txt"), mark: "assume-unchanged" }], total: 1, unreadable: None },
        }],
    };
    let begun: RemovalBegun = RemovalBegun { id: String::from("0123abcd-1-2") };
    let line: String = deletions_line(&begun, worktree.as_path(), removal_words(worktree.as_path(), true).as_slice(), &deletions, 1_700_000_000_000);
    assert!(line.contains("\"nested\":[{\"worktree\":\"/fixtures/outer/nested\""), "the nested worktree is named: {line}");
    assert!(line.contains("\"files\":[\"personal.txt\",\"build/\"]"), "its files are named: {line}");
    assert!(line.contains("\"filesShown\":2,\"filesTotal\":2"), "their count is written: {line}");
    assert!(line.contains("\"changed\":[\"tracked.txt\",\"staged.txt\"]"), "the tracked files it changed are named too: {line}");
    assert!(line.contains("\"changedShown\":2,\"changedTotal\":2"), "and counted: {line}");
    assert!(line.contains("\"hidden\":[{\"path\":\"hidden.txt\",\"mark\":\"assume-unchanged\"}]"), "the marked file and its mark are named: {line}");
    assert!(line.contains("\"hiddenShown\":1,\"hiddenTotal\":1"), "and counted: {line}");
    assert!(line.contains("\"command\":[\"worktree\",\"remove\",\"--force\",\"/fixtures/outer\"]"), "a forced removal says so: {line}");
}

#[test]
fn a_record_of_a_listing_that_cannot_be_read_says_so_rather_than_naming_nothing() {
    let deletions: RemovalDeletions = RemovalDeletions {
        ignored: DeletedEntries {
            shown: Vec::default(),
            total: 0,
            unreadable: Some(String::from("Git could not list the files it ignores in /fixtures/outer (permission denied)")),
        },
        nested: Vec::new(),
    };
    let begun: RemovalBegun = RemovalBegun { id: String::from("0123abcd-1-2") };
    let line: String = deletions_line(&begun, Path::new("/fixtures/outer"), &[OsString::from("merge")], &deletions, 1_700_000_000_000);
    assert!(line.contains("\"unreadable\":\"Git could not list the files it ignores"), "{line}");
    assert!(line.contains("\"ignoredShown\":0,\"ignoredTotal\":0"), "an empty list and an unread one are told apart: {line}");
    assert!(deletions.anything(), "a listing that could not be read is something to record");
}

#[test]
fn a_removal_that_deleted_nothing_a_person_could_miss_writes_no_record() {
    let root: PathBuf = fixture("deletions-nothing");
    let common: PathBuf = root.join("common");
    std::fs::create_dir_all(common.as_path()).expect("a common Git directory");
    let worktree: PathBuf = root.join("outer");
    std::fs::create_dir_all(worktree.as_path()).expect("the worktree a removal took");
    assert_eq!(
        record_removal_begins(common.as_path(), worktree.as_path(), removal_words(worktree.as_path(), false).as_slice(), &RemovalDeletions::default()),
        None,
        "nothing to delete, so nothing to write down"
    );
    assert!(!common.join(DELETIONS_LOG).exists(), "a record of nothing would fill the log with lines nobody needs");
}

#[test]
fn the_record_is_appended_to_the_shared_log_and_reads_back() {
    let root: PathBuf = fixture("deletions-record");
    let common: PathBuf = root.join("common");
    std::fs::create_dir_all(common.as_path()).expect("a common Git directory");
    let worktree: PathBuf = root.join("outer");
    std::fs::create_dir_all(worktree.as_path()).expect("the worktree a removal took");
    let deletions: RemovalDeletions = RemovalDeletions {
        ignored: DeletedEntries::from_listing(listing_of(&["build/"]).as_slice()),
        nested: Vec::new(),
    };
    let first: RemovalBegun = record_removal_begins(common.as_path(), worktree.as_path(), removal_words(worktree.as_path(), false).as_slice(), &deletions)
        .expect("a removal that deletes writes down what it is about to delete");
    record_removal_deleted(common.as_path(), &first, worktree.as_path());
    let second: RemovalBegun = record_removal_begins(common.as_path(), worktree.as_path(), removal_words(worktree.as_path(), true).as_slice(), &deletions)
        .expect("the second removal writes its own line");
    record_removal_deleted(common.as_path(), &second, worktree.as_path());
    let written: String = std::fs::read_to_string(common.join(DELETIONS_LOG)).expect("the record of the deletion");
    let lines: Vec<&str> = written.lines().filter(|line: &&str| return !line.is_empty()).collect();
    assert_eq!(lines.len(), 4, "two lines for each of two removals: {written}");
    assert_eq!(lines.iter().filter(|line: &&&str| return line.contains("\"outcome\":\"begun\"")).count(), 2, "{written}");
    assert_eq!(lines.iter().filter(|line: &&&str| return line.contains("\"outcome\":\"deleted\"")).count(), 2, "{written}");
    for line in &lines {
        assert!(line.contains(&format!("\"worktree\":{}", crate::json_record::quote(worktree.to_string_lossy().as_ref()))), "{line}");
    }
    for line in lines.iter().filter(|line: &&&str| return line.contains("\"begun\"")) {
        assert!(line.contains("\"ignored\":[\"build/\"]"), "the list is in the line written before the deletion: {line}");
    }
    for line in lines.iter().filter(|line: &&&str| return line.contains("\"deleted\"")) {
        assert!(line.contains(first.id.as_str()) || line.contains(second.id.as_str()), "a confirming line carries its pair's id: {line}");
    }
    assert!(written.contains("--force"), "each command is written as it was run: {written}");
}

/// What:
///  The ignored listing names only what the ignore rules hide,
///  and the untracked one names everything Git does not track.
/// Why:
///   A removal of the worktree around a nested one takes that worktree's untracked files,
///  the ignored ones included,
///  and a record that named the ignored ones alone would leave a person's file out of it.
/// What:
///  A confirming line names its own removal and nothing else,
///  so a reader pairs the two lines of one removal and sees the other's separately.
/// Why:
///  The pair is what says a removal finished:
///  a begun line whose id has no confirming line is the one a person has to look at the worktree for.
/// What:
///  A `git status --porcelain -z` answer becomes the paths it names,
///  the rename's two sides included.
/// Why:
///  Each record carries two status letters and a space before its path,
///  and a rename is two records,
///  so a reader that took the record whole would name ` M a.txt` and lose the old side.
#[test]
fn a_status_listing_names_the_paths_it_carries_and_both_sides_of_a_rename() {
    let found: DeletedEntries = from_status_listing(b" M a.txt\0A  staged.txt\0R  new.txt\0old.txt\0?? untracked.txt\0");
    assert_eq!(found.shown, vec![String::from("a.txt"), String::from("staged.txt"), String::from("new.txt"), String::from("old.txt"), String::from("untracked.txt")], "{found:?}");
    assert_eq!(found.total, 5, "every record is counted: {found:?}");
}

/// What:
///  The path a rename came from is named whole, even when its third byte is a space.
/// Why:
///  `git status --porcelain=v1 -z` writes a rename as the new path with `XY` before it and then the old path alone,
///  and a parser that strips three bytes from any record whose third byte is a space turns `ab c.txt` into `c.txt`,
///  a file that never was:
///  the refusal then names it and the deletions log records it,
///  which is the only thing left of the file (a review of this change, finding 1).
#[test]
fn the_path_a_rename_came_from_is_named_whole() {
    let found: DeletedEntries = from_status_listing(b"R  renamed.txt\0ab c.txt\0");
    assert_eq!(found.shown, vec![String::from("renamed.txt"), String::from("ab c.txt")], "the old path is named whole: {found:?}");
    assert_eq!(found.total, 2, "both sides of the rename are counted: {found:?}");
}

/// What:
///  The record after a rename's old path is read with its own state again,
///  even when the old path's second byte is an `R` or a `C`.
/// Why:
///  A path is arbitrary bytes, so a bare old path can look like a rename record;
///  a reader that read the old path's bytes as a state would then name the next file with its status in front of it
///  (` M next.txt`).
#[test]
fn a_record_after_a_rename_is_read_with_its_own_state_again() {
    let found: DeletedEntries = from_status_listing(b"R  renamed.txt\0aR.txt\0 M next.txt\0");
    assert_eq!(
        found.shown,
        vec![String::from("renamed.txt"), String::from("aR.txt"), String::from("next.txt")],
        "the old path and the record after it are each named as they are: {found:?}"
    );
}

/// What:
///  An `ls-files -v -z` answer becomes the marked files and their marks,
///  and an entry with no mark is left out.
/// Why:
///  The tag is one byte whose case is the whole answer for `assume-unchanged`,
///  and `S` is `skip-worktree`;
///  every other entry is a file whose edit `git status` already reports.
#[test]
fn a_marked_listing_names_the_files_and_their_marks() {
    let found: HiddenMarks = HiddenMarks::from_listing(b"H plain.txt\0h hidden.txt\0S skipped.txt\0");
    assert_eq!(
        found.shown,
        vec![
            HiddenEntry { path: String::from("hidden.txt"), mark: "assume-unchanged" },
            HiddenEntry { path: String::from("skipped.txt"), mark: "skip-worktree" },
        ],
        "{found:?}"
    );
    assert_eq!(found.total, 2, "a file with no mark is not counted: {found:?}");
}

#[test]
fn a_confirming_line_carries_the_id_of_the_line_it_confirms() {
    let worktree: PathBuf = PathBuf::from("/fixtures/outer");
    let begun: RemovalBegun = RemovalBegun { id: String::from("0123abcd-1700000000000-4242") };
    let line: String = deleted_line(&begun, worktree.as_path(), 1_700_000_000_001);
    assert!(line.contains("\"outcome\":\"deleted\""), "{line}");
    assert!(line.contains("\"id\":\"0123abcd-1700000000000-4242\""), "{line}");
    assert!(line.contains("\"worktree\":\"/fixtures/outer\""), "{line}");
    assert!(!line.contains("\"ignored\""), "the list is the first line's, and a second copy would double the log: {line}");
    assert!(line.ends_with("}\n"), "one line: {line}");
}

#[test]
fn the_two_listings_differ_in_the_files_they_name() {
    let (common, worktree, admin): (PathBuf, PathBuf, PathBuf) = a_repository_with_a_worktree("deletions-listings", "build/\nignored.log\n");
    std::fs::create_dir_all(worktree.join("build").as_path()).expect("an ignored directory with a file in it");
    std::fs::write(worktree.join("build/out.o").as_path(), b"build output\n").expect("a file in the ignored directory");
    std::fs::write(worktree.join("ignored.log").as_path(), b"an ignored file\n").expect("an ignored file");
    std::fs::write(worktree.join("personal.txt").as_path(), b"a file Git does not track and does not ignore\n").expect("an untracked file");
    let context: crate::transaction_git::GitContext = git_context();
    let ignored: DeletedEntries = DeletedEntries::ignored_of(&context, worktree.as_path(), Some(admin.as_path()));
    assert_eq!(ignored.shown, vec![String::from("build/"), String::from("ignored.log")], "the ignore rules name those two and no more");
    let everything: DeletedEntries = DeletedEntries::untracked_and_ignored_of(&context, worktree.as_path(), Some(admin.as_path()));
    assert!(everything.shown.contains(&String::from("personal.txt")), "the untracked file is named too: {everything:?}");
    assert!(everything.shown.contains(&String::from("build/")), "the ignored directory is named: {everything:?}");
    assert!(everything.total >= ignored.total, "the wider listing names at least what the narrower one does");
    assert!(!common.join(DELETIONS_LOG).exists(), "listing alone writes no record");
}
