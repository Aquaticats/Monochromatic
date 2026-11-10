//! Controls for the pieces of the worktree pass:
//! the stages read from index entries,
//! the records
//!  that put a conflict back,
//! the closing words,
//! and what the pass refuses to replace when another command moved it.
//! The pass as a whole runs through the built executable in
//!  `binary_word_merge_tests.rs`.

use super::*;
use crate::merge_second_pass::tests::{Change, World, await_it, corpus, runner, world};
use crate::merge_second_pass::{GitReply, resolved_record};
use crate::merge_second_pass_worktree_unchanged::LARGEST_RECORDED_BYTES;
use crate::test_support::{git, git_context, git_output, git_text, remove};
use std::path::{Path, PathBuf};

/// An index entry of a path at a stage.
fn entry(path: &str, stage: u8, mode: u32, byte: u8) -> IndexEntry {
    return IndexEntry { path: path.as_bytes().to_vec(), mode, id: vec![byte; 20], stage, extended: 0 };
}

/// Only the named paths are read,
/// every stage lands where it belongs,
/// and a resolved entry is ignored.
#[test]
fn the_stages_of_the_named_paths_are_read_from_the_index() {
    let entries: Vec<IndexEntry> = vec![
        entry("a.md", 0, 0o100_644, 9),
        entry("b.md", 1, 0o100_644, 1),
        entry("b.md", 2, 0o100_755, 2),
        entry("b.md", 3, 0o100_644, 3),
        entry("c.md", 2, 0o100_644, 4),
        entry("c.md", 3, 0o100_644, 5),
    ];
    let only: BTreeSet<Vec<u8>> = [b"b.md".to_vec(), b"c.md".to_vec()].into_iter().collect();
    let conflicts: Vec<Conflict> = conflicts_of_index(entries.as_slice(), &only);
    assert_eq!(conflicts.len(), 2);
    assert_eq!(conflicts[0].path, b"b.md".to_vec());
    assert_eq!(conflicts[0].base, Some(Stage { mode: 0o100_644, oid: "01".repeat(20) }));
    assert_eq!(conflicts[0].ours.as_ref().map(|stage| { return stage.mode; }), Some(0o100_755));
    assert_eq!(conflicts[0].theirs, Some(Stage { mode: 0o100_644, oid: "03".repeat(20) }));
    assert_eq!((conflicts[1].path.as_slice(), conflicts[1].base.is_none()), (b"c.md".as_slice(), true));
    let none: BTreeSet<Vec<u8>> = BTreeSet::new();
    assert!(conflicts_of_index(entries.as_slice(), &none).is_empty());
}

/// Putting a conflict back removes the resolved entry first and then lists each stage the path had,
/// in order.
#[test]
fn the_restore_records_remove_the_resolved_entry_and_list_the_stages() {
    let conflict = Conflict {
        path: b"d/x.md".to_vec(),
        base: Some(Stage { mode: 0o100_644, oid: String::from("aa") }),
        ours: Some(Stage { mode: 0o100_644, oid: String::from("bb") }),
        theirs: Some(Stage { mode: 0o100_755, oid: String::from("cc") }),
    };
    let expected: Vec<u8> = [b"0 00\td/x.md\0".as_slice(), b"100644 aa 1\td/x.md\0", b"100644 bb 2\td/x.md\0", b"100755 cc 3\td/x.md\0"].concat();
    assert_eq!(restore_records(&conflict), expected);
}

/// The closing words differ by whether a conflicted file remains,
/// and name the operation;
/// the line after a finish that did not end the operation names the command that ends it and the state the index is in,
/// and never the failing step's own sentence about itself.
#[test]
fn the_closing_words_say_whether_a_conflict_remains() {
    assert!(closing_words("rebase", 0).contains("no conflicted file is left") && closing_words("rebase", 0).contains("finish the rebase"));
    assert!(closing_words("merge", 2).contains("still lists as conflicted") && closing_words("merge", 2).contains("git diff --cached"));
    for (operation, ending) in [("merge", "`git commit`"), ("pull", "`git commit`"), ("rebase", "`git rebase --continue`"), ("cherry-pick", "`git cherry-pick --continue`"), ("revert", "`git revert --continue`")] {
        let line: String = unfinished_words(operation, "notes.md", Some(&[]), ending);
        assert!(line.contains(format!("the {operation} did not finish").as_str()) && line.contains("the files the second merge pass staged as resolved are notes.md"), "{line}");
        assert!(line.contains("no file is conflicted"), "{line}");
        assert!(line.contains(ending), "{line}");
        assert!(!line.contains("Nothing in the repository changed"), "the last word is not the failing step's sentence about itself: {line}");
    }
    let one: Vec<String> = vec![String::from("notes.md")];
    let line: String = unfinished_words("rebase", "a.md", Some(one.as_slice()), "`git rebase --continue`");
    assert!(line.contains("notes.md is conflicted again where the ending step stopped") && line.contains("resolve it"), "{line}");
    assert!(!line.contains("the conflict is gone") && !line.contains("no file is conflicted"), "the state is read, not assumed: {line}");
    let two: Vec<String> = vec![String::from("a.md"), String::from("b.md")];
    let several: String = unfinished_words("rebase", "c.md", Some(two.as_slice()), "`git rebase --continue`");
    assert!(several.contains("a.md, b.md are conflicted again") && several.contains("resolve them"), "{several}");
}

/// What:
///  A `pull` whose rebase stopped is told the rebase's command,
/// because the state file says which operation is in progress.
/// Why:
///   `git pull` runs either a merge or a rebase as a child,
/// so the operation's own word cannot name the command that ends it
/// (cross-vendor review of this branch,
/// finding 6).
#[test]
fn the_ending_command_is_the_one_the_state_files_name() {
    let root: PathBuf = crate::test_support::fixture("second-pass-ending-command");
    let git_dir: PathBuf = root.join("git");
    std::fs::create_dir_all(git_dir.join("rebase-merge")).expect("the rebase state");
    assert_eq!(ending_command(git_dir.as_path(), "pull"), "`git rebase --continue`", "a rebasing pull");
    assert_eq!(ending_command(git_dir.as_path(), "rebase"), "`git rebase --continue`");
    std::fs::remove_dir(git_dir.join("rebase-merge")).expect("remove the rebase state");
    std::fs::write(git_dir.join("MERGE_HEAD"), b"0".repeat(40)).expect("the merge state");
    assert_eq!(ending_command(git_dir.as_path(), "pull"), "`git commit`", "a merging pull");
    std::fs::remove_file(git_dir.join("MERGE_HEAD")).expect("remove the merge state");
    std::fs::write(git_dir.join("CHERRY_PICK_HEAD"), b"0".repeat(40)).expect("the pick state");
    assert_eq!(ending_command(git_dir.as_path(), "cherry-pick"), "`git cherry-pick --continue`");
    std::fs::remove_file(git_dir.join("CHERRY_PICK_HEAD")).expect("remove the pick state");
    std::fs::write(git_dir.join("REVERT_HEAD"), b"0".repeat(40)).expect("the revert state");
    assert_eq!(ending_command(git_dir.as_path(), "revert"), "`git revert --continue`");
    std::fs::remove_file(git_dir.join("REVERT_HEAD")).expect("remove the revert state");
    assert_eq!(ending_command(git_dir.as_path(), "merge"), "`git commit`", "the operation's word is the fallback");
    remove(root.as_path());
}

/// A runner that records every start and fails the command it is told to.
struct Scripted {
    /// The command word and the standard input of every start,
    /// in order.
    calls: std::sync::Mutex<Vec<(String, Vec<u8>)>>,
    /// The command word that exits with status 1.
    failing: &'static str,
}

/// Recording the starts instead of running Git.
impl GitRunner for Scripted {
    fn run<'a>(
        &'a self,
        arguments: &'a [&'a str],
        input: Option<&'a [u8]>,
        _index_file: Option<&'a Path>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<GitReply, String>> + Send + 'a>> {
        return Box::pin(async move {
            self.calls.lock().expect("no control panics while it holds the starts").push((arguments[0].to_owned(), input.map(<[u8]>::to_vec).unwrap_or_default()));
            let code: Option<i32> = if arguments[0] == self.failing { Some(1) } else { Some(0) };
            return Ok(GitReply { code, stdout: Vec::new(), stderr: b"boom".to_vec() });
        });
    }
}

/// One merged file with the stages it came from.
fn merged_file() -> Merged {
    let stage = |mode: u32, oid: &str| { return Some(Stage { mode, oid: oid.to_owned() }); };
    let stages = Conflict { path: b"a.md".to_vec(), base: stage(0o100_644, "aa"), ours: stage(0o100_644, "bb"), theirs: stage(0o100_644, "cc") };
    return Merged { path: b"a.md".to_vec(), mode: 0o100_644, bytes: b"merged\n".to_vec(), parts: Vec::new(), stages };
}

/// A file is staged and then written to the worktree;
/// nothing else runs when both work.
#[test]
fn a_merged_file_is_staged_and_then_checked_out() {
    let runner = Scripted { calls: std::sync::Mutex::new(Vec::new()), failing: "none" };
    await_it(install(&runner, &[merged_file()], &[String::from("dd")])).expect("both steps work");
    let calls = runner.calls.lock().expect("no control panics while it holds the starts");
    assert_eq!(calls.iter().map(|(word, _)| { return word.as_str(); }).collect::<Vec<&str>>(), vec!["update-index", "checkout-index"]);
    assert_eq!(calls[0].1, resolved_record(0o100_644, "dd", b"a.md"));
    assert_eq!(calls[1].1, b"a.md\0".to_vec());
}

/// A worktree write that fails puts the conflict stages back,
/// so the index never says resolved over a file with markers.
#[test]
fn a_failed_checkout_puts_the_conflict_stages_back() {
    let runner = Scripted { calls: std::sync::Mutex::new(Vec::new()), failing: "checkout-index" };
    let error: String = await_it(install(&runner, &[merged_file()], &[String::from("dd")])).expect_err("the checkout failed");
    assert!(error.contains("boom"), "{error}");
    let calls = runner.calls.lock().expect("no control panics while it holds the starts");
    assert_eq!(calls.iter().map(|(word, _)| { return word.as_str(); }).collect::<Vec<&str>>(), vec!["update-index", "checkout-index", "update-index"]);
    assert_eq!(calls[2].1, restore_records(&merged_file().stages));
}

/// A failed staging stops before the worktree is written.
#[test]
fn a_failed_staging_writes_nothing_to_the_worktree() {
    let runner = Scripted { calls: std::sync::Mutex::new(Vec::new()), failing: "update-index" };
    assert!(await_it(install(&runner, &[merged_file()], &[String::from("dd")])).is_err());
    assert_eq!(runner.calls.lock().expect("no control panics while it holds the starts").len(), 1);
}

/// The index entries of a repository's index,
/// as the pass reads them in process.
fn entries_of(repo: &Path) -> Vec<IndexEntry> {
    let bytes: Vec<u8> = std::fs::read(repo.join(".git/index")).expect("the index");
    return parse_index(bytes.as_slice(), crate::repo_read_layout::ObjectFormat::Sha1).expect("an index the in-process reader takes");
}

/// The text of the index entry at stage 0 of a path,
/// or nothing when the path has no resolved entry.
fn staged_text(repo: &Path, path: &str) -> Option<String> {
    let shown: std::process::Output = crate::test_support::git_output(repo, &["show", format!(":0:{path}").as_str()]);
    return if shown.status.code() == Some(0) { Some(String::from_utf8_lossy(&shown.stdout).into_owned()) } else { None };
}

/// A sparse file of `length` bytes at `path`:
/// longer than a conflicted file can be,
/// without writing the bytes no control reads.
fn sparse_file_of(path: &Path, length: u64) {
    let file: std::fs::File = std::fs::File::create(path).expect("the file");
    file.set_len(length).expect("a sparse file of that length");
}

/// What:
///  A worktree file that was too large to compare before the pass and is too large again when it installs
/// is not "unchanged":
/// it stays as the person left it and the line names it.
/// Why:
///   Both reads recorded nothing,
/// so the comparison of two nothings said "the same bytes" and the pass staged its own merge
///  over a file it never compared,
/// which can run to exit `0`
/// (cross-vendor review round 3,
/// finding 1).
#[test]
fn a_path_too_large_to_compare_twice_is_left_and_named() {
    let (world, entries) = stopped_merge("second-pass-worktree-too-large-twice");
    let path: PathBuf = world.repo.join("a.md");
    sparse_file_of(path.as_path(), LARGEST_RECORDED_BYTES + 1);
    let (file, oid, held) = merged_of(&world, entries.as_slice());
    assert_eq!(held.get(b"a.md".as_slice()), Some(&None), "a file past the limit is recorded as nothing to compare");
    // Different bytes, and still past the limit: the second read records nothing either.
    sparse_file_of(path.as_path(), LARGEST_RECORDED_BYTES + 2);
    let fresh: Vec<IndexEntry> = entries_of(world.repo.as_path());
    let before = Before { worktree_root: world.repo.as_path(), entries: fresh.as_slice(), held: &held };
    let kept: Kept = still_the_same(&before, vec![file], vec![oid]);
    assert!(kept.files.is_empty(), "nothing is installed over a file the pass never read: {} merged file(s) would be", kept.files.len());
    assert_eq!(kept.unreadable, vec![b"a.md".to_vec()], "the path nobody compared is named");
    assert!(kept.moved.is_empty(), "no edit was seen, so none is claimed: {:?}", kept.moved);
    assert_eq!(std::fs::metadata(path.as_path()).expect("the file").len(), LARGEST_RECORDED_BYTES + 2, "the newer file is still there");
    remove(world.root.as_path());
}

/// What:
///  A worktree path the pass cannot read at all is left and named as one it could not compare.
/// Why:
///   A read that fails says nothing about the bytes,
/// and no person can be told their file changed when nobody read it
///  (cross-vendor review round 3,
/// finding 1,
/// the other half:
/// "inability to compare is accepted as equality").
#[test]
fn a_path_that_cannot_be_read_is_left_and_named() {
    let (world, entries) = stopped_merge("second-pass-worktree-cannot-read");
    let path: PathBuf = world.repo.join("a.md");
    std::fs::remove_file(path.as_path()).expect("the file Git left");
    std::fs::create_dir(path.as_path()).expect("a directory where the file was");
    let (file, oid, held) = merged_of(&world, entries.as_slice());
    assert_eq!(held.get(b"a.md".as_slice()), Some(&None), "a path that cannot be read is recorded as nothing to compare");
    let fresh: Vec<IndexEntry> = entries_of(world.repo.as_path());
    let before = Before { worktree_root: world.repo.as_path(), entries: fresh.as_slice(), held: &held };
    let kept: Kept = still_the_same(&before, vec![file], vec![oid]);
    assert!(kept.files.is_empty(), "nothing is installed over a path the pass never read: {} merged file(s) would be", kept.files.len());
    assert_eq!(kept.unreadable, vec![b"a.md".to_vec()], "the path nobody read is named");
    assert!(path.is_dir(), "what is at the path is untouched");
    remove(world.root.as_path());
}

/// How long an answer that must not wait may take in a control.
const PATIENCE: std::time::Duration = std::time::Duration::from_secs(20);

/// A named pipe at `path`,
/// which blocks whoever opens it for reading until a writer comes.
fn make_pipe(path: &Path) {
    use std::os::unix::ffi::OsStrExt as _;
    let name = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("path text");
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let result: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(result, 0, "mkfifo {}", path.display());
}

/// What:
///  The recorded state of `paths`,
/// answered inside `PATIENCE`.
/// Why:
///   A read that waits has no end of its own,
/// and a test that waited with it could only be stopped from outside;
/// the panic here names what waited and why it must not.
fn held_within(root: &Path, paths: &BTreeSet<Vec<u8>>) -> BTreeMap<Vec<u8>, Held> {
    let (sender, receiver) = std::sync::mpsc::channel::<BTreeMap<Vec<u8>, Held>>();
    let handed: PathBuf = root.to_path_buf();
    let asked: BTreeSet<Vec<u8>> = paths.clone();
    std::thread::spawn(move || {
        let _ = sender.send(held_paths(handed.as_path(), &asked));
    });
    return match receiver.recv_timeout(PATIENCE) {
        Ok(held) => held,
        Err(_) => panic!("the worktree snapshot did not answer within {PATIENCE:?}"),
    };
}

/// What:
///  A named pipe with no writer at a conflicted path is refused,
/// not waited for.
/// Why:
///   `File::open` on a pipe waits for a writer that may never come,
/// and that wait sits in the middle of a foreground command with no deadline and no way to
///  end it
/// (cross-vendor review round 3,
/// finding 2).
#[test]
fn a_named_pipe_with_no_writer_is_not_waited_for() {
    let (world, entries) = stopped_merge("second-pass-worktree-pipe-no-writer");
    let path: PathBuf = world.repo.join("a.md");
    std::fs::remove_file(path.as_path()).expect("the file Git left");
    make_pipe(path.as_path());
    let created: BTreeSet<Vec<u8>> = [b"a.md".to_vec()].into_iter().collect();
    let held: BTreeMap<Vec<u8>, Held> = held_within(world.repo.as_path(), &created);
    assert_eq!(held.get(b"a.md".as_slice()), Some(&None), "a path that is no plain file is nothing to compare");
    drop(entries);
    remove(world.root.as_path());
}

/// What:
///  A named pipe whose writer holds it open and says nothing is refused too.
/// Why:
///   A writer that keeps the pipe open takes the wait from the open to the read,
/// and the byte limit bounds neither,
///  so the command waited for ever with the file open
/// (cross-vendor review round 3,
/// finding 2,
/// the other half).
#[test]
fn a_named_pipe_whose_writer_says_nothing_is_not_waited_for() {
    let (world, entries) = stopped_merge("second-pass-worktree-pipe-silent-writer");
    let path: PathBuf = world.repo.join("a.md");
    std::fs::remove_file(path.as_path()).expect("the file Git left");
    make_pipe(path.as_path());
    // One handle that is both ends keeps a writer open without a second process and without a thread that could block.
    let writer: std::fs::File = std::fs::OpenOptions::new().read(true).write(true).open(path.as_path()).expect("a writer for the pipe");
    let created: BTreeSet<Vec<u8>> = [b"a.md".to_vec()].into_iter().collect();
    let held: BTreeMap<Vec<u8>, Held> = held_within(world.repo.as_path(), &created);
    assert_eq!(held.get(b"a.md".as_slice()), Some(&None), "a path that is no plain file is nothing to compare");
    drop(writer);
    drop(entries);
    remove(world.root.as_path());
}

/// A repository whose merge of the two branches stopped with `a.md` conflicted,
/// with the index entries the pass reads after the command.
fn stopped_merge(name: &str) -> (World, Vec<IndexEntry>) {
    let world: World = world(
        name,
        &[("a.md", corpus("md1b", "base").as_slice())],
        &[("a.md", Change::Write(corpus("md1b", "ours").as_slice()))],
        &[("a.md", Change::Write(corpus("md1b", "theirs").as_slice()))],
    );
    let merged: std::process::Output = git_output(world.repo.as_path(), &["merge", "--no-edit", "theirs"]);
    assert_eq!(merged.status.code(), Some(1), "the merge stops on the conflict: {}", String::from_utf8_lossy(&merged.stderr));
    let entries: Vec<IndexEntry> = entries_of(world.repo.as_path());
    return (world, entries);
}

/// The one file of a stopped merge,
/// merged by word,
/// with its blob written.
fn merged_of(world: &World, entries: &[IndexEntry]) -> (Merged, String, BTreeMap<Vec<u8>, Held>) {
    let created: BTreeSet<Vec<u8>> = [b"a.md".to_vec()].into_iter().collect();
    let conflicts: Vec<Conflict> = conflicts_of_index(entries, &created);
    let context: GitContext = git_context();
    let pass: ContextRunner<'_> = runner(&context, world.repo.as_path());
    let verdicts: Vec<Verdict> = await_it(second_pass(&pass, conflicts.as_slice(), AttributeSource::Invocation)).expect("the pass runs");
    let [Verdict::Merged(file)] = verdicts.as_slice() else { panic!("the one file resolves by word: {verdicts:?}") };
    let oid: String = await_it(write_blob(&pass, file.bytes.as_slice())).expect("the merged blob");
    let held: BTreeMap<Vec<u8>, Held> = held_paths(world.repo.as_path(), &created);
    return ((**file).clone(), oid, held);
}

/// What:
///  A resolution another command staged while the pass was computing is never replaced:
/// the pass names the path and leaves the index and the file exactly as that command left them.
/// Why:
///   The owner's rule is that user data is never discarded.
/// A person who resolves the file and runs `git add` while the pass works
///  holds a resolution the pass never read,
/// and staging over it loses that work at exit `0`.
#[test]
fn a_resolution_another_command_staged_is_left_alone() {
    let (world, entries) = stopped_merge("second-pass-worktree-staged");
    let (file, oid, held) = merged_of(&world, entries.as_slice());
    // Another command resolves the file and stages it, after the pass read the stages.
    std::fs::write(world.repo.join("a.md"), b"a resolution another command wrote\n").expect("the resolution");
    git(world.repo.as_path(), &["add", "--", "a.md"]);
    let fresh: Vec<IndexEntry> = entries_of(world.repo.as_path());
    let before = Before { worktree_root: world.repo.as_path(), entries: fresh.as_slice(), held: &held };
    let kept: Kept = still_the_same(&before, vec![file], vec![oid]);
    assert_eq!(kept.moved, vec![b"a.md".to_vec()], "the path that moved is named");
    assert!(kept.files.is_empty(), "nothing is installed over it");
    await_it(install(&runner(&git_context(), world.repo.as_path()), kept.files.as_slice(), kept.oids.as_slice())).expect("nothing to install is not a failure");
    assert_eq!(staged_text(world.repo.as_path(), "a.md"), Some(String::from("a resolution another command wrote\n")), "the index keeps the other command's resolution");
    assert_eq!(std::fs::read(world.repo.join("a.md")).expect("the file"), b"a resolution another command wrote\n".to_vec(), "the worktree keeps it too");
    remove(world.root.as_path());
}

/// What:
///  The other half of the same control:
/// a path that did not move is staged and written,
/// so the check can say `unchanged` as well as `moved`.
/// Why:
///   A guard that refused everything would pass the control above while losing the pass.
#[test]
fn a_path_that_did_not_move_is_installed() {
    let (world, entries) = stopped_merge("second-pass-worktree-unmoved");
    let (file, oid, held) = merged_of(&world, entries.as_slice());
    let expected: Vec<u8> = file.bytes.clone();
    let fresh: Vec<IndexEntry> = entries_of(world.repo.as_path());
    let before = Before { worktree_root: world.repo.as_path(), entries: fresh.as_slice(), held: &held };
    let kept: Kept = still_the_same(&before, vec![file], vec![oid]);
    assert!(kept.moved.is_empty(), "nothing moved: {:?}", kept.moved);
    await_it(install(&runner(&git_context(), world.repo.as_path()), kept.files.as_slice(), kept.oids.as_slice())).expect("the install works");
    assert_eq!(std::fs::read(world.repo.join("a.md")).expect("the file"), expected, "the worktree holds the merged bytes");
    assert_eq!(staged_text(world.repo.as_path(), "a.md"), Some(String::from_utf8_lossy(expected.as_slice()).into_owned()), "and the index does too");
    assert_eq!(git_text(world.repo.as_path(), &["ls-files", "--unmerged"]), "", "the conflict stages are gone");
    remove(world.root.as_path());
}

/// What:
///  A worktree file a person edited after the pass read the stages is left,
/// even when the index still holds the conflict stages.
/// Why:
///   The person's edit is a resolution in progress,
/// and writing the worktree file over it discards it.
#[test]
fn a_worktree_file_edited_after_it_was_read_is_left_alone() {
    let (world, entries) = stopped_merge("second-pass-worktree-edited");
    let (file, oid, held) = merged_of(&world, entries.as_slice());
    std::fs::write(world.repo.join("a.md"), b"half of my own resolution\n").expect("the edit");
    let fresh: Vec<IndexEntry> = entries_of(world.repo.as_path());
    let before = Before { worktree_root: world.repo.as_path(), entries: fresh.as_slice(), held: &held };
    let kept: Kept = still_the_same(&before, vec![file], vec![oid]);
    assert_eq!(kept.moved, vec![b"a.md".to_vec()], "the edited file is named");
    assert!(kept.files.is_empty(), "nothing is installed over it");
    assert_eq!(std::fs::read(world.repo.join("a.md")).expect("the file"), b"half of my own resolution\n".to_vec());
    remove(world.root.as_path());
}

/// What:
///  A line about an index that could not be read again says so,
/// and never that no file is conflicted.
/// Why:
///   The read failing left the list of conflicted paths empty,
/// so the line claimed a state nobody had
/// (cross-vendor review round 2,
/// finding 7,
/// which is outside the ordered list of the round).
#[test]
fn a_line_about_an_unread_index_does_not_claim_a_clean_one() {
    let line: String = unfinished_words("rebase", "a.md", None, "`git rebase --continue`");
    assert!(line.contains("could not be read"), "{line}");
    assert!(!line.contains("no file is conflicted"), "{line}");
    assert!(line.contains("git rebase --continue"), "the command that ends it is still named: {line}");
    // The known states still say what they hold.
    assert!(unfinished_words("rebase", "a.md", Some(&[]), "`git rebase --continue`").contains("no file is conflicted"));
    let one: Vec<String> = vec![String::from("b.md")];
    assert!(unfinished_words("rebase", "a.md", Some(one.as_slice()), "`git rebase --continue`").contains("b.md is conflicted again"));
}
