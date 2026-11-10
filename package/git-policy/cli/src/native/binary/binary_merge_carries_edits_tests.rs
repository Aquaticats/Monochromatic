//! What:
//!  A `git merge` through the built executable over a checkout that holds the person's own unsaved or staged work
//!  in the paths the merge changes.
//! Why:
//!  The owner's rules (`SPEC.md`,
//! "Merges and commits never fail"):
//! "Manual merge should never fail too.",
//!  "Auto merging should never fail.",
//! "never discard user data".
//! Native Git refuses such a merge
//!  (`error: Your local changes to the following files would be overwritten by merge`,
//! exit 2),
//! and so did the
//!  wrapper,
//! which is the answer "Carry edits forward (Recommended)"
//! (`SPEC.md:3331-3332`) replaces.
//! Every control here compares the checkout with what the person had and with the merge,
//!  never with a message alone:
//! a merge that lands must keep every byte of the person's work where they left it,
//! and a
//!  conflict must keep both sides in the file and still exit 0.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const before = snapshot(repo); const ran = await wrapped(['merge', '--no-edit', 'feature']);
//! expect(ran.code).toBe(0); expect(read('a.txt')).toContain('the person line'); expect(parents(head)).toHaveLength(2);
//! ```

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::support::{Fixture, Observed, direct, executable, fixture, git, observe, repository, without_wait_notices};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The eight lines of `a.txt` both sides start from.
const BASE_LINES: &str = "one\ntwo\nthree\nfour\nfive\nsix\nseven\neight\n";

/// What `feature` makes of the third line.
pub(super) const FEATURE_THIRD: &str = "three feature";

/// What `main` makes of the first line.
const MAIN_FIRST: &str = "one main";

/// The line the person adds in the controls that stay clear of the merge's own lines.
pub(super) const PERSON_LINE: &str = "the person's own line";

/// What:
///  A repository whose `main` and `feature` diverge:
/// `feature` changes the third line of `a.txt`,
///  adds `added.txt` and deletes `gone.txt`;
/// `main` changes the first line of `a.txt`.
/// Why:
///   The merge's paths are known exactly,
/// so a control can put the person's work on a line the merge does not touch,
///  or on the one it changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// base({ 'a.txt': LINES, 'keep.txt': 'keep base\n', 'gone.txt': 'gone base\n' }); branch('feature', ...);
/// ```
pub(super) fn diverged(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    std::fs::write(repo.join("a.txt"), BASE_LINES).expect("a.txt");
    std::fs::write(repo.join("keep.txt"), b"keep base\n").expect("keep.txt");
    std::fs::write(repo.join("gone.txt"), b"gone base\n").expect("gone.txt");
    git(fixture, repo.as_path(), &["add", "--", "a.txt", "keep.txt", "gone.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("a.txt"), BASE_LINES.replace("three", FEATURE_THIRD)).expect("a.txt");
    std::fs::write(repo.join("added.txt"), b"added by feature\n").expect("added.txt");
    git(fixture, repo.as_path(), &["rm", "--quiet", "--", "gone.txt"]);
    git(fixture, repo.as_path(), &["add", "--", "a.txt", "added.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=feature work"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("a.txt"), BASE_LINES.replace("one", MAIN_FIRST)).expect("a.txt");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main work"]);
    return repo;
}

/// `main`'s `a.txt` as the person found it.
pub(super) fn on_main_a() -> String {
    return BASE_LINES.replace("one", MAIN_FIRST);
}

/// One file's bytes as text,
/// or nothing when the file is not there.
pub(super) fn text_at(repo: &Path, name: &str) -> Option<String> {
    return std::fs::read(repo.join(name)).ok().map(|bytes| { return String::from_utf8_lossy(bytes.as_slice()).into_owned(); });
}

/// A wrapped command run to its end,
/// without the wait notices a slow step may print.
pub(super) fn run(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> Observed {
    let observed: Observed = observe(&mut wrapped_commit(fixture, repo, arguments), b"");
    return Observed { stderr: without_wait_notices(observed.stderr.as_slice()), ..observed };
}

/// The raw porcelain status,
/// untracked files included.
pub(super) fn status(fixture: &Fixture, repo: &Path) -> String {
    return git_text(fixture, repo, &["--no-optional-locks", "status", "--porcelain=v1", "--untracked-files=all"]);
}

/// The object the index holds for a path,
/// as `git ls-files --stage` prints it.
pub(super) fn index_entry(fixture: &Fixture, repo: &Path, path: &str) -> String {
    return git_text(fixture, repo, &["--no-optional-locks", "ls-files", "--stage", "--", path]);
}

/// The object `HEAD` holds for a path,
/// or the empty text when `HEAD` holds none.
pub(super) fn head_object(fixture: &Fixture, repo: &Path, path: &str) -> String {
    let output = direct(fixture)
        .current_dir(repo)
        .args(["rev-parse", "--quiet", "--verify", &format!("HEAD:{path}")])
        .output()
        .expect("the oracle starts");
    return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
}

/// The kinds of `refs/cli-git/` a merge may keep a commit on,
/// which a finished merge leaves empty.
pub(super) fn side_refs(fixture: &Fixture, repo: &Path) -> String {
    return git_text(fixture, repo, &["for-each-ref", "--format=%(refname)", "refs/cli-git"]);
}

/// The merge records of a worktree,
/// which a finished merge leaves empty.
pub(super) fn records(repo: &Path) -> String {
    let directory: PathBuf = repo.join(".git/monochromatic-git-policy-cli-merges");
    let mut names: Vec<String> = std::fs::read_dir(directory.as_path())
        .map(|entries| {
            return entries
                .flatten()
                .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
                .collect();
        })
        .unwrap_or_default();
    names.sort();
    return names.join(" ");
}

/// What:
///  Every promise a finished merge keeps,
/// checked from one place:
/// two parents,
/// no `MERGE_HEAD`,
/// no unmerged entry,
/// no record and no side ref left,
/// and a repository Git itself calls sound.
/// Why:
///   The owner's bar is that the index and the worktree hold either what they held before the command
/// or the whole
///  result,
/// and that no `MERGE_HEAD` is left by a merge that did not finish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(parents(head)).toHaveLength(2); expect(exists('.git/MERGE_HEAD')).toBe(false);
/// expect(records()).toBe(''); expect(sideRefs()).toBe(''); expect(fsck()).toBe(0);
/// ```
pub(super) fn assert_merged(fixture: &Fixture, repo: &Path, label: &str) {
    let parents: Vec<String> = git_text(fixture, repo, &["rev-list", "--parents", "-n", "1", "HEAD"])
        .split_whitespace()
        .map(String::from)
        .collect();
    assert_eq!(parents.len(), 3, "{label}: a merge commit with two parents, not {:?}", parents);
    assert!(!repo.join(".git/MERGE_HEAD").exists(), "{label}: no MERGE_HEAD is left");
    assert_eq!(git_text(fixture, repo, &["--no-optional-locks", "ls-files", "--unmerged"]), "", "{label}: no unmerged entry");
    assert_eq!(records(repo), "", "{label}: no merge record is left");
    assert_eq!(side_refs(fixture, repo), "", "{label}: no side ref is left");
    let fsck = direct(fixture).current_dir(repo).args(["fsck", "--strict", "--no-dangling"]).output().expect("fsck starts");
    assert!(fsck.status.success(), "{label}: git fsck: {}", String::from_utf8_lossy(fsck.stderr.as_slice()));
}

/// The text of a failure message,
/// for an assertion line.
pub(super) fn said(observed: &Observed) -> String {
    return String::from_utf8_lossy(observed.stderr.as_slice()).into_owned();
}

/// An unsaved edit on a line the merge does not change:
/// the merge lands and the edit is still there,
/// unsaved.
#[test]
fn an_unsaved_edit_away_from_the_merges_lines_is_carried_and_the_merge_lands() {
    let fixture: Fixture = fixture("carry-unsaved-edit");
    let repo: PathBuf = diverged(&fixture, "repo");
    let edited: String = format!("{}{PERSON_LINE}\n", on_main_a());
    std::fs::write(repo.join("a.txt"), &edited).expect("the person's edit");
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", said(&observed));
    assert_merged(&fixture, repo.as_path(), "unsaved edit");
    let after: String = text_at(repo.as_path(), "a.txt").expect("a.txt is there");
    assert!(after.contains(MAIN_FIRST), "the merge's first line lands: {after:?}");
    assert!(after.contains(FEATURE_THIRD), "the merge's third line lands: {after:?}");
    assert_eq!(after, format!("{}{PERSON_LINE}\n", on_main_a().replace("three", FEATURE_THIRD)), "exactly the merged file with the person's line");
    assert_eq!(status(&fixture, repo.as_path()), " M a.txt\n", "the edit is still unsaved");
    assert_eq!(
        index_entry(&fixture, repo.as_path(), "a.txt").split_whitespace().nth(1),
        Some(head_object(&fixture, repo.as_path(), "a.txt").as_str()),
        "the index holds the merge's own entry"
    );
    remove_settled(&fixture);
}

/// A staged edit on a line the merge does not change:
/// the merge lands and the edit is still staged,
/// over the merged content.
#[test]
fn a_staged_edit_away_from_the_merges_lines_is_carried_and_stays_staged() {
    let fixture: Fixture = fixture("carry-staged-edit");
    let repo: PathBuf = diverged(&fixture, "repo");
    let staged: String = format!("{}{PERSON_LINE}\n", on_main_a());
    std::fs::write(repo.join("a.txt"), &staged).expect("the person's edit");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", said(&observed));
    assert_merged(&fixture, repo.as_path(), "staged edit");
    assert_eq!(status(&fixture, repo.as_path()), "M  a.txt\n", "the edit is still staged, with nothing unstaged over it");
    let cached: String = git_text(&fixture, repo.as_path(), &["diff", "--cached", "--unified=0", "--", "a.txt"]);
    assert!(cached.contains(&format!("+{PERSON_LINE}")), "the staged difference holds the person's line: {cached}");
    assert!(!cached.contains(FEATURE_THIRD) && !cached.contains(MAIN_FIRST), "the merge's own lines are not staged: {cached}");
    assert_eq!(
        text_at(repo.as_path(), "a.txt").unwrap_or_default().as_str(),
        format!("{}{PERSON_LINE}\n", on_main_a().replace("three", FEATURE_THIRD)).as_str(),
        "the worktree holds the merged file with the person's line"
    );
    remove_settled(&fixture);
}

/// An edit of the very line the merge changes:
/// the file holds conflict markers with both sides,
/// the command exits 0,
/// and it names the file and what to do.
#[test]
fn a_conflicting_edit_is_kept_with_markers_and_the_command_exits_zero() {
    let fixture: Fixture = fixture("carry-conflict");
    let repo: PathBuf = diverged(&fixture, "repo");
    std::fs::write(repo.join("a.txt"), BASE_LINES.replace("one", MAIN_FIRST).replace("three", "three person")).expect("the person's edit");
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(observed.code, Some(0), "a conflict the person must judge does not fail the merge: {}", said(&observed));
    assert_merged(&fixture, repo.as_path(), "conflicting edit");
    let after: String = text_at(repo.as_path(), "a.txt").expect("a.txt is there");
    assert!(after.contains("<<<<<<<") && after.contains("=======") && after.contains(">>>>>>>"), "the file holds conflict markers: {after:?}");
    assert!(after.contains("three person"), "the person's side is in the file: {after:?}");
    assert!(after.contains(FEATURE_THIRD), "the merge's side is in the file: {after:?}");
    assert!(said(&observed).contains("a.txt"), "the command names the file: {}", said(&observed));
    assert!(said(&observed).contains("conflict markers"), "the command says what is in the file: {}", said(&observed));
    assert_eq!(status(&fixture, repo.as_path()), " M a.txt\n", "the markers are the person's unsaved work");
    remove_settled(&fixture);
}

/// An untracked file where the merge adds that path:
/// both sides are kept in the file,
/// the merge lands,
/// and the command exits 0.
#[test]
fn an_untracked_file_where_the_merge_adds_the_path_keeps_both_sides() {
    let fixture: Fixture = fixture("carry-untracked-add");
    let repo: PathBuf = diverged(&fixture, "repo");
    std::fs::write(repo.join("added.txt"), b"the person's untracked file\n").expect("the person's file");
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", said(&observed));
    assert_merged(&fixture, repo.as_path(), "untracked add");
    let after: String = text_at(repo.as_path(), "added.txt").expect("added.txt is there");
    assert!(after.contains("the person's untracked file"), "the person's bytes are kept: {after:?}");
    assert!(after.contains("added by feature"), "the merge's bytes are kept: {after:?}");
    assert!(!head_object(&fixture, repo.as_path(), "added.txt").is_empty(), "the merge's version is in the commit");
    assert!(said(&observed).contains("added.txt"), "the command names the file: {}", said(&observed));
    remove_settled(&fixture);
}

/// The person's edit in a file the merge deletes:
/// the file stays with their bytes,
/// the merge lands,
/// and the command names the file.
#[test]
fn an_edit_in_a_file_the_merge_deletes_is_kept() {
    let fixture: Fixture = fixture("carry-edit-in-deleted");
    let repo: PathBuf = diverged(&fixture, "repo");
    std::fs::write(repo.join("gone.txt"), format!("gone base\n{PERSON_LINE}\n")).expect("the person's edit");
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", said(&observed));
    assert_merged(&fixture, repo.as_path(), "edit in a deleted file");
    assert_eq!(text_at(repo.as_path(), "gone.txt").unwrap_or_default(), format!("gone base\n{PERSON_LINE}\n"), "the person's bytes are kept");
    assert!(head_object(&fixture, repo.as_path(), "gone.txt").is_empty(), "the merge's deletion is in the commit");
    assert!(said(&observed).contains("gone.txt"), "the command names the file: {}", said(&observed));
    remove_settled(&fixture);
}

/// What:
///  The person deleted a path the merge changes:
/// the deletion stands in the checkout,
///  the merge lands,
/// and the line says where the merge's version is instead of naming a file that
///  holds bytes the path does not have.
/// Why:
///  A line that says "the file holds what you wrote" over an absent file sends the person looking for their
///  work in the wrong place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rm('a.txt'); const ran = await wrapped(['merge', '--no-edit', 'feature']);
/// expect(read('a.txt')).toBeUndefined(); expect(ran.stderr).not.toSay('holds what you wrote');
/// ```
#[test]
fn a_path_the_person_deleted_that_the_merge_changes_is_named_as_deleted() {
    let fixture: Fixture = fixture("carry-deleted-path");
    let repo: PathBuf = diverged(&fixture, "repo");
    std::fs::remove_file(repo.join("a.txt")).expect("the person's deletion");
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", said(&observed));
    assert_merged(&fixture, repo.as_path(), "deleted path");
    assert_eq!(text_at(repo.as_path(), "a.txt"), None, "the person's deletion stands");
    assert!(!head_object(&fixture, repo.as_path(), "a.txt").is_empty(), "the merge's version is in the merge commit");
    let words: String = said(&observed);
    assert!(words.contains("a.txt"), "the line names the path: {words}");
    assert!(words.contains("keeps your own deletion"), "the line says the checkout keeps their deletion: {words}");
    assert!(!words.contains("holds what you wrote"), "the line claims no file that holds their bytes: {words}");
    assert!(!words.contains("your bytes are kept at"), "the line claims no bytes of theirs at the path: {words}");
    assert!(words.contains("git restore --source=HEAD --staged --worktree -- a.txt"), "the line names how to take the merge's version: {words}");
    remove_settled(&fixture);
}

/// What:
///  The person deleted the path and the index holds that deletion:
/// the checkout and the index both keep it,
///  and the line names the same way back to the merge's version.
/// Why:
///  A deletion the index holds leaves no entry for `git restore -- a.txt` to restore from,
///  so the advice has to name
///  the merge commit as its source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['rm', '--quiet', '--', 'a.txt']); const ran = await wrapped(['merge', '--no-edit', 'feature']);
/// expect(status()).toBe('D  a.txt\n'); expect(ran.stderr).toSay('restore --source=HEAD');
/// ```
#[test]
fn a_deletion_the_person_staged_that_the_merge_changes_is_named_as_deleted() {
    let fixture: Fixture = fixture("carry-deleted-staged");
    let repo: PathBuf = diverged(&fixture, "repo");
    git(&fixture, repo.as_path(), &["rm", "--quiet", "--", "a.txt"]);
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", said(&observed));
    assert_merged(&fixture, repo.as_path(), "staged deletion");
    assert_eq!(text_at(repo.as_path(), "a.txt"), None, "the person's deletion stands");
    assert_eq!(status(&fixture, repo.as_path()), "D  a.txt\n", "the deletion is still staged");
    let words: String = said(&observed);
    assert!(words.contains("keeps your own deletion"), "the line says the checkout keeps their deletion: {words}");
    assert!(!words.contains("holds what you wrote"), "the line claims no file that holds their bytes: {words}");
    assert!(words.contains("git restore --source=HEAD --staged --worktree -- a.txt"), "the line names how to take the merge's version: {words}");
    remove_settled(&fixture);
}

/// What:
///  The person's edit in a path the merge deletes:
/// the merge commit holds no such path,
///  and the line says so and names a
///  command that takes that deletion.
/// Why:
///  `git restore -- gone.txt` restores from the index,
/// and the index holds no entry for a path the merge
///  deleted,
/// so the old advice failed with a pathspec error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('gone.txt', 'gone base\nthe person\'s own line\n'); const ran = await wrapped(['merge', '--no-edit', 'feature']);
/// expect(ran.stderr).toSay('merge commit holds no gone.txt'); expect(ran.stderr).not.toSay('restore -- gone.txt');
/// ```
#[test]
fn a_path_the_merge_deletes_is_named_with_the_merges_absence() {
    let fixture: Fixture = fixture("carry-deleted-by-merge");
    let repo: PathBuf = diverged(&fixture, "repo");
    std::fs::write(repo.join("gone.txt"), format!("gone base\n{PERSON_LINE}\n")).expect("the person's edit");
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", said(&observed));
    assert_merged(&fixture, repo.as_path(), "merge deletes the path");
    let words: String = said(&observed);
    assert!(words.contains("The merge commit holds no gone.txt"), "the line says the merge's version is no file: {words}");
    assert!(!words.contains("git restore -- gone.txt"), "the line does not name a restore the index cannot make: {words}");
    assert!(words.contains("`rm -- gone.txt`"), "the line names the command that takes the merge's deletion: {words}");
    remove_settled(&fixture);
}

/// A `git pull` that merges,
/// over an unsaved edit the pull's merge does not touch:
/// the pull lands and the edit is still there.
#[test]
fn a_pull_that_merges_carries_an_unsaved_edit() {
    let fixture: Fixture = fixture("carry-pull");
    let repo: PathBuf = diverged(&fixture, "repo");
    let remote: PathBuf = fixture.root.join("remote.git");
    git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "remote.git"]);
    git(&fixture, repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(&fixture, repo.as_path(), &["push", "--quiet", "origin", "feature:main"]);
    git(&fixture, repo.as_path(), &["config", "branch.main.remote", "origin"]);
    git(&fixture, repo.as_path(), &["config", "branch.main.merge", "refs/heads/main"]);
    let edited: String = format!("{}{PERSON_LINE}\n", on_main_a());
    std::fs::write(repo.join("a.txt"), &edited).expect("the person's edit");
    let observed: Observed = run(&fixture, repo.as_path(), &["pull", "--no-rebase", "--no-edit"]);
    assert_eq!(observed.code, Some(0), "{}", said(&observed));
    assert_merged(&fixture, repo.as_path(), "pull");
    let after: String = text_at(repo.as_path(), "a.txt").expect("a.txt is there");
    assert!(after.contains(FEATURE_THIRD) && after.contains(&format!("{PERSON_LINE}\n")), "the merge landed and the edit is there: {after:?}");
    assert_eq!(status(&fixture, repo.as_path()), " M a.txt\n", "the edit is still unsaved");
    remove_settled(&fixture);
}

/// A checkout with no work in the merge's paths is completed exactly as before,
/// and no carry is left behind.
#[test]
fn a_clean_checkout_merges_with_no_trace_of_a_carry() {
    let fixture: Fixture = fixture("carry-clean");
    let repo: PathBuf = diverged(&fixture, "repo");
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", said(&observed));
    assert_merged(&fixture, repo.as_path(), "clean");
    assert_eq!(status(&fixture, repo.as_path()), "", "nothing is left uncommitted");
    remove_settled(&fixture);
}

/// What:
///  An editor that appends one line to the message it was handed,
/// records the file it read,
/// and counts the runs it
///  made.
/// Why:
///   The line shows in the merge commit,
/// so a control can tell the message the person wrote from the one Git
///  would have written,
/// and the count shows the editor opened once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const editor = script(`echo run >> '${runs}'; cp "$1" '${read}'; printf '\nedited\n' >> "$1"`);
/// ```
fn appending_editor(fixture: &Fixture, name: &str) -> PathBuf {
    let path: PathBuf = fixture.root.join(name);
    let script: String = format!(
        "#!/bin/sh\necho run >> '{runs}'\ncp \"$1\" '{read}'\nprintf '\\nedited in the editor\\n' >> \"$1\"\n",
        runs = fixture.root.join(format!("{name}-runs")).display(),
        read = fixture.root.join(format!("{name}-read")).display()
    );
    executable(path.as_path(), script.as_bytes());
    return path;
}

/// What:
///  Run a merge that opens the editor named,
/// as a person at a terminal would,
/// and return what they saw.
/// Why:
///   A control's standard input is no terminal,
/// so `GIT_MERGE_AUTOEDIT=yes` is what makes the merge open the editor,
///  which is the same decision the wrapper reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// observe(wrapped(['merge', 'feature'], { GIT_EDITOR: editor, GIT_MERGE_AUTOEDIT: 'yes' }));
/// ```
fn run_with_editor(fixture: &Fixture, repo: &Path, editor: &Path, arguments: &[&str]) -> Observed {
    let observed: Observed = observe(
        wrapped_commit(fixture, repo, arguments).env("GIT_EDITOR", editor).env("GIT_MERGE_AUTOEDIT", "yes"),
        b"",
    );
    return Observed { stderr: without_wait_notices(observed.stderr.as_slice()), ..observed };
}

/// What:
///  A merge that opens an editor carries the person's unsaved edit forward,
/// and the message they write is the merge's.
/// Why:
///   Native Git performs the merge before the editor opens,
/// so it refuses a checkout that holds the person's own work in
///  the paths the merge changes (`error: Your local changes to the following files would be overwritten by merge`,
///  exit 2),
/// and the wrapper forwarded such a merge for the same reason.
/// This is the one case "Carry edits forward
///  (Recommended)" left.
#[test]
fn a_merge_that_opens_an_editor_carries_an_unsaved_edit() {
    let fixture: Fixture = fixture("carry-editor-unsaved");
    let repo: PathBuf = diverged(&fixture, "repo");
    let editor: PathBuf = appending_editor(&fixture, "editor");
    let edited: String = format!("{}{PERSON_LINE}\n", on_main_a());
    std::fs::write(repo.join("a.txt"), &edited).expect("the person's edit");
    let observed: Observed = run_with_editor(&fixture, repo.as_path(), editor.as_path(), &["merge", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", said(&observed));
    assert_merged(&fixture, repo.as_path(), "editor and unsaved edit");
    let after: String = text_at(repo.as_path(), "a.txt").expect("a.txt is there");
    assert_eq!(
        after,
        format!("{}{PERSON_LINE}\n", on_main_a().replace("three", FEATURE_THIRD)),
        "the merged file with the person's line"
    );
    assert_eq!(status(&fixture, repo.as_path()), " M a.txt\n", "the edit is still unsaved");
    let message: String = git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%B"]);
    assert!(message.contains("edited in the editor"), "the merge carries the message the person wrote: {message:?}");
    let read: String = std::fs::read_to_string(fixture.root.join("editor-read")).expect("what the editor read");
    assert!(read.starts_with("Merge branch 'feature'\n"), "the editor opened on the merge's own message: {read:?}");
    assert!(read.contains("# Please enter a commit message"), "and on the help lines Git writes: {read:?}");
    assert_eq!(std::fs::read_to_string(fixture.root.join("editor-runs")).expect("the editor's runs"), "run\n", "the editor opened once");
    assert!(!repo.join(".git/MERGE_MSG").exists(), "no merge message file is left behind");
    remove_settled(&fixture);
}

/// What:
///  A merge that opens an editor carries the person's staged edit forward,
/// and it stays staged.
/// Why:
///   Native Git refuses such a merge as well,
/// although the merge may touch none of the staged paths,
/// and the
///  message the person writes belongs to a merge that is made from objects.
#[test]
fn a_merge_that_opens_an_editor_carries_a_staged_edit() {
    let fixture: Fixture = fixture("carry-editor-staged");
    let repo: PathBuf = diverged(&fixture, "repo");
    let editor: PathBuf = appending_editor(&fixture, "editor");
    let staged: String = format!("{}{PERSON_LINE}\n", on_main_a());
    std::fs::write(repo.join("a.txt"), &staged).expect("the person's edit");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    let observed: Observed = run_with_editor(&fixture, repo.as_path(), editor.as_path(), &["merge", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", said(&observed));
    assert_merged(&fixture, repo.as_path(), "editor and staged edit");
    assert_eq!(status(&fixture, repo.as_path()), "M  a.txt\n", "the edit is still staged, with nothing unstaged over it");
    let cached: String = git_text(&fixture, repo.as_path(), &["diff", "--cached", "--unified=0", "--", "a.txt"]);
    assert!(cached.contains(&format!("+{PERSON_LINE}")), "the staged difference holds the person's line: {cached}");
    assert!(!cached.contains(FEATURE_THIRD) && !cached.contains(MAIN_FIRST), "the merge's own lines are not staged: {cached}");
    let message: String = git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%B"]);
    assert!(message.contains("edited in the editor"), "the merge carries the message the person wrote: {message:?}");
    remove_settled(&fixture);
}

/// What:
///  An editor that leaves no message ends the merge with status 1 and changes nothing.
/// Why:
///   Git aborts a commit whose edited message is empty (`error: Empty commit message.`),
/// that ending is the
///  person's,
/// and no record,
/// ref or message file may be left by a merge that was not made.
#[test]
fn an_editor_that_leaves_no_message_leaves_the_checkout_as_it_was() {
    let fixture: Fixture = fixture("carry-editor-empty");
    let repo: PathBuf = diverged(&fixture, "repo");
    let before: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let edited: String = format!("{}{PERSON_LINE}\n", on_main_a());
    std::fs::write(repo.join("a.txt"), &edited).expect("the person's edit");
    let editor: PathBuf = fixture.root.join("empty-editor");
    executable(editor.as_path(), b"#!/bin/sh\nprintf '# only a comment\\n' > \"$1\"\n");
    let observed: Observed = run_with_editor(&fixture, repo.as_path(), editor.as_path(), &["merge", "feature"]);
    assert_eq!(observed.code, Some(1), "{}", said(&observed));
    assert!(said(&observed).contains("the editor left an empty message"), "the sentence says what stopped it: {}", said(&observed));
    assert!(said(&observed).contains("nothing in the repository changed"), "and what the person will find: {}", said(&observed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), before, "no merge was made");
    assert_eq!(text_at(repo.as_path(), "a.txt").unwrap_or_default(), edited, "the person's bytes are where they left them");
    assert_eq!(status(&fixture, repo.as_path()), " M a.txt\n", "nothing was staged by the attempt");
    assert!(!repo.join(".git/MERGE_MSG").exists(), "no merge message file is left behind");
    assert_eq!(records(repo.as_path()), "", "no merge record is left");
    assert_eq!(side_refs(&fixture, repo.as_path()), "", "no side ref is left");
    remove_settled(&fixture);
}

/// What:
///  An editor that ends with a failure ends the merge with status 1 and changes nothing.
/// Why:
///   The person's own program failing is the person's ending,
/// as a signer that fails is,
/// and the sentence
///  names the editor so they can repair it.
#[test]
fn an_editor_that_fails_leaves_the_checkout_as_it_was() {
    let fixture: Fixture = fixture("carry-editor-failed");
    let repo: PathBuf = diverged(&fixture, "repo");
    let before: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let edited: String = format!("{}{PERSON_LINE}\n", on_main_a());
    std::fs::write(repo.join("a.txt"), &edited).expect("the person's edit");
    let editor: PathBuf = fixture.root.join("failing-editor");
    executable(editor.as_path(), b"#!/bin/sh\nexit 3\n");
    let observed: Observed = run_with_editor(&fixture, repo.as_path(), editor.as_path(), &["merge", "feature"]);
    assert_eq!(observed.code, Some(1), "{}", said(&observed));
    assert!(said(&observed).contains("there was a problem with the editor"), "the sentence says what happened: {}", said(&observed));
    assert!(
        said(&observed).contains(editor.to_str().expect("the script's path is text")),
        "and names the editor: {}",
        said(&observed)
    );
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), before, "no merge was made");
    assert_eq!(text_at(repo.as_path(), "a.txt").unwrap_or_default(), edited, "the person's bytes are where they left them");
    assert!(!repo.join(".git/MERGE_MSG").exists(), "no merge message file is left behind");
    assert_eq!(records(repo.as_path()), "", "no merge record is left");
    remove_settled(&fixture);
}

/// What:
///  A checkout with nothing of the person's own work in the merge's paths is native Git's merge,
/// editor and all.
/// Why:
///   The wrapper acts on the case native Git refuses;
/// everywhere else the merge that opens an editor is Git's own,
/// and this
///  control keeps that boundary.
#[test]
fn a_clean_checkout_that_opens_an_editor_still_merges() {
    let fixture: Fixture = fixture("carry-editor-clean");
    let repo: PathBuf = diverged(&fixture, "repo");
    let editor: PathBuf = appending_editor(&fixture, "editor");
    let observed: Observed = run_with_editor(&fixture, repo.as_path(), editor.as_path(), &["merge", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", said(&observed));
    assert_merged(&fixture, repo.as_path(), "editor and clean checkout");
    let message: String = git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%B"]);
    assert!(message.contains("edited in the editor"), "the merge carries the message the person wrote: {message:?}");
    assert_eq!(status(&fixture, repo.as_path()), "", "nothing is left uncommitted");
    remove_settled(&fixture);
}
