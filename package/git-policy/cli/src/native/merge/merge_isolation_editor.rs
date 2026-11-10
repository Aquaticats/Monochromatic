//! What:
//!  The editor a merge from objects opens for its message:
//!  the file it reads,
//!  the program Git would start,
//!  and the message that comes back.
//! Why:
//!  A merge started from a terminal opens the person's editor
//! (`default_edit_option` in Git's `builtin/merge.c`),
//! and native Git performs that merge before the editor opens,
//!  so a checkout that holds the person's own work in the paths the merge changes
//!  makes it refuse before anything is edited
//! (`doc/planning/cli-git-merge-with-staged-index.md`).
//!  That work is carried forward instead (`SPEC.md`,
//! "A merge whose paths hold the person's own work carries it
//!  forward"),
//! so the editor the merge would have opened is this file's:
//! the message is written to `<git-dir>/MERGE_MSG` with the help lines
//!  Git adds,
//! the program `git var GIT_EDITOR` names runs on it through the shell,
//! as `launch_editor` runs one,
//!  and what it leaves is cleaned the way a commit that opens an editor cleans it
//! (`COMMIT_MSG_CLEANUP_ALL`:
//!  comment lines and trailing whitespace go).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const message = await editMergeMessage({ session, gitDir, message }); // throws EditorRefusal
//! ```

/// The questions an isolated merge asks Git.
use crate::merge_isolation_git_reads::Session;
/// Real Git.
use crate::transaction_git::GitOutput;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};
/// The editor runs as the person's own program,
/// with the terminal and the streams they gave this command.
use std::process::{Command, ExitStatus};

/// What:
///  The help lines Git writes under the merge message when the editor opens,
/// for the default comment character.
/// Why:
///   They are the words the person expects to read above their message;
/// each line is a comment,
/// so the cleanup drops
///  them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const HELP = "# Please enter a commit message to explain why this merge is necessary,\n...";
/// ```
const HELP_LINES: &str = concat!(
    "# Please enter a commit message to explain why this merge is necessary,\n",
    "# especially if it merges an updated upstream into a topic branch.\n",
    "#\n",
    "# Lines starting with '#' will be ignored, and an empty message aborts\n",
    "# the commit.\n",
);

/// What:
///  The shell Git starts an editor through.
/// Why:
///   `run-command.c` runs an editor as `sh -c '<editor> "$@"' '<editor>' <file>`,
/// so an editor configured as
///  `code --wait` or `emacsclient -nw` receives its file the way it does under native Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SHELL = '/bin/sh';
/// ```
const SHELL: &str = "/bin/sh";

/// What:
///  The place the editor of a merge opens its file in.
/// Why:
///   The editor needs the invocation's Git,
/// with the global options the person gave as a prefix,
///  and the one
/// Git directory whose message file the merge writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EditorPlace = { session: Session; gitDir: string };
/// ```
pub struct EditorPlace<'a> {
    /// The invocation's repository,
    /// with its global options as a prefix.
    pub session: &'a Session,
    /// The Git directory the merge message file is written in.
    pub git_dir: &'a Path,
}

/// What:
///  Why the editor left no message the merge can be committed with.
/// Why:
///   Each ending is reported with its own words,
/// and none of them changes anything in the repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EditorRefusal = { kind: 'no-editor' } | { kind: 'stopped'; detail: string } | { kind: 'empty' };
/// ```
#[derive(Debug)]
pub enum EditorRefusal {
    /// Git resolved no editor for this environment;
    /// native Git is the command,
    /// and it dies its own way.
    NoEditor,
    /// The message file could not be written,
    /// the editor could not be started,
    /// or it ended with a failure;
    /// the detail names what happened.
    Stopped(String),
    /// The editor left nothing to commit.
    Empty,
}

/// What:
///  The program Git would start for this command,
/// or nothing when this environment resolves none.
/// Why:
///   `git var GIT_EDITOR` applies Git's own order (`GIT_EDITOR`,
/// `core.editor`,
/// `VISUAL`,
/// `EDITOR`,
///  and the terminal check),
/// so the person's editor is the one that opens,
/// with the global options of this command included.
///  An environment that resolves none (a dumb terminal and no editor set) leaves native Git to say so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function editorCommand(session: Session): string | undefined; // git var GIT_EDITOR
/// ```
pub fn editor_command(session: &Session) -> Option<String> {
    let output: GitOutput = session.run(&["var", "GIT_EDITOR"]).ok()?;
    if !output.succeeded() {
        return None;
    }
    let text: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    if text.is_empty() {
        return None;
    }
    return Some(text);
}

/// What:
///  Open the editor on the merge message and return what the person left.
/// Why:
///   The program comes from Git's own resolution,
/// so the file and the message are the ones native Git would have handed over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const message = editMergeMessage({ session, gitDir, message }); // throws EditorRefusal
/// ```
pub fn edit_message(place: &EditorPlace<'_>, message: &[u8]) -> Result<Vec<u8>, EditorRefusal> {
    let Some(editor) = editor_command(place.session) else {
        return Err(EditorRefusal::NoEditor);
    };
    return edit_with(place, message, editor.as_str());
}

/// What:
///  The message this merge commits with,
/// asking the person's editor only once.
/// Why:
///   A round made again because the branch moved keeps the message the person wrote,
/// so no one is asked to type it
///  twice,
/// and a merge that never reaches its commit leaves no message and nothing else behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function askedOnce(kept, place, message): Uint8Array; // throws EditorRefusal
/// ```
pub fn asked_once(kept: &mut Option<Vec<u8>>, place: &EditorPlace<'_>, message: &[u8]) -> Result<Vec<u8>, EditorRefusal> {
    if let Some(written) = kept {
        return Ok(written.clone());
    }
    let edited: Vec<u8> = edit_message(place, message)?;
    *kept = Some(edited.clone());
    return Ok(edited);
}

/// What:
///  The same edit with the editor named,
/// for the callers that already know it and for the controls.
/// Why:
///   The file,
/// its help lines,
/// the shell and the cleanup are the same whether the program came from Git or from a control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const message = editWith({ session, gitDir }, message, editor); // throws EditorRefusal
/// ```
pub fn edit_with(place: &EditorPlace<'_>, message: &[u8], editor: &str) -> Result<Vec<u8>, EditorRefusal> {
    let path: PathBuf = place.git_dir.join("MERGE_MSG");
    let mut template: Vec<u8> = message.to_vec();
    if let Some(lines) = help_lines(place.session) {
        template.extend_from_slice(lines.as_bytes());
    }
    if let Err(error) = std::fs::write(path.as_path(), template.as_slice()) {
        return Err(EditorRefusal::Stopped(format!("the merge message file {} could not be written: {error}", path.display())));
    }
    let ended: Result<ExitStatus, String> = launch(editor, path.as_path());
    // The message file is this command's while the merge is unfinished: Git keeps `MERGE_MSG` for the length of the
    //  merge and removes it with the merge's state, and a merge that is not made here leaves none behind.
    let left: std::io::Result<Vec<u8>> = std::fs::read(path.as_path());
    let _removed = std::fs::remove_file(path.as_path());
    let status: ExitStatus = match ended {
        Ok(status) => status,
        Err(detail) => return Err(EditorRefusal::Stopped(detail)),
    };
    if !status.success() {
        return Err(EditorRefusal::Stopped(format!("there was a problem with the editor {editor} ({status})")));
    }
    let written: Vec<u8> = match left {
        Ok(bytes) => bytes,
        Err(error) => return Err(EditorRefusal::Stopped(format!("the message the editor left in {} could not be read: {error}", path.display()))),
    };
    let cleaned: Vec<u8> = clean_edited(place.session, written.as_slice())?;
    if cleaned.is_empty() {
        return Err(EditorRefusal::Empty);
    }
    return Ok(cleaned);
}

/// What:
///  Start the editor on the message file and wait for it,
/// with the terminal this command was given.
/// Why:
///   The person's editor is the wait and it has no ceiling (`README.md`,
/// "a commit that opens the person's
///  editor prints nothing and has no ceiling");
/// its own exit status says what happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawn('/bin/sh', ['-c', `${editor} "$@"`, editor, path], { stdio: 'inherit' }).status
/// ```
fn launch(editor: &str, path: &Path) -> Result<ExitStatus, String> {
    let mut command: Command = Command::new(SHELL);
    command.arg("-c").arg(format!("{editor} \"$@\"")).arg(editor).arg(path);
    return match command.status() {
        Ok(status) => Ok(status),
        Err(error) => Err(format!("the editor {editor} could not be started: {error}")),
    };
}

/// What:
///  The message cleaned the way a commit that opens an editor cleans it:
/// comment lines and trailing whitespace gone,
/// blank lines collapsed,
///  one final line feed.
/// Why:
///   `git merge` cleans an edited message with `COMMIT_MSG_CLEANUP_ALL`,
/// and `git stripspace
///  --strip-comments` is that cleanup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cleaned = await git(['stripspace', '--strip-comments'], written);
/// ```
fn clean_edited(session: &Session, written: &[u8]) -> Result<Vec<u8>, EditorRefusal> {
    let cleaned: GitOutput = match session.run_with_input(&["stripspace", "--strip-comments"], written.to_vec()) {
        Ok(output) => output,
        Err(reason) => return Err(EditorRefusal::Stopped(reason)),
    };
    if !cleaned.succeeded() {
        return Err(EditorRefusal::Stopped(format!("git stripspace --strip-comments failed: {}", cleaned.error_text())));
    }
    return Ok(cleaned.stdout);
}

/// What:
///  Git's help lines for this repository,
/// or nothing when its comment character is not `#`.
/// Why:
///   The cleanup drops the lines that start with the configured comment character,
/// so help lines written with `#`
///  would stay in the message under `core.commentChar=;`,
/// and a message with no help lines is what the person reads then.
///  `git var` resolves no comment character,
/// so the two settings Git accepts for one are read here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function helpLines(session: Session): string | undefined; // '#' when nothing else is configured
/// ```
fn help_lines(session: &Session) -> Option<&'static str> {
    for key in ["core.commentChar", "core.commentString"] {
        let named: GitOutput = session.run(&["config", "--get", key]).ok()?;
        if !named.succeeded() {
            continue;
        }
        let text: String = String::from_utf8_lossy(named.stdout.as_slice()).trim().to_owned();
        return if text == "#" { Some(HELP_LINES) } else { None };
    }
    return Some(HELP_LINES);
}

/// The message file and the editor that reads it.
#[cfg(test)]
#[path = "merge_isolation_editor_tests.rs"]
mod tests;
