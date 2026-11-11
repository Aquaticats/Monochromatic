//! Controls for the cause a failure record names.

use super::*;

/// The cause a record names is the first of Git's own lines that gives a reason,
/// never the remote it pushed to,
/// which Git writes above every reference's line of a push's report.
///  `2026-11-10`:
/// 20 of the day's retry lines named only
///  `To https://github.com/Aquaticats/Monochromatic`,
/// and the reason was on the line after it
///  (`doc/planning/git-policy-cli-push-jams-2026-11-10.md`).
#[test]
fn the_cause_a_retry_names_is_a_reason_and_not_the_remote() {
    let words: &str = "To https://github.com/example/repository\n ! [remote rejected]       main -> main (missing necessary objects)\nerror: failed to push some refs to 'https://github.com/example/repository'\n";
    assert_eq!(
        cause_line(words),
        Some("! [remote rejected]       main -> main (missing necessary objects)"),
        "the reason and not the remote"
    );
    assert_eq!(
        cause_line("fatal: 'DISABLED' does not appear to be a git repository\n"),
        Some("fatal: 'DISABLED' does not appear to be a git repository"),
        "a first line that is a reason is the cause"
    );
    assert_eq!(cause_line("To ../other\n"), Some("To ../other"), "words that name only the remote keep it");
    assert_eq!(cause_line(""), None);
    assert_eq!(cause_line("\n \n"), None, "blank lines name nothing");
}
