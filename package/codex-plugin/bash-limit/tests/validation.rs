//! Refusal paths and initialization through the native executable.

/// What:
/// `mod` includes the shared fixture module;
/// `use` imports standard-library names.
/// Why:
/// Native tests share disposable setup without using a TypeScript harness.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as support from './support';
/// import * as fs from 'node:fs';
/// ```
mod support;
/// Import file operations,
/// errors,
/// and captured subprocess output.
use std::{fs, io, process::Output};
/// Import the operating-system string type that can hold bytes that are not UTF-8.
use std::ffi::OsStr;
/// Import `from_bytes`,
/// which builds such a string from raw argument bytes.
use std::os::unix::ffi::OsStrExt;

/// What:
/// `fn` declares a function;
/// `&Output` borrows captured data from its owner.
/// Why:
/// Every denial must have a blocking exit,
/// an empty success stream,
/// and a diagnostic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function denied(output: ProcessOutput): void { ... }
/// ```
fn denied(output: &Output) {
    // What: `assert_eq!` fails a test with both values; `Some` means a present exit code.
    // Why: Codex's blocking hook contract requires status 2 rather than a generic failure.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // assert.equal(output.status, 2);
    // ```
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(!output.stderr.is_empty(), "{output:?}");
}

/// What:
/// `#[test]` registers a nextest case;
/// `Result<()>` carries success or a setup error.
/// Why:
/// CLI and JSON validation must be exercised through the executable entry point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('rejects invalid invocation', () => { ... });
/// ```
#[test]
fn rejects_invalid_invocation() -> io::Result<()> {
    // What: A borrowed list of borrowed lists allows different argument counts.
    // Why: No arguments, extra arguments, and invalid modes are distinct CLI branches.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const cases: readonly (readonly string[])[] = [ ... ];
    // ```
    let cases: &[&[&str]] = &[
        &[], &["--hook"], &["--hook", "relative"],
        &["--unknown", "/"], &["--hook", "/", "extra"],
        &["--initialize", "/nonexistent-codex-fixture/not-a-slice"],
    ];
    for arguments in cases {
        // Borrow output for the denial assertion; `?` propagates fixture process failures.
        denied(&support::invoke(arguments, "{}")?);
    }
    for input in [
        "{", "{}", "{\"hook_event_name\":\"PostToolUse\",\"tool_name\":\"Bash\"}",
        "{\"hook_event_name\":\"PreToolUse\",\"tool_name\":\"Read\"}",
        "{\"hook_event_name\":\"PreToolUse\",\"tool_name\":\"Bash\"}",
        "{\"hook_event_name\":\"PreToolUse\",\"tool_name\":\"Bash\",\"tool_input\":{\"command\":5}}",
    ] {
        // Lend arguments and captured output to the common helpers.
        denied(&support::invoke(&["--hook", "/"], input)?);
    }
    // What: `Ok(())` returns success without a payload; `Err` would return a failure.
    // Why: Passing native tests need no return data.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return;
    // ```
    return Ok(());
}

/// What:
/// `#[test]` registers a nextest case for the argument boundary;
/// `Result<()>` carries success or a setup error.
/// Why:
/// An argument whose bytes are not UTF-8 must still block the call with the documented status 2;
/// `std::env::args` panics instead,
/// and a panic exits 101,
/// which Codex reports as a failed hook run and then executes the tool anyway.
#[test]
fn refuses_arguments_that_are_not_utf8() -> io::Result<()> {
    // Build raw argument bytes, keeping the invalid sequence intact.
    let not_utf8_parent = OsStr::from_bytes(b"/no-such-fixture/\xff-worker");
    let not_utf8_mode = OsStr::from_bytes(b"--hook\xff");
    let not_utf8_extra = OsStr::from_bytes(b"\xff");
    // What: A borrowed list of borrowed lists covers each argument position independently.
    // Why: The mode, the parent path, and an extra argument are decoded in separate branches.
    let cases: &[&[&OsStr]] = &[
        &[OsStr::new("--hook"), not_utf8_parent],
        &[not_utf8_mode, OsStr::new("/")],
        &[OsStr::new("--hook"), OsStr::new("/"), not_utf8_extra],
    ];
    for arguments in cases {
        // Lend the raw argument list and a JSON event body to the native binary.
        let output = support::invoke_os(arguments, "{}")?;
        // The shared denial check proves the blocking exit and the empty success stream.
        denied(&output);
        // Decode the diagnostic, replacing invalid bytes only in the test's own reading.
        let diagnostic = String::from_utf8_lossy(&output.stderr).into_owned();
        assert!(diagnostic.starts_with("Codex worker limits: "), "{diagnostic}");
        assert!(diagnostic.contains("UTF-8"), "{diagnostic}");
    }
    return Ok(());
}

/// What:
/// `#[test]` registers a nextest case for the first refusal inside initialization;
/// `Result<()>` carries success or a setup error.
/// Why:
/// A path that does not exist and names no final component must be refused before the hook starts any slice,
/// and no other case in the suite reaches that arm.
#[test]
fn refuses_an_initialize_path_without_a_unit_name() -> io::Result<()> {
    // `..` leaves the final component absent while the path stays absolute, as the entry point requires.
    let output = support::invoke(&["--initialize", "/nonexistent-codex-fixture/.."], "")?;
    // The shared denial check proves the blocking exit and the empty success stream.
    denied(&output);
    // Read the diagnostic, whose wording belongs to the no-unit-name arm alone.
    let diagnostic = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(diagnostic.contains("has no unit name"), "{diagnostic}");
    return Ok(());
}

/// Reject changed or unreadable controls and absent or invalid membership targets.
#[test]
fn rejects_changed_controls_and_missing_membership() -> io::Result<()> {
    let input = support::event("printf 'must-not-run'")?;
    for (file, value) in [
        ("memory.max", "max"), ("memory.max", "2147483648"),
        ("memory.swap.max", "1"), ("pids.max", "max"),
        ("cpu.max", "max 100000"), ("cpu.max", "0 100000"),
        ("cpu.max", "200000 0"), ("cpu.max", ""), ("cpu.max", "200000"),
        ("cpu.max", "200000 invalid"), ("cpu.max", "1 2 3"),
    ] {
        let parent = support::fixture()?;
        // Convert the path without replacing invalid text; fixture failure should be explicit.
        let path = parent.to_str().ok_or_else(|| { return io::Error::other("fixture path encoding"); })?;
        fs::write(parent.join(file), value)?;
        // Borrow the event and output for the refusal checks.
        denied(&support::invoke(&["--hook", path], &input)?);
        fs::remove_dir_all(parent)?;
    }
    for file in ["memory.max", "cpu.max", "codex/cgroup.procs"] {
        let parent = support::fixture()?;
        // Borrow path text while this fixture remains alive.
        let path = parent.to_str().ok_or_else(|| { return io::Error::other("fixture path encoding"); })?;
        fs::remove_file(parent.join(file))?;
        // Deny the missing file before testing the non-file membership case.
        denied(&support::invoke(&["--hook", path], &input)?);
        if file == "codex/cgroup.procs" {
            fs::create_dir(parent.join(file))?;
            // A directory cannot receive a PID and must also be rejected by the hook.
            denied(&support::invoke(&["--hook", path], &input)?);
        }
        fs::remove_dir_all(parent)?;
    }
    return Ok(());
}

/// Repeated initialization creates the child and preserves an explicit finite CPU increase.
#[test]
fn initialization_preserves_controls() -> io::Result<()> {
    let parent = support::fixture()?;
    // Borrow lossless fixture text for repeated native invocations.
    let path = parent.to_str().ok_or_else(|| { return io::Error::other("fixture path encoding"); })?;
    fs::remove_dir_all(parent.join("codex"))?;
    fs::write(parent.join("cpu.max"), "400000 100000\n")?;
    for _iteration in [0, 1] {
        // Pass separate native arguments, without invoking a shell for the hook itself.
        let output = support::invoke(&["--initialize", path], "")?;
        assert!(output.status.success(), "{output:?}");
        assert_eq!(output.stdout, b"{}");
        assert!(output.stderr.is_empty());
        assert!(parent.join("codex").is_dir());
        assert_eq!(fs::read_to_string(parent.join("cpu.max"))?, "400000 100000\n");
    }
    fs::remove_dir_all(parent)?;
    return Ok(());
}
