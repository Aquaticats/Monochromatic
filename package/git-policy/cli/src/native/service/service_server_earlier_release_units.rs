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

use crate::service_server_start_through_the_manager::{ManagerStart, PROBE_WAIT};
use std::path::{Path, PathBuf};

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
        directory.join("monochromatic-git-policy-cli-worktree-notes"),
        directory.join("cli-git@.service"),
        directory.join("cli-git@.socket"),
        directory.join("cli-git.service"),
        directory.join("cli-git.socket"),
    ];
}

/// What:
///  Removes the unit files and the notes folder that an earlier release wrote,
///  and leaves every path it did not write.
/// Why:
///  A plain unlink needs no manager and no reload:
/// the manager forgets the unit when it next reads its units, which is its own business,
///  and until then the stop below is what frees the address.
/// The notes folder holds paths of worktrees,
///  so it goes whole,
/// and a folder of that name that holds no note of this program is left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const path of unitPaths(dir)) if (isOurs(path)) rm(path)
/// ```
pub fn remove_earlier_unit_files(runtime_dir: &Path) -> Vec<PathBuf> {
    let mut removed: Vec<PathBuf> = Vec::new();
    for path in unit_paths(runtime_dir) {
        let ours: bool = match std::fs::symlink_metadata(&path) {
            // A folder: the notes folder is this program's by its name alone, and holds nothing but its notes.
            Ok(metadata) if metadata.is_dir() => std::fs::remove_dir_all(&path).is_ok(),
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
///  Stops the socket units that hold this worktree's address, with the manager's own program.
/// Why:
///  A loaded socket unit keeps the name bound, so no server of this release can own it.
/// The stop returns when the manager has queued it, so the bound here is `PROBE_WAIT` and the answer is only whether it
///  was asked;
/// the address is looked at again by the caller, which is the fact that matters.
/// The earlier single service's socket is stopped too:
/// it holds one address for every worktree, which nothing of this release reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// systemctl('--user', 'stop', `monochromatic-git-policy-cli@${instance}.socket`, 'cli-git.socket')
/// ```
pub fn stop_earlier_socket_units(_manager: &ManagerStart, instance: &str) -> bool {
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
                let _ended = child.kill();
                let _reaped = child.wait();
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
pub fn retire_earlier_units(runtime_dir: &Path, manager: Option<&ManagerStart>, instance: &str) -> Vec<PathBuf> {
    let removed: Vec<PathBuf> = remove_earlier_unit_files(runtime_dir);
    if let Some(found) = manager {
        let _asked: bool = stop_earlier_socket_units(found, instance);
    }
    return removed;
}

/// Controls of the earlier release's units stay out of the release executable.
#[cfg(test)]
#[path = "service_server_earlier_release_units_tests.rs"]
mod tests;
