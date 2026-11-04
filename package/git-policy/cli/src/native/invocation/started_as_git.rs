//! What:
//!  The marker that every start of the real Git carries,
//! and the check that stops a cli-git executable which finds it was
//!  started as that Git.
//! Why:
//!  Which file the wrapper selected as Git can be a launcher,
//! a link,
//! a copy or another release of the wrapper,
//! and no test
//!  of that file can recognise every one of them.
//! What no spelling hides is the start itself:
//! the wrapper that starts
//!  "Git" puts its own process number,
//! that process's birth identity and the way it starts the program (replacing itself or
//!  as a child) in the environment of the start.
//! A cli-git executable that finds itself in the very process,
//! or the child,
//!  that a wrapper started as Git has been started as Git,
//! whatever stood in between,
//! and stops at once.
//!  Anything started further down (a hook,
//! an alias,
//! an editor a real Git started) has another process in between and
//!  passes:
//! running `git` there through an installed wrapper stays legitimate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // child_process.spawn(realGit, args, { env: { ...env, CLI_GIT_STARTED_AS_GIT: `${process.pid}\x1f${identity}\x1fchild` } });
//! // if (marker && marker.pid === process.ppid && sameBirth(marker)) { stop('started as Git by a cli-git process'); }
//! ```

/// The one lookup of a named variable in an injected environment.
use crate::child_environment::{FORWARD_TARGET_VARIABLE, environment_value};
/// What:
///  The birth identity of a process,
/// which changes when its number is reused.
/// Why:
///   A process number alone could name a later process after the first one ended;
/// the identity cannot.
use crate::process_identity::{current_birth_identity, process_birth_identity};
/// What:
///  `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes (siblings `&str`/`String` must be UTF-8).
/// Why:
///   Environment values and paths need not be UTF-8.
use std::ffi::{OsStr, OsString};
/// `Path` is a borrowed filesystem path of raw bytes (owned sibling `PathBuf`).
use std::path::Path;
/// The child-process builder that receives the marker.
use std::process::Command;

/// What:
///  The name of the marker variable.
///       `&str` is a borrowed text baked into the program (owned sibling:
/// `String`).
/// Why:
///   Every start of the real Git sets it,
/// and every wrapper start reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STARTED_AS_GIT_VARIABLE = 'CLI_GIT_STARTED_AS_GIT';
/// ```
pub const STARTED_AS_GIT_VARIABLE: &str = "CLI_GIT_STARTED_AS_GIT";

/// The byte between the three fields of the marker:
/// the unit separator,
/// which a process number,
/// a birth identity or a mode
///  word never contains.
const FIELD_SEPARATOR: char = '\u{1f}';

/// The mode word of a start that replaces the starting process.
const REPLACE_WORD: &str = "replace";

/// The mode word of a start that runs beside the starting process,
/// which waits for it.
const CHILD_WORD: &str = "child";

/// What:
///  How the process that starts "Git" starts it.
///       `#[derive(...)]` generates copying,
/// debug printing and `==`.
/// Why:
///   The two ways leave different traces.
/// A replaced process keeps its number,
/// so the started program is the very process
///  the marker names;
/// a child has the starter as its parent.
/// The check looks at the trace its mode leaves and nowhere else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StartMode = 'replace' | 'child';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartMode {
    /// The starter replaces its own program with the one it starts (`exec`);
    /// the process number stays.
    Replace,
    /// The starter runs the program as a child and waits for it.
    Child,
}

/// What:
///  What a marker says,
/// taken apart.
///       `u32` is an unsigned 32-bit integer (siblings `u64`,
/// `i32`):
/// the type of a Linux process number.
///       `String` is owned text (sibling `&str`,
/// a borrowed view).
/// Why:
///   The check compares these three facts with the live process;
/// keeping them in one value lets a test hand the check
///  a marker without touching the real environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StartMarker = { starterPid: number; starterIdentity: string; mode: StartMode };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartMarker {
    /// The process number of the wrapper that started the program as Git.
    pub starter_pid: u32,
    /// The birth identity of that process;
    /// empty where this platform has none.
    pub starter_identity: String,
    /// How the program was started.
    pub mode: StartMode,
}

/// What:
///  Build the marker value for a start by the process `own_pid` with birth identity `own_identity`.
///       `&str` borrows the identity text;
/// `OsString` is the owned value an environment holds.
/// Why:
///   One function writes the format and `parse_marker` reads it,
/// so the two cannot drift apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = [pid, identity, mode === 'replace' ? 'replace' : 'child'].join('\x1f');
/// ```
pub fn marker_value(mode: StartMode, own_pid: u32, own_identity: &str) -> OsString {
    let word: &str = if mode == StartMode::Replace { REPLACE_WORD } else { CHILD_WORD };
    // `format!` builds owned text; `OsString::from` turns it into environment text.
    return OsString::from(format!("{own_pid}{FIELD_SEPARATOR}{own_identity}{FIELD_SEPARATOR}{word}"));
}

/// What:
///  Take a marker value apart.
///       `Option<StartMarker>` is "the marker,
/// or nothing when the text is not one".
/// Why:
///   A variable some other program set to other text must not stop a wrapper,
/// so anything that is not exactly three
///  fields of the right kinds is no marker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseMarker(value: string): StartMarker | undefined;
/// ```
pub fn parse_marker(value: &OsStr) -> Option<StartMarker> {
    // `?` on an `Option` returns `None` from this function when the value is absent; here the text must be UTF-8.
    let text: &str = value.to_str()?;
    let mut fields = text.split(FIELD_SEPARATOR);
    let pid_text: &str = fields.next()?;
    let identity: &str = fields.next()?;
    let word: &str = fields.next()?;
    if fields.next().is_some() {
        return None;
    }
    // `.parse::<u32>().ok()?` reads the digits, turning a failure into `None`.
    let starter_pid: u32 = pid_text.parse::<u32>().ok()?;
    let mode: StartMode = match word {
        REPLACE_WORD => StartMode::Replace,
        CHILD_WORD => StartMode::Child,
        _ => return None,
    };
    // `Some(...)` is the "present" case of an `Option`.
    return Some(StartMarker { starter_pid, starter_identity: identity.to_string(), mode });
}

/// What:
///  The birth identity of this process,
/// or empty text where none can be read.
/// Why:
///   A start is marked even where the platform cannot name a birth;
/// the check then compares process numbers alone.
///  Only Linux reads it here,
/// from one small file that stays cached for the life of the process;
/// the other platforms
///  would start a program to ask,
/// which every real-Git start must not pay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = process.platform === 'linux' ? readBirth(process.pid) ?? '' : '';
/// ```
fn own_identity() -> String {
    #[cfg(target_os = "linux")]
    {
        // `.unwrap_or_default()` substitutes empty text when the identity cannot be read.
        return current_birth_identity().unwrap_or_default();
    }
    #[cfg(not(target_os = "linux"))]
    {
        return String::new();
    }
}

/// What:
///  Put the marker in the environment of a real-Git start.
///       `&mut Command` lends the builder for changing.
/// Why:
///   Every place that starts the real Git calls this,
/// so a wrapper that arrives as Git can tell.
/// `Command::env` overrides
///  an earlier value of the same name,
/// so a start that replaces the process can override the child marker that the shared
///  builder put there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// command.env.CLI_GIT_STARTED_AS_GIT = markerValue(mode, process.pid, ownIdentity());
/// ```
pub fn mark_real_git_start(command: &mut Command, mode: StartMode) {
    command.env(STARTED_AS_GIT_VARIABLE, marker_value(mode, std::process::id(), own_identity().as_str()));
}

/// What:
///  What the check asks of the live process.
///       `&dyn Fn(u32) -> Option<String>` borrows any function from a process number to its birth identity.
/// Why:
///   A control hands the check made-up numbers and identities;
/// production hands it the process's own facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProcessFacts = { ownPid: number; parentPid: number; identityOf: (pid: number) => string | undefined };
/// ```
pub struct ProcessFacts<'a> {
    /// This process's number.
    pub own_pid: u32,
    /// The number of this process's parent.
    pub parent_pid: u32,
    /// The birth identity of the process with a number;
    /// `None` when it cannot be read.
    pub identity_of: &'a dyn Fn(u32) -> Option<String>,
}

/// What:
///  Whether the marker names the process this one was started by,
/// in the way its mode says.
/// Why:
///   `Replace`:
/// the starter replaced itself,
/// so this is that very process (same number,
/// same birth).
/// `Child`:
/// the starter
///  ran this program as its child,
/// so the starter is this process's parent.
/// In both a identity the platform can read must
///  agree,
/// so a later process that reuses the number is not taken for the starter.
/// A process further down has another
///  process in between and is not the start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startedAsGit(marker: StartMarker, facts: ProcessFacts): boolean;
/// ```
pub fn started_as_git(marker: &StartMarker, facts: &ProcessFacts<'_>) -> bool {
    let candidate: u32 = if marker.mode == StartMode::Replace { facts.own_pid } else { facts.parent_pid };
    if candidate != marker.starter_pid {
        return false;
    }
    if marker.starter_identity.is_empty() {
        return true;
    }
    // What: `match` on the identity read: a readable identity must be the recorded one; an unreadable one cannot
    //       contradict it, so the numbers decide.
    // Why:  Stopping one start too many is repaired by the person who reads the message; missing a start is not.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const found = facts.identityOf(candidate); return found === undefined || found === marker.starterIdentity;
    // ```
    match (facts.identity_of)(candidate) {
        Some(found) => return found == marker.starter_identity,
        None => return true,
    }
}

/// What:
///  The marker of an environment,
/// if it carries one.
/// Why:
///   The caller's environment is passed in,
/// never read from the process,
/// so a control sets any value it likes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const marker = parseMarker(environment.CLI_GIT_STARTED_AS_GIT);
/// ```
pub fn marker_of(environment: &[(OsString, OsString)]) -> Option<StartMarker> {
    let value: OsString = environment_value(environment, STARTED_AS_GIT_VARIABLE)?;
    return parse_marker(value.as_os_str());
}

/// What:
///  The words a person reads when a cli-git executable stops because it was started as Git.
///       `&Path` borrows this executable's path for the message.
/// Why:
///   The message names the executable,
/// the process that started it and how,
/// the Git that process chose when the
///  environment says so,
/// the `PATH` this process saw,
/// why stopping is right,
/// and every way out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusalText(marker, ownExecutable, environment): string;
/// ```
pub fn refusal_text(marker: &StartMarker, own_executable: &Path, environment: &[(OsString, OsString)]) -> String {
    let how: String = if marker.mode == StartMode::Replace {
        format!("this is process {}, which a cli-git wrapper replaced with this program", marker.starter_pid)
    } else {
        format!("this is a child of the cli-git wrapper that runs as process {} and waits for its answer", marker.starter_pid)
    };
    let chosen: String = match environment_value(environment, FORWARD_TARGET_VARIABLE) {
        Some(target) => format!(" The file that process chose as Git is {}.", Path::new(&target).display()),
        None => String::new(),
    };
    let search_path: String = match environment_value(environment, "PATH") {
        Some(value) => format!(" PATH here is {}.", Path::new(&value).display()),
        None => String::from(" PATH is not set here."),
    };
    return format!(
        "this cli-git executable ({}) was started as Git by a cli-git wrapper: {how}.{chosen}{search_path} \
         A cli-git wrapper never runs itself as Git, so it stops here instead of starting one more wrapper, which would \
         repeat without end. This happens when the first `git` that PATH offers after the wrapper leads back to cli-git: \
         a link to it, a copy of it, its toolchain launcher, or another release of it. To fix it, put the directory of the \
         real Git (for example /usr/bin) on the PATH of the program that started the first cli-git, or remove the entry that \
         leads back to cli-git from that PATH.",
        own_executable.display()
    );
}

/// What:
///  The birth identity of the process with a number, read from the operating system now.
///       `.ok().flatten()` turns a read failure and an absent process into the same `None`.
/// Why:
///   The check treats an identity it cannot read as no contradiction, so both cases need no distinction here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function liveIdentityOf(pid: number): string | undefined { try { return readBirth(pid); } catch { return undefined; } }
/// ```
fn live_identity_of(pid: u32) -> Option<String> {
    return process_birth_identity(i64::from(pid)).ok().flatten();
}

/// What:
///  Whether this process was started as Git by a cli-git wrapper,
/// read from the live process,
/// and if so the words to
///  say so.
///       `Option<String>` is "the message,
/// or nothing when this process was not started as Git".
/// Why:
///   The executable calls it once,
/// before it plans anything.
/// It reads the parent's number and,
/// only when the marker's
///  number matches,
/// one small process file for the birth identity,
/// so an ordinary start costs a variable lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusalWhenStartedAsGit(environment, ownExecutable): string | undefined;
/// ```
pub fn refusal_when_started_as_git(environment: &[(OsString, OsString)], own_executable: &Path) -> Option<String> {
    let marker: StartMarker = marker_of(environment)?;
    let facts: ProcessFacts<'_> = ProcessFacts { own_pid: std::process::id(), parent_pid: parent_pid(), identity_of: &live_identity_of };
    if !started_as_git(&marker, &facts) {
        return None;
    }
    return Some(refusal_text(&marker, own_executable, environment));
}

/// What:
///  The process number of this process's parent.
/// Why:
///   A child start leaves the starter as the parent;
/// the standard library does not expose it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parentPid = process.ppid;
/// ```
#[cfg(unix)]
fn parent_pid() -> u32 {
    // SAFETY: `getppid` takes no argument, reads no memory of this process and cannot fail.
    let parent: libc::pid_t = unsafe { libc::getppid() };
    // `u32::try_from(..)` refuses a negative number, which `getppid` never returns; `.unwrap_or(0)` then means no parent.
    return u32::try_from(parent).unwrap_or(0);
}

/// Without a parent number the child check matches nothing (this crate runs on Linux only).
#[cfg(not(unix))]
fn parent_pid() -> u32 {
    return 0;
}

/// Disposable-fact controls of the marker and the check stay out of the release executable.
#[cfg(test)]
#[path = "started_as_git_tests.rs"]
mod tests;
