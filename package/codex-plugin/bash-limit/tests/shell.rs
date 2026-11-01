//! Native shell behavior and the attachment failure guard.

/// What:
/// `mod` includes a sibling Rust module;
/// `use` imports names from other modules.
/// Why:
/// Share fixture setup while keeping shell behavior separate from input validation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as support from './support';
/// import * as fs from 'node:fs';
/// ```
mod support;
/// Import filesystem,
/// error,
/// filename,
/// and subprocess operations.
use std::{fs, io, path::Path, process::{Command, Output}};

/// What:
/// `&Path` and `&str` borrow values;
/// `Result<String>` returns owned text or an error.
/// Why:
/// The caller retains the fixture and original text while receiving a new command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rewrite(parent: string, original: string): string { /* may throw */ }
/// ```
fn rewrite(parent: &Path, original: &str) -> io::Result<String> {
    // What: `ok_or_else` turns an absent value into an error; `?` propagates failure.
    // Why: Path encoding must remain exact, including quotes and spaces.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const path = requireUtf8(parent);
    // ```
    let path = parent.to_str().ok_or_else(|| { return io::Error::other("fixture path is not UTF-8"); })?;
    // Borrow the generated event and arguments for the native process.
    let output = support::invoke(&["--hook", path], &support::event(original)?)?;
    // What: `assert!` and `assert_eq!` are Rust's built-in test checks, failing with diagnostics.
    // Why: Missing rewrites must fail the test before any shell executes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // assert(output.status === 0);
    // ```
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    // Parse the actual process output, returning parse failures as test errors.
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(io::Error::other)?;
    assert_eq!(response["hookSpecificOutput"]["hookEventName"], "PreToolUse");
    assert_eq!(response["hookSpecificOutput"]["permissionDecision"], "allow");
    // Borrow the rewritten command from the parsed response.
    let command = response["hookSpecificOutput"]["updatedInput"]["command"].as_str()
        .ok_or_else(|| { return io::Error::other("hook omitted rewritten command"); })?;
    // What: `Ok` signals success; `to_owned` copies borrowed text into owned storage.
    // Why: The parsed response is dropped here, so the caller needs its own command string.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return command;
    // ```
    return Ok(command.to_owned());
}

/// Execute borrowed shell source with the same clean Bash launch used in the baseline.
fn shell(command: &str) -> io::Result<Output> {
    return Command::new("/bin/bash").args(["--noprofile", "--norc", "-c", command]).output();
}

/// What:
/// `#[test]` registers a native test for nextest;
/// `Result<()>` permits setup failures.
/// Why:
/// Test shell syntax and exit status at the executable consumer boundary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('preserves shell source and behavior', () => { ... });
/// ```
#[test]
fn preserves_shell_source_and_behavior() -> io::Result<()> {
    let parent = support::fixture()?;
    for original in [
        "",
        "printf '%s\\n' 'quotes: \" \\ $HOME $(untouched) `untouched` ; ../'",
        "cat <<'END'\nquotes: '\" \\ $HOME $(untouched) `untouched` ; ../\nEND\n",
        "worker_value='preserved'; worker_fn() { printf '%s' \"$worker_value\"; }; worker_fn",
        "printf 'before-exit\\n'; exit 17",
    ] {
        // Lend the fixture path and original source to the hook runner.
        let rewritten = rewrite(&parent, original)?;
        assert!(rewritten.ends_with(original));
        let expected = shell(original)?;
        // Lend the rewrite to Bash without interpreting it in the test process.
        let actual = shell(&rewritten)?;
        assert_eq!(actual.status, expected.status);
        assert_eq!(actual.stdout, expected.stdout);
        assert_eq!(actual.stderr, expected.stderr);
        let member = fs::read_to_string(parent.join("codex/cgroup.procs"))?;
        // Parse the recorded PID as an unsigned number; u32 suffices for kernel process IDs.
        assert!(member.parse::<u32>().is_ok(), "{member}");
    }
    fs::remove_dir_all(parent)?;
    return Ok(());
}

/// Register the guard check,
/// including a successful execution as its positive control.
#[test]
fn attachment_failure_prevents_payload() -> io::Result<()> {
    let parent = support::fixture()?;
    // Borrow the independent fixture for a command with visible evidence of execution.
    let rewritten = rewrite(&parent, "printf 'payload-executed'")?;
    // Execute the allowed case before breaking its destination.
    let allowed = shell(&rewritten)?;
    assert_eq!(allowed.stdout, b"payload-executed");
    fs::remove_dir_all(parent.join("codex"))?;
    // Execute the same already-generated command after the attachment target disappears.
    let blocked = shell(&rewritten)?;
    // Some is the present exit-code variant; None would mean termination by a signal.
    assert_eq!(blocked.status.code(), Some(70));
    assert!(blocked.stdout.is_empty());
    assert!(!blocked.stderr.is_empty());
    fs::remove_dir_all(parent)?;
    return Ok(());
}
