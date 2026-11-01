//! Controls for the merge command-line reader.

use super::*;

/// The arguments as the operating-system strings the wrapper holds.
fn region(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(OsString::from).collect();
}

/// The supported shape of a command line, or a failure naming it.
fn supported(words: &[&str]) -> MergeShape {
    match parse_merge_shape(&region(words)) {
        ParsedMerge::Supported(shape) => return shape,
        ParsedMerge::Unsupported(reason) => panic!("{words:?} should be supported but: {reason}"),
    }
}

/// The reason a command line is forwarded, or a failure naming it.
fn unsupported(words: &[&str]) -> String {
    match parse_merge_shape(&region(words)) {
        ParsedMerge::Supported(shape) => panic!("{words:?} should be forwarded but read as {shape:?}"),
        ParsedMerge::Unsupported(reason) => return reason,
    }
}

/// One branch name is the whole command.
#[test]
fn a_bare_branch_is_supported_with_every_default() {
    let shape: MergeShape = supported(&["feature"]);
    assert_eq!(shape.reference, OsString::from("feature"));
    assert_eq!(shape.message, None);
    assert_eq!(shape.fast_forward, FastForwardChoice::Default);
    assert_eq!(shape.signing, Signing::Default);
    assert!(!shape.no_edit && !shape.quiet);
    assert_eq!(shape.stat, None);
    assert_eq!(shape.log, None);
    assert!(!shape.allow_unrelated_histories);
}

/// The message options Git accepts for one value.
#[test]
fn every_spelling_of_the_message_reads_the_same_bytes() {
    for words in [
        vec!["-m", "text", "feature"],
        vec!["-mtext", "feature"],
        vec!["--message=text", "feature"],
        vec!["--message", "text", "feature"],
        vec!["feature", "-m", "text"],
    ] {
        assert_eq!(supported(&words).message, Some(b"text\n".to_vec()), "{words:?}");
    }
}

/// Git joins several values with a blank line, ending each in a line feed.
#[test]
fn several_messages_are_joined_by_a_blank_line() {
    let shape: MergeShape = supported(&["-m", "one", "-m", "two\n", "-m", "three", "feature"]);
    assert_eq!(shape.message, Some(b"one\n\ntwo\n\nthree\n".to_vec()));
}

/// A message with non-UTF-8 bytes keeps them.
#[test]
fn a_message_keeps_its_bytes() {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        let mut words: Vec<OsString> = vec![OsString::from("-m"), OsString::from_vec(vec![0xff, b'x'])];
        words.push(OsString::from("feature"));
        let ParsedMerge::Supported(shape) = parse_merge_shape(&words) else {
            panic!("a binary message is a message");
        };
        assert_eq!(shape.message, Some(vec![0xff, b'x', b'\n']));
    }
}

/// The fast-forward, edit, quiet, stat and unrelated-history switches.
#[test]
fn the_switches_are_read() {
    assert_eq!(supported(&["--no-ff", "f"]).fast_forward, FastForwardChoice::Never);
    assert_eq!(supported(&["--ff", "f"]).fast_forward, FastForwardChoice::Allowed);
    assert!(supported(&["--no-edit", "f"]).no_edit);
    assert!(supported(&["-q", "f"]).quiet);
    assert!(supported(&["--quiet", "f"]).quiet);
    assert_eq!(supported(&["--no-stat", "f"]).stat, Some(false));
    assert_eq!(supported(&["-n", "f"]).stat, Some(false));
    assert_eq!(supported(&["--stat", "f"]).stat, Some(true));
    assert!(supported(&["--allow-unrelated-histories", "f"]).allow_unrelated_histories);
    assert_eq!(supported(&["--log", "f"]).log, Some(true));
    assert_eq!(supported(&["--no-log", "f"]).log, Some(false));
}

/// Output-only options change nothing about the commit.
#[test]
fn output_only_options_are_accepted_and_ignored() {
    for option in ["-v", "--verbose", "--no-progress", "--no-verify", "--no-signoff", "--no-autostash"] {
        let shape: MergeShape = supported(&[option, "feature"]);
        assert_eq!(shape.reference, OsString::from("feature"), "{option}");
    }
}

/// Every signing spelling.
#[test]
fn signing_options_are_read() {
    assert_eq!(supported(&["-S", "f"]).signing, Signing::Sign(None));
    assert_eq!(supported(&["--gpg-sign", "f"]).signing, Signing::Sign(None));
    assert_eq!(supported(&["-SABCD", "f"]).signing, Signing::Sign(Some(b"ABCD".to_vec())));
    assert_eq!(supported(&["--gpg-sign=ABCD", "f"]).signing, Signing::Sign(Some(b"ABCD".to_vec())));
    assert_eq!(supported(&["--no-gpg-sign", "f"]).signing, Signing::Never);
}

/// Shapes whose result the wrapper does not reproduce are forwarded.
#[test]
fn shapes_that_cannot_be_made_exact_are_forwarded() {
    for words in [
        vec!["--squash", "f"],
        vec!["--no-commit", "f"],
        vec!["--commit", "f"],
        vec!["-s", "ours", "f"],
        vec!["--strategy=ours", "f"],
        vec!["-X", "theirs", "f"],
        vec!["--ff-only", "f"],
        vec!["--edit", "f"],
        vec!["-e", "f"],
        vec!["--signoff", "f"],
        vec!["--cleanup=strip", "f"],
        vec!["--autostash", "f"],
        vec!["--verify-signatures", "f"],
        vec!["--abort"],
        vec!["--continue"],
        vec!["--quit"],
        vec!["-F", "file", "f"],
        vec!["--no-e", "f"],
        vec!["--log=3", "f"],
        vec!["-m", "x", "--log", "f"],
        vec!["f", "g"],
        vec![],
        vec!["--"],
    ] {
        let reason: String = unsupported(&words);
        assert!(!reason.is_empty(), "{words:?}");
    }
}

/// An option value that is not UTF-8 is not guessed at.
#[test]
fn an_option_that_is_not_text_is_forwarded() {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        let words: Vec<OsString> = vec![OsString::from_vec(vec![b'-', 0xff]), OsString::from("f")];
        assert!(matches!(parse_merge_shape(&words), ParsedMerge::Unsupported(_)));
    }
}

/// A branch name that is not UTF-8 is still one branch.
#[test]
fn a_reference_that_is_not_text_is_accepted() {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        let words: Vec<OsString> = vec![OsString::from_vec(vec![b'b', 0xff])];
        let ParsedMerge::Supported(shape) = parse_merge_shape(&words) else {
            panic!("a binary branch name is a branch name");
        };
        assert_eq!(shape.reference.as_encoded_bytes(), &[b'b', 0xff]);
    }
}
