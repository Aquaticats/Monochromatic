//! What:
//! Whether a path under the repository is unreachable because the repository's link to the 860 drive dangles,
//! and the plain message that says so,
//! by asking the installed repository link check (`package/repository-link/check`) once,
//! on a failure path only.
//! Why:
//! After the repository moves to the 860 drive its old path is a link to the copy there.
//! An unreadable lock file or a launcher folder that cannot be made then has one common cause the person can act on
//! (unlock the drive),
//! and the check already words it.
//! The launcher stays fast:
//! nothing here runs unless a read or a write already failed,
//! or an install is about to write under the link,
//! and with no identity file (`~/.config/monochromatic-dev/repository-link.json`) nothing starts.
//! The check program is the one installed outside the repository,
//! so it never dangles itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // repository-link-cause.ts
//! export function repositoryLinkVerdict(home: string, path: string, deadlineMs: number): LinkVerdict
//! ```

/// Imports `Duration` and `Instant` from `std::time`.
use std::time::{Duration, Instant};
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// The identity file the cut-over writes,
/// below the home directory.
pub const IDENTITY_RELATIVE_PATH: &str = ".config/monochromatic-dev/repository-link.json";

/// The installed check program,
/// below the home directory:
/// outside the repository,
/// so it runs when the repository is away.
pub const CHECK_RELATIVE_PATH: &str = ".local/share/monochromatic-dev/repository-link/repository-link-check";

/// How long the check program may run before it is killed and the answer is that it did not answer.
/// The check waits five seconds for the drive itself (`package/repository-link/check/src/constants.ts`,
/// `DEFAULT_DEADLINE_SECONDS`),
/// and the owner's rule is "nothing should ever wait forever or indefinitely";
/// this bound is that wait plus the time to start Node.
pub const CHECK_DEADLINE: Duration = Duration::from_secs(10);

/// The homes whose repository the check already said is usable in this process:
/// an install asks for one launcher after another,
/// and starting the check for each would cost a Node start apiece.
/// A not-ready answer is never remembered.
static KNOWN_READY: std::sync::Mutex<Vec<PathBuf>> = std::sync::Mutex::new(Vec::new());

/// How often the wait for the check program looks at it.
const CHECK_POLL: Duration = Duration::from_millis(10);

/// What:
/// What asking the repository link check came to.
/// Why:
/// A caller prints the message of `NotReady`,
/// and goes on for every other answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinkVerdict = { kind: 'not-configured' | 'ready' } | { kind: 'not-ready'; message: string } | { kind: 'check-unavailable'; why: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkVerdict {
    /// There is no identity file,
    /// or the path is not under the repository:
    /// nothing was asked.
    NotConfigured,
    /// The check says the repository is usable.
    Ready,
    /// The check says it is not,
    /// or did not answer in time;
    /// this is its plain message.
    NotReady {
        /// The plain message.
        message: String,
    },
    /// The check program could not be started;
    /// this says why.
    CheckUnavailable {
        /// Why the program could not be started.
        why: String,
    },
}

/// The text of one field of the identity file,
/// empty when the file or the field is missing or not text.
fn identity_field(home: &Path, field: &str) -> String {
    let Ok(text) = std::fs::read_to_string(home.join(IDENTITY_RELATIVE_PATH)) else {
        return String::new();
    };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) else {
        return String::new();
    };
    return match parsed.get(field) {
        Some(serde_json::Value::String(value)) => value.clone(),
        _ => String::new(),
    };
}

/// The path with every link resolved as far as it exists,
/// and the part that does not exist appended with its `.` and `..` segments applied.
/// A link to a drive that is not mounted does not resolve,
/// but a path through it still lies under the repository.
fn resolved_as_far_as_it_exists(path: &Path) -> PathBuf {
    for candidate in path.ancestors() {
        let Ok(real) = std::fs::canonicalize(candidate) else {
            continue;
        };
        let Ok(rest) = path.strip_prefix(candidate) else {
            continue;
        };
        let mut resolved: PathBuf = real;
        for component in rest.components() {
            match component {
                std::path::Component::Normal(name) => resolved.push(name),
                std::path::Component::ParentDir => {
                    resolved.pop();
                }
                std::path::Component::CurDir | std::path::Component::RootDir | std::path::Component::Prefix(_) => {}
            }
        }
        return resolved;
    }
    return path.to_path_buf();
}

/// What:
/// Whether a path lies under the repository the identity file names:
/// under its link path,
/// under its approved canonical root,
/// or under either with the part that exists resolved.
/// Why:
/// A launcher folder or a lock file is under the repository only by one of its spellings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// underRepositoryLink(home, path)
/// ```
pub fn under_repository_link(home: &Path, path: &Path) -> bool {
    let resolved: PathBuf = resolved_as_far_as_it_exists(path);
    for field in ["linkPath", "canonicalRoot"] {
        let root: String = identity_field(home, field);
        if root.is_empty() {
            continue;
        }
        let root_path: &Path = Path::new(root.as_str());
        let spelled: bool = path.starts_with(root_path);
        // The link path is resolved through its parent only: the final link is the thing a path goes through.
        let resolved_root: PathBuf = match (root_path.parent(), root_path.file_name()) {
            (Some(parent), Some(name)) => resolved_as_far_as_it_exists(parent).join(name),
            _ => root_path.to_path_buf(),
        };
        if spelled || resolved.starts_with(resolved_root.as_path()) || resolved.starts_with(resolved_as_far_as_it_exists(root_path)) {
            return true;
        }
    }
    return false;
}

/// Whether the check already said this home's repository is usable in this process.
fn home_is_known_ready(home: &Path) -> bool {
    let Ok(known) = KNOWN_READY.lock() else {
        return false;
    };
    for ready_home in known.iter() {
        if ready_home == home {
            return true;
        }
    }
    return false;
}

/// Waits for a started check program until the deadline,
/// and returns its exit status,
/// or nothing when the deadline passed (the program is killed).
fn wait_for_check(child: &mut std::process::Child, deadline: Duration) -> Option<std::process::ExitStatus> {
    let give_up_at: Instant = Instant::now() + deadline;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) => {}
            Err(_unknown) => return None,
        }
        if Instant::now() >= give_up_at {
            let _killed = child.kill();
            let _reaped = child.wait();
            return None;
        }
        std::thread::sleep(CHECK_POLL);
    }
}

/// What:
/// Asks the installed repository link check whether the repository is usable,
/// when `path` is under the repository the identity file names.
/// Why:
/// The check words every cause (drive not mounted,
/// folder missing,
/// a real folder in place of the link) once,
/// so no launcher words them again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repositoryLinkVerdict(home, path, deadline)
/// ```
pub fn repository_link_verdict(home: &Path, path: &Path, deadline: Duration) -> LinkVerdict {
    if !home.join(IDENTITY_RELATIVE_PATH).is_file() || !under_repository_link(home, path) {
        return LinkVerdict::NotConfigured;
    }
    if home_is_known_ready(home) {
        return LinkVerdict::Ready;
    }
    let program: PathBuf = home.join(CHECK_RELATIVE_PATH);
    let mut child: std::process::Child = match std::process::Command::new(program.as_path())
        .env("HOME", home)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(started) => started,
        Err(error) => return LinkVerdict::CheckUnavailable { why: format!("{} could not be started: {error}", program.display()) },
    };
    let Some(status) = wait_for_check(&mut child, deadline) else {
        return LinkVerdict::NotReady {
            message: format!("the repository link check did not answer within {} s, so the state of the repository is unknown", deadline.as_secs_f64()),
        };
    };
    if status.success() {
        if let Ok(mut known) = KNOWN_READY.lock() {
            known.push(home.to_path_buf());
        }
        return LinkVerdict::Ready;
    }
    let mut message: String = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        let _read = std::io::Read::read_to_string(&mut stderr, &mut message);
    }
    return LinkVerdict::NotReady { message: message.trim().to_string() };
}

/// The unit tests of the repository link cause.
#[cfg(test)]
#[path = "repository_link_cause_tests.rs"]
mod tests;
