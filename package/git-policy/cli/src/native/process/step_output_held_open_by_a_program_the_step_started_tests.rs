//! Controls for the sentences that name a program the step started and the stream it still holds.
//! Why:
//!  The name in those sentences is the one a program chose for itself (`/proc/<pid>/comm`),
//!  and the sentence is the person's pointer to which program to end,
//!  so the name must reach the terminal as text that can act on nothing (`AGENTS.md`,
//!  rule `SYB`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(outputHeldOpenText('a step', [{ pid: 7, command: '\u001b[2K' }])).not.toContain('\u001b');
//! ```

use super::*;

/// What:
///  A program's name as a program may set it:
///  a terminal escape that erases the line,
///  a line break that would start a line this program never wrote,
///  and a closing parenthesis that would end the parentheses around the name.
/// Why:
///  Those are the shapes the repository's security rules list for text that crosses into a line,
///  and they fit the fifteen bytes a process name may hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NAME_A_PROGRAM_CHOSE = '\u001b[2K\n)';
/// ```
const NAME_A_PROGRAM_CHOSE: &str = "\u{1b}[2K\n)";

/// What:
///  Why a sentence about a held-open stream must read as text a terminal cannot act on:
///  it is one line,
///  no byte of the name erases or ends it,
///  and the program is still named by its number and its own words.
/// Why:
///  A line a person reads to find a program is the one thing the name could be used to forge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theSentenceCannotBeRewritten(sentence: string, pid: number): void;
/// ```
fn the_sentence_cannot_be_rewritten(
    sentence: &str,
    pid: i64,
) {
    assert!(
        !sentence.chars().any(char::is_control),
        "no control character of the name reached the sentence: {sentence:?}"
    );
    assert!(
        !sentence.contains('\n'),
        "the sentence holds no line break a terminal would end a line at: {sentence:?}"
    );
    assert!(
        sentence.contains("\\u{1b}[2K\\n\\)"),
        "the name is shown with its escape, its line break and its parenthesis spelled out: {sentence:?}"
    );
    assert!(
        sentence.contains(format!("{pid}").as_str()),
        "the sentence still names which program it is: {sentence:?}"
    );
}

/// A program whose name holds an escape,
/// a line break and a closing parenthesis is named by a sentence that stays one
/// line,
/// and its number still says which process it is.
#[test]
fn an_adversarial_name_cannot_rewrite_the_held_open_sentence() {
    let holder: ProcessUser = ProcessUser {
        pid: 7,
        command: Some(String::from(NAME_A_PROGRAM_CHOSE)),
    };
    let sentence: String = output_held_open_text("a step", &[holder]);
    the_sentence_cannot_be_rewritten(sentence.as_str(), 7);
}

/// The sentence that says the standard error stops where the program holds it names that program the same way.
#[test]
fn an_adversarial_name_cannot_rewrite_the_cut_short_sentence() {
    let holder: ProcessUser = ProcessUser {
        pid: 9,
        command: Some(String::from(NAME_A_PROGRAM_CHOSE)),
    };
    let sentence: String = error_cut_short_text("a step", &[holder]);
    the_sentence_cannot_be_rewritten(sentence.as_str(), 9);
}

/// A holder whose name could not be read is named by its number alone,
/// so the sentence never claims a program it cannot name.
#[test]
fn a_holder_with_no_readable_name_is_named_by_its_number() {
    let holder: ProcessUser = ProcessUser {
        pid: 11,
        command: None,
    };
    let sentence: String = output_held_open_text("a step", &[holder]);
    assert!(
        sentence.contains("process 11 still holds"),
        "the sentence names the process by its number: {sentence:?}"
    );
    assert!(
        !sentence.contains('('),
        "no name is invented for a process whose name could not be read: {sentence:?}"
    );
}

/// A stream no living program holds any more says so,
/// and names no number at all.
#[test]
fn a_stream_nothing_holds_says_so() {
    let sentence: String = output_held_open_text("a step", &[]);
    assert!(
        sentence.contains("a program it started, which no longer shows in the process table"),
        "{sentence:?}"
    );
}
