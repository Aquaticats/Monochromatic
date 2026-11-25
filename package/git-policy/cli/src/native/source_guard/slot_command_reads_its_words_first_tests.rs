//! What:
//!  The check that the slot glue a forwarded Git command reaches reads that command's words before it reads
//!  anything the words did not ask for.
//! Why:
//!  `forward_action` reaches the slot glue for every forwarded command,
//!  so a `git status` or a `git commit` pays whatever the glue reads before the words say the command is
//!  not one of its own (`invocation/entry_zfs_slot_commands.rs`,
//!  "Every forwarded command asks,
//! so the repair word is read before the working directory is").
//!  A working directory read is a system call,
//!  and the command's answer cannot show the order:
//!  a failed read ends in the same `None` the words alone end in,
//!  so `.ok()?` hides the cost from every module test.
//!  This check reads the release source instead,
//!  where the order is the code,
//!  and fails when a read of the working directory,
//!  of a record or of the registry stands before the read of the command's words,
//!  or when nothing returns between them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // source_guard/slot_command_reads_its_words_first_tests.ts: one ordering check over one release file.
//! ```

use std::path::PathBuf;

use super::blocking_code_off_the_runtime_tests::read_the_code_of;
use crate::source_scan_tests::source_root;

/// The one release file whose order this check reads,
/// relative to the source root.
const SLOT_GLUE: &str = "invocation/entry_zfs_slot_commands.rs";

/// The functions every forwarded command reaches,
/// in the order the caller runs them.
const FORWARDED_FUNCTIONS: [&str; 3] = ["move_refusal_of", "before_a_prune", "before_a_repair"];

/// The code that reads a command's own words:
/// the worktree subcommand and the tokens after it.
const WORD_READS: [&str; 3] = ["worktree_subcommand(", "command.word()", "command.arguments.get("];

/// What one later step costs,
/// as the failure names it:
/// the code,
/// and what every forwarded command pays when the step stands before the words are read.
const LATER_STEPS: [(&str, &str); 7] = [
    ("std::env::current_dir", "a read of the process's working directory"),
    ("start_directory(", "a parse of the start directory the global options move"),
    ("parse_move_source(", "a read of the registry and of the filesystem for the move's source"),
    ("refuse_move_of_a_bound_slot(", "a read of the binding record of the slot a move names"),
    ("guard_the_prune(", "a read of every record a prune would drop"),
    ("repair_paths(", "a read of the registry for the paths a repair reports"),
    ("the_links_report(", "a read of the link and of the registration of each repaired path"),
];

/// What:
///  The code lines of one function of `text`,
///  with their numbers in the file.
/// Why:
///  The order of two statements is a question about one function,
///  and a comment or a literal that names either statement is not a statement;
///  the reading masks both,
///  so a doc line about the working directory neither passes nor fails the order.
fn body_lines(text: &str, name: &str) -> Result<Vec<(usize, String)>, String> {
    let masked: Vec<String> = read_the_code_of(text)
        .map_err(|error| return format!("{name} cannot be read from this file: {error}"))?
        .masked;
    let signature: String = format!("fn {name}(");
    let mut body: Vec<(usize, String)> = Vec::new();
    let mut depth: usize = 0;
    let mut opened: bool = false;
    for (index, line) in masked.iter().enumerate() {
        if body.is_empty() && !line.contains(signature.as_str()) {
            continue;
        }
        depth += line.matches('{').count();
        depth = depth.saturating_sub(line.matches('}').count());
        opened = opened || depth > 0;
        body.push((index + 1, line.clone()));
        if opened && depth == 0 {
            return Ok(body);
        }
    }
    if body.is_empty() {
        return Err(format!("this file declares no function named `{name}`, so the order this check reads is not here"));
    }
    return Err(format!("the braces of `{name}` do not close, so its body cannot be read"));
}

/// What:
///  The first line of `body` that holds one of `patterns`,
///  as the line's number and the position of the pattern it matched.
/// Why:
///  The check compares the first read of the words with the first later step,
///  so each is the earliest line that holds any of its spellings.
fn first_line_holding(body: &[(usize, String)], patterns: &[&str]) -> Option<(usize, usize)> {
    for (number, line) in body {
        if let Some(position) = patterns.iter().position(|candidate| return line.contains(candidate)) {
            return Some((*number, position));
        }
    }
    return None;
}

/// What:
///  What is wrong with one forwarded function's order,
///  or `Ok` when it reads the command's words first and returns before its first later step.
/// Why:
///  The rule is the words first,
///  an early return,
///  and only then the reads,
///  and each arm names the line a person edits.
fn order_of(name: &str, body: &[(usize, String)]) -> Result<(), String> {
    let steps: Vec<&str> = LATER_STEPS.iter().map(|(pattern, _)| return *pattern).collect();
    let Some((step_line, step)) = first_line_holding(body, steps.as_slice()) else {
        return Err(format!(
            "`{name}` reads nothing, so this check no longer covers what it was written for"
        ));
    };
    let Some((word_line, word)) = first_line_holding(body, &WORD_READS) else {
        return Err(format!(
            "`{name}` runs {} (`{}`) at line {step_line} without reading the command's words at all",
            LATER_STEPS[step].1, LATER_STEPS[step].0
        ));
    };
    if step_line < word_line {
        return Err(format!(
            "`{name}` runs {} (`{}`) at line {step_line} before it reads the command's words (`{}`) at line \
             {word_line}",
            LATER_STEPS[step].1, LATER_STEPS[step].0, WORD_READS[word]
        ));
    }
    let returns: bool = body
        .iter()
        .any(|(number, line)| return *number >= word_line && *number < step_line && line.contains("return"));
    if !returns {
        return Err(format!(
            "`{name}` reads the command's words at line {word_line} and reaches {} at line \
             {step_line} with no return between them",
            LATER_STEPS[step].1
        ));
    }
    return Ok(());
}

/// What:
///  Every backward step the slot glue's forwarded functions take,
///  each as `<file>: <what is wrong>`.
/// Why:
///  A failure names the file,
///  the function,
///  both lines and the cost,
///  so a person edits the statement the message names.
fn backward_steps_in(text: &str, names: &[&str]) -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();
    for name in names {
        let checked: Result<(), String> =
            body_lines(text, name).and_then(|body| return order_of(name, body.as_slice()));
        if let Err(failure) = checked {
            failures.push(format!("{SLOT_GLUE}: {failure}"));
        }
    }
    return failures;
}

/// What:
///  A command's words decide every read the slot glue makes.
/// Why:
///  This is the defect the check exists for:
///  `move_refusal_of` read the working directory and parsed the start directory for every forwarded
///  command,
///  and the words that would have returned first were read after them.
#[test]
fn every_forwarded_slot_command_reads_its_words_before_it_reads_anything_else() {
    let path: PathBuf = source_root().join(SLOT_GLUE);
    let text: String = std::fs::read_to_string(path.as_path())
        .unwrap_or_else(|error| panic!("{} is a release file of this crate: {error}", path.display()));
    let failures: Vec<String> = backward_steps_in(text.as_str(), &FORWARDED_FUNCTIONS);
    assert!(
        failures.is_empty(),
        "the slot glue runs for every forwarded command, so a read before the command's words is paid by a `git \
         status` and a `git commit`; read the words and return first:\n{}",
        failures.join("\n")
    );
}

/// What:
///  The check passes a function that reads its words,
///  returns,
///  and only then reads,
///  even with a comment above the guard that names the read.
/// Why:
///  A control that cannot pass proves nothing,
///  and the comment shows the masked reading does not count prose as a statement.
#[test]
fn a_function_that_reads_its_words_first_passes() {
    let text: &str = "\
fn move_refusal_of(command: &ResolvedCommand) -> Option<Action> {
    // The working directory is read only after the word above says this is a move.
    if worktree_subcommand(command) != Some(b\"move\") {
        return None;
    }
    let current: PathBuf = std::env::current_dir().ok()?;
    return None;
}
";
    assert_eq!(backward_steps_in(text, &["move_refusal_of"]), Vec::<String>::new());
}

/// What:
///  The check fails a function that reads before its words,
///  one that reads after them with no return between,
///  and the shape this round fixed,
///  a function that reads and never reads the words at all.
/// Why:
///  Each arm is a way the read came first,
///  so the control above fails when any of them comes back.
#[test]
fn the_check_names_a_read_before_the_words_and_a_missing_return() {
    let early: &str = "\
fn move_refusal_of(command: &ResolvedCommand, common_dir: Option<&Path>) -> Option<Action> {
    let current: PathBuf = std::env::current_dir().ok()?;
    if worktree_subcommand(command) != Some(b\"move\") {
        return None;
    }
    return None;
}
";
    let failures: Vec<String> = backward_steps_in(early, &["move_refusal_of"]);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert!(failures[0].contains("std::env::current_dir"), "{failures:?}");
    assert!(failures[0].contains("line 2"), "the read's line is named: {failures:?}");
    let late: &str = "\
fn before_a_prune(command: &ResolvedCommand) -> Option<Action> {
    if worktree_subcommand(command) == Some(b\"prune\") {
        let guard: PruneGuard = guard_the_prune(command);
    }
    return None;
}
";
    let refused: Vec<String> = backward_steps_in(late, &["before_a_prune"]);
    assert_eq!(refused.len(), 1, "{refused:?}");
    assert!(refused[0].contains("no return between them"), "{refused:?}");
    let wordless: &str = "\
fn move_refusal_of(command: &ResolvedCommand, common_dir: Option<&Path>) -> Option<Action> {
    let common: &Path = common_dir?;
    let current: PathBuf = std::env::current_dir().ok()?;
    return None;
}
";
    let absent: Vec<String> = backward_steps_in(wordless, &["move_refusal_of"]);
    assert_eq!(absent.len(), 1, "{absent:?}");
    assert!(absent[0].contains("without reading the command's words at all"), "{absent:?}");
}

/// What:
///  The reading refuses a file without the function,
///  and one whose braces do not balance,
///  rather than passing either.
/// Why:
///  A renamed function and a half-read file would otherwise report the order as right by reading nothing.
#[test]
fn the_reading_refuses_what_it_cannot_place() {
    let renamed: Result<Vec<(usize, String)>, String> = body_lines("fn other() {\n}\n", "move_refusal_of");
    assert!(renamed.is_err(), "a file without the function is an error");
    let unbalanced: Result<Vec<(usize, String)>, String> =
        body_lines("fn move_refusal_of() {\n    let a: u8 = 1;\n", "move_refusal_of");
    assert!(unbalanced.is_err(), "a body that does not close is an error");
    let braces_in_a_literal: Result<Vec<(usize, String)>, String> = body_lines(
        "fn move_refusal_of() {\n    let a: u8 = format!(\"{}\", 1);\n}\n",
        "move_refusal_of",
    );
    assert_eq!(
        braces_in_a_literal.expect("the literal's braces close nothing").len(),
        3,
        "the body ends at its own brace"
    );
}
