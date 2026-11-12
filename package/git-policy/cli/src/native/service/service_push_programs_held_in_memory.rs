//! What:
//!  The programs a push the commit service runs finds,
//!  and the transport that push speaks through,
//!  carried from the command that asked for it and held only in the service's memory.
//! Why:
//!  The service takes the environment the user manager started it with,
//! and the person committing can see neither that list nor a way to change it:
//! on 2026-11-12 the manager's `PATH` held no folder `git-lfs` is installed in while the shell that
//!  committed had one,
//! so two runs' commits landed and their pushes were refused for a program their own shell could start.
//! A push must not fail where the same push typed in the same shell would succeed,
//! and the shell that asked for the push is the only place its list exists.
//!  Nothing of it is written:
//! `GIT_SSH`,
//! `GIT_SSH_COMMAND` and `GIT_PROXY_COMMAND` are whole command lines that can hold a password,
//! a record of them outlives the command that made it,
//! and every program of the account can read it.
//! The owner's decision of 2026-11-12T11:58:45-05:00 is "Memory only",
//! so the values live in the memory of the service process that runs the push,
//! and a service that restarted has none of them
//! (`push_without_its_transport.rs` says so to the person instead of pushing through another transport).
//!  One entry belongs to one push record and never to a worktree or a branch:
//! two commands of one worktree may queue pushes a moment apart,
//! and each push must run with the values of the command that asked for it,
//! never with those of a later one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const programs = programsOf(env); remember(gitDir, pushRecordId, programs); // the service, before it runs that record
//! programs = held(gitDir, pushRecordId); forget(gitDir, pushRecordId); // once the record can no longer run
//! ```

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// What:
///  The variables a push's children find their programs through,
///  by the names Git and `git-lfs` read them.
/// Why:
///  `PATH` is where `git-lfs`, a credential helper and `ssh` are found;
/// `GIT_SSH`,
/// `GIT_SSH_COMMAND` and `GIT_PROXY_COMMAND` name those programs outright,
/// and `GIT_EXEC_PATH` is where Git looks for `git-upload-pack` and its other subcommands.
/// A login prompt is not carried:
/// the service's pushes carry the never-prompt pair by the owner's decision
/// (`doc/troubleshooting/git-askpass-dialog-despite-terminal-prompt-off.md`),
/// so `GIT_ASKPASS` and `SSH_ASKPASS` would be overwritten anyway and are left out rather than pretending to travel.
pub const CARRIED: [&str; 5] = ["PATH", "GIT_EXEC_PATH", "GIT_SSH", "GIT_SSH_COMMAND", "GIT_PROXY_COMMAND"];

/// What:
///  The carried variables that name the transport a push speaks through.
/// Why:
///  A push whose command set one of these must never run through another after a service restart:
/// the person chose the way out of their network,
/// the service's own environment may name none at all,
/// and a credential that lets the push out may be gone with the command that had it.
/// `PATH` and `GIT_EXEC_PATH` are not here on purpose:
/// they say where programs are found,
/// never which program or which host,
/// and a push whose list is gone searches the service's own and says so in its refusal
/// (`lfs_pre_push::ProgramSearch`).
pub const TRANSPORT: [&str; 3] = ["GIT_SSH", "GIT_SSH_COMMAND", "GIT_PROXY_COMMAND"];

/// What:
///  The variable a service's push carries so its refusals can say the list came from the command that asked for it.
/// Why:
///   The values are the environment's,
/// and nothing in an environment says where a value came from,
/// so the push marks the list it was handed.
/// A push without the mark searched the service's own environment,
/// which the manager fixed.
pub const FROM_THE_REQUEST: &str = "MONOCHROMATIC_PUSH_PROGRAMS_OF_THE_REQUEST";

/// What:
///  The variable a service's push carries when no list came with the request it belongs to,
///  so its refusals can say the list is the service's own.
/// Why:
///   A service that restarted has none of the settings of the command that asked for the push,
/// and the refusal must not
///  let the person believe their own shell was searched.
pub const FROM_THE_SERVICE: &str = "MONOCHROMATIC_PUSH_PROGRAMS_OF_THE_SERVICE";

/// What:
///  The program search list and the transport settings of the command that asked for a push.
/// Why:
///   They travel together because they come from one place and are needed at one moment:
/// the four names Git reads to find a program,
/// the `PATH` it finds one through,
/// and whether the transport itself was chosen by that command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushPrograms = { variables: [string, string][]; transport: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PushPrograms {
    /// The variables that command set, in [`CARRIED`] order, as text.
    /// A value that is not text is left out:
    /// it cannot be carried without inventing bytes,
    /// and a lossy value could name a folder no shell searched.
    pub variables: Vec<(String, String)>,
    /// Whether that command set one of [`TRANSPORT`],
    /// read from the environment before a value that is not text is left out,
    /// so a transport this release cannot spell still stops a push whose settings are gone.
    pub transport: bool,
}

/// What:
///  The programs of the environment this process was given.
/// Why:
///   A wrapper command is the person's shell:
/// its environment is the list the same push typed there would search,
/// and it is the only process that knows what that list resolves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const programsOfThisProcess = () => programsOf(process.env);
/// ```
pub fn of_this_process() -> PushPrograms {
    return of(std::env::vars_os().collect::<Vec<(OsString, OsString)>>().as_slice());
}

/// What:
///  The programs of `environment`,
///  in the order [`CARRIED`] names them.
/// Why:
///   A name that is set more than once keeps its last value,
/// as it does for Git's own configuration,
/// and a name or value that is not text is left out,
/// so every value here can be written in a protocol line and read back whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// CARRIED.flatMap((name) => { const value = lastOf(env, name); return value === undefined || !isText(value) ? [] : [[name, value]]; })
/// ```
pub fn of(environment: &[(OsString, OsString)]) -> PushPrograms {
    let mut variables: Vec<(String, String)> = Vec::new();
    for name in CARRIED {
        let mut value: Option<&OsString> = None;
        for (held, held_value) in environment.iter().rev() {
            if held == OsStr::new(name) {
                value = Some(held_value);
                break;
            }
        }
        let Some(held_value) = value else {
            continue;
        };
        let (Some(plain_name), Some(plain_value)) = (OsStr::new(name).to_str(), held_value.to_str()) else {
            continue;
        };
        variables.push((String::from(plain_name), String::from(plain_value)));
    }
    let mut transport: bool = false;
    for name in TRANSPORT {
        transport = transport || last_value(environment, name).is_some();
    }
    return PushPrograms { variables, transport };
}

/// What:
///  The last value `environment` gives `name`.
/// Why:
///   A name that is set twice keeps its last value,
/// and the transport question is asked of the environment itself rather than of the text-only list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lastValue = (env, name) => env.findLast(([held]) => held === name)?.[1];
/// ```
fn last_value(environment: &[(OsString, OsString)], name: &str) -> Option<OsString> {
    for (held, value) in environment.iter().rev() {
        if held == OsStr::new(name) {
            return Some(value.clone());
        }
    }
    return None;
}

impl PushPrograms {
    /// What:
    ///  Whether this list names `name`,
    ///  which a push's environment is built from.
    /// Why:
    ///   The base's own bookkeeping asks which of the five names travelled,
    /// so a push never carries a value from two sources.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const has = (name) => this.variables.some(([held]) => held === name);
    /// ```
    pub fn holds(&self, name: &str) -> bool {
        return self.variables.iter().any(|(held, _value)| { return held == name; });
    }

    /// What:
    ///  The pairs a child's environment takes.
    /// Why:
    ///   The overlay is built from this list,
    /// and the order the variables travel in is the order [`CARRIED`] names.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// this.variables.map(([name, value]) => [name, value]);
    /// ```
    pub fn as_pairs(&self) -> Vec<(OsString, OsString)> {
        return self.variables.iter().map(|(name, value)| { return (OsString::from(name), OsString::from(value)); }).collect();
    }
}

/// What one held entry is:
/// the worktree whose push record the values belong to,
/// that record's id,
/// and the values.
/// Why:
///   The id is the push record's own,
/// so a value can never be taken by the push of another commit,
/// even on the same branch of the same worktree.
struct Held {
    /// The worktree's own Git directory, as the request named it.
    git_dir: PathBuf,
    /// The push record's id, which names its kind, commit and branch.
    item: String,
    /// The values the command that asked for that push gave.
    programs: PushPrograms,
}

/// Lock kept:
///  the values each queued push record still has,
/// one entry per record.
/// The sections are one lookup, one replacement and one removal of an entry,
/// and nothing is read from or written to the device under the lock,
/// so a landing that queues a push and the round that runs one never wait on storage.
/// The values are a few hundred bytes each and are dropped when their push can no longer run,
/// so the list holds the pushes in flight and nothing else.
static HELD: Mutex<Vec<Held>> = Mutex::new(Vec::new());

/// What:
///  Holds `programs` for the push record `item` of `git_dir`,
///  in place of any values an earlier landing put there.
/// Why:
///   The landing is the only caller that has the request and the commit it became,
/// so it is where a request's values meet the record that will run them.
/// A record whose id is held again takes the newer values:
/// the same commit landing twice is the same push,
/// and the second landing's command is the one that asked for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// remember(gitDir, item, programs); // void
/// ```
pub fn remember(git_dir: &Path, item: &str, programs: PushPrograms) {
    let mut held: std::sync::MutexGuard<'_, Vec<Held>> = HELD.lock().expect("the programs of the pushes in flight");
    let entry: Held = Held { git_dir: git_dir.to_path_buf(), item: String::from(item), programs };
    match held.iter_mut().find(|existing| { return existing.git_dir == entry.git_dir && existing.item == entry.item; }) {
        Some(existing) => *existing = entry,
        None => held.push(entry),
    }
}

/// What:
///  The values the push record `item` of `git_dir` was given,
///  or nothing when no command of this service gave any.
/// Why:
///   Nothing is the answer for a record a service restart made it re-hold,
/// and for one an earlier release queued:
/// the push then either searches the service's own list and says so,
/// or stops because the transport of the command that asked for it is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// held(gitDir, item); // PushPrograms | undefined
/// ```
pub fn held(git_dir: &Path, item: &str) -> Option<PushPrograms> {
    let held: std::sync::MutexGuard<'_, Vec<Held>> = HELD.lock().expect("the programs of the pushes in flight");
    return held
        .iter()
        .find(|existing| { return existing.git_dir == git_dir && existing.item == item; })
        .map(|found| { return found.programs.clone(); });
}

/// What:
///  Forgets the values of the push record `item` of `git_dir`,
///  because that record can no longer run.
/// Why:
///   A record is settled once:
/// it was pushed,
/// it failed for good,
/// or nothing was left to push.
/// The values outlive nothing else,
/// so a service that keeps running does not keep a credential of a finished push.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forget(gitDir, item); // void
/// ```
pub fn forget(git_dir: &Path, item: &str) {
    let mut held: std::sync::MutexGuard<'_, Vec<Held>> = HELD.lock().expect("the programs of the pushes in flight");
    held.retain(|existing| { return !(existing.git_dir == git_dir && existing.item == item); });
}

/// What:
///  Forgets every value of the push records of `git_dir`.
/// Why:
///   A worktree whose Git directory is gone has no push that can run,
/// and a service that is told its repository is gone gives the values up with the records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetEvery(gitDir); // void
/// ```
pub fn forget_every(git_dir: &Path) {
    let mut held: std::sync::MutexGuard<'_, Vec<Held>> = HELD.lock().expect("the programs of the pushes in flight");
    held.retain(|existing| { return existing.git_dir != git_dir; });
}

/// The controls of the list and of its memory are in the sidecar.
#[cfg(test)]
#[path = "service_push_programs_held_in_memory_tests.rs"]
mod tests;
