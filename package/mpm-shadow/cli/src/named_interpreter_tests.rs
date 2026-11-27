//! What:
//! Unit tests of the reader that says which interpreter a program names.
//! Why:
//! Two sentences send a person to that file,
//! and the file's own bytes are the only evidence for which one it is:
//! a script names it on the first line,
//! a compiled program names it in an entry of its own program-header table.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('namedInterpreter', () => { it('reads a shebang', () => { ... }); });
//! ```

/// Imports the reader and its answers from the module under test.
use crate::named_interpreter::{
    MissingInterpreter,
    NamedInterpreter,
    missing_interpreter,
    named_interpreter,
};
/// Imports the hand-built compiled program's bytes and their two layouts.
use crate::program_bytes::{
    NARROW,
    WIDE,
    elf,
};
/// Imports the scratch-directory helper (see `test_scratch.rs`).
use crate::test_scratch::scratch;
/// Imports the file-system module (writes the fixture program files).
use std::fs;
/// Imports the borrowed path type.
use std::path::Path;

/// A script names the program after `#!` as its interpreter.
#[test]
fn a_script_names_the_program_after_the_hash_and_bang() {
    let directory = scratch();
    let program: std::path::PathBuf = directory.path().join("mpm");
    fs::write(&program, "#!/bin/sh\necho hi\n").expect("write the fixture script");
    match named_interpreter(&program) {
        NamedInterpreter::Script {
            interpreter,
        } => assert_eq!(interpreter, Path::new("/bin/sh")),
        other => panic!("a script names its first line's program: {other:?}"),
    }
}

/// The rest of the first line is the interpreter's arguments,
/// so only the first word is the program.
#[test]
fn a_script_line_with_arguments_still_names_only_the_program() {
    let directory = scratch();
    let program: std::path::PathBuf = directory.path().join("mpm");
    fs::write(&program, "#!/usr/bin/env bash\n").expect("write the fixture script");
    match named_interpreter(&program) {
        NamedInterpreter::Script {
            interpreter,
        } => assert_eq!(interpreter, Path::new("/usr/bin/env")),
        other => panic!("the kernel starts the first word of the line: {other:?}"),
    }
}

/// A first line that names no program is not a script this code can read.
#[test]
fn a_script_line_with_no_program_cannot_be_read_as_a_program() {
    let directory = scratch();
    let program: std::path::PathBuf = directory.path().join("mpm");
    fs::write(&program, "#!\n").expect("write the fixture script");
    assert!(
        matches!(named_interpreter(&program), NamedInterpreter::Unknown),
        "an empty interpreter path is no interpreter"
    );
}

/// A 64-bit little-endian program names the dynamic linker its entry holds.
#[test]
fn a_wide_little_endian_program_names_the_linker_its_entry_holds() {
    let directory = scratch();
    let program: std::path::PathBuf = directory.path().join("mpm");
    fs::write(&program, elf(&WIDE, false, Some("/lib64/ld-fixture.so\0")))
        .expect("write the fixture program");
    match named_interpreter(&program) {
        NamedInterpreter::DynamicLinker {
            linker,
        } => assert_eq!(linker, Path::new("/lib64/ld-fixture.so")),
        other => panic!("the entry's payload is the linker: {other:?}"),
    }
}

/// The 32-bit,
/// most-significant-byte-first layout carries the same answer.
#[test]
fn a_narrow_big_endian_program_names_its_linker_too() {
    let directory = scratch();
    let program: std::path::PathBuf = directory.path().join("mpm");
    fs::write(&program, elf(&NARROW, true, Some("/lib/ld-fixture.so\0")))
        .expect("write the fixture program");
    match named_interpreter(&program) {
        NamedInterpreter::DynamicLinker {
            linker,
        } => assert_eq!(linker, Path::new("/lib/ld-fixture.so")),
        other => panic!("the 32-bit layout names the same entry: {other:?}"),
    }
}

/// A compiled program with no interpreter entry names no interpreter,
/// so its own file is the only file it needs.
#[test]
fn a_program_with_no_interpreter_entry_names_none() {
    let directory = scratch();
    let program: std::path::PathBuf = directory.path().join("mpm");
    fs::write(&program, elf(&WIDE, false, None)).expect("write the fixture program");
    assert!(
        matches!(named_interpreter(&program), NamedInterpreter::NoInterpreter),
        "no interpreter entry means no interpreter to name"
    );
}

/// A text file that is not a script is not a program this code can read either.
#[test]
fn a_text_file_that_is_not_a_script_cannot_be_read_as_a_program() {
    let directory = scratch();
    let program: std::path::PathBuf = directory.path().join("mpm");
    fs::write(&program, "this is not a program\n").expect("write the fixture file");
    assert!(matches!(
        named_interpreter(&program),
        NamedInterpreter::Unknown
    ));
}

/// A program file cut inside its own header cannot be read as a program.
#[test]
fn a_file_cut_inside_its_header_cannot_be_read_as_a_program() {
    let directory = scratch();
    let program: std::path::PathBuf = directory.path().join("mpm");
    fs::write(&program, [0x7f, b'E', b'L', b'F']).expect("write the cut file");
    assert!(matches!(
        named_interpreter(&program),
        NamedInterpreter::Unknown
    ));
}

/// A path that holds nothing answers nothing,
/// which is the case a vanished program makes.
#[test]
fn a_path_that_is_not_there_cannot_be_read() {
    let directory = scratch();
    assert!(matches!(
        named_interpreter(&directory.path().join("gone")),
        NamedInterpreter::Unknown
    ));
}

/// The running test program is a compiled program on this machine,
/// and the answer for it must be one of the two a compiled program can give.
#[test]
fn a_real_compiled_program_answers_as_a_compiled_program() {
    let executable: std::path::PathBuf =
        std::env::current_exe().expect("the test program's own path");
    let head: Vec<u8> = fs::read(&executable).expect("read the test program");
    if !head.starts_with(&[0x7f, b'E', b'L', b'F']) {
        return;
    }
    assert!(
        matches!(
            named_interpreter(&executable),
            NamedInterpreter::DynamicLinker { .. } | NamedInterpreter::NoInterpreter
        ),
        "a compiled program names a linker or none, never a first line"
    );
}

/// A script whose own first line names an interpreter that is not there is the case the message has to
/// name,
/// so the answer carries that interpreter.
#[test]
fn a_script_whose_interpreter_is_not_there_answers_with_that_interpreter() {
    let directory = scratch();
    let program: std::path::PathBuf = directory.path().join("mpm");
    fs::write(&program, "#!/nonexistent/interpreter\nexit 0\n").expect("write the fixture script");
    match missing_interpreter(&program) {
        MissingInterpreter::Script {
            interpreter,
        } => assert_eq!(interpreter, Path::new("/nonexistent/interpreter")),
        other => panic!("the interpreter the first line names is missing: {other:?}"),
    }
}

/// A script whose interpreter is there leaves nothing for the message to name.
#[test]
fn a_script_whose_interpreter_is_there_says_nothing() {
    let directory = scratch();
    let program: std::path::PathBuf = directory.path().join("mpm");
    fs::write(&program, "#!/bin/sh\nexit 0\n").expect("write the fixture script");
    assert!(matches!(
        missing_interpreter(&program),
        MissingInterpreter::SaysNothing
    ));
}

/// A compiled program whose dynamic linker is not there is the other case the message names.
#[test]
fn a_compiled_program_whose_linker_is_not_there_answers_with_that_linker() {
    let directory = scratch();
    let program: std::path::PathBuf = directory.path().join("mpm");
    fs::write(
        &program,
        elf(&WIDE, false, Some("/nowhere/ld-fixture.so\0")),
    )
    .expect("write the fixture program");
    match missing_interpreter(&program) {
        MissingInterpreter::DynamicLinker {
            linker,
        } => assert_eq!(linker, Path::new("/nowhere/ld-fixture.so")),
        other => panic!("the linker the program's own file names is missing: {other:?}"),
    }
}

/// A compiled program whose linker is there leaves nothing for the message to name either.
#[test]
fn a_compiled_program_whose_linker_is_there_says_nothing() {
    let directory = scratch();
    let program: std::path::PathBuf = directory.path().join("mpm");
    fs::write(&program, elf(&WIDE, false, Some("/bin/sh\0"))).expect("write the fixture program");
    assert!(matches!(
        missing_interpreter(&program),
        MissingInterpreter::SaysNothing
    ));
}

/// A program that is gone says nothing,
/// which is the case that keeps both kinds of interpreter in the message.
#[test]
fn a_program_that_is_gone_says_nothing() {
    let directory = scratch();
    assert!(matches!(
        missing_interpreter(&directory.path().join("gone")),
        MissingInterpreter::SaysNothing
    ));
}
