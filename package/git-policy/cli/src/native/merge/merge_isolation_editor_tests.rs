//! What:
//!  The editor a merge from objects opens:
//! the file it reads,
//! the message that comes back,
//!  and each way it can end without one.
//! Why:
//!  The bytes the editor reads are the merge message Git writes,
//! so a person sees the merge they asked for and the
//!  help lines Git would have shown;
//! a merge that is not committed leaves no message file behind.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const edited = editWith({ session, gitDir }, b"Merge branch 'feature'\n", script);
//! ```

use super::*;
use crate::test_support::{executable, fixture, git, git_context, remove, repository};
use crate::transaction_git::GitContext;
use std::ffi::OsString;
use std::path::PathBuf;

/// A script that copies the file it was handed to `seen` and appends `line` to it.
fn scripted_editor(parent: &Path, name: &str, body: &str) -> PathBuf {
    let path: PathBuf = parent.join(name);
    executable(path.as_path(), body.as_bytes());
    return path;
}

/// The session of a control,
/// naming a repository by the global options of the command.
fn session_in(repo: &Path) -> Session {
    let mut context: GitContext = git_context();
    context.global_prefix = vec![OsString::from("-C"), repo.as_os_str().to_os_string()];
    return Session { context };
}

/// The editor reads Git's own message file with Git's help lines under the message,
/// and what it leaves is cleaned as a commit that opens an editor cleans it.
#[test]
fn the_editor_reads_gits_message_file_and_its_message_is_cleaned() {
    let parent: PathBuf = fixture("merge-editor-file");
    let repo: PathBuf = repository(parent.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let written: PathBuf = parent.join("what-the-editor-read");
    let body: String = format!(
        "#!/bin/sh\ncp \"$1\" '{written}'\nprintf '\\nedited in the editor  \\n' >> \"$1\"\n",
        written = written.display()
    );
    let editor: PathBuf = scripted_editor(parent.as_path(), "editor", body.as_str());
    let session: Session = session_in(repo.as_path());
    let place: EditorPlace<'_> = EditorPlace { session: &session, git_dir: git_dir.as_path() };
    let edited: Vec<u8> = edit_with(&place, b"Merge branch 'feature'\n", editor.to_str().expect("the script's path is text"))
        .expect("the editor ran");
    assert_eq!(
        std::fs::read_to_string(written.as_path()).expect("what the editor read"),
        format!("Merge branch 'feature'\n{HELP_LINES}"),
        "the merge message and Git's help lines"
    );
    assert_eq!(edited, b"Merge branch 'feature'\n\nedited in the editor\n".to_vec(), "the cleaned message");
    assert!(!git_dir.join("MERGE_MSG").exists(), "the message file is not left behind");
    remove(parent.as_path());
}

/// A comment character that is not `#` opens the message alone,
/// because the help lines would not be stripped.
#[test]
fn another_comment_character_opens_the_message_alone() {
    let parent: PathBuf = fixture("merge-editor-comment-char");
    let repo: PathBuf = repository(parent.as_path(), "repo");
    git(repo.as_path(), &["config", "core.commentChar", ";"]);
    let written: PathBuf = parent.join("what-the-editor-read");
    let body: String = format!(
        "#!/bin/sh\ncp \"$1\" '{written}'\nprintf 'kept # because the comment character is a semicolon\\n' >> \"$1\"\n",
        written = written.display()
    );
    let editor: PathBuf = scripted_editor(parent.as_path(), "editor", body.as_str());
    let session: Session = session_in(repo.as_path());
    let git_dir: PathBuf = repo.join(".git");
    let place: EditorPlace<'_> = EditorPlace { session: &session, git_dir: git_dir.as_path() };
    let edited: Vec<u8> = edit_with(&place, b"Merge branch 'feature'\n", editor.to_str().expect("the script's path is text"))
        .expect("the editor ran");
    assert_eq!(
        std::fs::read_to_string(written.as_path()).expect("what the editor read"),
        "Merge branch 'feature'\n",
        "the message with no help lines"
    );
    assert_eq!(
        edited,
        b"Merge branch 'feature'\nkept # because the comment character is a semicolon\n".to_vec(),
        "a line starting with `#` is the person's text under another comment character"
    );
    remove(parent.as_path());
}

/// What:
///  An environment that resolves no editor is native Git's,
/// and so is one whose Git will not answer the question.
/// Why:
///   Native Git words that ending itself,
/// and this command changes nothing before it asks.
#[test]
fn an_environment_that_resolves_no_editor_leaves_native_git_the_command() {
    let parent: PathBuf = fixture("merge-editor-none");
    let repo: PathBuf = repository(parent.as_path(), "repo");
    let session: Session = session_in(repo.as_path());
    let git_dir: PathBuf = repo.join(".git");
    let muted: GitContext = GitContext {
        overlay: vec![(OsString::from("GIT_EDITOR"), OsString::from(""))],
        ..git_context()
    };
    let muted_session: Session = Session { context: muted };
    let place: EditorPlace<'_> = EditorPlace { session: &muted_session, git_dir: git_dir.as_path() };
    assert_eq!(editor_command(&muted_session), None, "an empty editor resolves none");
    assert!(matches!(edit_message(&place, b"Merge branch 'feature'\n"), Err(EditorRefusal::NoEditor)), "no editor, no message");
    let refusing: GitContext = GitContext {
        global_prefix: vec![OsString::from("-c"), OsString::from("not-a-setting")],
        ..session.context.clone()
    };
    assert_eq!(editor_command(&Session { context: refusing }), None, "a Git that refuses answers nothing");
    remove(parent.as_path());
}

/// What:
///  An editor that ends with a failure stops the merge,
/// and the message file goes with it.
/// Why:
///   Nothing of the checkout has changed when the editor runs,
/// so the refusal is the whole of the outcome,
/// and a
///  `MERGE_MSG` left behind would name a merge that will never be made.
#[test]
fn an_editor_that_ends_with_a_failure_stops_the_merge() {
    let parent: PathBuf = fixture("merge-editor-failed");
    let repo: PathBuf = repository(parent.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let editor: PathBuf = scripted_editor(parent.as_path(), "editor", "#!/bin/sh\nexit 3\n");
    let session: Session = session_in(repo.as_path());
    let place: EditorPlace<'_> = EditorPlace { session: &session, git_dir: git_dir.as_path() };
    let refused: EditorRefusal = edit_with(&place, b"Merge branch 'feature'\n", editor.to_str().expect("the script's path is text"))
        .expect_err("the editor failed");
    let EditorRefusal::Stopped(detail) = refused else {
        panic!("a failing editor is reported as what stopped: {refused:?}");
    };
    assert!(detail.contains("there was a problem with the editor"), "{detail}");
    assert!(detail.contains(editor.to_str().expect("text")), "the detail names the editor: {detail}");
    assert!(!git_dir.join("MERGE_MSG").exists(), "no message file is left");
    remove(parent.as_path());
}

/// What:
///  An editor that leaves nothing stops the merge too,
/// because a merge commit has no message.
/// Why:
///   Git aborts a commit whose edited message is empty (`error: Empty commit message.`),
/// and so does this.
#[test]
fn an_editor_that_leaves_nothing_stops_the_merge() {
    let parent: PathBuf = fixture("merge-editor-empty");
    let repo: PathBuf = repository(parent.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let editor: PathBuf = scripted_editor(parent.as_path(), "editor", "#!/bin/sh\nprintf '# only a comment\\n' > \"$1\"\n");
    let session: Session = session_in(repo.as_path());
    let place: EditorPlace<'_> = EditorPlace { session: &session, git_dir: git_dir.as_path() };
    let refused: EditorRefusal = edit_with(&place, b"Merge branch 'feature'\n", editor.to_str().expect("the script's path is text"))
        .expect_err("the editor left no message");
    assert!(matches!(refused, EditorRefusal::Empty), "{refused:?}");
    assert!(!git_dir.join("MERGE_MSG").exists(), "no message file is left");
    remove(parent.as_path());
}

/// What:
///  A message the person wrote is asked for once,
/// however many rounds the merge needs.
/// Why:
///   A round made again because the branch moved must not reopen the editor on a fresh message,
/// which would lose
///  the words the person typed.
#[test]
fn the_person_is_asked_for_the_message_once() {
    let parent: PathBuf = fixture("merge-editor-once");
    let repo: PathBuf = repository(parent.as_path(), "repo");
    let runs: PathBuf = parent.join("runs");
    let body: String = format!(
        "#!/bin/sh\necho run >> '{runs}'\nprintf 'the person wrote this\\n' >> \"$1\"\n",
        runs = runs.display()
    );
    let editor: PathBuf = scripted_editor(parent.as_path(), "editor", body.as_str());
    let context: GitContext = GitContext {
        overlay: vec![(OsString::from("GIT_EDITOR"), editor.as_os_str().to_os_string())],
        ..session_in(repo.as_path()).context
    };
    let session: Session = Session { context };
    let git_dir: PathBuf = repo.join(".git");
    let mut kept: Option<Vec<u8>> = None;
    let place: EditorPlace<'_> = EditorPlace { session: &session, git_dir: git_dir.as_path() };
    let first: Vec<u8> =
        asked_once(&mut kept, &place, b"Merge branch 'feature'\n").expect("the editor ran");
    let second: Vec<u8> =
        asked_once(&mut kept, &place, b"Merge branch 'feature'\n").expect("the message was kept");
    assert_eq!(first, second, "the same message both times");
    assert_eq!(
        first,
        b"Merge branch 'feature'\nthe person wrote this\n".to_vec(),
        "the words the person typed are the message"
    );
    assert_eq!(std::fs::read_to_string(runs.as_path()).expect("the runs"), "run\n", "the editor ran once");
    remove(parent.as_path());
}

/// The editor Git resolves for this command is the one it names,
/// space and options included,
/// with the command's own global options applied.
#[test]
fn the_editor_is_the_one_git_resolves() {
    let parent: PathBuf = fixture("merge-editor-resolution");
    let repo: PathBuf = repository(parent.as_path(), "repo");
    let session: Session = session_in(repo.as_path());
    let configured: GitContext = GitContext {
        overlay: vec![(OsString::from("GIT_EDITOR"), OsString::from("/some/editor --wait"))],
        ..session.context.clone()
    };
    assert_eq!(editor_command(&Session { context: configured }), Some(String::from("/some/editor --wait")));
    let prefixed: GitContext = GitContext {
        global_prefix: vec![OsString::from("-c"), OsString::from("core.editor=/configured/editor")],
        overlay: vec![(OsString::from("GIT_EDITOR"), OsString::from("/from/the/environment"))],
        ..session.context.clone()
    };
    assert_eq!(editor_command(&Session { context: prefixed }), Some(String::from("/from/the/environment")), "the environment wins over core.editor, as it does under Git");
    remove(parent.as_path());
}
