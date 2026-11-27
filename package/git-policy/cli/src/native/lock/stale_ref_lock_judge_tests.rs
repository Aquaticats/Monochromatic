//! Controls for the margins of the reference-lock judge:
//! each must clear the measurement it was set from.

use super::*;

/// A lock younger than the longest live hold measured may still be live,
/// so the margin must exceed that hold.
#[test]
fn the_margin_exceeds_the_longest_live_reference_step_measured() {
    assert!(
        MARGIN > LONGEST_LIVE_REF_STEP,
        "{MARGIN:?} must exceed {LONGEST_LIVE_REF_STEP:?}"
    );
}

/// The margin is the multiple of the measurement,
/// rounded up to whole seconds and no further.
#[test]
fn the_margin_is_the_multiple_of_the_measurement_rounded_up_to_a_whole_second() {
    let wanted: Duration =
        LONGEST_LIVE_REF_STEP * u32::try_from(MARGIN_MULTIPLE).expect("a small multiple");
    assert_eq!(MARGIN.subsec_nanos(), 0);
    assert!(MARGIN >= wanted, "{MARGIN:?} must cover {wanted:?}");
    assert!(
        MARGIN - wanted < Duration::from_secs(1),
        "{MARGIN:?} is a second or more over {wanted:?}"
    );
}

/// What:
///  The name another program chose for the process that holds the judged file:
///  a closing parenthesis that would end the parentheses around it,
///  an escape that would erase the line,
///  a line break that would start a line this program never wrote,
///  and a right-to-left override that would reorder it.
/// Why:
///  The name comes from `/proc/<pid>/comm`,
///  which the program sets to any bytes it likes,
///  and the verdict is the line a person reads to find it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NAME_A_PROGRAM_CHOSE = 'x) FORGED' + ESCAPE + '[2K' + LINE_BREAK + '\u202e';
/// ```
const NAME_A_PROGRAM_CHOSE: &str = "x) FORGED\u{1b}[2K\n\u{202e}";

/// What:
///  The name shown in the sentence that says a live process holds the judged file.
/// Why:
///  The four shapes are written out,
///  so the sentence keeps one line and the parentheses around the name stay the program's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SHOWN_NAME = 'x\\) FORGED\\u{1b}[2K\\n\\u{202e}';
/// ```
const SHOWN_NAME: &str = "x\\) FORGED\\u{1b}[2K\\n\\u{202e}";

/// What:
///  The verdict of a lock and of a staging file that a live process has open are one sentence,
///  built by `a_held_file_verdict`,
///  and it shows the name so no byte of it can end the parentheses
///  or start a line.
/// Why:
///  The two judgements say the same words about the same kernel-chosen name,
///  so a name that rewrote one of them would rewrite both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(aHeldFileVerdict(4242, NAME_A_PROGRAM_CHOSE)).toBe(`process 4242 (${ SHOWN_NAME }) has the file open`);
/// ```
#[test]
fn a_name_that_would_end_the_parentheses_cannot_rewrite_the_held_file_verdict() {
    let line: String = a_held_file_verdict(4242, Some(NAME_A_PROGRAM_CHOSE));
    assert_eq!(
        line,
        format!("process 4242 ({SHOWN_NAME}) has the file open")
    );
    assert_eq!(line.matches('\n').count(), 0, "{line:?}");
}

/// A process whose name could not be read shows nothing between the parentheses,
/// so the sentence never claims a program it cannot name.
#[test]
fn a_holder_with_no_readable_name_is_named_by_its_number() {
    assert_eq!(
        a_held_file_verdict(4242, None),
        "process 4242 () has the file open"
    );
}
