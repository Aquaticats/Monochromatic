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
use crate::plain_file::read_plain_file;
use crate::private_storage::{create_private_directory_all, replace_private_file_unsynced};
use crate::worktree_convert_inspection::GIT_FILE_LIMIT;
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
///   `tries_the_record_allows` is the record's bound and never the count it reached:
///   the count rises when a pass killed after the words is recovered again,
///   and the words a second telling writes over the first must say what the first said.
///   A record with no kept answer does not claim the service stopped or that no tool ran,
///   because either may have happened:
///   it says what is true, that no answer was kept, and where the rest is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = (worktree, commands, triesAllowed, lastFailure) => `monochromatic-git-policy-cli: the package state of ${worktree} was not relinked, so its links still name the checkout it was copied from. ${lastFailure === undefined ? `No tool's answer was kept for this record, so what stopped it is only in the commit service's journal.` : `The wrapper tried ${triesAllowed} times and gave up. The last try failed with: ${lastFailure.replace(/[. \\n]+$/, '')}.`} Running the same command again would fail the same way, so something only you can change has to come first; after that, ${commands.map((c) => `\`${c}\``).join(' or ')} finishes the job.`;
/// ```
pub fn words_for(worktree: &Path, entry: &RelinkEntry, tries_the_record_allows: u32, last_failure: Option<&str>) -> String {
    let commands: Vec<String> = commands_for(worktree, &entry.tools);
    let opening: String = format!("monochromatic-git-policy-cli: the package state of {} was not relinked, so its links still name the checkout it was copied from.", worktree.display());
    // The tool's own words are quoted as they came, minus the stop that would double the sentence's own period.
    let what_happened: String = match last_failure.map(|reason| return reason.trim_end_matches(['.', ' ', '\n'])) {
        Some(reason) => format!("The wrapper tried {tries_the_record_allows} times and gave up. The last try failed with: {reason}."),
        None => String::from("No tool's answer was kept for this record, so what stopped it is only in the commit service's journal."),
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
///   The pointer is read the way every other `.git` reader of this crate reads one:
///   the open takes the type from the handle, without following a link and without waiting,
///   and a file past `GIT_FILE_LIMIT` is refused rather than read into this process's memory.
///   A pointer is one line, and a give-up pass may not be the thing that exhausts the host
///   (a 2 GiB pointer file was read whole before this: 523 MB peak and the rest in swap).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitDir = (worktree) => stat(join(worktree, '.git')).isDirectory() ? join(worktree, '.git') : parsePointer(readPlainFile(join(worktree, '.git'), GIT_FILE_LIMIT));
/// ```
pub fn git_dir_of_a_worktree(worktree: &Path) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStrExt as _;
    let pointer: PathBuf = worktree.join(".git");
    // A stat opens nothing, so a pipe or a link at the name cannot hold it.
    let Ok(found) = std::fs::metadata(pointer.as_path()) else { return None };
    if found.is_dir() {
        return Some(pointer);
    }
    // A name that is not a plain file, a link, and a file past the bound are all nothing this may write in.
    let bytes: Vec<u8> = read_plain_file(pointer.as_path(), GIT_FILE_LIMIT).ok()?;
    let line: &[u8] = bytes.trim_ascii_end();
    let named: &[u8] = line.strip_prefix(b"gitdir: ")?;
    let named_path: PathBuf = PathBuf::from(std::ffi::OsStr::from_bytes(named));
    return Some(if named_path.is_absolute() { named_path } else { worktree.join(named_path) });
}

/// What:
///  Where the words about `worktree` go,
///  when this repository can prove that the worktree at that path is the one the record is about:
///  its own Git directory.
///  Nothing when it cannot be proved,
///  and then the caller writes in the directory the record was drained from.
/// Why:
///   The pointer alone proves nothing:
///   an interrupted removal leaves a checkout whose pointer names a directory that is gone,
///   and a notice written into a directory made again there is one no command in that checkout can read
///   (Git refuses the worktree: `gitfile does not point to a valid repository`).
///   The registration's own `gitdir` and `commondir` back-links are what a working Git directory of this repository
///   holds, so the words go where a command reads them or somewhere the caller can be sure of.
///   A registration made after the record was queued is another worktree at the same path:
///   the administrative directory is named after the path,
///   so a later worktree would otherwise show words about an earlier one,
///   and the moment the record carries is the only thing that tells them apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const target = (drain, worktree, queuedAt) => { const admin = gitDirOf(worktree); const proved = registrationOf(commonDirOf(drain), worktree); return admin && proved && sameFile(admin, proved) && mtime(join(worktree, '.git')) <= instantOf(queuedAt) ? proved : undefined; };
/// ```
pub fn notice_target_of_a_worktree(drain_git_dir: &Path, worktree: &Path, queued_at: &str) -> Option<PathBuf> {
    let named: PathBuf = git_dir_of_a_worktree(worktree)?;
    // The registration proves the directory is a worktree of this repository, at this path, with the back-links a
    //  working Git directory of it holds.
    let common: PathBuf = crate::common_directory_of_a_git_directory::common_directory_of(drain_git_dir);
    let proved: PathBuf = crate::worktree_registration_proof::registration_of(common.as_path(), worktree).ok()?;
    if std::fs::canonicalize(named.as_path()).ok().as_deref() != Some(proved.as_path()) {
        return None;
    }
    // A registration made after the record was queued is a worktree added at that path later. Both moments are
    //  compared in milliseconds, which is the resolution a record's `created` is written with: a `.git` written in the
    //  same millisecond as the record that names it is this worktree's, and a file's own moment is finer than that.
    let registered: std::time::SystemTime = std::fs::symlink_metadata(worktree.join(".git")).ok()?.modified().ok()?;
    let queued: std::time::SystemTime = crate::iso_time::instant_of_iso(queued_at)?;
    if milliseconds_of(registered) > milliseconds_of(queued) {
        return None;
    }
    return Some(proved);
}

/// What:
///  `moment` in whole milliseconds since the Unix epoch,
///  the resolution a record's own stamp is written with.
/// Why:
///   A file's moment is finer than a millisecond, so comparing the two as they are would call a `.git` written inside
///   the record's own millisecond a later registration and hold the words back from the worktree they are about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const millis = (moment) => Math.floor(moment.getTime());
/// ```
fn milliseconds_of(moment: std::time::SystemTime) -> u128 {
    return moment.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis();
}

/// What:
///  Write the notice one given-up record leaves,
///  named after the record,
///  replacing any words an earlier telling of that record left.
/// Why:
///   The record is gone from the queue by the time the person reads the notice,
///   so the name it leaves has to be one a command can show and remove without knowing the record.
///   A pass killed between the words and the move to `failed` is told again by the next pass
///   (`work_item_recovery.rs`),
///   and the record's reason is kept until it is in `failed`,
///   so the second telling words exactly what the first did:
///   the same bytes over the same name,
///   which a command either reads whole or finds replaced.
///   Words that could not replace the first would hold the record unsettled for ever,
///   because the recovery settles nothing while the person cannot be told.
///   A Git directory that is not there refuses them,
///   rather than being made again for them: a directory made again is one no command in the worktree reads,
///   and the caller has somewhere a command does read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = (gitDir, id, text) => { mkdir(noticeDirectory(gitDir)); replaceFile(join(noticeDirectory(gitDir), id), text) };
/// ```
pub fn record_notice(git_dir: &Path, id: &str, text: &str) -> Result<(), String> {
    // The words never bring a Git directory into being: a directory that is not there is no place a command in that
    //  worktree reads, and making one again is how a notice came to land where Git refuses the worktree
    //  (`fatal: gitfile does not point to a valid repository`). The caller writes where it knows a command reads.
    if !git_dir.is_dir() {
        return Err(format!("{} is no directory that exists, so no command there would read words written in it", git_dir.display()));
    }
    let directory: PathBuf = git_dir.join(NOTICE_DIRECTORY);
    create_private_directory_all(directory.as_path()).map_err(|error| format!("the notice directory {} could not be made ({error})", directory.display()))?;
    let path: PathBuf = directory.join(id);
    return replace_private_file_unsynced(path.as_path(), text.as_bytes()).map_err(|error| format!("the notice {} could not be written ({error})", path.display()));
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
/// const noticeFor = (item) => item.kind === 'relink' && decode(item.parent) ? wordsFor(item.repository, decode(item.parent), MAX_TRIES, recall(item.gitDir, item.id)) : undefined;
/// ```
pub fn notice_for_given_up(item: &crate::work_item::WorkItem, tries_the_record_allows: u32) -> Option<String> {
    if item.kind != crate::work_item::WorkKind::Relink {
        return None;
    }
    let entry: RelinkEntry = match decode(item.parent.as_str()) {
        Ok(decoded) => decoded,
        Err(_unreadable) => return None,
    };
    let last_failure: Option<String> = crate::package_repair_relink_failure::recall(item.git_dir.as_path(), item.id().as_str());
    return Some(words_for(item.repository.as_path(), &entry, tries_the_record_allows, last_failure.as_deref()));
}

/// Notice controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_relink_notice_tests.rs"]
mod tests;
