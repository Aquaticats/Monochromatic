//! Controls for diagnostic line formatting.

use super::*;

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

/// The user and the password of the URL in the fake Git output; neither is a real credential.
const FAKE_USER: &str = "fake-user";
/// The fake password, short enough that the repository scanner's `url-userinfo-credential` rule does not match it.
const FAKE_PASSWORD: &str = "fakepass1";

/// What Git says when a remote refuses: its words carry the remote's URL with userinfo.
fn git_words_naming_a_remote() -> String {
    return format!("git fetch said: fatal: unable to access 'https://{FAKE_USER}:{FAKE_PASSWORD}@example.invalid/team/repo.git/': 403");
}

/// The userinfo of a URL in the message never reaches the line, and the rest of the message stays.
#[test]
fn a_diagnostic_line_hides_the_userinfo_of_a_url_in_its_message() {
    for level in ["debug", "warn"] {
        let line: String = diagnostic_line(level, "t", "fetch", git_words_naming_a_remote().as_str());
        assert!(!line.contains(FAKE_USER), "{line}");
        assert!(!line.contains(FAKE_PASSWORD), "{line}");
        assert!(
            line.contains(format!("'https://{}@example.invalid/team/repo.git/': 403", crate::lfs_output_mask::CREDENTIALS_MASK).as_str()),
            "{line}"
        );
    }
}

/// A control character inside the userinfo does not carry the rest of it past the mask.
#[test]
fn a_diagnostic_line_hides_userinfo_that_holds_a_control_character() {
    let line: String = diagnostic_line("debug", "t", "fetch", format!("https://{FAKE_USER}\u{85}{FAKE_PASSWORD}@example.invalid/").as_str());
    assert!(!line.contains(FAKE_USER), "{line}");
    assert!(!line.contains(FAKE_PASSWORD), "{line}");
}

/// The variable that marks the child process of the stream controls.
const CHILD_VARIABLE: &str = "CLI_GIT_DIAGNOSTIC_CHILD";

/// What the child prints, so the parent can tell that its stream was read.
const CHILD_TAG: &str = "streamControl";

/// The body the parent starts in a child: it prints one debug and one warning line carrying Git's words.
/// Run without the child's variable it returns at once and checks nothing.
#[test]
fn child_prints_a_debug_and_a_warning_line_naming_a_remote() {
    if std::env::var_os(CHILD_VARIABLE).is_none() {
        return;
    }
    debug(CHILD_TAG, git_words_naming_a_remote().as_str());
    warn(CHILD_TAG, git_words_naming_a_remote().as_str());
}

/// Standard error of a child that runs the test binary on `child_prints_a_debug_and_a_warning_line_naming_a_remote`,
///  with the verbose switch set or removed.
fn child_stderr(verbose: bool) -> String {
    let module: &str = module_path!();
    let path: String = format!(
        "{}::child_prints_a_debug_and_a_warning_line_naming_a_remote",
        module.split_once("::").map_or(module, |(_, rest)| rest)
    );
    let mut command: std::process::Command = std::process::Command::new(crate::test_support::this_test_binary().expect("the test binary"));
    command
        .args(["--exact", path.as_str(), "--nocapture", "--test-threads=1"])
        .env(CHILD_VARIABLE, "1")
        .env_remove("MONOCHROMATIC_WARN")
        .stdin(std::process::Stdio::null());
    if verbose {
        command.env("MONOCHROMATIC_VERBOSE", "true");
    } else {
        command.env_remove("MONOCHROMATIC_VERBOSE");
    }
    let output: std::process::Output = command.output().expect("the child test runs");
    let said: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    let heard: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert!(output.status.success(), "the child failed:\n{said}\n{heard}");
    // A name that matches no test would end with success and run nothing.
    assert!(said.contains("running 1 test") && said.contains("1 passed"), "the child ran no test named {path}:\n{said}\n{heard}");
    return heard;
}

/// With the verbose switch on, the debug line Git's words ride in reaches standard error with the userinfo masked;
///  the warning line is masked too, and with the switch off only the warning appears.
#[test]
fn git_words_in_a_verbose_debug_line_reach_standard_error_without_the_userinfo() {
    let verbose: String = child_stderr(true);
    let debug_line: &str = verbose.lines().find(|line| return line.starts_with("[debug]")).expect(verbose.as_str());
    assert!(debug_line.contains(CHILD_TAG), "{verbose}");
    assert!(debug_line.contains("403"), "{verbose}");
    assert!(verbose.lines().any(|line| return line.starts_with("[warn]")), "{verbose}");
    assert!(!verbose.contains(FAKE_USER), "{verbose}");
    assert!(!verbose.contains(FAKE_PASSWORD), "{verbose}");
    assert!(verbose.contains(crate::lfs_output_mask::CREDENTIALS_MASK), "{verbose}");
    // The positive control for the switch: without it the debug line is absent, so the line above came from the switch.
    let quiet: String = child_stderr(false);
    assert!(!quiet.lines().any(|line| return line.starts_with("[debug]")), "{quiet}");
    assert!(quiet.lines().any(|line| return line.starts_with("[warn]")), "{quiet}");
    assert!(!quiet.contains(FAKE_PASSWORD), "{quiet}");
}
