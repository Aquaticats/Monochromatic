//! What:
//! Round 6 tests of the command line that errors tell the person to run again.
//! Why:
//! Fifth review 5c N5: joining arguments with spaces loses their boundaries.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('rerunCommand', () => { it(...) })
//! ```

use std::ffi::OsString;

use crate::shim::rerun_command;

/// Turns text into the arguments of a command.
fn arguments(texts: &[&str]) -> Vec<OsString> {
    return texts.iter().map(OsString::from).collect();
}

/// Round 6 (5c N5): an argument with a space stays one argument.
#[test]
fn an_argument_with_a_space_is_quoted() {
    let line = rerun_command(&arguments(&["build", "--manifest-path", "project with spaces/Cargo.toml"]));

    assert_eq!(line, "cargo build --manifest-path 'project with spaces/Cargo.toml'");
}

/// Round 6 (5c N5): quotes and shell characters cannot end the quoting early.
#[test]
fn quotes_and_shell_characters_are_escaped() {
    let line = rerun_command(&arguments(&["run", "--", "it's; rm -rf /"]));

    assert_eq!(line, "cargo run -- 'it'\\''s; rm -rf /'");
}

/// Round 6 (5c N5): plain arguments are left alone.
#[test]
fn plain_arguments_are_not_quoted() {
    let line = rerun_command(&arguments(&["build", "--release", "-p", "a-b_c.d", "--features=x,y"]));

    assert_eq!(line, "cargo build --release -p a-b_c.d --features=x,y");
}

/// Round 6 (5c N5): an empty argument is still an argument.
#[test]
fn an_empty_argument_is_written_as_empty_quotes() {
    assert_eq!(rerun_command(&arguments(&["run", ""])), "cargo run ''");
}
