//! What:
//!  The marker every start of the real Git carries:
//!  its value,
//!  the way it is read back,
//!  and the one lookup of it in an environment.
//! Why:
//!  The guard that stops a wrapper which was started as Git,
//!  the chain record a launcher cannot erase and the
//!  builders that start Git all speak this text,
//! so its format lives in one module they read instead of one of
//!  them.
//!  A marker older than the count (`pid`,
//! identity and mode only) is read as a first start,
//!  which is what it was.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const marker = [pid, identity, mode, starts].join('\x1f'); parseMarker(markerText);
//! ```

/// The one lookup of a named variable in an injected environment.
use crate::child_environment::environment_value;
/// `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes (siblings `&str`/`String` must be UTF-8).
use std::ffi::{OsStr, OsString};

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

/// What:
///  The most starts of real Git one chain may hold before a wrapper stops it.
/// Why:
///   The guard below recognises the process a marker names,
/// and a script named `git` that starts the wrapper as a child puts
///  itself between the two,
/// so the same wrapper start arrives looking like one further down a chain
///  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 1).
/// Counting the starts ends any such chain,
///  however many processes stand between two of them.
/// A finite chain is legitimate and must run:
/// nine `!` aliases that each run the next through `git`,
/// where Git's own exec directory offers the wrapper,
/// reach the wrapper once per alias and need a tenth start for the command itself
///  (`a_finite_chain_of_aliases_reaches_the_command_the_person_asked_for`,
/// measured there).
/// So the bound sits above a chain three times as long,
///  32 starts,
/// and a chain that leads back without a real Git of the person's between its levels ends at the 33rd
///  start,
/// measured at 1.3 s from the first wrapper to the refusal.
/// A stop one level too early is repaired by
///  the person who reads the message;
/// a chain that never ends is not.
pub const MOST_STARTS_IN_ONE_CHAIN: u32 = 32;

/// What:
/// The byte between the fields of the marker:
/// the unit separator,
/// which a process number,
/// a birth identity or a mode
///  word never contains.
pub const FIELD_SEPARATOR: char = '\u{1f}';

/// The mode word of a start that replaces the starting process.
const REPLACE_WORD: &str = "replace";

/// The mode word of a start that runs beside the starting process,
/// which waits for it.
const CHILD_WORD: &str = "child";

/// The mode word of a start the wrapper makes to read the repository itself,
/// whose output it captures and whose standard input it closes.
const QUERY_WORD: &str = "query";

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
    /// The starter runs the program to read the repository,
    /// with its output captured and its standard input closed,
    /// so nothing of the person's can run under it:
    /// a wrapper that arrives as the Git of such a start came from the file that Git is,
    /// which
    ///  is the wrapper leading back to itself.
    Query,
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
///   The check compares these facts with the live process;
/// keeping them in one value lets a test hand the check
///  a marker without touching the real environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StartMarker = { starterPid: number; starterIdentity: string; mode: StartMode; starts: number };
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
    /// How many marked starts of real Git this start is from the wrapper the person ran,
    /// counting itself:
    /// one for a first start.
    pub starts: u32,
}

/// What:
///  Build the marker value for start `starts` by the process `own_pid` with birth identity `own_identity`.
///       `&str` borrows the identity text;
/// `OsString` is the owned value an environment holds.
/// Why:
///   One function writes the format and `parse_marker` reads it,
/// so the two cannot drift apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = [pid, identity, { replace: 'replace', child: 'child', query: 'query' }[mode], starts].join('\x1f');
/// ```
pub fn marker_value(mode: StartMode, own_pid: u32, own_identity: &str, starts: u32) -> OsString {
    let word: &str = match mode {
        StartMode::Replace => REPLACE_WORD,
        StartMode::Child => CHILD_WORD,
        StartMode::Query => QUERY_WORD,
    };
    // `format!` builds owned text; `OsString::from` turns it into environment text.
    return OsString::from(format!("{own_pid}{FIELD_SEPARATOR}{own_identity}{FIELD_SEPARATOR}{word}{FIELD_SEPARATOR}{starts}"));
}

/// What:
///  Take a marker value apart.
///       `Option<StartMarker>` is "the marker,
/// or nothing when the text is not one".
/// Why:
///   A variable some other program set to other text must not stop a wrapper,
/// so anything that is not three
///  or four fields of the right kinds is no marker.
/// A marker of three fields is one an earlier release wrote,
/// which  never counted its chain:
/// it is read as the first start,
/// which is what it was.
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
    // `.parse::<u32>().ok()?` reads the digits, turning a failure into `None`; a marker of three fields has no count.
    let starts: u32 = match fields.next() {
        Some(counted) => counted.parse::<u32>().ok()?,
        None => 1,
    };
    if fields.next().is_some() {
        return None;
    }
    // `.parse::<u32>().ok()?` reads the starter's number, turning a failure into `None`.
    let starter_pid: u32 = pid_text.parse::<u32>().ok()?;
    let mode: StartMode = match word {
        REPLACE_WORD => StartMode::Replace,
        CHILD_WORD => StartMode::Child,
        QUERY_WORD => StartMode::Query,
        _ => return None,
    };
    // `Some(...)` is the "present" case of an `Option`.
    return Some(StartMarker { starter_pid, starter_identity: identity.to_string(), mode, starts });
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
