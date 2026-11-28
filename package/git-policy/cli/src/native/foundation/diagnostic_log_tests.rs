//! Controls for the door over a diagnostic line's words:
//! with the verbose switch on,
//! a debug record and a warning reach standard error with the userinfo of a URL masked,
//! and with it off only the warning appears.
//!
//! The words themselves are controlled beside the file that holds them
//! (`the_words_of_a_diagnostic_line_and_the_lines_this_thread_wrote_tests.rs`):
//! one line per record,
//! the exact values the two switches answer to,
//! and the userinfo staying off a line.

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
/// A copy of the function of the same name beside the words' own control:
/// the two controls sit in two sidecars after the words moved out of this file,
/// and neither sidecar reaches the other's items.
fn git_words_naming_a_remote() -> String {
    return format!(
        "git fetch said: fatal: unable to access 'https://{FAKE_USER}:{FAKE_PASSWORD}@example.invalid/team/repo.git/': 403"
    );
}

/// The variable that marks the child process of the stream controls.
const CHILD_VARIABLE: &str = "CLI_GIT_DIAGNOSTIC_CHILD";

/// What the child prints,
/// so the parent can tell that its stream was read.
const CHILD_TAG: &str = "streamControl";

/// The body the parent starts in a child:
/// it prints one debug and one warning line carrying Git's words.
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
        module
            .split_once("::")
            .map_or(module, |(_, rest)| return rest)
    );
    let mut environment: Vec<(&str, &std::ffi::OsStr)> =
        vec![(CHILD_VARIABLE, std::ffi::OsStr::new("1"))];
    if verbose {
        environment.push(("MONOCHROMATIC_VERBOSE", std::ffi::OsStr::new("true")));
    }
    // The warning switch is never inherited, and neither is the verbose one in the arm that must run without it.
    let mut cleared: Vec<&str> = vec!["MONOCHROMATIC_WARN"];
    if !verbose {
        cleared.push("MONOCHROMATIC_VERBOSE");
    }
    // The child's pass is this control's assertion; what this reads is its standard error.
    let output: std::process::Output =
        crate::child_test_process::passed(crate::child_test_process::ChildTest {
            exact_test: path.as_str(),
            environment: environment.as_slice(),
            cleared_environment: cleared.as_slice(),
            ..crate::child_test_process::ChildTest::default()
        });
    return String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
}

/// With the verbose switch on,
/// the debug line Git's words ride in reaches standard error with the userinfo masked;
///  the warning line is masked too,
///  and with the switch off only the warning appears.
#[test]
fn git_words_in_a_verbose_debug_line_reach_standard_error_without_the_userinfo() {
    let verbose: String = child_stderr(true);
    let debug_line: &str = verbose
        .lines()
        .find(|line| return line.starts_with("[debug]"))
        .expect(verbose.as_str());
    assert!(debug_line.contains(CHILD_TAG), "{verbose}");
    assert!(debug_line.contains("403"), "{verbose}");
    assert!(
        verbose
            .lines()
            .any(|line| return line.starts_with("[warn]")),
        "{verbose}"
    );
    assert!(!verbose.contains(FAKE_USER), "{verbose}");
    assert!(!verbose.contains(FAKE_PASSWORD), "{verbose}");
    assert!(
        verbose.contains(crate::lfs_output_mask::CREDENTIALS_MASK),
        "{verbose}"
    );
    // The positive control for the switch: without it the debug line is absent, so the line above came from the switch.
    let quiet: String = child_stderr(false);
    assert!(
        !quiet.lines().any(|line| return line.starts_with("[debug]")),
        "{quiet}"
    );
    assert!(
        quiet.lines().any(|line| return line.starts_with("[warn]")),
        "{quiet}"
    );
    assert!(!quiet.contains(FAKE_PASSWORD), "{quiet}");
}
