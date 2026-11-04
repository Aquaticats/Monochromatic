//! What:
//!  Short-option cluster cases:
//!  arity inside a cluster,
//!  the typo check and `h`.
//! Why:
//!  `-am`,
//!  `-ma` and `-ua` differ only by which letter takes a value;
//!  a wrong reading
//!      hides `--all` from a policy or invents it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseOptions(['-am', 'msg'])).toEqual(...)
//! ```

/// Tokenizer result types,
///  queries and the shared synthetic table.
use crate::command_options::{
    Arity, DEFAULT_MODE, Occurrence, OptionError, OptionErrorKind, OptionSpec, OptionValue,
    ParsedOptions, parse_options, row,
};
use crate::command_options_query::{is_enabled, is_stated};
use crate::command_test_support::{
    ALL, MESSAGE, NO_VERIFY, UNTRACKED, os_arguments, parse_synthetic,
};

/// Shorthand for an expected refusal at token 0.
fn refusal(kind: OptionErrorKind) -> Result<ParsedOptions, OptionError> {
    return Err(OptionError { kind, token: 0 });
}

/// A value-taking letter ends the cluster and takes the next token when nothing is attached.
#[test]
fn a_required_value_letter_takes_the_rest_or_the_next_token() {
    let detached: ParsedOptions = parse_synthetic(&["-am", "msg", "x"]).expect("valid");
    assert_eq!(
        detached.occurrences,
        vec![
            Occurrence {
                id: ALL,
                negated: false,
                value: None,
                token: 0
            },
            Occurrence {
                id: MESSAGE,
                negated: false,
                value: Some(OptionValue::Detached { token: 1 }),
                token: 0
            },
        ]
    );
    assert_eq!(detached.leading, vec![2]);
    let attached: ParsedOptions = parse_synthetic(&["-amhello", "x"]).expect("valid");
    assert_eq!(
        attached.occurrences[1].value,
        Some(OptionValue::Attached {
            token: 0,
            offset: 3
        })
    );
    assert_eq!(attached.leading, vec![1]);
    // `-ma`: `a` is the message, not `--all`.
    let value_letter: ParsedOptions = parse_synthetic(&["-ma"]).expect("valid");
    assert!(!is_stated(&value_letter, ALL));
    assert!(is_enabled(&value_letter, MESSAGE));
}

/// An optional-value letter takes attached bytes only,
///  never the next token.
#[test]
fn an_optional_value_letter_never_takes_the_next_token() {
    let attached: ParsedOptions = parse_synthetic(&["-ua", "x"]).expect("valid");
    assert!(!is_stated(&attached, ALL));
    assert_eq!(
        attached.occurrences[0].value,
        Some(OptionValue::Attached {
            token: 0,
            offset: 2
        })
    );
    assert_eq!(attached.leading, vec![1]);
    let bare: ParsedOptions = parse_synthetic(&["-au", "x"]).expect("valid");
    assert!(is_enabled(&bare, ALL) && is_enabled(&bare, UNTRACKED));
    assert_eq!(bare.occurrences[1].value, None);
    assert_eq!(bare.leading, vec![1]);
}

/// Flag letters combine in any order.
#[test]
fn flag_letters_combine() {
    let flags: ParsedOptions = parse_synthetic(&["-na", "-an"]).expect("valid");
    assert_eq!(flags.occurrences.len(), 4);
    assert!(is_enabled(&flags, NO_VERIFY) && is_enabled(&flags, ALL));
}

/// Git refuses letters that spell a long option (`check_typos`,
///  parse-options.c:622-640).
#[test]
fn refuses_single_dash_long_spellings() {
    for values in [
        vec!["-all"],
        vec!["-amend"],
        vec!["-no-verify"],
        vec!["-hard"],
    ] {
        assert_eq!(
            parse_synthetic(values.as_slice()),
            refusal(OptionErrorKind::SingleDashLongOption),
            "{values:?}"
        );
    }
    // Two letters are too short for the typo check, and a value is not letters.
    assert!(parse_synthetic(&["-an"]).is_ok());
    assert!(parse_synthetic(&["-mall"]).is_ok());
    assert!(parse_synthetic(&["-uall"]).is_ok());
}

/// A lone `-h` asks for usage even when a row declares `h`;
///  with other arguments the
/// declared row is used (parse-options.c:1049-1051).
#[test]
fn a_lone_h_is_help_even_when_declared() {
    let table: &[OptionSpec] = &[row(1, Some(b'h'), Some("human"), Arity::None, true)];
    assert_eq!(
        parse_options(os_arguments(&["-h"]).as_slice(), table, DEFAULT_MODE, &[]),
        refusal(OptionErrorKind::HelpRequested)
    );
    let declared: ParsedOptions = parse_options(
        os_arguments(&["-h", "x"]).as_slice(),
        table,
        DEFAULT_MODE,
        &[],
    )
    .expect("declared letter");
    assert_eq!(declared.occurrences.len(), 1);
    assert_eq!(declared.leading, vec![1]);
}

/// An undeclared `h` is a usage request;
///  any other undeclared letter is refused.
#[test]
fn reads_help_and_refuses_unknown_letters() {
    for values in [vec!["-h"], vec!["-ah"], vec!["-h", "x"]] {
        assert_eq!(
            parse_synthetic(values.as_slice()),
            refusal(OptionErrorKind::HelpRequested),
            "{values:?}"
        );
    }
    for values in [vec!["-ax"], vec!["-xall"], vec!["-x"], vec!["-H"]] {
        assert_eq!(
            parse_synthetic(values.as_slice()),
            refusal(OptionErrorKind::UnknownOption),
            "{values:?}"
        );
    }
}
