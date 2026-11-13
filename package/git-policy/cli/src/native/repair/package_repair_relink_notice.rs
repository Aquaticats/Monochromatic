//! What:
//!  The notice a relink leaves for the person when the commit service gives the record up,
//!  and the lines a later command in that worktree shows of it.
//! Why:
//!  The relinking runs in the service, after the command that added the worktree returned,
//!  so a command that never finishes the job has nothing attached to it that could say so.
//!  A work record that reached `failed` is scanned by nothing:
//!  without a notice the person keeps a worktree whose package links name the checkout it was copied from,
//!  and learns of it only when a tool fails in it.
//!  The notice holds one line,
//!  and it names the commands that finish the job by hand,
//!  so the person needs to read no record and no journal.
//!  It is a plain file in the worktree's own Git directory,
//!  so a command in the worktree reads one directory and finds nothing when nothing was given up.
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
///  The one line a person reads about a relink that was given up on.
/// Why:
///   It names the worktree,
///   how many times the service tried,
///   and every command that finishes the job,
///   because the record itself is not where anyone looks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = (worktree, commands, tries) => `monochromatic-git-policy-cli: the package state of ${worktree} was not relinked after ${tries} tries. Run ${commands.map((c) => `\`${c}\``).join(', then ')} to finish it by hand.`;
/// ```
pub fn words_for(worktree: &Path, entry: &RelinkEntry, tries: u32) -> String {
    let commands: Vec<String> = commands_for(worktree, &entry.tools);
    return format!(
        "monochromatic-git-policy-cli: the package state of {} was not relinked after {tries} tries, so its links still name the checkout it was copied from. Run `{}` to finish it by hand.",
        worktree.display(),
        commands.join("`, then `")
    );
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
    return Some(words_for(item.repository.as_path(), &entry, tries));
}

/// Notice controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_relink_notice_tests.rs"]
mod tests;
