//! What:
//!  The notice a relink leaves for the person when the commit service gives the record up,
//!  and the lines a later command in that worktree shows of it.
//! Why:
//!  The relinking runs in the service, after the command that added the worktree returned,
//!  so a command that never finishes the job has nothing attached to it that could say so.
//!  A work record that reached `failed` is scanned by nothing:
//!  without a notice the person keeps a worktree whose package links name the checkout it was copied from,
//!  and learns of it only when a tool fails in it.
//!  The notice holds what the last try failed with,
//!  which only the round that ran the tool saw (`package_repair_relink_failure.rs`),
//!  and the commands the record asked for,
//!  so the person reads why the wrapper stopped and what is left to do about it.
//!  It is a plain file in the relinked worktree's own Git directory,
//!  so a command in that worktree reads one directory and finds nothing when nothing was given up,
//!  and no command in another worktree answers for it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordGivenUp(gitDir, worktree, tools); for (const line of takeNotices(gitDir)) stderr.write(line + '\n');
//! ```

use crate::package_repair_pnpm::manual_command;
use crate::package_repair_relink::{RelinkEntry, RelinkTool, decode};
use crate::package_repair_toolchain::MANUAL_COMMAND;
use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use std::path::{Path, PathBuf};

/// Directory under the worktree's own Git directory that holds one notice per relink given up on.
pub const NOTICE_DIRECTORY: &str = "monochromatic-git-policy-cli/relink-notices";

/// What:
///  The commands that finish the relinking of `worktree` by hand,
///  one per tool the record asked for.
/// Why:
///   The two tools are run in the worktree and each rewrites its own links,
///   so a person who runs one and not the other is left with half the work;
///   the pnpm command names the worktree itself and works from any directory,
///   while the toolchain command is the repository's own task and is run in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commands = (worktree, tools) => tools.map((tool) => tool === 'pnpm' ? manualCommand(worktree) : 'mise run toolchain:install');
/// ```
fn commands_for(worktree: &Path, tools: &[RelinkTool]) -> Vec<String> {
    let mut commands: Vec<String> = Vec::with_capacity(tools.len());
    for tool in tools {
        commands.push(match tool {
            RelinkTool::Pnpm => manual_command(worktree),
            RelinkTool::Toolchain => format!("cd -- {} && {MANUAL_COMMAND}", crate::shell_quote::shell_quote(worktree.as_os_str().as_encoded_bytes())),
        });
    }
    return commands;
}

/// What:
///  The one line a person reads about a relink that was given up on:
///  what the worktree holds now,
///  whether a tool answered and with what,
///  and the command the record's tool finishes the job with.
/// Why:
///   It names the worktree,
///   and it says what failed rather than only how often the service tried,
///   because a count names nothing a person can change.
///   The command is offered as the step that follows that change,
///   never as the fix on its own:
///   the wrapper ran it,
///   and it cannot run anything else for the tools the record asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = (worktree, commands, tries, lastFailure) => `monochromatic-git-policy-cli: the package state of ${worktree} was not relinked, so its links still name the checkout it was copied from. ${lastFailure === undefined ? `The commit service stopped ${tries} times before it could finish the job (a restart, a kill or a crash each time), so no tool's answer was ever recorded.` : `The wrapper tried ${tries} times and gave up. The last try failed with: ${lastFailure.replace(/[. \\n]+$/, '')}.`} Running the same command again would fail the same way, so something only you can change has to come first; after that, ${commands.map((c) => `\`${c}\``).join(' or ')} finishes the job.`;
/// ```
pub fn words_for(worktree: &Path, entry: &RelinkEntry, tries: u32, last_failure: Option<&str>) -> String {
    let commands: Vec<String> = commands_for(worktree, &entry.tools);
    let opening: String = format!("monochromatic-git-policy-cli: the package state of {} was not relinked, so its links still name the checkout it was copied from.", worktree.display());
    // The tool's own words are quoted as they came, minus the stop that would double the sentence's own period.
    let what_happened: String = match last_failure.map(|reason| return reason.trim_end_matches(['.', ' ', '\n'])) {
        Some(reason) => format!("The wrapper tried {tries} times and gave up. The last try failed with: {reason}."),
        None => format!("The commit service stopped {tries} times before it could finish the job (a restart, a kill or a crash each time), so no tool's answer was ever recorded."),
    };
    return format!(
        "{opening} {what_happened} Running the same command again would fail the same way, so something only you can change has to come first; after that, `{}` finishes the job.",
        commands.join("` or `")
    );
}

/// What:
///  The Git directory of `worktree`:
///  `.git` itself when it is a directory,
///  else the path its pointer names,
///  so a notice is written where a command run in that worktree reads it.
/// Why:
///   The work records live in the Git directory of the worktree whose command added the new one,
///   and a notice written there is shown by the next command in that worktree
///   about worktrees the person may never enter.
///   The pointer of a removed worktree does not resolve,
///   and then nothing will run in it again and nothing is owed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitDir = (worktree) => stat(join(worktree, '.git')).isDirectory() ? join(worktree, '.git') : parsePointer(readFile(join(worktree, '.git')));
/// ```
pub fn git_dir_of_a_worktree(worktree: &Path) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStrExt as _;
    let pointer: PathBuf = worktree.join(".git");
    let Ok(found) = std::fs::metadata(pointer.as_path()) else { return None };
    if found.is_dir() {
        return Some(pointer);
    }
    // The pointer is a file only: a link, a pipe or anything else at that name is no Git directory this may write in.
    if !found.is_file() {
        return None;
    }
    let bytes: Vec<u8> = std::fs::read(pointer.as_path()).ok()?;
    let line: &[u8] = bytes.trim_ascii_end();
    let named: &[u8] = line.strip_prefix(b"gitdir: ")?;
    let named_path: PathBuf = PathBuf::from(std::ffi::OsStr::from_bytes(named));
    return Some(if named_path.is_absolute() { named_path } else { worktree.join(named_path) });
}

/// What:
///  Write the notice one given-up record leaves,
///  named after the record,
///  so two records of one worktree do not answer for each other.
/// Why:
///   The record is gone from the queue by the time the person reads the notice,
///   so the name it leaves has to be one a command can show and remove without knowing the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = (gitDir, id, text) => { mkdir(noticeDirectory(gitDir)); writeFile(join(noticeDirectory(gitDir), id), text) };
/// ```
pub fn record_notice(git_dir: &Path, id: &str, text: &str) -> Result<(), String> {
    let directory: PathBuf = git_dir.join(NOTICE_DIRECTORY);
    create_private_directory_all(directory.as_path()).map_err(|error| format!("the notice directory {} could not be made ({error})", directory.display()))?;
    let path: PathBuf = directory.join(id);
    return write_private_file_unsynced(path.as_path(), text.as_bytes()).map_err(|error| format!("the notice {} could not be written ({error})", path.display()));
}

/// What:
///  The notices this worktree's Git directory holds,
///  each removed after it was read,
///  so the person is told once.
/// Why:
///   A notice is worded already,
///   so showing it is one read of each file;
///   a file that cannot be read is left for the next command rather than lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const take = (gitDir) => readdir(join(gitDir, NOTICE_DIRECTORY)).flatMap((name) => { const text = readFile(...); unlink(...); return [text] });
/// ```
pub fn take_notices(git_dir: &Path) -> Vec<String> {
    let directory: PathBuf = git_dir.join(NOTICE_DIRECTORY);
    let Ok(names) = std::fs::read_dir(directory.as_path()) else { return Vec::new() };
    let mut lines: Vec<String> = Vec::new();
    for name in names.flatten() {
        let path: PathBuf = name.path();
        let Ok(text) = std::fs::read_to_string(path.as_path()) else { continue };
        // The read is what the person is owed; a notice that cannot be removed is shown again rather than lost.
        let _removed = std::fs::remove_file(path.as_path());
        lines.push(text);
    }
    return lines;
}

/// What:
///  The notice a record of this kind leaves when it is given up on,
///  or nothing when the record is of another kind or its payload cannot be read.
/// Why:
///   The words name the tools the record asked for,
///   which only the payload knows,
///   and a record whose payload cannot be read names no command;
///   the service's journal line about that record is then the person's only word,
///   and it says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noticeFor = (item) => item.kind === 'relink' && decode(item.parent) ? wordsFor(item.repository, decode(item.parent), MAX_TRIES) : undefined;
/// ```
pub fn notice_for_given_up(item: &crate::work_item::WorkItem, tries: u32) -> Option<String> {
    if item.kind != crate::work_item::WorkKind::Relink {
        return None;
    }
    let entry: RelinkEntry = match decode(item.parent.as_str()) {
        Ok(decoded) => decoded,
        Err(_unreadable) => return None,
    };
    let last_failure: Option<String> = crate::package_repair_relink_failure::recall(item.git_dir.as_path(), item.id().as_str());
    return Some(words_for(item.repository.as_path(), &entry, tries, last_failure.as_deref()));
}

/// Notice controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_relink_notice_tests.rs"]
mod tests;
