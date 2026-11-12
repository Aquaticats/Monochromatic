//! What:
//!  The program search list a worktree's pushes must use,
//! recorded where the service that runs them can read it:
//! the `PATH` and the program-naming variables of the shell that committed.
//! Why:
//!  The commit service takes the environment the user manager started it with,
//! and the person committing can see
//!  neither that list nor change it:
//! on 2026-11-12 the manager's `PATH` held no folder `git-lfs` is installed in while the shell that
//!  committed had one,
//! so two agents' commits landed and their pushes were refused for a program their own shell could start.
//! A push must not fail where the same push typed in the same shell would succeed,
//! and the client knows what its own shell
//!  resolves,
//! so the client leaves that list beside the work records and the push searches it.
//!
//! It lives in the worktree's own Git directory,
//! under the folder the work records already use,
//! so both sides find it without the
//!  socket path or the state directory:
//! the client knows the Git directory of the request,
//! and the service knows the Git directory of the
//!  work it drains.
//! Every client refreshes it before it submits,
//! so a commit uses the shell that made it,
//! and a push retried after the client is gone uses the
//!  last list that shell left.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writePushPrograms(gitDir, env); const list = readPushPrograms(gitDir); // the service, before every push
//! ```

use crate::private_storage::{create_private_directory_all, read_regular_file, replace_private_file_unsynced};
use serde_json::{Map, Value};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The version of the recorded shape,
/// refused when it is another.
pub const VERSION: u64 = 1;

/// The folder under a worktree's own Git directory that holds this record,
/// the one the durable work records use.
pub const FOLDER: &str = "monochromatic-git-policy-cli";

/// The record's file name.
pub const FILE: &str = "push-programs.json";

/// What:
///  The variable a service's push carries when it searches a record a client left.
/// Why:
///   The refusal says which list it searched,
/// and the list is the environment's,
/// so the environment names the record it came from.
/// A
///  manual push never sets it,
/// and then the refusal says it searched the command's own list.
pub const RECORD_VARIABLE: &str = "MONOCHROMATIC_PUSH_PROGRAM_SEARCH_RECORD";

/// What:
///  The variable a service's push carries when no client left a record,
/// naming where it would have been.
/// Why:
///   A person reading the refusal must learn that the searched list is the manager's,
/// not their shell's.
pub const MISSING_VARIABLE: &str = "MONOCHROMATIC_PUSH_PROGRAM_SEARCH_MISSING";

/// The most a record may hold,
/// so a damaged or hostile file is refused rather than read whole.
pub const RECORD_LIMIT: u64 = 64 * 1024;

/// What:
///  The variables a push's children find their programs through,
/// by the names Git and `git-lfs` read them.
/// Why:
///   `PATH` is where `git-lfs`,
/// a credential helper and `ssh` are found;
/// `GIT_SSH`,
/// `GIT_SSH_COMMAND` and `GIT_PROXY_COMMAND` name those programs
///  outright,
/// and `GIT_EXEC_PATH` is where Git looks for `git-upload-pack` and its other subcommands.
/// A login prompt is not carried:
/// the service's pushes carry the never-prompt pair by the owner's decision
/// (`doc/troubleshooting/git-askpass-dialog-despite-terminal-prompt-off.md`),
/// so `GIT_ASKPASS` and
///  `SSH_ASKPASS` would be overwritten anyway and are left out rather than pretending to travel.
pub const CARRIED: [&str; 5] = ["PATH", "GIT_EXEC_PATH", "GIT_SSH", "GIT_SSH_COMMAND", "GIT_PROXY_COMMAND"];

/// What:
///  Where a worktree's push search list is recorded.
/// Why:
///   Both sides find it from the worktree's own Git directory alone,
/// which the request names and the service validates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recordPath = (gitDir) => join(gitDir, 'monochromatic-git-policy-cli', 'push-programs.json');
/// ```
pub fn record_path(git_dir: &Path) -> PathBuf {
    return git_dir.join(FOLDER).join(FILE);
}

/// What:
///  The variables of `environment` a push carries,
/// in the order [`CARRIED`] names them,
/// with a value that is not text left out.
/// Why:
///   A record is JSON,
/// and a name or value that is not text cannot be written without inventing bytes:
/// dropping it leaves the push on the
///  service's own list for that variable,
/// which refuses rather than choosing another program.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// CARRIED.flatMap((name) => { const value = env[name]; return value === undefined || !isText(name) || !isText(value) ? [] : [[name, value]]; });
/// ```
pub fn carried(environment: &[(OsString, OsString)]) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    for name in CARRIED {
        // The last value of a name wins, as it does for Git's own configuration.
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
        found.push((String::from(plain_name), String::from(plain_value)));
    }
    return found;
}

/// What:
///  Record the program search list of this run for the worktree whose Git directory is `git_dir`.
/// Why:
///   The client that commits is the only process that knows the person's shell;
/// it writes the list before it submits,
/// and the service reads it when it pushes,
/// which may be after the client has exited or after a restart.
/// The write is one rename over the name,
/// so a push never reads half a list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writePushPrograms(gitDir, env); // { v: 1, variables: { PATH: ..., GIT_SSH: ... } }
/// ```
pub fn write(git_dir: &Path, environment: &[(OsString, OsString)]) -> std::io::Result<()> {
    let mut variables: Map<String, Value> = Map::new();
    for (name, value) in carried(environment) {
        variables.insert(name, Value::from(value));
    }
    let record: Value = Value::Object(Map::from_iter([
        (String::from("v"), Value::from(VERSION)),
        (String::from("variables"), Value::Object(variables)),
    ]));
    let mut text: String = record.to_string();
    text.push('\n');
    let folder: PathBuf = git_dir.join(FOLDER);
    create_private_directory_all(folder.as_path())?;
    return replace_private_file_unsynced(record_path(git_dir).as_path(), text.as_bytes());
}

/// What:
///  The recorded list,
/// or nothing when there is none,
/// it cannot be read,
/// or it is not the shape this release writes.
/// Why:
///   A record this cannot read is no evidence of what the shell searches:
/// the service then searches its own `PATH`,
/// which refuses
///  rather than choosing another program,
/// and the refusal says that no record was there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = readJson(recordPath(gitDir)); return record?.v === 1 ? Object.entries(record.variables) : undefined;
/// ```
pub fn read(git_dir: &Path) -> Option<Vec<(OsString, OsString)>> {
    let bytes: Vec<u8> = read_regular_file(record_path(git_dir).as_path(), RECORD_LIMIT).ok()?;
    let text: String = String::from_utf8(bytes).ok()?;
    let document: Value = serde_json::from_str(text.as_str()).ok()?;
    if document.get("v").and_then(Value::as_u64) != Some(VERSION) {
        return None;
    }
    let variables: &Map<String, Value> = document.get("variables")?.as_object()?;
    let mut found: Vec<(OsString, OsString)> = Vec::new();
    for (name, value) in variables {
        if !CARRIED.contains(&name.as_str()) {
            continue;
        }
        found.push((OsString::from(name), OsString::from(value.as_str()?)));
    }
    // A record without the search list is no list.
    let mut has_the_search_list: bool = false;
    for (name, _) in &found {
        has_the_search_list = has_the_search_list || name == OsStr::new("PATH");
    }
    if !has_the_search_list {
        return None;
    }
    return Some(found);
}

/// The record's shape and the two faces of reading it are controlled in the test module.
#[cfg(test)]
#[path = "service_client_program_search_tests.rs"]
mod tests;
