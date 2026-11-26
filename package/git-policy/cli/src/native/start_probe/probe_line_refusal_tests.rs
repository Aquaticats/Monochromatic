//! What:
//!  Controls that each refusal says what a person needs:
//!  the line or the word it refused,
//!  in a sentence rather than a name.
//! Why:
//!  A record that says only "unreadable" sends a person nowhere.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(refusal.message()).toContain('probe-step');
//! ```

use super::LineRefusal;

/// Every refusal names the thing it refused.
#[test]
fn every_refusal_names_what_it_refused() {
    let cases: Vec<(LineRefusal, &str)> = vec![
        (
            LineRefusal::NotAStatusLine {
                text: String::from("hello"),
            },
            "hello",
        ),
        (
            LineRefusal::UnknownKey {
                word: String::from("colour"),
            },
            "colour",
        ),
        (
            LineRefusal::RepeatedKey {
                key: String::from("step"),
            },
            "step",
        ),
        (
            LineRefusal::MissingKey {
                key: String::from("answer"),
            },
            "answer",
        ),
        (
            LineRefusal::UnknownStep {
                word: String::from("sideways"),
            },
            "sideways",
        ),
        (
            LineRefusal::UnknownAnswer {
                word: String::from("maybe"),
            },
            "maybe",
        ),
        (
            LineRefusal::UnknownErrno {
                word: String::from("ENOPE"),
            },
            "ENOPE",
        ),
        (
            LineRefusal::UnknownReason {
                word: String::from("because"),
            },
            "because",
        ),
        (
            LineRefusal::WrongArm {
                expected: "move",
                word: String::from("teardown"),
            },
            "teardown",
        ),
        (
            LineRefusal::ValueNotOneWord {
                key: String::from("source"),
            },
            "source",
        ),
    ];
    for (refusal, named) in cases {
        assert!(
            refusal.message().contains(named),
            "{} lacks {named}",
            refusal.message()
        );
    }
}
