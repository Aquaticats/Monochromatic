//! The repository's own exclude line,
//! and that real Git honours it in a linked worktree.

use super::{TRASH_EXCLUDE_LINE, ensure_trash_is_excluded};
use crate::file_size_limit_child_test_support::{LIMITED_DIRECTORY_VARIABLE, run_child_under_file_size_limit};
use crate::test_support::{fixture, git, git_text, remove, repository};
use crate::worktree_trash::TRASH_PARENT;
use crate::worktree_trash_ignore_rule::trash_location_is_ignored;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

/// Write a file and its directories.
fn put(root: &Path, path: &str, bytes: &[u8]) {
    let full: PathBuf = root.join(path);
    std::fs::create_dir_all(full.parent().expect("a parent")).expect("directories");
    std::fs::write(full, bytes).expect("the file");
}

/// The bytes of a fixture file.
fn read(root: &Path, path: &str) -> String {
    return String::from_utf8(std::fs::read(root.join(path)).expect("the file")).expect("text");
}

/// What:
///  The line is added once,
/// the person's own patterns are kept as they were,
/// and a file that did not end in a newline
///  still gets the line as a line of its own.
/// Why:
///   `info/exclude` is the person's file;
/// the wrapper adds one line to it and must never rewrite what is there.
#[test]
fn the_line_joins_the_persons_patterns_without_rewriting_them() {
    let root: PathBuf = fixture("trash-exclude-line");
    put(root.as_path(), ".git/info/exclude", b"node_modules/\n*.log");
    let added: bool = ensure_trash_is_excluded(root.join(".git").as_path()).expect("the line is written");
    assert!(added, "the line was not there before");
    assert_eq!(read(root.as_path(), ".git/info/exclude"), format!("node_modules/\n*.log\n{TRASH_EXCLUDE_LINE}\n"), "the person's bytes are kept and the line joins them");
    let again: bool = ensure_trash_is_excluded(root.join(".git").as_path()).expect("read again");
    assert!(!again, "a second add of the same worktree writes nothing");
    assert_eq!(read(root.as_path(), ".git/info/exclude"), format!("node_modules/\n*.log\n{TRASH_EXCLUDE_LINE}\n"), "and the file is exactly as it was");
    remove(root.as_path());
}

/// What:
///  An exclude pattern that is not text is the person's own bytes,
/// and the line joins them without changing one of
///  them,
/// so Git goes on ignoring the file the pattern names and `git add --all` stages none of it.
/// Why:
///  `info/exclude` is the person's file,
/// and a byte that is not UTF-8 can spell a name the person's Git
///  ignores;
/// a text round trip replaces that byte with three others,
/// which silently unpublishes the pattern and
///  puts the file it ignored into the next commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(readFileSync(exclude)).toEqual(Buffer.concat([Buffer.from([0xff, 0x0a]), Buffer.from('/.claude/cli-git-trash/\n')]));
/// ```
#[test]
fn a_pattern_that_is_not_text_is_kept_byte_for_byte() {
    let root: PathBuf = fixture("trash-exclude-not-text");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let name: &OsStr = OsStr::from_bytes(b"secret-\xff");
    std::fs::write(repo.join(name), b"the person's own file\n").expect("the file the pattern names");
    std::fs::write(repo.join(".git/info/exclude"), b"secret-\xff\n").expect("the person's pattern");
    assert!(git(repo.as_path(), &[OsStr::new("check-ignore"), OsStr::new("--quiet"), OsStr::new("--"), name]).status.success(), "control: the person's pattern ignores the file");
    let added: bool = ensure_trash_is_excluded(repo.join(".git").as_path()).expect("the line is written");
    assert!(added, "the line was not there before");
    assert_eq!(std::fs::read(repo.join(".git/info/exclude")).expect("the file"), b"secret-\xff\n/.claude/cli-git-trash/\n".to_vec(), "not one byte of the person's pattern changed");
    assert!(git(repo.as_path(), &[OsStr::new("check-ignore"), OsStr::new("--quiet"), OsStr::new("--"), name]).status.success(), "and the pattern still ignores the file it names");
    remove(root.as_path());
}

/// Child of the control below:
/// ask for the line in a process that may grow no file past its size limit,
/// so the write is
///  stopped half way.
#[test]
#[ignore = "started as a child process, under a file-size limit, by the control of a refused write"]
fn child_adds_the_line_under_a_file_size_limit() {
    let root: PathBuf = PathBuf::from(std::env::var_os(LIMITED_DIRECTORY_VARIABLE).expect("directory"));
    assert!(ensure_trash_is_excluded(root.join(".git").as_path()).is_err(), "a write past the process's limit fails");
}

/// What:
///  A write the system refuses half way leaves every pattern the person wrote exactly as it was,
/// and the call
///  answers the reason instead of claiming the line was added.
/// Why:
///   The file is the person's,
/// and the write that adds one line opens it for truncation before anything is
///  written;
/// a full disk,
/// a quota or a kill in that window would leave an empty or half-written file,
/// and every
///  file it ignored becomes stageable.
/// The size limit is the privilege-free way to make that write fail after the open
///  (measured on this host:
/// the limit refuses the bytes and the file keeps what it had).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(runUnderFileSizeLimit(0)).not.toBe(0);
/// expect(readFileSync(exclude)).toEqual(Buffer.from('node_modules/\n*.log\n'));
/// ```
#[test]
fn a_write_the_system_refuses_keeps_every_pattern_the_person_wrote() {
    let root: PathBuf = fixture("trash-exclude-refused");
    put(root.as_path(), ".git/info/exclude", b"node_modules/\n*.log\n");
    let status: std::process::ExitStatus = run_child_under_file_size_limit("worktree::worktree_trash_exclude_line::tests::child_adds_the_line_under_a_file_size_limit", 0, root.as_path());
    assert!(status.success(), "the child's write must fail under the limit, so its own control holds: {status:?}");
    assert_eq!(std::fs::read(root.join(".git/info/exclude")).expect("the file"), b"node_modules/\n*.log\n".to_vec(), "the person's patterns are all still there, byte for byte");
    remove(root.as_path());
}

/// A repository with no `info/exclude` at all gets one holding the line.
#[test]
fn a_repository_with_no_exclude_file_gets_one() {
    let root: PathBuf = fixture("trash-exclude-absent");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::remove_file(repo.join(".git/info/exclude")).expect("Git writes one at init");
    let added: bool = ensure_trash_is_excluded(repo.join(".git").as_path()).expect("the line is written");
    assert!(added);
    assert_eq!(read(repo.as_path(), ".git/info/exclude"), format!("{TRASH_EXCLUDE_LINE}\n"));
    remove(root.as_path());
}

/// What:
///  Real Git honours the repository's line in a linked worktree:
/// before the line the trash place is not ignored
///  and shows as untracked,
/// after it `git check-ignore` names the repository's own file and `git add --all` stages
///  nothing of a trash that stands there.
/// Why:
///   A linked worktree reads `$GIT_COMMON_DIR/info/exclude`,
/// never its own worktree's,
/// and that is measured rather than
///  assumed:
/// a line written for the worktree alone leaves the trash stageable.
/// The trash a target commit could stage is another
///  command's state,
/// so the proof is `git add --all` itself and not the ignore query alone.
#[test]
fn real_git_keeps_a_standing_trash_out_of_the_index_of_a_linked_worktree() {
    let root: PathBuf = fixture("trash-exclude-git");
    let repo: PathBuf = repository(root.as_path(), "repo");
    put(repo.as_path(), "tracked.txt", b"tracked\n");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--detach", linked.to_str().expect("text")]);
    let trash: String = format!("{TRASH_PARENT}/{}/d0", "a".repeat(40));
    put(linked.as_path(), format!("{trash}/payload/credentials").as_str(), b"another worktree's token\n");
    assert!(!trash_location_is_ignored(&crate::test_support::git_context(), linked.as_path()), "without the line the trash place is not ignored");
    assert_eq!(git_text(linked.as_path(), &["status", "--porcelain"]), "?? .claude/", "and Git shows the trash as untracked");
    let added: bool = ensure_trash_is_excluded(repo.join(".git").as_path()).expect("the line is written");
    assert!(added);
    assert!(trash_location_is_ignored(&crate::test_support::git_context(), linked.as_path()), "with the line the trash place is ignored");
    assert_eq!(git_text(linked.as_path(), &["status", "--porcelain"]), "", "Git lists nothing untracked any more");
    git(linked.as_path(), &["add", "--all"]);
    assert_eq!(git_text(linked.as_path(), &["diff", "--cached", "--name-only"]), "", "and `git add --all` stages nothing of it");
    remove(root.as_path());
}

/// What:
///  A target commit that stops ignoring the trash place does not open it again,
/// and neither does one that tries to
///  re-include it.
/// Why:
///   The commit is what changes the ignore rules,
/// so it must not be able to decide whether another command's state
///  is stageable.
/// `info/exclude` outranks nothing a commit can write for a path below a directory it excludes,
/// and the
///  measurement is `git add --all` after the commit,
/// with the trash standing.
#[test]
fn a_target_commit_that_stops_ignoring_the_trash_place_cannot_stage_it() {
    let root: PathBuf = fixture("trash-exclude-target");
    let repo: PathBuf = repository(root.as_path(), "repo");
    put(repo.as_path(), "tracked.txt", b"tracked\n");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--detach", linked.to_str().expect("text")]);
    ensure_trash_is_excluded(repo.join(".git").as_path()).expect("the line is written");
    let trash: String = format!("{TRASH_PARENT}/{}/d0", "b".repeat(40));
    put(linked.as_path(), format!("{trash}/payload/credentials").as_str(), b"another worktree's token\n");
    for rules in [".gitignore ignores nothing under .claude:\nnode_modules/\n", ".gitignore tries to re-include the trash parent:\n.claude/\n!.claude/cli-git-trash/\n"] {
        put(linked.as_path(), ".gitignore", rules.as_bytes());
        git(linked.as_path(), &["add", "--all"]);
        let staged: String = git_text(linked.as_path(), &["diff", "--cached", "--name-only"]);
        assert_eq!(staged, ".gitignore", "{rules}: only the commit's own file is staged and never the trash");
        git(linked.as_path(), &["reset", "--quiet"]);
    }
    remove(root.as_path());
}

/// What:
///  A named pipe at the repository's exclude file refuses the trash line at once,
///  and names the file and what stands there.
/// Why:
///   This step runs on the add's own path, where nothing bounds a synchronous read:
/// a plain read of a pipe with no writer never returns, and the add would wait for ever for a line it could not write.
/// A refusal is the shape the add already has for an exclude file it cannot write:
/// the trash is removed in the foreground instead.
///
/// In TS you'd write (pseudocode):
/// 
#[test]
fn a_named_pipe_at_the_exclude_file_never_holds_the_trash_line() {
    let root: PathBuf = fixture("trash-exclude-pipe");
    let common: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(common.join("info")).expect("the Git directory's info folder");
    let exclude: PathBuf = common.join("info/exclude");
    {
        use std::os::unix::ffi::OsStrExt as _;
        let named: std::ffi::CString = std::ffi::CString::new(exclude.as_os_str().as_bytes()).expect("a fixture path holds no NUL byte");
        // SAFETY: the string is NUL-terminated and outlives the call, and the name is a fresh path in the fixture.
        let made: libc::c_int = unsafe { libc::mkfifo(named.as_ptr(), 0o600) };
        assert_eq!(made, 0, "a named pipe at {}: {}", exclude.display(), std::io::Error::last_os_error());
    }
    let covered: PathBuf = root.clone();
    let waiting: Option<Result<bool, String>> = crate::test_support::finished_within_at(std::time::Duration::from_secs(10), move || return ensure_trash_is_excluded(common.as_path()));
    let written: Result<bool, String> = waiting.expect("the trash line must not wait on a named pipe at the exclude file");
    let Err(reason) = written else { panic!("a named pipe at the exclude file is refused: {written:?}") };
    assert!(reason.contains("exclude") && reason.contains("not a regular file"), "the refusal names the file and what stands there: {reason}");
    remove(covered.as_path());
}

/// What:
///  A form of the line that Git does not ignore by is not taken for the wrapper's own:
///  the line is added,
///  and Git then ignores the trash.
/// Why:
///   Git strips a pattern's trailing spaces and keeps its leading ones and its tabs,
/// so ` /.claude/cli-git-trash/` and a line with a trailing tab are lines the trash is not ignored by.
/// Treating either as already there would append nothing and leave the trash stageable,
/// which is the one thing the line exists to prevent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const form of [" /.claude/cli-git-trash/", "/.claude/cli-git-trash/\t"]) { put(exclude, form + "\n"); expect(ensureTrashIsExcluded(gitDir)).toBe(true); expect(gitCheckIgnore(repo)).toBe(false) }
/// ```
#[test]
fn a_line_git_does_not_ignore_by_is_never_taken_for_the_wrappers_own() {
    for form in [" /.claude/cli-git-trash/", "/.claude/cli-git-trash/\t"] {
        let root: PathBuf = fixture("trash-exclude-trimmed-form");
        let repo: PathBuf = repository(root.as_path(), "repo");
        put(repo.as_path(), ".git/info/exclude", format!("{form}\n").as_bytes());
        put(repo.as_path(), ".claude/cli-git-trash/credentials.txt", b"another command's token\n");
        assert!(!trash_location_is_ignored(&crate::test_support::git_context(), repo.as_path()), "{form:?}: Git does not ignore the trash by that line");
        let added: bool = ensure_trash_is_excluded(repo.join(".git").as_path()).expect("the line is written");
        assert!(added, "{form:?}: the wrapper does not take a form Git does not ignore by for its own");
        assert!(trash_location_is_ignored(&crate::test_support::git_context(), repo.as_path()), "{form:?}: with the line added Git ignores the trash");
        let bytes: String = read(repo.as_path(), ".git/info/exclude");
        assert!(bytes.contains(format!("{form}\n").as_str()), "{form:?}: the person's line stays as they wrote it: {bytes:?}");
        assert!(bytes.lines().any(|line| return line == "/.claude/cli-git-trash/"), "{form:?}: and the wrapper's own line is a line of its own: {bytes:?}");
        remove(root.as_path());
    }
}
