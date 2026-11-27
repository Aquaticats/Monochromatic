//! What:
//! The part of the shim tests that watches a start that cannot happen:
//! a real `mpm` whose interpreter is missing gets the full explanation and the failure status,
//! a real `mpm` whose file another process is writing is explained as a replacement in progress,
//! the program the shadow starts learns how deep the nesting is,
//! and at the bound the shadow starts nothing and says why.
//! Why:
//! A person reading the output must learn which program failed,
//! at which stage,
//! and what to do,
//! and a nesting that would run the shadow for ever has to stop before it starts anything:
//! the marker file is what proves nothing started.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shim/a_start_that_fails_and_the_nesting_bound.test.ts
//! test('a real mpm that cannot start is explained', () => { ... });
//! ```

/// What:
/// Imports `BufRead`,
/// the trait that adds `read_line` to the pipe the holder reports through,
/// and the buffer reader built over it.
/// Why:
/// The test waits for the holder to say the file is open before it starts the shadow,
/// so a start can never run before the file is being written.
use std::io::{
    BufRead,
    BufReader,
};
/// Imports the borrowed path type the holder takes.
use std::path::Path;
/// Imports the child process,
/// the command that starts it,
/// the standard streams it gets,
/// and the pipe into it that ends its hold when it is dropped.
use std::process::{
    Child,
    ChildStdin,
    Command,
    Output,
    Stdio,
};

use mpm_shadow::constants::EXEC_FAILED_EXIT_CODE;
use mpm_shadow::nesting::MAX_NESTING as NESTING_LIMIT;

use crate::{
    fixture_root,
    start_shadow,
    start_shadow_with,
    write_fake_mpm,
    write_stale_copies,
};

/// A real mpm that cannot be started (its interpreter is missing) gets the full explanation,
/// the exit status 126,
/// and no cleanup.
#[test]
fn a_real_mpm_that_cannot_start_is_explained_with_command_program_stage_cause_and_status() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/nonexistent/interpreter\n");
    write_stale_copies(root.path());
    let program: String = root.path().join("real").join("mpm").display().to_string();

    let output: Output = start_shadow(root.path(), &["installed"]);

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(
        output.status.code(),
        Some(EXEC_FAILED_EXIT_CODE),
        "{stderr}"
    );
    for expected in [
        "mpm installed",
        &program,
        "starting the real `mpm`",
        "126",
        "could not be started",
        "post-run cleanup did not run",
    ] {
        assert!(stderr.contains(expected), "{expected}: {stderr}");
    }
    assert!(!stderr.contains("deleted stale"), "{stderr}");
    assert!(
        root.path()
            .join("cargo/registry/cache/index.crates.io-fixture/monochromatic-lint-0.1.0.crate")
            .exists(),
        "no cleanup ran"
    );
}

/// The program the shadow starts learns how deep it is.
#[test]
fn the_program_it_starts_gets_a_nesting_depth_one_deeper_than_its_own() {
    let root = fixture_root();
    write_fake_mpm(
        &root.path().join("real"),
        "#!/bin/sh\nprintf '%s' \"$MPM_SHADOW_NESTING\"\n",
    );

    let top: Output = start_shadow(root.path(), &[]);
    assert_eq!(String::from_utf8_lossy(&top.stdout), "1");

    let deeper: Output = start_shadow_with(root.path(), &[], &[("MPM_SHADOW_NESTING", "2")]);
    assert_eq!(String::from_utf8_lossy(&deeper.stdout), "3");
}

/// At the bound the shadow starts nothing and says why.
#[test]
fn a_shadow_nested_to_the_limit_refuses_to_start_another_program() {
    let root = fixture_root();
    let marker: std::path::PathBuf = root.path().join("ran");
    write_fake_mpm(
        &root.path().join("real"),
        &format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
    );

    let output: Output = start_shadow_with(
        root.path(),
        &["installed"],
        &[("MPM_SHADOW_NESTING", &NESTING_LIMIT.to_string())],
    );

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(
        output.status.code(),
        Some(EXEC_FAILED_EXIT_CODE),
        "{stderr}"
    );
    assert!(!marker.exists(), "the program must not have started");
    for expected in [
        "mpm installed",
        "started by another mpm-shadow",
        "itself",
        "126",
    ] {
        assert!(stderr.contains(expected), "{expected}: {stderr}");
    }
}

/// What:
/// A child process that holds one file open for writing,
/// released when this value is dropped.
/// Why:
/// `execve` answers `Text file busy` for a file that any process holds open for writing,
/// so the refusal needs a holder,
/// and the holder has to be another process:
/// a write descriptor in the test process is inherited by every child another test thread forks
/// (issue 6527),
/// and it would then be the holder of a start that is meant to succeed.
struct HeldOpenForWriting {
    /// The `sh` process whose descriptor number three is the file open for writing.
    holder: Child,
}

/// What:
/// `impl Drop for HeldOpenForWriting` closes the pipe into the holder and waits for it to end.
/// Why:
/// Closing the pipe is how the holder learns its work is over,
/// and waiting for it keeps it from outliving the test that started it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holder.stdin.end(); holder.waitSync();
/// ```
impl Drop for HeldOpenForWriting {
    /// Ends the hold and reaps the holder.
    fn drop(&mut self) {
        // What:     `.stdin.take()` moves the pipe out of the child record and drops it here.
        // Why:      Dropping the last handle on the pipe gives the holder's `cat` its end of file,
        //           which is the only thing it waits for.
        let pipe: Option<ChildStdin> = self.holder.stdin.take();
        drop(pipe);
        let _ = self.holder.wait();
    }
}

/// What:
/// `fn hold_open_for_writing(path) -> HeldOpenForWriting` starts a `sh`
/// that opens `path` for appending and keeps it open until the returned value is dropped.
/// Why:
/// This is the one write a start can meet:
/// the system refuses to start a file that is being written,
/// and the shadow must say what that means.
/// The holder appends nothing,
/// so the program in the file stays the one the test wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holdOpenForWriting(path: string): Held {
///   const child = spawn('sh', ['-c', 'exec 3>>"$1"; printf "held\n"; cat > /dev/null', 'sh', path], { stdio: ['pipe', 'pipe', 'inherit'] });
///   readOneLine(child.stdout); return { child };
/// }
/// ```
fn hold_open_for_writing(path: &Path) -> HeldOpenForWriting {
    // What:     `Command::new("/bin/sh")` starts a shell with three arguments after the script:
    //           `sh` (its own `$0`) then `path` (its `$1`).
    // Why:      `exec 3>>"$1"` opens the file for appending on descriptor three of the shell itself,
    //           so the descriptor belongs to a process of the test's own and never to the test.
    let mut holder: Child = Command::new("/bin/sh")
        .arg("-c")
        .arg("exec 3>>\"$1\" || exit 9; printf 'held\\n'; cat > /dev/null")
        .arg("sh")
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("start the holder of the program file");
    // What:     `.stdout.take()` moves the reporting pipe out of the child record.
    // Why:      The line `held` is the proof the file is open before the shadow starts,
    //           so the test never races the holder.
    let reporting = holder.stdout.take().expect("the holder's report");
    let mut reader: BufReader<std::process::ChildStdout> = BufReader::new(reporting);
    let mut line: String = String::new();
    reader.read_line(&mut line).expect("read the holder's report");
    assert_eq!(line, "held\n", "the holder did not report the open file");
    return HeldOpenForWriting {
        holder,
    };
}

/// Every file under `directory`, as paths relative to it, sorted.
fn relative_paths(directory: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut pending: Vec<std::path::PathBuf> = vec![directory.to_path_buf()];
    while let Some(next) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&next) else {
            continue;
        };
        for entry in entries.flatten() {
            let path: std::path::PathBuf = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let relative: String = path
                .strip_prefix(directory)
                .expect("a path below the root")
                .display()
                .to_string();
            found.push(relative);
        }
    }
    found.sort();
    return found;
}

/// A start the system refuses because the real `mpm`'s file is being written says the file is being replaced,
/// names the command to run again,
/// prints no advice that cannot show the error again,
/// and leaves the cargo home exactly as it found it.
#[test]
fn a_start_refused_because_the_program_is_being_written_says_the_file_is_being_replaced() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\nexit 0\n");
    write_stale_copies(root.path());
    let program: std::path::PathBuf = root.path().join("real").join("mpm");
    let cargo_before: Vec<String> = relative_paths(&root.path().join("cargo"));

    let output: Output = {
        // What:      The holder lives for this block only, so the file is being written when the shadow starts
        //            and is no longer being written when the block ends.
        // Why:       Nothing else can release it early, and a panic still drops it.
        let _held: HeldOpenForWriting = hold_open_for_writing(&program);
        start_shadow(root.path(), &["installed"])
    };

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(
        output.status.code(),
        Some(EXEC_FAILED_EXIT_CODE),
        "{stderr}"
    );
    assert!(stderr.contains("Text file busy"), "{stderr}");
    for expected in [
        "mpm installed",
        "is being written by another program",
        "run the same command again",
        "126",
    ] {
        assert!(stderr.contains(expected), "{expected}: {stderr}");
    }
    assert!(
        !stderr.contains("by hand to see the same error"),
        "the advice names a step that shows nothing once the file is closed: {stderr}"
    );
    assert!(!stderr.contains("deleted stale"), "{stderr}");
    assert_eq!(
        relative_paths(&root.path().join("cargo")),
        cargo_before,
        "the skipped cleanup left nothing behind and deleted nothing"
    );
}
