//! What:
//!  The answers the settling word prints without a run:
//!  each way its words or its two confirming values are wrong,
//!  and the refusal about where the invocation stands.
//! Why:
//!  Every one of these is decided before a record or the pool is read,
//!  so a control on this machine proves them and never opens this machine's pool.
//!  The command has no `--help`,
//!  which the control that types one asserts:
//!  its usage refusal prints the line a person copies.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(aSettlingCommandAnswered(parseSettlingArguments(['one']), noPlace).code).toBe(2);
//! ```

use crate::worktree_clone_cancel_and_settle_commands::a_settling_command_answered::a_settling_command_answered;
use crate::worktree_clone_cancel_and_settle_commands::the_answer_one_word_prints::AnAnswer;
use crate::worktree_clone_cancel_and_settle_commands::the_place_the_two_commands_run_in::TheRoute;
use crate::worktree_clone_record_settling::settling_request::parse_settling_arguments;
use std::ffi::OsString;

/// The refusal a control hands in where a run has to know its place,
/// which no stop of the words may reach.
fn no_place() -> Result<TheRoute, AnAnswer> {
    return Err(AnAnswer::on_standard_error(
        1,
        String::from("this control's invocation names no place, and no answer may need one"),
    ));
}

/// One invocation written as the words a person types.
fn words(typed: &[&str]) -> Vec<OsString> {
    return typed
        .iter()
        .map(|word| return OsString::from(word))
        .collect();
}

/// What:
///  That the slot alone answers the caller's status with the sentence that names
///  the two flags,
///  and reaches no place.
/// Why:
///  A person who read half the design's line learns the rest of it from the
///  refusal,
///  and a refusal that needed a repository would refuse them twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = aSettlingCommandAnswered(parseSettlingArguments(['one']), noPlace());
/// expect(answer.code).toBe(2); expect(answer.stdout).toBe('');
/// ```
#[test]
fn the_slot_alone_answers_the_two_flags_it_needs() {
    let answer: AnAnswer = a_settling_command_answered(
        parse_settling_arguments(words(&["one"]).as_slice()),
        no_place(),
    );
    assert_eq!(answer.code, 2, "the caller's mistake is status 2");
    assert!(
        answer.stdout.is_empty(),
        "a refusal writes nothing on standard output"
    );
    assert!(
        answer.stderr.contains("--confirm-removal-of")
            && answer.stderr.contains("--confirm-clone-guid"),
        "the refusal names the two flags and the line they stand in: {}",
        answer.stderr
    );
    assert!(
        answer.stderr.ends_with('\n'),
        "the sentence ends its line: {:?}",
        answer.stderr
    );
}

/// What:
///  That a value still carrying its placeholder is the caller's mistake,
///  answered with the caller's status.
/// Why:
///  A blind paste of the printed line is the mistake the placeholder exists for,
///  and it must not read as a refusing machine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(answer(['one', '--confirm-removal-of', 'one', '--confirm-clone-guid', '<the last four decimal digits of cloneGuid=>']).code).toBe(2);
/// ```
#[test]
fn a_pasted_placeholder_answers_the_callers_status() {
    let answer: AnAnswer = a_settling_command_answered(
        parse_settling_arguments(
            words(&[
                "one",
                "--confirm-removal-of",
                "one",
                "--confirm-clone-guid",
                "<the last four decimal digits of cloneGuid=>",
            ])
            .as_slice(),
        ),
        no_place(),
    );
    assert_eq!(answer.code, 2);
    assert!(
        answer.stderr.contains("placeholder"),
        "the refusal says the value is still the printed placeholder: {}",
        answer.stderr
    );
}

/// What:
///  That `--help` is no option of this command,
///  so it answers the usage refusal rather than a help text.
/// Why:
///  The command's grammar is one operand and two flags,
///  and a help text of its own would be a second grammar to keep in step with
///  the line every refusal prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(answer(['--help']).code).toBe(2);
/// expect(answer(['--help']).stderr).toContain('settle-worktree-clone-record');
/// ```
#[test]
fn help_is_no_option_of_this_command() {
    let answer: AnAnswer = a_settling_command_answered(
        parse_settling_arguments(words(&["--help"]).as_slice()),
        no_place(),
    );
    assert_eq!(answer.code, 2);
    assert!(
        answer
            .stderr
            .contains("`--help` is no option of this command"),
        "{}",
        answer.stderr
    );
    assert!(
        answer
            .stderr
            .contains("settle-worktree-clone-record <slot>"),
        "the refusal prints the line a person copies: {}",
        answer.stderr
    );
}

/// What:
///  That a place which cannot be read answers with the machine's status and its
///  own sentence.
/// Why:
///  A person who typed the command correctly and stood in the wrong folder reads
///  why,
///  and a place no command could name is the machine's condition rather than
///  their mistake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = aSettlingCommandAnswered(parseSettlingArguments(['one','--confirm-removal-of','one','--confirm-clone-guid','3102']), noPlace());
/// expect(answer).toEqual({ code: 1, stdout: '', stderr: '...\n' });
/// ```
#[test]
fn a_place_that_cannot_be_read_answers_its_own_sentence() {
    let answer: AnAnswer = a_settling_command_answered(
        parse_settling_arguments(
            words(&[
                "one",
                "--confirm-removal-of",
                "one",
                "--confirm-clone-guid",
                "3102",
            ])
            .as_slice(),
        ),
        no_place(),
    );
    assert_eq!(answer.code, 1);
    assert!(
        answer.stdout.is_empty(),
        "a refusal writes nothing on standard output"
    );
    assert_eq!(
        answer.stderr,
        "this control's invocation names no place, and no answer may need one\n"
    );
}
