//! Controls for the words a diagnostic line is made of:
//! one line per record,
//! the exact values the two switches answer to,
//! and the userinfo of a URL in a message staying off the line.
//!
//! They came with the words out of `diagnostic_log_tests.rs`,
//! which keeps the control of the door over them:
//! a verbose debug line and a warning reaching standard error with the userinfo masked.

use super::*;

/// The user and the password of the URL in the fake Git output;
/// neither is a real credential.
const FAKE_USER: &str = "fake-user";
/// The fake password,
/// short enough that the repository scanner's `url-userinfo-credential` rule does not match it.
const FAKE_PASSWORD: &str = "fakepass1";

/// What Git says when a remote refuses:
/// its words carry the remote's URL with userinfo.
///
/// A copy of the function of the same name beside the door's own control:
/// the two controls sit in two sidecars after the words moved out of the door's file,
/// and neither sidecar reaches the other's items.
fn git_words_naming_a_remote() -> String {
    return format!(
        "git fetch said: fatal: unable to access 'https://{FAKE_USER}:{FAKE_PASSWORD}@example.invalid/team/repo.git/': 403"
    );
}

/// A diagnostic is one line,
///  whatever its message holds.
#[test]
fn a_diagnostic_is_one_line() {
    assert_eq!(
        diagnostic_line(
            "debug",
            "2026-10-06T21:40:39.524Z",
            "recovery",
            "owner 7 alive"
        ),
        "[debug] [2026-10-06T21:40:39.524Z] [monochromatic-git-policy-cli] [recovery] owner 7 alive\n"
    );
    assert_eq!(
        diagnostic_line("warn", "t", "capture", "a\nb\r\nc\u{1b}[31m"),
        "[warn] [t] [monochromatic-git-policy-cli] [capture] a b  c [31m\n"
    );
}

/// Only the exact values switch the levels.
#[test]
fn the_switches_need_exact_values() {
    assert!(is_true(std::ffi::OsString::from("true")));
    assert!(!is_true(std::ffi::OsString::from("TRUE")));
    assert!(!is_true(std::ffi::OsString::from("1")));
    assert!(is_false(std::ffi::OsString::from("false")));
    assert!(!is_false(std::ffi::OsString::from("0")));
}

/// The userinfo of a URL in the message never reaches the line,
/// and the rest of the message stays.
#[test]
fn a_diagnostic_line_hides_the_userinfo_of_a_url_in_its_message() {
    for level in ["debug", "warn"] {
        let line: String =
            diagnostic_line(level, "t", "fetch", git_words_naming_a_remote().as_str());
        assert!(!line.contains(FAKE_USER), "{line}");
        assert!(!line.contains(FAKE_PASSWORD), "{line}");
        assert!(
            line.contains(
                format!(
                    "'https://{}@example.invalid/team/repo.git/': 403",
                    crate::lfs_output_mask::CREDENTIALS_MASK
                )
                .as_str()
            ),
            "{line}"
        );
    }
}

/// A control character inside the userinfo does not carry the rest of it past the mask.
#[test]
fn a_diagnostic_line_hides_userinfo_that_holds_a_control_character() {
    let line: String = diagnostic_line(
        "debug",
        "t",
        "fetch",
        format!("https://{FAKE_USER}\u{85}{FAKE_PASSWORD}@example.invalid/").as_str(),
    );
    assert!(!line.contains(FAKE_USER), "{line}");
    assert!(!line.contains(FAKE_PASSWORD), "{line}");
}

/// What:
///  The sentence about a stopped reader names the bound the queue gave it,
///  so a person reads how long the reader was given.
/// Why:
///   The words of that line are what the queue says instead of the lines that reached nobody,
///  and the bound is the one thing in them this process chose.
#[test]
fn the_sentence_about_a_stopped_reader_names_the_bound_it_was_given() {
    /// What every reading of the sentence starts with.
    const BEFORE_THE_BOUND: &str = "the reader of this queue's stream took no line within ";
    /// What every reading of the sentence ends with.
    const AFTER_THE_BOUND: &str = ", so the command stops waiting for it";
    for within in [
        std::time::Duration::from_millis(250),
        std::time::Duration::from_secs(2),
    ] {
        let said: String = the_reader_stopped_sentence(within);
        assert!(
            said.starts_with(BEFORE_THE_BOUND),
            "the sentence says which reader stopped: {said}"
        );
        assert!(
            said.contains(&format!("{within:?}")),
            "the sentence names the bound the queue held the reader to: {said}"
        );
        assert!(
            said.ends_with(AFTER_THE_BOUND),
            "the sentence says what the command does next: {said}"
        );
    }
}
