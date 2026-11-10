//! Disposable files and native process helpers shared by integration tests.

/// What:
/// `use` imports names;
/// `::` separates module names.
/// Why:
/// Tests need files,
/// paths,
/// errors,
/// and child processes without a test framework dependency.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as fs from 'node:fs';
/// import { spawnSync } from 'node:child_process';
/// ```
use std::{ffi::OsStr, fs, io, io::Write, path::{Path, PathBuf}, process::{Command, Output, Stdio}};
/// Import the trait that adds `from_mode` to `Permissions`.
use std::os::unix::fs::PermissionsExt;

/// What:
/// `pub pub const OWNER_ONLY_MODE: u32 = 0o700;` is the mode rule ATH requires for the shared scratch root:
/// read,
/// write and search for the owner and nothing for anyone else.
/// Why:
/// The fixture sets it and `tests/fixture.rs` asserts it,
/// both from this one value,
/// so a change to the requirement is one edit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const OWNER_ONLY_MODE = 0o700;
/// ```
pub const OWNER_ONLY_MODE: u32 = 0o700;

/// What:
/// `pub fn` exports a function;
/// `Result<PathBuf>` carries a path or an I/O error.
/// `PathBuf` owns its filename;
/// sibling `&Path` borrows an existing filename.
/// Why:
/// Each test needs its own directory that remains usable after this function returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function fixture(): string { /* may throw */ }
/// ```
pub fn fixture() -> io::Result<PathBuf> {
    // What: `map_err` converts an error; `?` returns early when an operation fails.
    // Why: A missing injected home must fail instead of touching an unintended directory.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const home = requireEnv('HOME');
    // ```
    let home = std::env::var("HOME").map_err(io::Error::other)?;
    return fixture_in(Path::new(&home));
}

/// What:
/// `pub fn fixture_in(home: &Path) -> io::Result<PathBuf>` declares the fixture builder for one home directory,
/// borrowed so a test can pass a disposable home instead of the process's own.
/// Why:
/// The scratch root's mode only shows up when the directory does not exist yet,
/// which is exactly the injected-home case,
/// and a test that changes `HOME` for the whole process would race every sibling test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function fixtureIn(home: string): string { ... }
/// ```
pub fn fixture_in(home: &Path) -> io::Result<PathBuf> {
    // Own the scratch path so tests never depend on a particular username.
    let scratch = home.join("temp/agent");
    // What: `&scratch` lends this value without transferring ownership.
    // Why: Directory creation and mktemp both need the same filename.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // mkdirSync(scratch, { recursive: true });
    // ```
    fs::create_dir_all(&scratch)?;
    // What: `set_permissions` replaces the mode `create_dir_all` left behind, and `from_mode` reads the octal
    // literal. `?` returns the error when the directory cannot be changed.
    // Why: `create_dir_all` applies the process umask (typically `0755`), and rule ATH requires the shared scratch
    // root to be `0700` because trust checks reject group and other bits. Setting it after creation also corrects a
    // directory an earlier run left looser, which a creation-time mode cannot do.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // chmodSync(scratch, 0o700);
    // ```
    fs::set_permissions(&scratch, fs::Permissions::from_mode(OWNER_ONLY_MODE))?;
    // Request a private unique directory; its quote and space exercise shell escaping.
    let output = Command::new("mktemp")
        .arg("--directory")
        .arg(scratch.join("codex-'quoted path'-XXXXXXXX"))
        .output()?;
    if !output.status.success() {
        // What: `Err` signals failure; sibling `Ok` signals success.
        // Why: A failed fixture setup must not become an empty or shared test path.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // throw new Error('mktemp could not create the fixture');
        // ```
        // Decode stderr for the failure report, replacing only invalid diagnostic bytes.
        return Err(io::Error::other(String::from_utf8_lossy(&output.stderr).into_owned()));
    }
    // Decode mktemp's bytes without substituting invalid characters.
    let text = String::from_utf8(output.stdout).map_err(io::Error::other)?;
    // Own the filename after removing mktemp's final newline.
    let parent = PathBuf::from(text.trim_end());
    fs::create_dir(parent.join("codex"))?;
    for (file, value) in [
        ("memory.max", "4294967296\n"), ("memory.swap.max", "0\n"),
        ("pids.max", "2048\n"), ("cpu.max", "200000 100000\n"),
        ("codex/cgroup.procs", ""),
    ] {
        fs::write(parent.join(file), value)?;
    }
    // Return the successfully constructed independent directory.
    return Ok(parent);
}

/// Borrow an argument list and stdin text,
/// returning the actual binary's captured output.
/// `&[&str]` borrows a variable-size list;
/// `[T; N]` fixes its length and `Vec<T>` owns it.
/// Borrowing avoids copying caller-owned command arguments.
pub fn invoke(arguments: &[&str], input: &str) -> io::Result<Output> {
    // What: `map` borrows each `&str` as an `&OsStr` without allocating the text.
    // Why: One launch path serves both callers, so plain-text and raw-byte arguments cannot drift apart.
    let os_arguments: Vec<&OsStr> = arguments.iter().map(|argument| { return OsStr::new(argument); }).collect();
    return invoke_os(&os_arguments, input);
}

/// Borrow an argument list of raw operating-system strings and stdin text,
/// returning the actual binary's captured output.
/// Why:
/// An argument that is not UTF-8 has no `&str` spelling,
/// so refusing one needs its own entry point into the same launch.
pub fn invoke_os(arguments: &[&OsStr], input: &str) -> io::Result<Output> {
    // What: `env!` embeds Cargo's executable path at compile time; `mut` permits writing stdin.
    // Why: Tests exercise this crate's executable, including entry-point validation and exit codes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const child = spawn(builtBinary, args, { stdio: 'pipe' });
    // ```
    let mut child = Command::new(env!("CARGO_BIN_EXE_codex-bash-limit"))
        .args(arguments).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    // Borrow stdin temporarily; an absent pipe is an explicit fixture failure.
    let pipe = child.stdin.as_mut().ok_or_else(|| {
        // Return the error from this zero-argument closure (`||`).
        return io::Error::other("native hook stdin pipe missing");
    })?;
    // Convert borrowed text to borrowed UTF-8 bytes for the pipe.
    pipe.write_all(input.as_bytes())?;
    // wait_with_output closes stdin before waiting, allowing read-to-end in the hook.
    return child.wait_with_output();
}

/// Encode a command event through serde_json,
/// borrowing the original shell text.
pub fn event(command: &str) -> io::Result<String> {
    // Serialize the command at the JSON boundary, preserving every shell character.
    let encoded = serde_json::to_string(command).map_err(io::Error::other)?;
    // Own a mutable string; `String` can grow while sibling `&str` only borrows text.
    let mut input = String::from("{\"hook_event_name\":\"PreToolUse\",\"tool_name\":\"Bash\",\"tool_input\":{\"command\":");
    // Lend the encoded JSON string to the destination buffer.
    input.push_str(&encoded);
    input.push_str("}}");
    return Ok(input);
}
