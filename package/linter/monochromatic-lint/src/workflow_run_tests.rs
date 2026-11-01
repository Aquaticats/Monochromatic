//! What:
//!  Controls for the `run:` and `shell:` value readers.
//! Why:
//!  A body read short hides lines from the rules,
//!  and a form read loosely lets a script escape them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('workflow run values', () => { /* plain, block, refused forms, shell */ });
//! ```

/// Import the readers under test.
use super::{capture_run, capture_shell};
use crate::workflow_lines::{Content, parse_entry, read_lines};
use crate::workflow_model::{BodyLine, RunBody, Shell, WalkError};

/// Read the first line's key as a `run:` and return the body and the next line index.
fn run_of(text: &str) -> Result<(RunBody, usize), WalkError> {
    let lines = read_lines(text).expect("lines");
    let entry = parse_entry(&lines[0]);
    let Content::Key {
        column,
        value,
        value_offset,
        ..
    } = entry.content
    else {
        panic!("fixture has no key");
    };
    return capture_run(&lines, 0, column, value, value_offset);
}

/// Read the first line's key as a `shell:`.
fn shell_of(text: &str) -> Result<Shell, WalkError> {
    let lines = read_lines(text).expect("lines");
    let entry = parse_entry(&lines[0]);
    let Content::Key {
        value,
        value_offset,
        ..
    } = entry.content
    else {
        panic!("fixture has no key");
    };
    return capture_shell(&lines[0], value, value_offset);
}

/// A plain command is one range without its comment.
#[test]
fn plain_commands() {
    assert_eq!(
        run_of("run: echo hi"),
        Ok((RunBody::Plain { start: 5, end: 12 }, 1))
    );
    assert_eq!(
        run_of("run: echo hi  # note"),
        Ok((RunBody::Plain { start: 5, end: 12 }, 1))
    );
    assert_eq!(
        run_of("- run: x\n  shell: bash\n"),
        Ok((RunBody::Plain { start: 7, end: 8 }, 1))
    );
}

/// A block literal collects its indented lines and stops at the next key.
#[test]
fn block_literals() {
    let text: &str = "- run: |\n    a\n    b\n  shell: bash\n";
    let Ok((RunBody::Block { lines }, next)) = run_of(text) else {
        panic!("not a block");
    };
    assert_eq!(next, 3);
    assert_eq!(
        lines,
        [
            BodyLine {
                envelope: 9,
                payload: 13,
                end: 15
            },
            BodyLine {
                envelope: 15,
                payload: 19,
                end: 21
            }
        ]
    );
}

/// Chomping indicators are fine,
///  inner blank lines stay,
///  and trailing blank lines go.
#[test]
fn chomping_and_blank_lines() {
    for header in ["|", "|-", "|+", "| # note"] {
        let text: String = format!("run: {header}\n  a\n\n  b\n\n\nnext: 1\n");
        let Ok((RunBody::Block { lines }, next)) = run_of(text.as_str()) else {
            panic!("not a block: {header}");
        };
        assert_eq!(lines.len(), 3, "{header}");
        assert_eq!(next, 6, "{header}");
        // The blank line keeps no payload bytes.
        assert_eq!(lines[1].payload, lines[1].end - 1);
    }
}

/// A deeper first line sets the body indentation.
#[test]
fn body_indentation_comes_from_the_first_line() {
    let Ok((RunBody::Block { lines }, _)) = run_of("run: |\n      a\n        b\n") else {
        panic!("not a block");
    };
    assert_eq!(lines[0].payload - lines[0].envelope, 6);
    assert_eq!(lines[1].payload - lines[1].envelope, 6);
}

/// A comment less indented than the body ends it,
///  and other text that is less indented is refused.
#[test]
fn shallow_lines_end_or_refuse() {
    let Ok((RunBody::Block { lines }, next)) = run_of("- run: |\n      a\n    # c\n  other: 1\n")
    else {
        panic!("not a block");
    };
    assert_eq!((lines.len(), next), (1, 2));
    let error: WalkError = run_of("- run: |\n      a\n    b\n").expect_err("shallow");
    assert!(error.message.contains("indented less"), "{}", error.message);
}

/// Windows line endings stay in the line range and out of the text.
#[test]
fn carriage_returns() {
    let Ok((RunBody::Block { lines }, _)) = run_of("run: |\r\n  a\r\n  b\r\n") else {
        panic!("not a block");
    };
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].end - lines[0].payload, 3);
}

/// Every other form is refused with advice.
#[test]
fn unsupported_forms_are_refused() {
    for text in [
        "run: |2\n   a\n",
        "run: >\n  a\n",
        "run: >-\n  a\n",
        "run: \"echo a\"",
        "run: 'echo a'",
        "run: *anchor",
        "run: &anchor echo",
        "run: !!str echo",
        "run: [a, b]",
        "run: {a: b}",
        "run: %thing",
        "run:\n  nested: 1\n",
        "run: # only a comment\n",
        "run: echo a\n  continued\n",
    ] {
        assert!(run_of(text).is_err(), "{text:?}");
    }
    assert_eq!(run_of("run: >\n  a\n").expect_err("folded").offset, 5);
}

/// A plain command may be followed by blank lines,
///  comments and the next key.
#[test]
fn plain_command_ends_at_the_next_key() {
    assert!(run_of("run: echo a\n\n# c\nnext: 1\n").is_ok());
    assert!(run_of("- run: echo a\n  next: 1\n").is_ok());
}

/// Plain and quoted shells are read without quotes or comments.
#[test]
fn shell_values() {
    assert_eq!(
        shell_of("shell: bash"),
        Ok(Shell {
            value: String::from("bash"),
            offset: 7
        })
    );
    assert_eq!(
        shell_of("shell: bash -e {0}  # c").expect("shell").value,
        "bash -e {0}"
    );
    assert_eq!(shell_of("shell: \"pwsh\"").expect("shell").value, "pwsh");
    assert_eq!(shell_of("shell: 'cmd'").expect("shell").value, "cmd");
    assert_eq!(
        shell_of("shell: ${{ matrix.shell }}").expect("shell").value,
        "${{ matrix.shell }}"
    );
}

/// Empty,
///  block,
///  flow and mismatched-quote shells are refused.
#[test]
fn bad_shell_values_are_refused() {
    for text in [
        "shell:",
        "shell: |",
        "shell: >",
        "shell: [a]",
        "shell: &a bash",
        "shell: \"bash",
        "shell: \"a\"b\"",
        "shell: \"\"",
    ] {
        let result: Result<Shell, WalkError> = shell_of(text);
        if text == "shell: \"\"" {
            assert_eq!(result.expect("empty quoted").value, "");
            continue;
        }
        assert!(result.is_err(), "{text:?}");
    }
}
