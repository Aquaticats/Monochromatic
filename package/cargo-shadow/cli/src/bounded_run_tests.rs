//! The tests of the bounded child runs:
//! no child,
//! and no reader of its output,
//! is waited for without a bound.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use std::process::Command;
// Imports `std::time::{Duration, Instant}`.
// What:     `Duration` is a length of time and `Instant` a point on the monotonic clock (siblings: the wall-clock `SystemTime`).
// Why:      Spacing between requests is a length of time, measured on a clock that never jumps back.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: T[] / Set / Map / number
// ```
use std::time::{Duration, Instant};

// Imports `crate::bounded_run::{Bound, RunError, outpu...`.
use crate::bounded_run::{Bound, RunError, output_within};

/// A bound with words the tests look for.
fn bound(limit: Duration) -> Bound {
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    return Bound { limit, what: "the slow command".to_string(), rerun: "slow-command --again".to_string() };
}

/// A child that outlives its bound is stopped and the error names it and the command to run again.
#[test]
fn a_child_that_outlives_its_bound_is_stopped_and_named() {
    let started = Instant::now();

    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    let result = output_within(Command::new("sleep").arg("5"), None, &bound(Duration::from_millis(200)));

    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    assert!(started.elapsed() < Duration::from_secs(3), "the child was waited for: {:?}", started.elapsed());
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let message: String = result.expect_err("a timeout").to_string();
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(message.contains("the slow command") && message.contains("slow-command --again"), "{message}");
}

/// A child that finishes returns its output and status.
#[test]
fn a_child_that_finishes_returns_its_output() {
    // `None`: the present or the absent value of an `Option`.
    // Borrow `&bound`: lends the value without copying or giving it away.
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    let output = output_within(Command::new("sh").args(["-c", "echo out; echo err >&2; exit 3"]), None, &bound(Duration::from_secs(10))).expect("runs");

    // Borrow `&output.stdout`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "out");
    // Borrow `&output.stderr`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(String::from_utf8_lossy(&output.stderr).trim(), "err");
    // `Some`: the present or the absent value of an `Option`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(output.status.code(), Some(3));
}

/// A grandchild that keeps the pipes open after the child exited does not hold the caller.
#[test]
fn a_grandchild_that_keeps_the_pipes_open_does_not_hold_the_caller() {
    let started = Instant::now();

    // `None`: the present or the absent value of an `Option`.
    // Borrow `&bound`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let output = output_within(Command::new("sh").args(["-c", "sleep 5 & echo done"]), None, &bound(Duration::from_secs(10))).expect("runs");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
    // Borrow `&output.stdout`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "done");
}

/// Output larger than a pipe buffer does not deadlock,
/// and input larger than a pipe buffer is delivered.
#[test]
fn large_input_and_output_do_not_deadlock() {
    // Typed binding: `input: Vec<u8>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let input: Vec<u8> = vec![b'x'; 1_000_000];

    // `Some`: the present or the absent value of an `Option`.
    // Borrow `&mut Command`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let output = output_within(&mut Command::new("cat"), Some(&input), &bound(Duration::from_secs(20))).expect("runs");

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(output.stdout.len(), input.len());
}

/// A child that exits without reading its input is reaped,
/// and the failed write is not an error of the call.
#[test]
fn a_child_that_ignores_its_input_is_reaped() {
    // Typed binding: `input: Vec<u8>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let input: Vec<u8> = vec![b'x'; 1_000_000];

    // `Some`: the present or the absent value of an `Option`.
    // Borrow `&input`: lends the value without copying or giving it away.
    let result = output_within(Command::new("sh").args(["-c", "exit 0"]), Some(&input), &bound(Duration::from_secs(10)));

    // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
    // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Ok(value)  is  return value;
    // // Err(error) is  throw error;   (the caller sees it as a returned value)
    // ```
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(result.is_ok(), "{:?}", result.map(|output| { return output.status; }));
}

/// A program that does not exist is an I/O error naming nothing but the system's words.
#[test]
fn a_program_that_does_not_exist_is_an_io_error() {
    // `None`: the present or the absent value of an `Option`.
    // Borrow `&mut Command`: lends the value without copying or giving it away.
    let result = output_within(&mut Command::new("/nonexistent/program"), None, &bound(Duration::from_secs(1)));

    // `Err(...)`: builds the success or the failure variant of a `Result`.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(matches!(result, Err(RunError::Io(_))));
}

/// Whether a process with this id still exists.
fn alive(pid: i32) -> bool {
    // Closure `|text|`: an anonymous function, like a TS arrow function.
    // Borrow `&format`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return std::path::Path::new(&format!("/proc/{pid}")).exists() && !std::fs::read_to_string(format!("/proc/{pid}/stat")).map(|text| { return text.contains(") Z "); }).unwrap_or(true);
}

/// Waits (a few milliseconds at a time,
/// at most a second) for a process to be gone.
fn gone_soon(pid: i32) -> bool {
    for _attempt in 0..100 {
        if !alive(pid) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    return false;
}

/// Reads the process id a script wrote to a file.
fn pid_in(path: &std::path::Path) -> i32 {
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    return std::fs::read_to_string(path).expect("pid file").trim().parse().expect("a pid");
}

/// Review finding (4c F2):
/// at the bound the whole process group is stopped,
/// not only the immediate child;
/// a descendant that could
/// still write would otherwise race the rollback.
#[test]
fn a_descendant_is_stopped_with_its_parent_at_the_bound() {
    let directory = crate::test_scratch::scratch();
    let pid_file = directory.path().join("pid");
    // Macro `format!`: builds or prints text (or a list) from a template.
    let script = format!("sleep 30 & echo $! > {}; sleep 30", pid_file.display());

    // `None`: the present or the absent value of an `Option`.
    // Borrow `&script`: lends the value without copying or giving it away.
    let result = output_within(Command::new("sh").args(["-c", &script]), None, &bound(Duration::from_millis(400)));

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(result.is_err());
    // Borrow `&pid_file`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(gone_soon(pid_in(&pid_file)), "the descendant still runs");
}

/// Review finding (4c F2):
/// a descendant that outlives the parent's normal exit is stopped before the call returns,
/// and the output
/// is complete.
#[test]
fn a_descendant_left_after_a_normal_exit_is_stopped() {
    let directory = crate::test_scratch::scratch();
    let pid_file = directory.path().join("pid");
    // Macro `format!`: builds or prints text (or a list) from a template.
    let script = format!("sleep 30 & echo $! > {}; echo done", pid_file.display());

    // `None`: the present or the absent value of an `Option`.
    // Borrow `&script`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let output = output_within(Command::new("sh").args(["-c", &script]), None, &bound(Duration::from_secs(10))).expect("runs");

    // Borrow `&output.stdout`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "done");
    // Borrow `&pid_file`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(gone_soon(pid_in(&pid_file)), "the descendant still runs");
}

/// Review finding (4c F4):
/// when a process outside the group keeps the output pipe open,
/// the output cannot be complete,
/// and the call
/// says so instead of returning a prefix.
/// Round 6 (5a F1):
/// a descendant that left the process group while holding the pipes is stopped,
/// so the output is complete and nothing of the run is left running.
#[test]
fn a_descendant_in_another_session_that_holds_the_pipes_is_stopped_and_the_output_is_complete() {
    let directory = crate::test_scratch::scratch();
    let pid_file = directory.path().join("pid");
    let script = format!("setsid sh -c 'echo $$ > {}; exec sleep 30' & echo started; sleep 0.3", pid_file.display());

    let result = output_within(Command::new("sh").args(["-c", &script]), None, &bound(Duration::from_secs(10)));

    let output = result.expect("the holder of the pipe was stopped, so the output is complete");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "started");
    assert!(gone_soon(pid_in(&pid_file)), "the descendant in another session still runs");
}

/// Review finding (4c P3):
/// the error names the command the user ran,
/// not a placeholder.
#[test]
fn a_timeout_names_the_command_the_user_ran() {
    // `Some`: the present or the absent value of an `Option`.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    crate::bounded_run::set_rerun(Some("cargo build --release".to_string()));
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    let named = Bound { limit: Duration::from_millis(200), what: "`cargo update`".to_string(), rerun: crate::bounded_run::rerun_text() };

    // `None`: the present or the absent value of an `Option`.
    // Borrow `&named`: lends the value without copying or giving it away.
    let result = output_within(Command::new("sleep").arg("5"), None, &named);

    // `None`: the present or the absent value of an `Option`.
    crate::bounded_run::set_rerun(None);
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `message: String`.
    let message: String = result.expect_err("a timeout").to_string();
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(message.contains("`cargo build --release`"), "{message}");
}
