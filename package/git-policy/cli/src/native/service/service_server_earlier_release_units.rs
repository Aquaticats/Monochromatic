//! What:
//!  What an earlier release left running and on disk for this account's commit services,
//!  and what this release does about it at the first command that meets it:
//!  the earlier release's unit files are removed from the login's unit directory,
//!  and the socket unit that holds a worktree's address is stopped,
//!  so this release's own server can own the address it was always meant to own.
//! Why:
//!  An earlier release started the service from a unit, and the manager's socket unit owns
//! `$XDG_RUNTIME_DIR/monochromatic-git-policy-cli/<instance>.sock` for as long as that unit is
//!  loaded. A wrapper that starts its own server cannot bind a name the manager holds, so a commit in the hours after
//!  an upgrade would be served by the old program for ever:
//! the manager starts the old service on every connection, the client asks it to leave, it exits, and the manager starts
//!  it again on the next connection. Removing the unit *file* does not free the name: the manager keeps a loaded unit
//!  until it reads its units again, and a reload is what this design refuses to ask for.
//!  So two steps, and the second is the one that matters:
//!  the files go (a plain unlink, which lets a later reload of the manager's own forget them, and costs nothing), and
//!  the socket unit of this worktree is stopped with the manager's own program, bounded, and only when that manager is
//!  this session's own (`ManagerStart::in_this_session`, issue 6293's gate).
//! A manager that cannot be asked, or is not this session's, leaves the old service serving: the commit is still made,
//!  the client says which release served it, and the next command tries again.
//!  Nothing here writes a unit file or asks for a reload.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // once, when this command met a service of an earlier release
//! unlink(join(runtimeUnitDir, 'monochromatic-git-policy-cli@.service'));
//! await bounded(() => systemctl('--user', 'stop', `monochromatic-git-policy-cli@${instance}.socket`));
//! ```

use crate::service_server_start_through_the_manager::{PROBE_WAIT, bus_is_ours, end_and_reap_off_the_path};
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "serviceServerEarlierReleaseUnits";

/// What:
///  The texts an earlier release wrote on its unit files' first lines,
///  and the names of the folders it wrote beside them.
/// Why:
///  A file of one of these names that carries none of them is somebody else's,
/// and this release never removes a file it did not write.
pub const OWN_MARKERS: [&str; 5] = ["(monochromatic-git-policy-cli-unit)", "(cli-git-unit)", "Description=Commit service for the git wrapper", "Description=Commit service socket for the git wrapper", "Description=Commit service of one worktree for the git wrapper"];

/// The program the manager stops for one worktree's socket.
/// It is the manager's own client, as `systemd-run` is the manager's own starter.
pub const STOP_PROGRAM: &str = "systemctl";

/// What:
///  The unit files an earlier release of this program writes below a runtime directory.
/// Why:
///  They are the two templates of the release that gave each worktree an instance,
/// the earlier single service's pair,
/// and the folder of notes that went beside them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unitFiles('/run/user/1000') // [...two templates, ...legacy pair, notes folder]
/// ```
pub fn unit_paths(runtime_dir: &Path) -> Vec<PathBuf> {
    let directory: PathBuf = runtime_dir.join("systemd").join("user");
    return vec![
        directory.join("monochromatic-git-policy-cli@.service"),
        directory.join("monochromatic-git-policy-cli@.socket"),
        directory.join(NOTES_FOLDER),
        directory.join("cli-git@.service"),
        directory.join("cli-git@.socket"),
        directory.join("cli-git.service"),
        directory.join("cli-git.socket"),
    ];
}

/// What:
///  The names of what an earlier release kept in its notes folder:
///  the folder itself,
///  one note per worktree,
///  and the link its units started the service through.
/// Why:
///  A note is a file named for an instance whose text is that worktree's Git directory as bytes
///  (or, in the release before that, the path itself);
/// the link is named `service-program` and is a symlink the wrapper replaced by a rename.
///  Those are this program's by name and by text, and nothing else in that folder is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NOTES_FOLDER = 'monochromatic-git-policy-cli-worktree-notes';
/// const NOTE_ENDING = '.git-dir';
/// const PROGRAM_LINK = 'service-program';
/// ```
pub const NOTES_FOLDER: &str = "monochromatic-git-policy-cli-worktree-notes";

/// The ending of one worktree's note in that folder.
pub const NOTE_ENDING: &str = ".git-dir";

/// The name of the link an earlier release's units started the service through.
pub const PROGRAM_LINK: &str = "service-program";

/// What:
///  Removes the unit files and the notes of an earlier release,
///  and leaves every path it did not write.
/// Why:
///  A plain unlink needs no manager and no reload:
/// the manager forgets the unit when it next reads its units, which is its own business,
///  and until then the stop below is what frees the address.
/// What goes from the notes folder is what this program is shown to have written, one entry at a time
///  (`remove_the_notes_of_this_program`):
/// the folder's name is this program's and the files inside it are the person's until their own name and text say
///  otherwise.
///  The review of `76a08c9fac` reproduced a person's file going with the folder when the folder was removed whole,
///  which is why the folder goes only once nothing but this program's notes was in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const path of unitPaths(dir)) if (isOurs(path)) rm(path)
/// ```
pub fn remove_earlier_unit_files(runtime_dir: &Path) -> Vec<PathBuf> {
    let mut removed: Vec<PathBuf> = Vec::new();
    for path in unit_paths(runtime_dir) {
        let ours: bool = match std::fs::symlink_metadata(&path) {
            // A folder: only the notes this program is shown to have written go, one at a time, and only then the folder.
            Ok(metadata) if metadata.is_dir() => remove_the_notes_of_this_program(&path, &mut removed),
            Ok(_) => match std::fs::read_to_string(&path) {
                Ok(text) if OWN_MARKERS.iter().any(|marker| { return text.contains(marker); }) => std::fs::remove_file(&path).is_ok(),
                _ => false,
            },
            Err(_not_there) => false,
        };
        if ours {
            removed.push(path);
        }
    }
    return removed;
}

/// What:
///  Removes the notes of this program from a notes folder, one entry at a time,
///  leaves every entry it did not write, and removes the folder only once nothing is left in it.
/// Why:
///  The folder's name is this program's and nothing inside it is until the entry's own name and text say so:
/// a person may keep a file of their own there,
///  and removing the folder whole discards it (the requirement is "never discard user data").
///  What is left is named, because a person who kept something there should know this program noticed it and left it
///  alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const entry of readdir(folder)) if (isOurNote(entry) || isOurProgramLink(entry)) rm(entry); else name(entry);
/// if (isEmpty(folder)) rmdir(folder);
/// ```
fn remove_the_notes_of_this_program(folder: &Path, removed: &mut Vec<PathBuf>) -> bool {
    let Ok(listing) = std::fs::read_dir(folder) else { return false };
    let mut left: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let path: PathBuf = entry.path();
        if !is_a_note_of_this_program(&path) {
            left.push(path);
            continue;
        }
        if std::fs::remove_file(&path).is_ok() {
            removed.push(path);
        }
    }
    if left.is_empty() {
        return std::fs::remove_dir(folder).is_ok();
    }
    crate::diagnostic_log::warn(
        TAG,
        format!(
            "{} holds what this program did not write, so it and its entries are left: {}",
            folder.display(),
            left.iter().map(|path| { return path.display().to_string(); }).collect::<Vec<String>>().join(", ")
        )
        .as_str(),
    );
    return false;
}

/// What:
///  Whether one entry of a notes folder is this program's:
///  the link its units started the service through, or a note whose name is an instance's and whose text is a Git
///  directory.
/// Why:
///  A name alone is not proof: a person's file may sit under a name like this program's.
/// The text is the second half of the proof, and a file that fails either half is left and named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isOurs(p) // service-program link, or /^[0-9a-f]{32}\.git-dir$/ holding gitdirHex= or one absolute path
/// ```
fn is_a_note_of_this_program(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(std::ffi::OsStr::to_str) else { return false };
    if name == PROGRAM_LINK {
        // The link holds no data of a person's, only a target, and this name is one only this program writes.
        return std::fs::symlink_metadata(path).map(|metadata| { return metadata.file_type().is_symlink(); }).unwrap_or(false);
    }
    let Some(instance) = name.strip_suffix(NOTE_ENDING) else { return false };
    let an_instance_name: bool = instance.len() == crate::service_instance::INSTANCE_LENGTH && instance.bytes().all(|byte| { return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte); });
    if !an_instance_name {
        return false;
    }
    let Ok(text) = std::fs::read_to_string(path) else { return false };
    return is_a_git_directory_record(text.as_str());
}

/// What:
///  Whether a note's text says which worktree the note belongs to:
///  the bytes of the Git directory as hexadecimal after `gitdirHex=`,
///  or, as the release before that wrote it, the path itself.
/// Why:
///  The text is what makes a file with a note's name this program's;
/// one with that name and a person's text is left, and named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isAGitDirectoryRecord('gitdirHex=2f7265706f2f2e676974') // true
/// ```
fn is_a_git_directory_record(text: &str) -> bool {
    let without_the_line_feed: &str = text.strip_suffix('\n').unwrap_or(text);
    if let Some(hexadecimal) = without_the_line_feed.strip_prefix(crate::path_bytes_in_records::GITDIR_HEX_PREFIX) {
        return !hexadecimal.is_empty() && hexadecimal.len() % 2 == 0 && hexadecimal.bytes().all(|byte| { return byte.is_ascii_hexdigit(); });
    }
    // One absolute path and nothing after it: a worktree's Git directory as the release before the bytes wrote it.
    return without_the_line_feed.starts_with('/')
        && !without_the_line_feed.contains('\0')
        && !without_the_line_feed.contains('\n')
        && without_the_line_feed.len() < 4096;
}

/// What:
///  Stops the socket units that hold this worktree's address, with the manager's own program.
/// Why:
///  A loaded socket unit keeps the name bound, so no server of this release can own it.
/// The stop returns when the manager has queued it, so the bound here is `PROBE_WAIT` and the answer is only whether it
///  was asked;
/// the address is looked at again by the caller, which is the fact that matters.
/// A child the bound ended is ended and reaped on a thread of its own, never here:
///  a client inside an uninterruptible call cannot be ended by a signal until that call returns,
///  and waiting for such a child on this path is a commit that waits for ever (the review of `76a08c9fac`, finding 2,
///  the same defect the probe of a manager had).
/// The earlier single service's socket is stopped too:
/// it holds one address for every worktree, which nothing of this release reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// systemctl('--user', 'stop', `monochromatic-git-policy-cli@${instance}.socket`, 'cli-git.socket')
/// ```
pub fn stop_with_the_managers_client(instance: &str) -> bool {
    return stop_with_the_managers_client_seamed(instance, Box::new(end_and_reap_off_the_path));
}

/// What:
///  The stop, with how a child the bound ended is dealt with given by the caller.
/// Why:
///  The control hands in an ending and reaping that never returns, which is the state a client inside an uninterruptible
///  call is in, and asserts that the ask still answers within its bound; a wait for the child on this path would fail it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stopSeamed(instance, endAndReap) // endAndReap runs on a thread of its own
/// ```
pub(crate) fn stop_with_the_managers_client_seamed(instance: &str, end_and_reap: Box<dyn FnOnce(std::process::Child) + Send>) -> bool {
    // What: The manager's own client is looked for on `PATH`, as the starter looks for its own.
    // Why:  A manager that is this session's own answers it, and one that is not never hears from this command; a machine
    //       without the client has no unit loaded either.
    let Some(stopping) = crate::service_server_start_through_the_manager::find_on_path(STOP_PROGRAM) else { return false };
    let mut command: std::process::Command = std::process::Command::new(stopping);
    command
        .args([
            "--user",
            "stop",
            format!("{}@{instance}.socket", crate::wrapper_names_old_and_new::SERVICE_UNIT.current).as_str(),
            format!("{}@{instance}.socket", crate::wrapper_names_old_and_new::SERVICE_UNIT.earlier).as_str(),
            format!("{}.socket", crate::wrapper_names_old_and_new::SERVICE_UNIT.earlier).as_str(),
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    let Ok(mut child) = command.spawn() else { return false };
    let deadline: std::time::Instant = std::time::Instant::now() + PROBE_WAIT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) if std::time::Instant::now() >= deadline => {
                // What: The child is ended and reaped on a thread of its own, and this command goes on without its answer.
                // Why:  A signal cannot end a child inside an uninterruptible call, and waiting for such a child here was the
                //       review's finding 2: a commit could wait for ever inside the stop of an earlier release's units.
                std::thread::spawn(move || {
                    end_and_reap(child);
                });
                return false;
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(10)),
            Err(_unreadable) => return false,
        }
    }
}

/// What:
///  What this release does about an earlier release's units, once per command:
///  the files go,
///  and the socket units are stopped when a manager of this session is there.
/// Why:
///   The order is the one that matters: a file that is gone cannot be loaded again,
/// and the stop is what frees the address now.
/// `manager` is the gate's own answer (`ManagerStart::in_this_session`): nothing is asked of a manager that is not this
///  session's, which is what keeps a command given a runtime directory of its own out of the account's manager.
/// Nothing is removed when nothing of an earlier release is there, which is the ordinary case after a login.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removed = removeEarlierUnitFiles(runtimeDir);
/// if (manager) stopEarlierSocketUnits(manager, instance);
/// return removed;
/// ```
pub fn retire_earlier_units(runtime_dir: &Path, named_runtime_dir: Option<&Path>, bus_address: Option<&str>, instance: &str) -> Vec<PathBuf> {
    let removed: Vec<PathBuf> = remove_earlier_unit_files(runtime_dir);
    // What: The manager is asked only when this command's runtime directory is the one the environment named and its bus is
    //       that session's own.
    // Why:  A command given a runtime directory of its own must leave the account's manager alone (issue 6293), and a folder
    //       this program fell back to is no session at all.
    if named_runtime_dir == Some(runtime_dir) && bus_is_ours(runtime_dir, bus_address) {
        let _asked: bool = stop_with_the_managers_client(instance);
    }
    return removed;
}

/// Controls of the earlier release's units stay out of the release executable.
#[cfg(test)]
#[path = "service_server_earlier_release_units_tests.rs"]
mod tests;
