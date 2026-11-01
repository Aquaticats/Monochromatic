//! What:
//!  Slot-position controls for inferred generic constants,
//!  compared by finding message.
//! Why:
//!  A constant hole and a nameable type hole both yield one finding;
//!  only the message shows which slot was resolved.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Check real sources in one loaded project and compare the ordered finding messages.
//! ```

/// Import actual findings and severities.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the disposable Cargo-backed fixture that runs the production semantic session.
use crate::rust_semantic_test_support::SemanticFixture;

/// Message reported when the hole stands in a constant parameter's slot.
const CONSTANT: &str = "Replace inferred '_' with an explicit constant value.";
/// Message reported when the hole stands in a nameable type parameter's slot.
const TYPE: &str = "Replace inferred '_' with an explicit Rust type.";

/// What:
///  Authored source and its independently expected finding messages,
///  in source order.
/// Why:
///  An equal finding count cannot hide a hole that was matched against a neighbouring parameter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Case = { name: string; source: string; messages: string[] };
/// ```
struct Case {
    /// Failure context printed by assertions.
    name: &'static str,
    /// Complete standalone Rust input for the owned fixture.
    source: &'static str,
    /// Expected ordinary findings;
    ///  semantic-processing failures are prohibited separately.
    messages: &'static [&'static str],
}

/// Compare the complete ordered message list and reject unavailable semantic information.
fn assert_case(fixture: &mut SemanticFixture, case: &Case) {
    let findings: Vec<Diagnostic> = fixture.check(case.source, Severity::Warn);
    // Borrow each message; the findings own their strings for the whole comparison.
    let mut messages: Vec<&str> = Vec::<&str>::new();
    for finding in &findings {
        assert!(!finding.processing_failure, "{}: {finding:?}", case.name);
        assert_eq!(finding.code, "rust/require-explicit-types", "{}", case.name);
        messages.push(finding.message.as_str());
    }
    assert_eq!(messages, case.messages, "{}", case.name);
}

/// Each hole is matched with the parameter in its own slot,
///  whatever precedes it in either list.
#[test]
fn holes_resolve_against_the_parameter_in_their_own_slot() {
    let mut fixture: SemanticFixture = SemanticFixture::new();
    let cases: &[Case] = &[
        Case {
            name: "constant hole after a type argument",
            source: "fn build<T: Copy, const N: usize>(value: T) -> [T; N] { return [value; N]; } fn main() { let values: [u8; 3] = build::<u8, _>(1_u8); }",
            messages: &[CONSTANT],
        },
        Case {
            name: "type hole after a constant argument",
            source: "fn build<const N: usize, T: Copy>(value: T) -> [T; N] { return [value; N]; } fn main() { let values: [u8; 3] = build::<3, _>(1_u8); }",
            messages: &[TYPE],
        },
        Case {
            name: "adjacent holes keep their own parameter kinds",
            source: "fn build<T: Copy, const N: usize>(value: T) -> [T; N] { return [value; N]; } fn main() { let values: [u8; 3] = build::<_, _>(1_u8); }",
            messages: &[TYPE, CONSTANT],
        },
        Case {
            name: "lifetime argument and parameter do not shift the slots",
            source: "struct View<'a, T, const N: usize> { items: &'a [T; N] } static ITEMS: [u8; 3] = [1_u8, 2_u8, 3_u8]; fn main() { let view: View<'static, u8, 3> = View::<'static, u8, _> { items: &ITEMS }; }",
            messages: &[CONSTANT],
        },
        Case {
            name: "method call arguments use the method's own parameters",
            source: "struct Maker; impl Maker { fn build<T: Copy, const N: usize>(&self, value: T) -> [T; N] { return [value; N]; } } fn main() { let values: [u8; 3] = Maker.build::<u8, _>(1_u8); }",
            messages: &[CONSTANT],
        },
    ];
    for case in cases {
        assert_case(&mut fixture, case);
    }
}
