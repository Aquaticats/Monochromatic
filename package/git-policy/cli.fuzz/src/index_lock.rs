//! What: Generated Git PID file text and `/proc/<pid>/stat` lines for the index-lock evidence
//!       readers, with the invariants of their parsers.
//! Why: A PID file that names a live process makes a native wrapper wait without limit, and a
//!      start tick read from a misparsed stat line decides whether that process owns the lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const { text, expected } of generatedPidText(data)) expect(parsePidFileText(text)).toBe(expected);
//! ```

/// The generator cursor shared with the owner records.
use super::owners::Cursor;
/// The PID file text reader under test.
use git_policy_cli::index_lock_evidence::parse_pid_file_text;
/// The JavaScript `trimEnd` the incumbent applies before reading a PID file.
use git_policy_cli::js_text::trim_javascript_end;
/// The safe-integer bound both readers apply.
use git_policy_cli::json_record::MAX_SAFE_INTEGER;
/// The stat-line reader under test.
use git_policy_cli::process_start::parse_linux_stat;

/// Prefixes a PID file may start with: the exact one, near misses, and a non-PID text.
const PID_PREFIXES: [&str; 5] = ["pid ", "pid", "PID ", "pid  ", "x"];

/// Suffixes a PID file may end with: none, JavaScript whitespace, and text that is not whitespace.
const PID_SUFFIXES: [&str; 5] = ["", "\n", "\r\n", " \t\n", "x"];

/// Whether a generated suffix is only JavaScript whitespace, in step with `PID_SUFFIXES`.
const SUFFIX_IS_WHITESPACE: [bool; 5] = [true, true, true, true, false];

/// Command-name pieces a generated stat line may contain, including the parentheses the reader must skip.
const COMMAND_PIECES: [&str; 7] = ["git", "a) b", " ", "(", ")", "\u{e9}", ""];

/// States a generated stat line may carry.
const STATES: [&str; 6] = ["R", "S", "D", "Z", "X", "T"];

/// What: One generated PID file text and the PID it must read as, or nothing.
/// Why:  The expectation is built from the chosen pieces, not computed by the parser under test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GeneratedPidText = { text: string; expected: number | undefined };
/// ```
pub fn generated_pid_text(data: &[u8]) -> (String, Option<i64>) {
    let mut cursor: Cursor<'_> = Cursor::new(data);
    let prefix_index: usize = usize::from(cursor.byte()) % PID_PREFIXES.len();
    let suffix_index: usize = usize::from(cursor.byte()) % PID_SUFFIXES.len();
    let (digits, value): (String, Option<i64>) = generated_digits(&mut cursor);
    let text: String = format!(
        "{}{digits}{}",
        PID_PREFIXES[prefix_index], PID_SUFFIXES[suffix_index]
    );
    let accepted: bool = prefix_index == 0 && SUFFIX_IS_WHITESPACE[suffix_index];
    if accepted {
        return (text, value);
    }
    return (text, None);
}

/// What: The digits of a generated PID and the value they read as, when they read as one.
/// Why:  Valid, zero, a leading-zero spelling, a value past the safe range and non-digits are
///       the cases the reader must split between accepting and refusing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedDigits(cursor): [string, number | undefined];
/// ```
fn generated_digits(cursor: &mut Cursor<'_>) -> (String, Option<i64>) {
    let choice: u8 = cursor.byte() % 5;
    if choice == 0 {
        let value: i64 = cursor.positive();
        return (value.to_string(), Some(value));
    }
    if choice == 1 {
        return (String::from("0"), None);
    }
    if choice == 2 {
        return (String::from("007"), Some(7));
    }
    if choice == 3 {
        return (String::from("9007199254740992"), None);
    }
    return (String::from("4x"), None);
}

/// What: The PID file reader on generated text and on raw bytes read as text.
/// Why:  Generated text pins the accepted spellings; raw bytes prove the reader never accepts a
///       value outside the safe range or a text the trailing-whitespace rule would leave unread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(parsePidFileText(text)).toBe(expected);
/// ```
pub fn check_pid_file_text(data: &[u8]) {
    let (text, expected): (String, Option<i64>) = generated_pid_text(data);
    assert_eq!(parse_pid_file_text(text.as_str()), expected, "{text:?}");
    let raw: String = String::from_utf8_lossy(data).into_owned();
    if let Some(pid) = parse_pid_file_text(raw.as_str()) {
        assert!((1..=MAX_SAFE_INTEGER).contains(&pid), "{pid}");
        let trimmed: &str = trim_javascript_end(raw.as_str());
        assert!(trimmed.starts_with("pid "), "{raw:?}");
        let rewritten: String = format!("pid {pid}\n");
        assert_eq!(parse_pid_file_text(rewritten.as_str()), Some(pid));
    }
}

/// What: One generated `/proc/<pid>/stat` line and the state and start ticks it must read as, or
///       nothing.
/// Why:  The command name may hold spaces and parentheses; the last `)` ends it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GeneratedStat = { line: string; expected: [string, number] | undefined };
/// ```
pub fn generated_linux_stat(data: &[u8]) -> (String, Option<(String, i64)>) {
    let mut cursor: Cursor<'_> = Cursor::new(data);
    let command_count: usize = usize::from(cursor.byte()) % 4;
    let mut command: String = String::new();
    for _ in 0..command_count {
        command.push_str(COMMAND_PIECES[usize::from(cursor.byte()) % COMMAND_PIECES.len()]);
    }
    let state: &str = STATES[usize::from(cursor.byte()) % STATES.len()];
    let (ticks_text, ticks): (String, Option<i64>) = generated_ticks(&mut cursor);
    let line: String = format!(
        "4242 ({command}) {state} 1 4242 4242 0 -1 4194560 100 0 0 0 1 2 0 0 20 0 1 0 {ticks_text} 1000 100 0\n"
    );
    match ticks {
        Some(value) => return (line, Some((String::from(state), value))),
        None => return (line, None),
    }
}

/// What: The start-tick field of a generated stat line and the value it must read as, when it does.
/// Why:  Ticks past the safe range and non-digit ticks must be refused, not rounded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedTicks(cursor): [string, number | undefined];
/// ```
fn generated_ticks(cursor: &mut Cursor<'_>) -> (String, Option<i64>) {
    let choice: u8 = cursor.byte() % 3;
    if choice == 0 {
        let value: i64 = cursor.positive();
        return (value.to_string(), Some(value));
    }
    if choice == 1 {
        return (String::from("9007199254740992"), None);
    }
    return (String::from("x42"), None);
}

/// What: The stat reader on generated lines and on raw bytes read as text.
/// Why:  A command name may hold any byte the process chose; the reader must still find the
///       state and ticks after the last `)`, and never return a state with a space in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(parseLinuxStat(line)).toEqual(expected);
/// ```
pub fn check_linux_stat(data: &[u8]) {
    let (line, expected): (String, Option<(String, i64)>) = generated_linux_stat(data);
    assert_eq!(parse_linux_stat(line.as_str()), expected, "{line:?}");
    let raw: String = String::from_utf8_lossy(data).into_owned();
    if let Some((state, ticks)) = parse_linux_stat(raw.as_str()) {
        assert!(!state.is_empty() && !state.contains(' '), "{state:?}");
        assert!((0..=MAX_SAFE_INTEGER).contains(&ticks), "{ticks}");
    }
}

/// Generator and invariant controls.
#[cfg(test)]
#[path = "index_lock_tests.rs"]
pub(crate) mod tests;
