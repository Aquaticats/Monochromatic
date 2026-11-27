//! What:
//! The part of the shim tests that watches a start that cannot happen:
//! a real `mpm` whose interpreter is missing gets the full explanation and the failure status,
//! a real `mpm` whose file another process is writing is refused once,
//! with what the refusal proves and what to do,
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

/// Imports the borrowed path type the cleanup snapshot takes.
use std::path::Path;
/// Imports the output of a finished start.
use std::process::Output;
/// Imports `Instant`,
/// the point in time a timed start is measured from.
use std::time::{
    Duration,
    Instant,
};

use mpm_shadow::constants::EXEC_FAILED_EXIT_CODE;
use mpm_shadow::nesting::MAX_NESTING as NESTING_LIMIT;

use crate::{
    fixture_root,
    held_programs::{
        hold_open_for_writing,
        hold_open_until,
    },
    start_shadow,
    start_shadow_with,
    through_a_child_process::{
        assert_compiled,
        copy_program,
    },
    write_fake_mpm,
    write_stale_copies,
};

/// A compiled program that is neither a script nor a shadow.
/// The control below needs a compiled program:
/// a script's first line names an interpreter,
/// so the message names the interpreter instead of the program,
/// and a script is started by its path rather than through the open file.
const COMPILED_PROGRAM: &str = "/usr/bin/true";

/// How long the control's stand-in holds the program open for writing.
/// A start under load ends millions of times sooner than this;
/// a start that repeated the refused attempt until the retry budget ran out
/// would still be inside the hold when the stand-in let the file go,
/// so it would start the program instead of being refused again.
const HELD_FOR: Duration = Duration::from_secs(3);

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
/// Every file under `directory`,
/// as paths relative to it,
/// sorted.
/// Why:
/// The cleanup test compares the set of files before and after a run that never started,
/// so it reads the whole set rather than one path.
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

/// A start the system refuses because the real `mpm`'s file is being written says what the refusal proves,
/// names the command to run again,
/// prints no advice that cannot show the error again,
/// and leaves the cargo home exactly as it found it.
#[test]
fn a_start_refused_because_the_program_is_being_written_says_what_the_refusal_proves_and_what_to_do()
 {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\nexit 0\n");
    write_stale_copies(root.path());
    let program: std::path::PathBuf = root.path().join("real").join("mpm");
    let cargo_before: Vec<String> = relative_paths(&root.path().join("cargo"));

    let output: Output = {
        // What:      The holder lives for this block only, so the file is being written when the shadow starts
        //            and is no longer being written when the block ends.
        // Why:       Nothing else can release it early, and a panic still drops it.
        let _held = hold_open_for_writing(&program);
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
        "is open for writing by some other process",
        "the interpreter named on the program's first line",
        "cannot tell which process holds that file",
        "run the same command again",
        "126",
    ] {
        assert!(stderr.contains(expected), "{expected}: {stderr}");
    }
    assert!(
        !stderr.contains("usual cause"),
        "no measurement shows a usual cause, so the message names none: {stderr}"
    );
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

/// What:
/// A start of a compiled program whose file another process holds open for writing is refused once,
/// inside the time the holder states,
/// with the words the unit test of the message also pins.
/// Why:
/// The refused start of a compiled program runs through the open file (`/proc/self/fd`),
/// which no other test or unit test reaches,
/// and a start that repeated the attempt would still be holding the file when the holder let it go,
/// so it would start the program instead of being refused again.
#[test]
fn a_refused_start_of_a_compiled_program_is_refused_once_inside_the_hold_with_the_words() {
    let root = fixture_root();
    let program: std::path::PathBuf = root.path().join("real").join("mpm");
    // What:     The program is copied by a child process, so this process holds no write descriptor on it.
    // Why:      A descriptor here would be inherited by a child another test thread forks, and that
    //           child's own start would then be refused (issue 6527).
    copy_program(Path::new(COMPILED_PROGRAM), &program);
    assert_compiled(&program);

    let mut holder = hold_open_until(&program, HELD_FOR);
    let started: Instant = Instant::now();
    let output: Output = start_shadow(root.path(), &["installed"]);
    let elapsed: Duration = started.elapsed();

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(
        output.status.code(),
        Some(EXEC_FAILED_EXIT_CODE),
        "{stderr}"
    );
    for expected in [
        "mpm installed",
        "Text file busy",
        "is open for writing by some other process",
        "the interpreter named on the program's first line",
        "cannot tell which process holds that file",
        "run the same command again",
        "writes a whole new copy into place",
    ] {
        assert!(stderr.contains(expected), "{expected}: {stderr}");
    }
    assert!(
        !stderr.contains("usual cause"),
        "no measurement shows a usual cause, so the message names none: {stderr}"
    );
    assert!(
        !stderr.contains("by hand"),
        "the advice names a step that shows nothing once the file is closed: {stderr}"
    );
    assert_eq!(
        stderr.matches("Text file busy").count(),
        1,
        "the refusal is reported once, not once per attempt: {stderr}"
    );
    assert!(
        elapsed < HELD_FOR,
        "the start ended {elapsed:?} after it began, and the holder states it keeps the file open for {HELD_FOR:?}: a start that waited for the write to end would end after the hold"
    );
    assert!(
        holder.still_holding(),
        "the start waited for the holder to release the file instead of giving up at once"
    );
}
