//! What:
//!  Controls for the command's grammar:
//!  one positional slot name,
//!  `--help`,
//!  and every caller mistake with its own sentence and exit `2`.
//! Why:
//!  A person who typed the wrong word must learn the shape before any record is read,
//!  and a name that is no slot name is one of the caller's mistakes
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  "The grammar" and "Exit codes and streams").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(parseCancelArguments(['one'])).toEqual({ kind: 'cancel', slot: 'one' });
//! ```

use super::{
    CANCEL_COMMAND_WORD,
    CancelInvocation,
    CancelMistake,
    parse_cancel_arguments,
    usage,
};
use std::ffi::OsString;

/// The words as the argument vector holds them.
fn words(arguments: &[&str]) -> Vec<OsString> {
    return arguments
        .iter()
        .map(|word| return OsString::from(word))
        .collect();
}

#[test]
fn one_slot_name_is_the_request() {
    match parse_cancel_arguments(words(&["one"]).as_slice())
        .expect("a plain slot name is the request")
    {
        CancelInvocation::Cancel(request) => assert_eq!(request.slot, "one"),
        other => panic!("a plain slot name must parse as a request, not {other:?}"),
    }
}

#[test]
fn the_help_is_printed_alone() {
    assert_eq!(
        parse_cancel_arguments(words(&["--help"]).as_slice()).expect("the help"),
        CancelInvocation::Help
    );
    assert_eq!(
        parse_cancel_arguments(words(&["-h"]).as_slice()).expect("the short help"),
        CancelInvocation::Help
    );
    let usage: String = usage();
    assert!(usage.contains(CANCEL_COMMAND_WORD), "{usage}");
    assert!(usage.contains("<slot>"), "{usage}");
}

#[test]
fn the_help_with_a_word_after_it_is_a_mistake() {
    match parse_cancel_arguments(words(&["--help", "one"]).as_slice())
        .expect_err("the help takes nothing after it")
    {
        CancelMistake::TooManyArguments {
            first,
            second,
        } => {
            assert_eq!(first, "--help");
            assert_eq!(second, "one");
        }
        other => panic!("a word after the help is too many arguments, not {other:?}"),
    }
}

#[test]
fn two_dashes_end_the_words_and_a_second_name_is_refused() {
    match parse_cancel_arguments(words(&["--", "-one"]).as_slice())
        .expect("a name after `--` is the slot")
    {
        CancelInvocation::Cancel(request) => assert_eq!(request.slot, "-one"),
        other => panic!("`--` then one name is a request, not {other:?}"),
    }
    match parse_cancel_arguments(words(&["--", "one", "two"]).as_slice())
        .expect_err("two names are too many")
    {
        CancelMistake::TooManyArguments {
            first,
            second,
        } => {
            assert_eq!(first, "one");
            assert_eq!(second, "two");
        }
        other => panic!("two names after `--` are too many arguments, not {other:?}"),
    }
}

#[test]
fn no_argument_at_all_is_a_mistake() {
    assert_eq!(
        parse_cancel_arguments(words(&[]).as_slice()).expect_err("no name"),
        CancelMistake::NoArgument
    );
    assert_eq!(
        parse_cancel_arguments(words(&["--"]).as_slice()).expect_err("no name after `--`"),
        CancelMistake::NoArgument
    );
}

#[test]
fn a_second_positional_is_one_mistake() {
    match parse_cancel_arguments(words(&["one", "two"]).as_slice())
        .expect_err("one slot is the whole grammar")
    {
        CancelMistake::TooManyArguments {
            first,
            second,
        } => {
            assert_eq!(first, "one");
            assert_eq!(second, "two");
        }
        other => panic!("a second positional is too many arguments, not {other:?}"),
    }
}

#[test]
fn an_option_is_a_mistake_because_the_command_takes_none() {
    for option in ["--force", "-f", "--prune"] {
        match parse_cancel_arguments(words(&[option]).as_slice())
            .expect_err("this command takes no option")
        {
            CancelMistake::AnOption {
                option: named,
            } => assert_eq!(named, option),
            other => panic!("`{option}` is an option, not {other:?}"),
        }
    }
}

#[test]
fn every_name_the_slot_rule_refuses_is_a_mistake() {
    for (name, reason) in [
        ("", "it is empty"),
        (".", "a path separator or names a folder above"),
        ("..", "a path separator or names a folder above"),
        ("a/b", "a path separator or names a folder above"),
        (".hidden", "it starts with a dot"),
        ("two words", "it holds a space"),
    ] {
        match parse_cancel_arguments(words(&[name]).as_slice())
            .expect_err("a name that is no slot name")
        {
            CancelMistake::NotASlotName {
                slot,
                reason: said,
            } => {
                assert_eq!(slot, name);
                assert!(
                    said.contains(reason),
                    "the refusal of `{name}` says why: {said}"
                );
            }
            other => panic!("`{name}` is not a slot name, not {other:?}"),
        }
    }
}

#[test]
fn an_argument_that_is_not_utf8_is_no_slot_name() {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::ffi::OsStringExt;
        let raw: OsString = OsString::from_vec(vec![b'a', 0xff, b'b']);
        match parse_cancel_arguments(&[raw]).expect_err("a name that is not UTF-8 is no slot name")
        {
            CancelMistake::NotASlotName {
                reason,
                ..
            } => assert!(reason.contains("not UTF-8"), "{reason}"),
            other => panic!("a name that is not UTF-8 is not a slot name, not {other:?}"),
        }
    }
}

#[test]
fn every_mistake_names_the_shape_the_command_takes() {
    let mistakes: [CancelMistake; 4] = [
        CancelMistake::NoArgument,
        CancelMistake::TooManyArguments {
            first: String::from("one"),
            second: String::from("two"),
        },
        CancelMistake::AnOption {
            option: String::from("--force"),
        },
        CancelMistake::NotASlotName {
            slot: String::from(".hidden"),
            reason: String::from("it starts with a dot"),
        },
    ];
    for mistake in mistakes {
        let said: String = mistake.message();
        assert!(said.contains(CANCEL_COMMAND_WORD), "{said}");
        assert!(said.contains("<slot>"), "{said}");
    }
}
