//! What:
//!  Every `argv.unit.test.ts` case ported to the tokenizer,
//!  plus boundaries and
//!       wrapper-only flags.
//! Why:
//!  The incumbent test is the behavior specification;
//!  cases where it disagrees with
//!      Git 2.56.0 are marked as divergences and follow Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseArgv({ args, spec })).toEqual(...)
//! ```

/// The tokenizer under test,
///  its queries and the shared argument builders.
use super::{
    Arity, Boundary, DEFAULT_MODE, Occurrence, OptionError, OptionErrorKind, OptionSpec,
    OptionValue, ParseMode, ParsedOptions, WrapperOccurrence, parse_options, row,
};
use crate::command_options_query::{positional_tokens, value_bytes, without_tokens};
use crate::command_test_support::{
    ALL, AMEND, MESSAGE, SYNTHETIC_TABLE, assert_table_invariants, os_arguments, parse_synthetic,
};
use std::ffi::OsString;

/// The declared surface of the incumbent `argv.unit.test.ts`:
///  one flag and one value option.
const INCUMBENT_TABLE: &[OptionSpec] = &[
    row(ALL, Some(b'a'), Some("all"), Arity::None, true),
    row(MESSAGE, Some(b'm'), Some("message"), Arity::Required, true),
];

/// Git's `PARSE_OPT_KEEP_UNKNOWN_OPT`.
const KEEP_UNKNOWN: ParseMode = ParseMode {
    keep_unknown: true,
    stop_at_non_option: false,
};

/// Tokenize against the incumbent test's two-row table under one mode.
fn parse(values: &[&str], mode: ParseMode) -> Result<ParsedOptions, OptionError> {
    return parse_options(os_arguments(values).as_slice(), INCUMBENT_TABLE, mode, &[]);
}

/// Shorthand for one expected message occurrence whose value is the next token.
fn message_from_next(token: usize) -> Occurrence {
    return Occurrence {
        id: MESSAGE,
        negated: false,
        value: Some(OptionValue::Detached { token: token + 1 }),
        token,
    };
}

/// Shorthand for one expected `-a` occurrence.
fn all_at(token: usize) -> Occurrence {
    return Occurrence {
        id: ALL,
        negated: false,
        value: None,
        token,
    };
}

/// Ported:
///  "takes a declared option value and keeps following positionals".
#[test]
fn takes_a_declared_option_value_and_keeps_following_positionals() {
    let parsed: ParsedOptions = parse(&["-m", "msg", "a.txt"], DEFAULT_MODE).expect("valid");
    assert_eq!(parsed.occurrences, vec![message_from_next(0)]);
    assert_eq!(parsed.leading, vec![2]);
}

/// Ported:
///  "counts a declared flag ..." and "counts every repeated declared flag occurrence".
#[test]
fn counts_declared_flags_without_consuming_positionals() {
    let single: ParsedOptions = parse(&["-a", "a.txt", "b.txt"], DEFAULT_MODE).expect("valid");
    assert_eq!(single.occurrences, vec![all_at(0)]);
    assert_eq!(single.leading, vec![1, 2]);
    let repeated: ParsedOptions = parse(&["-a", "-a"], DEFAULT_MODE).expect("valid");
    assert_eq!(repeated.occurrences, vec![all_at(0), all_at(1)]);
    assert!(repeated.leading.is_empty());
}

/// Divergence from five incumbent cases:
///  Git 2.56.0 refuses an undeclared option;
///  it never
/// lets one consume a following plain token (parse-options.c:1164-1165,
///  1224-1233).
#[test]
fn refuses_an_undeclared_option_as_git_does() {
    for values in [
        vec!["-q", "a.txt"],
        vec!["-q", "-a"],
        vec!["-q", "--", "a.txt"],
        vec!["-q"],
        vec!["--unknown=v", "a.txt"],
        vec!["--unknown"],
    ] {
        assert_eq!(
            parse(values.as_slice(), DEFAULT_MODE),
            Err(OptionError {
                kind: OptionErrorKind::UnknownOption,
                token: 0
            }),
            "{values:?}"
        );
    }
    assert_eq!(
        parse(&["-a", "-aq"], DEFAULT_MODE),
        Err(OptionError {
            kind: OptionErrorKind::UnknownOption,
            token: 1
        })
    );
}

/// Under Git's keep-unknown flag an undeclared option is kept and consumes nothing.
#[test]
fn keeps_an_undeclared_option_without_consuming_the_next_token() {
    for values in [vec!["-q", "a.txt"], vec!["--unknown=v", "a.txt"]] {
        let parsed: ParsedOptions = parse(values.as_slice(), KEEP_UNKNOWN).expect("kept");
        assert_eq!(parsed.unknown, vec![0], "{values:?}");
        assert_eq!(parsed.leading, vec![1], "{values:?}");
    }
    // A known letter before an unknown one still takes effect; the token is kept once.
    let mixed: ParsedOptions = parse(&["-aq"], KEEP_UNKNOWN).expect("kept");
    assert_eq!(mixed.occurrences, vec![all_at(0)]);
    assert_eq!(mixed.unknown, vec![0]);
    // Abbreviations are disabled under keep-unknown (parse-options.c:502-503).
    let abbreviated: ParsedOptions =
        parse(&["--mess=x", "--message=y"], KEEP_UNKNOWN).expect("kept");
    assert_eq!(abbreviated.unknown, vec![0]);
    assert_eq!(abbreviated.occurrences.len(), 1);
    // Without the flag the same abbreviation is accepted.
    let plain: ParsedOptions = parse(&["--mess=x"], DEFAULT_MODE).expect("valid");
    assert_eq!(plain.occurrences.len(), 1);
}

/// Ported:
///  "takes a declared joined option value";
///  also the empty joined value.
#[test]
fn takes_a_declared_joined_option_value() {
    let arguments: Vec<OsString> = os_arguments(&["--message=msg", "a.txt", "--message="]);
    let parsed: ParsedOptions =
        parse_options(arguments.as_slice(), INCUMBENT_TABLE, DEFAULT_MODE, &[]).expect("valid");
    let first: OptionValue = parsed.occurrences[0].value.expect("joined value");
    assert_eq!(
        first,
        OptionValue::Attached {
            token: 0,
            offset: 10
        }
    );
    assert_eq!(value_bytes(arguments.as_slice(), first), b"msg");
    let empty: OptionValue = parsed.occurrences[1].value.expect("empty joined value");
    assert_eq!(value_bytes(arguments.as_slice(), empty), b"");
    assert_eq!(parsed.leading, vec![1]);
}

/// Ported:
///  "names the offending token and region when refusing" and
/// "refuses a declared option missing its value".
#[test]
fn names_the_offending_token_when_a_value_is_missing() {
    for (values, token) in [
        (vec!["-a", "-m"], 1),
        (vec!["-m"], 0),
        (vec!["--message"], 0),
    ] {
        assert_eq!(
            parse(values.as_slice(), DEFAULT_MODE),
            Err(OptionError {
                kind: OptionErrorKind::MissingValue,
                token
            }),
            "{values:?}"
        );
    }
    let error: OptionError = parse(&["-a", "-m"], DEFAULT_MODE).expect_err("refused");
    assert!(error.to_string().contains("argument 1"), "{error}");
    assert!(error.to_string().contains("MissingValue"), "{error}");
}

/// Ported:
///  "takes a dash-led token as a declared option value".
#[test]
fn takes_a_dash_led_token_as_a_declared_option_value() {
    let parsed: ParsedOptions = parse(&["-m", "-a"], DEFAULT_MODE).expect("valid");
    assert_eq!(parsed.occurrences, vec![message_from_next(0)]);
    let terminator: ParsedOptions = parse(&["-m", "--", "x"], DEFAULT_MODE).expect("valid");
    assert_eq!(terminator.boundary, Boundary::End);
    assert_eq!(terminator.leading, vec![2]);
}

/// Ported:
///  "parses declared options appearing after positionals",
///  "treats a lone dash as
/// positional",
///  "treats every token after the terminator as positional" and
/// "parses declared flags before the terminator".
#[test]
fn separates_options_positionals_and_the_terminator() {
    let after: ParsedOptions = parse(&["a.txt", "-m", "msg"], DEFAULT_MODE).expect("valid");
    assert_eq!(after.leading, vec![0]);
    assert_eq!(after.occurrences, vec![message_from_next(1)]);
    let dash: ParsedOptions = parse(&["-", ""], DEFAULT_MODE).expect("valid");
    assert_eq!(dash.leading, vec![0, 1]);
    let terminated: ParsedOptions = parse(&["--", "-a"], DEFAULT_MODE).expect("valid");
    assert!(terminated.occurrences.is_empty());
    assert_eq!(terminated.boundary, Boundary::DashDash(0));
    assert_eq!(positional_tokens(&terminated, 2), vec![1]);
    let before: ParsedOptions = parse(&["-a", "--", "a.txt"], DEFAULT_MODE).expect("valid");
    assert_eq!(before.occurrences, vec![all_at(0)]);
    assert_eq!(positional_tokens(&before, 3), vec![2]);
}

/// Ported as a table invariant:
///  "rejects a declared spelling that cannot introduce an option".
#[test]
#[should_panic(expected = "nodash")]
fn rejects_a_declared_spelling_that_cannot_introduce_an_option() {
    assert_table_invariants(&[row(ALL, None, Some("-nodash"), Arity::None, true)]);
}

/// Duplicate letters are table defects,
///  as in Git's `parse_options_check`.
#[test]
#[should_panic(expected = "duplicate short")]
fn rejects_a_table_with_a_repeated_short_letter() {
    assert_table_invariants(&[
        row(ALL, Some(b'a'), Some("all"), Arity::None, true),
        row(AMEND, Some(b'a'), Some("amend"), Arity::None, true),
    ]);
}

/// The synthetic tables satisfy the invariants they are used to demonstrate.
#[test]
fn synthetic_tables_are_well_formed() {
    assert_table_invariants(SYNTHETIC_TABLE);
    assert_table_invariants(INCUMBENT_TABLE);
}

/// `--end-of-options` and stop-at-non-option end option parsing at different tokens.
#[test]
fn reports_each_boundary_kind() {
    let end: ParsedOptions = parse_synthetic(&["--end-of-options", "-a"]).expect("valid");
    assert_eq!(end.boundary, Boundary::EndOfOptions(0));
    assert_eq!(positional_tokens(&end, 2), vec![1]);
    let stop: ParseMode = ParseMode {
        keep_unknown: false,
        stop_at_non_option: true,
    };
    let stopped: ParsedOptions = parse(&["-a", "file", "-a", "--"], stop).expect("valid");
    assert_eq!(stopped.boundary, Boundary::NonOption(1));
    assert_eq!(stopped.occurrences, vec![all_at(0)]);
    assert_eq!(positional_tokens(&stopped, 4), vec![1, 2, 3]);
    let complete: ParsedOptions = parse_synthetic(&["x", "-a"]).expect("valid");
    assert_eq!(complete.boundary, Boundary::End);
    assert_eq!(positional_tokens(&complete, 2), vec![0]);
}

/// A wrapper-only flag counts only where Git would look for an option.
#[test]
fn finds_wrapper_flags_in_option_position_only() {
    let arguments: Vec<OsString> = os_arguments(&[
        "--no-enforce-only",
        "-m",
        "--no-enforce-only",
        "--cli-git-keep-going",
        "--",
        "--no-enforce-only",
    ]);
    let wrapper: &[&[u8]] = &[b"--cli-git-keep-going", b"--no-enforce-only"];
    let parsed: ParsedOptions =
        parse_options(arguments.as_slice(), SYNTHETIC_TABLE, DEFAULT_MODE, wrapper).expect("ok");
    assert_eq!(
        parsed.wrapper,
        vec![
            WrapperOccurrence { flag: 1, token: 0 },
            WrapperOccurrence { flag: 0, token: 3 },
        ]
    );
    assert_eq!(positional_tokens(&parsed, arguments.len()), vec![5]);
    // Without the wrapper list the same token is an option Git does not know.
    assert_eq!(
        parse_synthetic(&["--no-enforce-only"]),
        Err(OptionError {
            kind: OptionErrorKind::UnknownOption,
            token: 0
        })
    );
    assert_eq!(
        parse_synthetic(&["-a"]).expect("ok").wrapper,
        Vec::<WrapperOccurrence>::new()
    );
    // Removal is by position: the message value and the path keep their spelling.
    let mut full: Vec<OsString> = os_arguments(&["-C", "/r", "commit"]);
    full.extend(arguments.clone());
    assert_eq!(
        without_tokens(full.as_slice(), 3, &[0, 3]),
        os_arguments(&[
            "-C",
            "/r",
            "commit",
            "-m",
            "--no-enforce-only",
            "--",
            "--no-enforce-only"
        ])
    );
}

/// Bytes that are not UTF-8 pass through as values and paths without being decoded.
#[cfg(unix)]
#[test]
fn keeps_non_utf8_bytes_in_values_and_paths() {
    use crate::command_test_support::byte_argument;
    let arguments: Vec<OsString> = vec![
        byte_argument(b"-m\xff\xfe"),
        byte_argument(b"--message=\xff"),
        byte_argument(b"path-\xff"),
        OsString::from("-m"),
        byte_argument(b"\x80"),
    ];
    let snapshot: Vec<OsString> = arguments.clone();
    let parsed: ParsedOptions =
        parse_options(arguments.as_slice(), SYNTHETIC_TABLE, DEFAULT_MODE, &[]).expect("ok");
    let values: Vec<&[u8]> = vec![
        value_bytes(
            arguments.as_slice(),
            parsed.occurrences[0].value.expect("v"),
        ),
        value_bytes(
            arguments.as_slice(),
            parsed.occurrences[1].value.expect("v"),
        ),
        value_bytes(
            arguments.as_slice(),
            parsed.occurrences[2].value.expect("v"),
        ),
    ];
    assert_eq!(values, vec![&b"\xff\xfe"[..], &b"\xff"[..], &b"\x80"[..]]);
    assert_eq!(parsed.leading, vec![2]);
    assert_eq!(arguments, snapshot);
    // A non-ASCII byte in letter position is an unknown switch, not a panic.
    assert_eq!(
        parse_options(
            &[byte_argument(b"-\xff")],
            SYNTHETIC_TABLE,
            DEFAULT_MODE,
            &[]
        ),
        Err(OptionError {
            kind: OptionErrorKind::UnknownOption,
            token: 0
        })
    );
}
