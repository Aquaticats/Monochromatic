//! What:
//! The arrival marker a shadow leaves in the environment of the program it starts:
//! how it started it,
//! its process number,
//! its birth identity and its own name,
//! and the code that reads the marker back,
//! decides whether it was left for this process or for its direct parent,
//! and gives the one line a stopped shadow prints.
//! Why:
//! No check of a file sees what a script or a launcher with no record does,
//! so a start that a shadow makes is shown by the start itself:
//! a shadow that finds the mark left for its own process,
//! or for its direct parent,
//! stops at once,
//! and one left for anyone else (a script the real tool runs,
//! a reused process number) passes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const marker = `spawn ${process.pid} ${birthOf(process.pid)} ${crateName} ${version}`;
//! const arrival = arrivalAtThisProcess(); // undefined unless it was meant for us
//! ```

/// Imports the file-system module that reads `/proc/<pid>/stat` for a process's birth identity.
use std::fs;

/// Imports the command type a shadow adds the marker to before it starts the real program.
use std::process::Command;

/// Imports what this program says it is,
/// which every marker names.
use super::the_carried_identity_text::own_identity;

/// The environment variable that carries the arrival marker from a shadow to the program it starts.
pub const ARRIVAL_VARIABLE: &str = "MONOCHROMATIC_SHADOW_STARTED_BY";

/// The exit status of a shadow that was started by a shadow.
pub const ARRIVAL_EXIT_CODE: i32 = 2;

/// How a shadow starts the program it chose.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartMode {
    /// The shadow replaces itself with the program:
    /// the process number stays.
    Exec,
    /// The shadow runs the program as a child and waits:
    /// the child's parent is the shadow.
    Spawn,
}

/// What an arrival marker says,
/// taken apart.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Arrival {
    /// How the starting shadow started the program.
    pub mode: StartMode,
    /// The process number of the starting shadow.
    pub pid: u32,
    /// The birth identity of that process,
    /// which changes when its number is reused.
    pub birth: String,
    /// The crate name and version of the starting shadow.
    pub starter: String,
}

/// What:
/// `pub fn birth_in(stat_text: &str) -> String` is the start time in clock ticks since boot that a `/proc/<pid>/stat` text names (field 22),
/// or `?` when the text has no such field.
/// Why:
/// The name field of that file may hold spaces and brackets,
/// so the fields are counted after its last closing bracket.
pub fn birth_in(stat_text: &str) -> String {
    let Some((_, after_name)) = stat_text.rsplit_once(')') else {
        return "?".to_string();
    };
    return after_name
        .split_whitespace()
        .nth(19)
        .unwrap_or("?")
        .to_string();
}

/// What:
/// `pub fn process_birth(pid: u32) -> String` is the birth identity of a process,
/// or `?` when it cannot be read.
/// Why:
/// A process number alone can name a later process once the first has ended;
/// the start time cannot.
pub fn process_birth(pid: u32) -> String {
    return match fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(text) => birth_in(&text),
        Err(_) => "?".to_string(),
    };
}

/// What:
/// `pub fn arrival_value(mode: StartMode) -> String` is the marker a shadow puts in the environment of the program it starts.
/// Why:
/// The program,
/// if it is a shadow,
/// finds that it was started by this process,
/// and stops.
pub fn arrival_value(mode: StartMode) -> String {
    let word: &str = match mode {
        StartMode::Exec => "exec",
        StartMode::Spawn => "spawn",
    };
    let pid: u32 = std::process::id();
    return format!("{word} {pid} {} {}", process_birth(pid), own_identity());
}

/// What:
/// `pub fn parse_arrival(text: &str) -> Option<Arrival>` reads what `arrival_value` wrote;
/// any other text is `None`.
/// Why:
/// A value left by anything else,
/// or a damaged one,
/// is never believed:
/// the shadow behaves as if started fresh.
pub fn parse_arrival(text: &str) -> Option<Arrival> {
    let mut fields = text.splitn(4, ' ');
    let mode: StartMode = match fields.next()? {
        "exec" => StartMode::Exec,
        "spawn" => StartMode::Spawn,
        _ => return None,
    };
    let pid: u32 = fields.next()?.parse().ok()?;
    let birth: String = fields.next()?.to_string();
    let starter: String = fields.next()?.to_string();
    return Some(Arrival {
        mode,
        pid,
        birth,
        starter,
    });
}

/// What:
/// `pub fn is_meant_for(arrival: &Arrival, own: (u32, &str), parent: (u32, &str)) -> bool` says whether this process is the one the marker was left for.
/// Why:
/// A shadow that replaced itself is the very process the marker names (same number,
/// same birth);
/// a shadow that started a child is that child's parent.
/// Anyone else,
/// such as a script the real tool runs,
/// passes.
pub fn is_meant_for(
    arrival: &Arrival,
    own: (u32, &str),
    parent: (u32, &str),
) -> bool {
    let (pid, birth): (u32, &str) = match arrival.mode {
        StartMode::Exec => own,
        StartMode::Spawn => parent,
    };
    return arrival.pid == pid && arrival.birth == birth;
}

/// What:
/// `pub fn arrival_at_this_process() -> Option<Arrival>` is the marker left for this process,
/// when there is one.
/// Why:
/// It reads the environment and the two process identities once;
/// the shim stops when it answers `Some`.
pub fn arrival_at_this_process() -> Option<Arrival> {
    let text: String = std::env::var(ARRIVAL_VARIABLE).ok()?;
    let arrival: Arrival = parse_arrival(&text)?;
    let own_pid: u32 = std::process::id();
    let parent_pid: u32 = std::os::unix::process::parent_id();
    let meant: bool = is_meant_for(
        &arrival,
        (own_pid, &process_birth(own_pid)),
        (parent_pid, &process_birth(parent_pid)),
    );
    return if meant { Some(arrival) } else { None };
}

/// What:
/// `pub fn arrival_line(tool: &str, arrival: &Arrival) -> String` is the one line printed when a shadow was started by a shadow.
/// Why:
/// It names the stopped shadow,
/// the shadow that started it,
/// the tool and what to do,
/// so a reader who knows nothing about shadows can act.
pub fn arrival_line(
    tool: &str,
    arrival: &Arrival,
) -> String {
    return format!(
        "{own}: stopped at once, before doing anything, because it was started by {starter} (process {pid}) as the real `{tool}`; a shadow never starts a shadow. \
         Put the real `{tool}` before the shadows on PATH, or remove the program on PATH that leads back to a shadow. Exit status {ARRIVAL_EXIT_CODE} means this stop.",
        own = own_identity(),
        starter = arrival.starter,
        pid = arrival.pid,
    );
}

/// What:
/// `pub fn mark_start(command: &mut Command, mode: StartMode)` adds the arrival marker to the environment of a command that starts the real program.
/// Why:
/// Every start of a real program carries it,
/// so a shadow among the programs reached (a launcher with no record,
/// a script) stops on arrival.
pub fn mark_start(
    command: &mut Command,
    mode: StartMode,
) {
    command.env(ARRIVAL_VARIABLE, arrival_value(mode));
}
