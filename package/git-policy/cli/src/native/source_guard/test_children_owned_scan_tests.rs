//! What:
//!  The scans that read one test file for the two shapes this crate refuses:
//!  a child started without an owner,
//!  and a signal aimed at the process the test binary is.
//! Why:
//!  A crate's tests run as threads of one process,
//!  so a child no test thread owns outlives the run that started it,
//!  and a signal a test aims at its own process reaches every thread of the run.
//!  The scans answer only "where",
//!  never "why the site is still there": the ledger beside them says that.
//! Gotcha:
//!  The scans are textual.
//!  A process number carried away from `getpid()` in a variable is not seen,
//!  and a child started through a helper that is not `spawn_owned` is seen at the helper's own start.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function bareChildStarts(text: string): number[]
//! export function selfSignals(text: string): number[]
//! ```

use super::source_code_places_tests::is_code;
use super::squeezed;

/// The text that starts a child process,
/// which every test reaches through `test_support::spawn_owned` or through `test_support::own_group`.
const STARTS_A_CHILD: &str = ".spawn()";

/// The text of the call that signals one process.
const SIGNALS_A_PROCESS: &str = "libc::kill(";

/// The text of the call that signals one process group.
const SIGNALS_A_GROUP: &str = "libc::killpg(";

/// The text of the call that makes a started child's process group the test thread's,
/// so the thread's fixtures end the child however the test ended.
const OWNS_THE_GROUP: &str = "own_group(";

/// How many lines are read below a start for the ownership call,
/// counting the start's own line,
/// so a start and the `own_group` line after it belong together.
const OWNERSHIP_WINDOW: usize = 4;

/// The expressions that name the process the test binary is,
/// which no signal of a test may name as its target.
const NAMES_THIS_PROCESS: &[&str] = &["getpid()", "getpgrp()", "process::id()"];

/// What:
///  `fn occurrences(text: &str, needle: &str) -> Vec<usize>` lists the byte offset of every place `needle` appears in
///  `text`.
///  `usize` is the unsigned integer wide enough to index memory (siblings: `u32`, `u64`, `i32`).
/// Why:
///  Each scan below reports where it found something,
///  and a search that started again from the first byte would report one place twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function occurrences(text: string, needle: string): number[]
/// ```
fn occurrences(
    text: &str,
    needle: &str,
) -> Vec<usize> {
    // `mut` allows the list to grow; `Vec::new()` is an empty list.
    let mut found: Vec<usize> = Vec::new();
    let mut from: usize = 0;
    // `find` gives `Some(offset inside the tail)` or `None`, and the loop ends on `None`.
    while let Some(at) = text[from..].find(needle) {
        let index: usize = from + at;
        found.push(index);
        // The next search starts after this match, so no place is counted twice.
        from = index + needle.len();
    }
    return found;
}

/// What:
///  `fn line_number(text: &str, index: usize) -> usize` is the one-based number of the line the byte at `index` sits on.
/// Why:
///  A finding names a place a person can open,
///  and the scans read whole files rather than one line at a time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lineNumber(text: string, index: number): number
/// ```
fn line_number(
    text: &str,
    index: usize,
) -> usize {
    return text[..index].matches('\n').count() + 1;
}

/// What:
///  `fn owns_the_group_below(text: &str, line: usize, code: &[bool]) -> bool` says whether `own_group(` is called as code
///  on `line` or within the next few lines.
/// Why:
///  A child that must lead a session of its own (a terminal helper calls `setsid` between `fork` and `exec`) cannot be put
///  in a group by `spawn_owned`,
///  so the test starts it and hands its number to `own_group` at once;
///  that pair is an owned child, not a finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownsTheGroupBelow(text: string, line: number, code: boolean[]): boolean
/// ```
fn owns_the_group_below(
    text: &str,
    line: usize,
    code: &[bool],
) -> bool {
    let mut offset: usize = 0;
    // `enumerate` numbers the lines, so only the window the start opened is read and the offset stays exact.
    for (number, each) in text.split_inclusive('\n').enumerate() {
        let line_number: usize = number + 1;
        if line_number >= line && line_number < line + OWNERSHIP_WINDOW {
            for at in occurrences(each, OWNS_THE_GROUP) {
                if code.get(offset + at) == Some(&true) {
                    return true;
                }
            }
        }
        if line_number >= line + OWNERSHIP_WINDOW {
            break;
        }
        offset += each.len();
    }
    return false;
}

/// What:
///  `fn first_argument(text: &str, from: usize) -> &str` is the text of the first argument of a call whose `(` the
///  caller has already stepped over.
///  `&str` is a borrowed view of the text (sibling: `String`, the owned buffer).
/// Why:
///  A signal's target is its first argument,
///  and the target is what decides whether the signal reached the test's own process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstArgument(text: string, from: number): string
/// ```
fn first_argument(
    text: &str,
    from: usize,
) -> &str {
    // The call's own `(` was stepped over before this, so the first argument is one level deep.
    let mut depth: usize = 1;
    // `char_indices` gives the byte offset of each character, which is how the slice is cut.
    for (offset, character) in text[from..].char_indices() {
        if character == ',' && depth == 1 {
            return &text[from..from + offset];
        }
        if matches!(character, '(' | '[' | '{') {
            depth += 1;
            continue;
        }
        if matches!(character, ')' | ']' | '}') {
            depth -= 1;
            if depth == 0 {
                return &text[from..from + offset];
            }
        }
    }
    return &text[from..];
}

/// What:
///  `fn names_this_process(target: &str) -> bool` says whether a signal's target names the process the test binary is.
/// Why:
///  `getpid()`, `getpgrp()`, `std::process::id()` and the literal `0` all name this process,
///  its group or its session,
///  and a signal to any of them reaches every thread of a threaded run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namesThisProcess(target: string): boolean
/// ```
fn names_this_process(target: &str) -> bool {
    // Blanks are squeezed first, so `libc :: getpid ()` is the same target as `libc::getpid()`.
    let squeezed_target: String = squeezed(target);
    if squeezed_target == "0" {
        return true;
    }
    // `iter().any` stops at the first name that appears.
    return NAMES_THIS_PROCESS
        .iter()
        .any(|name| return squeezed_target.contains(name));
}

/// What:
///  `pub(super) fn bare_child_starts(text: &str) -> Vec<usize>` lists the line numbers of the child starts that name no
///  owner.
/// Why:
///  A start is owned only through `spawn_owned` (which is not this text) or through the `own_group` call beside it,
///  so every other start is a child the test thread's fixtures will not end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bareChildStarts(text: string): number[]
/// ```
pub(super) fn bare_child_starts(text: &str) -> Vec<usize> {
    let code: Vec<bool> = is_code(text);
    let mut found: Vec<usize> = Vec::new();
    for index in occurrences(text, STARTS_A_CHILD) {
        if code.get(index) != Some(&true) {
            continue;
        }
        let line: usize = line_number(text, index);
        if owns_the_group_below(text, line, &code) {
            continue;
        }
        found.push(line);
    }
    return found;
}

/// What:
///  `pub(super) fn self_signals(text: &str) -> Vec<usize>` lists the line numbers of the signals whose target names this
///  process.
/// Why:
///  The guard's signal handlers are process-wide,
///  so a signal a test sends itself reaches every test sharing the threaded run,
///  and wherever no handler is installed at that instant the signal's default action ends the whole test binary
///  (issue 6463).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function selfSignals(text: string): number[]
/// ```
pub(super) fn self_signals(text: &str) -> Vec<usize> {
    let code: Vec<bool> = is_code(text);
    let mut found: Vec<usize> = Vec::new();
    for needle in [SIGNALS_A_PROCESS, SIGNALS_A_GROUP] {
        for index in occurrences(text, needle) {
            if code.get(index) != Some(&true) {
                continue;
            }
            let target: &str = first_argument(text, index + needle.len());
            if names_this_process(target) {
                found.push(line_number(text, index));
            }
        }
    }
    // `sort` puts the places in file order; one line may hold both calls.
    found.sort_unstable();
    found.dedup();
    return found;
}
