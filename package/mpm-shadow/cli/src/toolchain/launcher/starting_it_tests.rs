//! What:
//! Unit tests of the two lines a launcher prints for a start the system answered with "no such file
//! or directory".
//! Why:
//! The file those lines name is read from the program's own bytes,
//! and the case where the program itself is gone cannot be reached by a test that starts a process:
//! nothing of a missing file can say which interpreter it needed,
//! so those words are pinned here.
//! `tests/launcher.rs` runs the two cases a fixture can reach through the built binary.

/// Imports `fs`,
/// which writes the fixture programs.
use std::fs;
/// Imports the error a refused start carries,
/// and the kind these tests choose it by.
use std::io::{
    Error,
    ErrorKind,
};
/// Imports the borrowed and the owned path types.
use std::path::{
    Path,
    PathBuf,
};

/// Imports the hand-built compiled program's bytes and their 64-bit layout.
use crate::program_bytes::{
    WIDE,
    elf,
};
/// Imports the scratch-directory helper (see `test_scratch.rs`).
use crate::test_scratch::scratch;
/// Imports which scope made a launcher,
/// which the remedy's one command is chosen by.
use crate::toolchain::scope::ScopeKind;

/// Imports the record the message reads the tree and the scope from.
use super::super::LaunchRecord;
/// Imports the two lines under test.
use super::missing_lines;

/// The tool the fixtures install.
const TOOL_ID: &str = "cargo:monochromatic-demo-tool";

/// What:
/// `fn record_in(tree, program) -> LaunchRecord` builds the record of a launcher of `program` in the
/// tree at `tree`.
/// Why:
/// The remedy names this tree's one command,
/// so the record has to name a tree.
fn record_in(
    tree: &Path,
    program: &Path,
) -> LaunchRecord {
    return LaunchRecord {
        tool_id: TOOL_ID.to_string(),
        lock_file: tree.join("toolchain.lock.json"),
        real: program.to_path_buf(),
        scope: ScopeKind::Repo,
    };
}

/// The one command this tree's remedy names.
fn repair_command_in(tree: &Path) -> String {
    return format!(
        "run `mpm install` in {} to make this tree's links again",
        tree.display()
    );
}

/// What:
/// A script whose own first line names an interpreter that is not there is named by that interpreter,
/// and the remedy installs that file.
/// Why:
/// `execve` answers one error for the program and for the interpreter,
/// and the script's first line says which one this start looked for.
#[test]
fn a_script_whose_interpreter_is_gone_names_that_interpreter() {
    let directory: tempfile::TempDir = scratch();
    let program: PathBuf = directory.path().join("tool");
    fs::write(&program, "#!/nonexistent/interpreter\nexit 0\n").expect("write the fixture script");
    let record: LaunchRecord = record_in(directory.path(), &program);

    let (cause, remedy) = missing_lines(&record, &program, &Error::from(ErrorKind::NotFound));

    for expected in [
        "the interpreter its first line names (`/nonexistent/interpreter`) is not there",
        "the program itself was there when this launcher checked the path",
    ] {
        assert!(cause.contains(expected), "{expected}: {cause}");
    }
    for expected in [
        "install the interpreter its first line names (`/nonexistent/interpreter`) if that is the file that is missing",
        &repair_command_in(directory.path()),
    ] {
        assert!(remedy.contains(expected), "{expected}: {remedy}");
    }
}

/// What:
/// A compiled program whose own file names a dynamic linker that is not there is named by that linker,
/// and the remedy installs the package that provides it.
/// Why:
/// A compiled program has no first line,
/// so a sentence about one is a sentence about a file the program does not have.
#[test]
fn a_compiled_program_whose_linker_is_gone_names_that_linker() {
    let directory: tempfile::TempDir = scratch();
    let program: PathBuf = directory.path().join("tool");
    fs::write(
        &program,
        elf(&WIDE, false, Some("/nowhere/ld-fixture.so\0")),
    )
    .expect("write the fixture program");
    let record: LaunchRecord = record_in(directory.path(), &program);

    let (cause, remedy) = missing_lines(&record, &program, &Error::from(ErrorKind::NotFound));

    for expected in [
        "its dynamic linker (`/nowhere/ld-fixture.so`) is not there",
        "a compiled program cannot start without it",
        "the program itself was there when this launcher checked the path",
    ] {
        assert!(cause.contains(expected), "{expected}: {cause}");
    }
    assert!(
        !cause.contains("first line"),
        "a compiled program has no first line: {cause}"
    );
    for expected in [
        "install or upgrade the package that provides `/nowhere/ld-fixture.so` if that linker is the file that is missing",
        &repair_command_in(directory.path()),
    ] {
        assert!(remedy.contains(expected), "{expected}: {remedy}");
    }
}

/// What:
/// A program whose own bytes are gone keeps both kinds of interpreter and claims neither,
/// and the remedy names this tree's command before installing anything again.
/// Why:
/// Nothing of a missing file can say which interpreter it needed,
/// and a message that named one kind would send half its readers to a file that is not the one missing.
#[test]
fn a_program_that_is_gone_keeps_both_kinds_of_interpreter_and_claims_neither() {
    let directory: tempfile::TempDir = scratch();
    let program: PathBuf = directory.path().join("gone");
    let record: LaunchRecord = record_in(directory.path(), &program);

    let (cause, remedy) = missing_lines(&record, &program, &Error::from(ErrorKind::NotFound));

    for expected in [
        "the program is gone, or the interpreter it names is",
        "the program after `#!` on a script's first line, or a compiled program's dynamic linker",
        "this launcher cannot say which of the two files is missing",
        "the program itself was there when this launcher checked the path",
    ] {
        assert!(cause.contains(expected), "{expected}: {cause}");
    }
    assert!(
        !cause.contains("(`/"),
        "no file's path is claimed when the program's bytes are gone: {cause}"
    );
    for expected in [
        &repair_command_in(directory.path()),
        "if that does not put the program back, install it again",
    ] {
        assert!(remedy.contains(expected), "{expected}: {remedy}");
    }
}
