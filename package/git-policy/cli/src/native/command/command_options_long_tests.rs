//! What:
//!  Long-option cases:
//!  exact names,
//!  abbreviations,
//!  ambiguity and `no-` negation.
//! Why:
//!  Git accepts `--am` for `--amend` and `--no-verify` as a positive option name;
//!  a
//!      list of exact spellings misses forms a policy must see.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseOptions(['--am'])).toEqual(...)
//! ```

/// Tokenizer types,
///  queries and the shared synthetic table.
use crate::command_options::{
    Arity, DEFAULT_MODE, Occurrence, OptionError, OptionErrorKind, OptionSpec, OptionValue,
    ParsedOptions, parse_options, row,
};
use crate::command_options_query::{is_enabled, is_stated, last_occurrence};
use crate::command_test_support::{
    ALL, ALLOW_EMPTY, ALLOW_EMPTY_MESSAGE, AMEND, NO_VERIFY, os_arguments, parse_synthetic,
};

/// Shorthand for an expected refusal at token 0.
fn refusal(kind: OptionErrorKind) -> Result<ParsedOptions, OptionError> {
    return Err(OptionError { kind, token: 0 });
}

/// A unique prefix resolves,
///  and a joined value starts after the `=` of the written name.
#[test]
fn resolves_unique_abbreviations() {
    let parsed: ParsedOptions =
        parse_synthetic(&["--am", "--mess=x", "--untracked=no"]).expect("valid");
    assert!(is_enabled(&parsed, AMEND));
    assert_eq!(
        parsed.occurrences[1].value,
        Some(OptionValue::Attached {
            token: 1,
            offset: 7
        })
    );
    assert_eq!(
        parsed.occurrences[2].value,
        Some(OptionValue::Attached {
            token: 2,
            offset: 12
        })
    );
}

/// A prefix shared by several names is refused,
///  including the empty prefix.
#[test]
fn refuses_ambiguous_abbreviations() {
    for values in [
        vec!["--a"],
        vec!["--allow"],
        vec!["--allow-"],
        vec!["--n"],
        vec!["--no"],
        vec!["--no-"],
        vec!["--no-a"],
        vec!["--=x"],
    ] {
        assert_eq!(
            parse_synthetic(values.as_slice()),
            refusal(OptionErrorKind::AmbiguousOption),
            "{values:?}"
        );
    }
}

/// An exact name wins over a longer name it prefixes,
///  in either table order.
#[test]
fn prefers_an_exact_name_over_an_abbreviation() {
    let exact: ParsedOptions =
        parse_synthetic(&["--allow-empty", "--allow-empty-m"]).expect("valid");
    assert_eq!(exact.occurrences[0].id, ALLOW_EMPTY);
    assert_eq!(exact.occurrences[1].id, ALLOW_EMPTY_MESSAGE);
    let reordered: &[OptionSpec] = &[
        row(
            ALLOW_EMPTY_MESSAGE,
            None,
            Some("allow-empty-message"),
            Arity::None,
            true,
        ),
        row(ALLOW_EMPTY, None, Some("allow-empty"), Arity::None, true),
    ];
    let later_exact: ParsedOptions = parse_options(
        os_arguments(&["--allow-empty"]).as_slice(),
        reordered,
        DEFAULT_MODE,
        &[],
    )
    .expect("exact match");
    assert!(is_enabled(&later_exact, ALLOW_EMPTY));
    assert!(!is_stated(&later_exact, ALLOW_EMPTY_MESSAGE));
}

/// `--no-<name>` turns an option off,
///  also in abbreviated form.
#[test]
fn negates_with_a_no_prefix() {
    let negated: ParsedOptions = parse_synthetic(&["--all", "--no-all", "--no-am"]).expect("ok");
    assert!(is_stated(&negated, ALL) && !is_enabled(&negated, ALL));
    assert_eq!(
        last_occurrence(&negated, AMEND),
        Some(Occurrence {
            id: AMEND,
            negated: true,
            value: None,
            token: 2
        })
    );
    let restored: ParsedOptions = parse_synthetic(&["--no-all", "--all"]).expect("ok");
    assert!(is_enabled(&restored, ALL));
}

/// A row named `no-verify` is on for `--no-verify` and off for `--verify` and `--no-no-verify`.
#[test]
fn reads_rows_whose_name_starts_with_no() {
    let on: ParsedOptions = parse_synthetic(&["--no-verify"]).expect("ok");
    assert!(is_enabled(&on, NO_VERIFY));
    let positive: ParsedOptions = parse_synthetic(&["--verify"]).expect("ok");
    assert!(is_stated(&positive, NO_VERIFY) && !is_enabled(&positive, NO_VERIFY));
    let double: ParsedOptions = parse_synthetic(&["--no-no-verify"]).expect("ok");
    assert!(is_stated(&double, NO_VERIFY) && !is_enabled(&double, NO_VERIFY));
    // `--no-no-` reaches only rows whose own name starts with `no-`.
    assert_eq!(
        parse_synthetic(&["--no-no-all"]),
        refusal(OptionErrorKind::UnknownOption)
    );
}

/// `PARSE_OPT_NONEG` rows have no negated form.
#[test]
fn refuses_negation_of_non_negatable_rows() {
    for values in [vec!["--no-hard"], vec!["--no-contains"], vec!["--no-har"]] {
        assert_eq!(
            parse_synthetic(values.as_slice()),
            refusal(OptionErrorKind::UnknownOption),
            "{values:?}"
        );
    }
    assert!(parse_synthetic(&["--hard", "--har"]).is_ok());
}

/// With one negatable row,
///  the very short negations `--n` and `--no` resolve to it.
///  `--no-`
/// stays ambiguous:
///  the row registers once as an empty abbreviation and once as a very
/// short negation,
///  and Git counts the second registration as a conflict
/// (parse-options.c:504-514,
///  564-571).
#[test]
fn resolves_a_very_short_negation_when_unique() {
    let only_all: &[OptionSpec] = &[row(ALL, Some(b'a'), Some("all"), Arity::None, true)];
    assert_eq!(
        parse_options(
            os_arguments(&["--no-"]).as_slice(),
            only_all,
            DEFAULT_MODE,
            &[]
        ),
        refusal(OptionErrorKind::AmbiguousOption)
    );
    for spelling in ["--n", "--no"] {
        let parsed: ParsedOptions = parse_options(
            os_arguments(&[spelling]).as_slice(),
            only_all,
            DEFAULT_MODE,
            &[],
        )
        .expect("unique negation");
        assert_eq!(
            parsed.occurrences,
            vec![Occurrence {
                id: ALL,
                negated: true,
                value: None,
                token: 0
            }],
            "{spelling}"
        );
    }
}

/// Names Git reserves or does not know.
#[test]
fn reads_help_and_refuses_unknown_names() {
    for values in [
        vec!["--help"],
        vec!["--help-all"],
        vec!["--git-completion-helper"],
        vec!["--git-completion-helper-all"],
    ] {
        assert_eq!(
            parse_synthetic(values.as_slice()),
            refusal(OptionErrorKind::HelpRequested),
            "{values:?}"
        );
    }
    // The completion request is special only when it is the sole argument.
    assert_eq!(
        parse_synthetic(&["-a", "--git-completion-helper"]),
        Err(OptionError {
            kind: OptionErrorKind::UnknownOption,
            token: 1
        })
    );
    for values in [vec!["--unknown"], vec!["--allx"], vec!["--message-x=1"]] {
        assert_eq!(
            parse_synthetic(values.as_slice()),
            refusal(OptionErrorKind::UnknownOption),
            "{values:?}"
        );
    }
}
