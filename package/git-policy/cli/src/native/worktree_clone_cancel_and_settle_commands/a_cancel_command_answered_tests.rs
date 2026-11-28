//! What:
//!  The answers the cancel word prints without a run:
//!  the usage,
//!  each caller's mistake,
//!  and the refusal about where the invocation stands.
//! Why:
//!  Every one of these is decided before a record or a mount is read,
//!  so a control on this machine proves them and never opens this machine's pool.
//!  The help and the mistakes are answered before the place is known,
//!  which is what the control that hands in no place at all asserts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(aCancelCommandAnswered(parseCancelArguments([]), noPlace).code).toBe(2);
//! ```

use crate::worktree_clone_cancel_and_settle_commands::a_cancel_command_answered::a_cancel_command_answered;
use crate::worktree_clone_cancel_and_settle_commands::the_answer_one_word_prints::AnAnswer;
use crate::worktree_clone_cancel_and_settle_commands::the_place_the_two_commands_run_in::TheRoute;
use crate::worktree_clone_removal_cancel::cancel_arguments::{
    CancelInvocation,
    CancelMistake,
    parse_cancel_arguments,
    usage,
};
use std::ffi::OsString;

/// The refusal a control hands in where a run has to know its place,
/// which the usage and the mistakes must never reach.
fn no_place() -> Result<TheRoute, AnAnswer> {
    return Err(AnAnswer::on_standard_error(
        2,
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
///  That `--help` prints the command's own usage and exits zero,
///  and that it reaches no place,
///  because the control's place refuses.
/// Why:
///  A usage that needed a repository would refuse a person asking what the
///  command takes in a folder that is none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(aCancelCommandAnswered(parseCancelArguments(['--help']), noPlace())).toEqual({ code: 0, stdout: usage() + '\n', stderr: '' });
/// ```
#[test]
fn the_usage_is_printed_and_exits_zero_without_a_place() {
    let answer: AnAnswer = a_cancel_command_answered(
        parse_cancel_arguments(words(&["--help"]).as_slice()),
        no_place(),
    );
    assert_eq!(answer.code, 0);
    assert_eq!(answer.stdout, format!("{}\n", usage()));
    assert!(
        answer.stderr.is_empty(),
        "the usage writes nothing on standard error"
    );
}

/// What:
///  That each caller's mistake answers with the command's own sentence,
///  the caller's status,
///  and nothing on standard output,
///  without reaching any place.
/// Why:
///  The word is admitted now,
///  so a mistake must not answer the namespace's usage text any more:
///  that is the answer this round removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const typed of [[], ['one', 'two'], ['--force'], ['bad/name']]) { expect(answer(typed).code).toBe(2); }
/// ```
#[test]
fn a_callers_mistake_answers_the_commands_own_sentence() {
    let mistakes: [&[&str]; 4] = [&[], &["one", "two"], &["--force"], &["bad/name"]];
    for typed in mistakes {
        let answer: AnAnswer =
            a_cancel_command_answered(parse_cancel_arguments(words(typed).as_slice()), no_place());
        assert_eq!(answer.code, 2, "`{typed:?}` is the caller's mistake");
        assert!(
            answer.stdout.is_empty(),
            "`{typed:?}` writes nothing on standard output"
        );
        assert!(
            answer.stderr.ends_with('\n'),
            "`{typed:?}` ends its line: {:?}",
            answer.stderr
        );
        assert!(
            !answer
                .stderr
                .contains("Usage: git monochromatic-git-policy-cli check"),
            "`{typed:?}` answers the command's own sentence rather than the namespace's usage: {}",
            answer.stderr
        );
    }
}

/// What:
///  That the first mistake's sentence is the one the words name,
///  and that a second word is named beside the first.
/// Why:
///  The two shapes are what a person types most often,
///  and a control that only read a status would pass on any sentence at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(answer([]).stderr).toContain('no slot was named');
/// expect(answer(['one', 'two']).stderr).toContain('`one` and `two`');
/// ```
#[test]
fn the_sentence_names_the_mistake_a_person_made() {
    let no_slot: AnAnswer =
        a_cancel_command_answered(parse_cancel_arguments(words(&[]).as_slice()), no_place());
    assert!(
        no_slot.stderr.contains("no slot was named"),
        "{}",
        no_slot.stderr
    );
    let two: AnAnswer = a_cancel_command_answered(
        parse_cancel_arguments(words(&["one", "two"]).as_slice()),
        no_place(),
    );
    assert!(two.stderr.contains("`one` and `two`"), "{}", two.stderr);
}

/// What:
///  That a place which cannot be read answers with its own status and sentence,
///  once the words themselves are the command's.
/// Why:
///  A person who typed the command correctly and stood in the wrong folder
///  reads why,
///  and the answer carries the line feed every sentence of this command ends in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = aCancelCommandAnswered(parseCancelArguments(['one']), noPlace());
/// expect(answer).toEqual({ code: 2, stdout: '', stderr: 'this control\'s invocation names no place, and no answer may need one\n' });
/// ```
#[test]
fn a_place_that_cannot_be_read_answers_its_own_sentence() {
    let answer: AnAnswer = a_cancel_command_answered(
        parse_cancel_arguments(words(&["one"]).as_slice()),
        no_place(),
    );
    assert_eq!(answer.code, 2);
    assert!(
        answer.stdout.is_empty(),
        "a refusal writes nothing on standard output"
    );
    assert_eq!(
        answer.stderr,
        "this control's invocation names no place, and no answer may need one\n"
    );
    let help: Result<CancelInvocation, CancelMistake> =
        parse_cancel_arguments(words(&["--help"]).as_slice());
    assert_eq!(help.expect("the usage parses"), CancelInvocation::Help);
}
