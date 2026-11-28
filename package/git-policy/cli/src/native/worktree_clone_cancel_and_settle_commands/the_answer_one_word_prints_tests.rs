//! What:
//!  That one answer carries one stream's text and one status,
//!  and that a text a person reads ends its line.
//! Why:
//!  A refusal written on both streams reads twice in a terminal,
//!  and a report whose last line has no line feed runs into the next prompt,
//!  so both are the answer's own decisions and are read here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(onStandardError(2, 'x').stdout).toBe('');
//! expect(onStandardOutput(0, 'x').lineTerminated().stdout).toBe('x\n');
//! ```

/// What:
///  That a text written on one stream leaves the other empty.
/// Why:
///  A caller that captures one stream reads one whole answer,
///  and a refusal that also wrote a report would be read as two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(answer.stderr).toBe('the refusal');
/// expect(answer.stdout).toBe('');
/// ```
#[test]
fn one_answer_carries_one_stream_s_text() {
    let refused: super::AnAnswer = super::AnAnswer::on_standard_error(
        2,
        String::from("no slot was named, so there is nothing to cancel."),
    );
    assert_eq!(refused.code, 2);
    assert_eq!(
        refused.stderr,
        "no slot was named, so there is nothing to cancel."
    );
    assert!(
        refused.stdout.is_empty(),
        "a refusal writes nothing on standard output"
    );
    let reported: super::AnAnswer =
        super::AnAnswer::on_standard_output(0, String::from("settled the records of `one`."));
    assert_eq!(reported.code, 0);
    assert_eq!(reported.stdout, "settled the records of `one`.");
    assert!(
        reported.stderr.is_empty(),
        "a report writes nothing on standard error"
    );
}

/// What:
///  That a text a person reads ends its line,
///  once,
///  and that an empty stream stays empty.
/// Why:
///  A sentence written with its own line feed and one added twice leaves a blank
///  line,
///  and a stream with no text gains nothing from the terminator.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(onStandardOutput(0, 'x').lineTerminated().stdout).toBe('x\n');
/// expect(onStandardOutput(0, 'x\n').lineTerminated().stdout).toBe('x\n');
/// expect(onStandardError(2, '').lineTerminated().stderr).toBe('');
/// ```
#[test]
fn a_text_a_person_reads_ends_its_line_once() {
    let added: super::AnAnswer =
        super::AnAnswer::on_standard_output(0, String::from("x")).line_terminated();
    assert_eq!(added.stdout, "x\n");
    let already: super::AnAnswer =
        super::AnAnswer::on_standard_output(0, String::from("x\n")).line_terminated();
    assert_eq!(
        already.stdout, "x\n",
        "a line feed the text carries is not doubled"
    );
    let empty: super::AnAnswer =
        super::AnAnswer::on_standard_error(2, String::new()).line_terminated();
    assert!(empty.stderr.is_empty(), "an empty stream stays empty");
}
