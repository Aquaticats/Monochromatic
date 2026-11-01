//! What:
//!  Unit tests for the message and help wording.
//! Why:
//!  Every placement and capture combination must name the failed condition and a concrete remedy,
//!  in neutral terms.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test.each(placements)('names the failed condition', (placement) => { ... });
//! ```

/// Import the text builders under test and the constants they embed.
use crate::message::{LIBRARY_NAME, LINT_NAME, capture_list, help_message, primary_message};
/// Import the placement categories to enumerate.
use crate::placement::Placement;

/// Every placement,
/// so a new variant added later fails the exhaustiveness of the loops below.
const PLACEMENTS: [Placement; 5] = [
    Placement::KeptByCallee,
    Placement::NotKeptByCallee,
    Placement::LetInitializer,
    Placement::UnresolvedCallee,
    Placement::Other,
];

/// Words that would judge code or authors instead of naming an operation or evidence.
const JUDGING_WORDS: [&str; 8] = [
    "bad", "wrong", "poor", "smell", "lazy", "mistake", "sloppy", "ugly",
];

/// Build an owned name list from literals.
fn names(items: &[&str]) -> Vec<String> {
    let mut list: Vec<String> = Vec::new();
    for item in items {
        list.push(String::from(*item));
    }
    return list;
}

/// An empty list renders as nothing,
/// one name as a backticked identifier,
/// several joined by commas.
#[test]
fn capture_list_renders_zero_one_and_many_names() {
    assert_eq!(capture_list(&names(&[])), "");
    assert_eq!(capture_list(&names(&["a"])), "`a`");
    assert_eq!(capture_list(&names(&["a", "b", "c"])), "`a`, `b`, `c`");
}

/// With captures,
/// the message names each capture and the failed placement condition.
#[test]
fn message_with_captures_names_them_and_the_failed_condition() {
    let captures: Vec<String> = names(&["state", "count"]);
    let expectations: [(Placement, &str); 4] = [
        (Placement::NotKeptByCallee, "not required to be `'static`"),
        (
            Placement::LetInitializer,
            "initializer of a `let` statement",
        ),
        (Placement::UnresolvedCallee, "not a declared function"),
        (Placement::Other, "not a direct argument"),
    ];
    for (placement, fragment) in expectations {
        let message: String = primary_message(&captures, placement);
        assert!(message.contains("`state`, `count`"), "{message}");
        assert!(message.contains(fragment), "{message}");
    }
}

/// Without captures,
/// the message says that nothing is captured,
/// and also names a failed placement when there is one.
#[test]
fn message_without_captures_says_nothing_is_captured() {
    let alone: String = primary_message(&names(&[]), Placement::KeptByCallee);
    assert!(alone.contains("captures no variables"), "{alone}");
    assert!(!alone.contains("also"), "{alone}");
    let both: String = primary_message(&names(&[]), Placement::Other);
    assert!(
        both.contains("captures no variables")
            && both.contains("also, it is not a direct argument"),
        "{both}"
    );
}

/// The remedy for a closure without captures is a named function.
#[test]
fn help_without_captures_names_a_function() {
    for placement in PLACEMENTS {
        let help: String = help_message(&names(&[]), placement);
        assert!(help.contains("named function"), "{help}");
        assert!(!help.contains(LINT_NAME), "{help}");
    }
}

/// The remedy for captured state shows the exact suppression attribute with this library and lint name.
#[test]
fn help_with_captures_shows_the_suppression_attribute() {
    let expected: String = format!(
        "#[cfg_attr(dylint_lib = \"{LIBRARY_NAME}\", expect({LINT_NAME}, reason = \"...\"))]"
    );
    for placement in PLACEMENTS {
        let help: String = help_message(&names(&["state"]), placement);
        assert!(help.contains(&expected), "{help}");
    }
}

/// The help for a let-bound closure explains the E0658 limit and the statement-level placement.
#[test]
fn help_for_let_initializer_mentions_the_statement_and_e0658() {
    let help: String = help_message(&names(&["state"]), Placement::LetInitializer);
    assert!(
        help.contains("`let` statement") && help.contains("E0658") && help.contains("restructure"),
        "{help}"
    );
    let other: String = help_message(&names(&["state"]), Placement::Other);
    assert!(!other.contains("E0658"), "{other}");
}

/// No message or help uses a word that judges the code or its author.
#[test]
fn messages_use_neutral_terms() {
    for placement in PLACEMENTS {
        for captures in [names(&[]), names(&["state"])] {
            let text: String = format!(
                "{} {}",
                primary_message(&captures, placement),
                help_message(&captures, placement)
            );
            for word in JUDGING_WORDS {
                assert!(!text.to_lowercase().contains(word), "{word} in {text}");
            }
        }
    }
}
