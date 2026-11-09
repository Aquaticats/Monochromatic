//! Controls for the line a waiting commit prints.

use super::*;

/// The first line names the commit Git builds,
/// says there is no limit and adds the hint;
/// a repeat is the same without the hint;
/// a
///  signed commit says why,
/// and a signer below it is named.
#[test]
fn the_line_names_the_step_the_limit_and_the_signer() {
    let ceiling: Duration = Duration::from_secs(1260);
    let first: String = notice_line(false, Duration::from_secs(12), true, &[], ceiling, true);
    assert!(first.starts_with("monochromatic-git-policy-cli: still waiting for the `git commit` this commit runs in its scratch repository, 12 seconds so far;"), "{first}");
    assert!(first.contains("stopped if it lasts 1260 seconds") && first.contains("Hint: it is still running"), "{first}");
    assert!(!first.contains("no time limit"), "{first}");
    assert!(!first.contains("signs"), "{first}");
    let repeat: String = notice_line(true, Duration::from_secs(22), false, &[(7, String::from("ssh-keygen"))], ceiling, true);
    assert!(!repeat.contains("Hint"), "{repeat}");
    assert!(repeat.contains("signs the commit because `commit.gpgsign` or `-S` asks for it") && repeat.contains("`ssh-keygen` (process 7) runs below it"), "{repeat}");
    assert!(first.ends_with('\n') && repeat.ends_with('\n'));
}

/// What:
///  A step that cannot be stopped is said to be one the commit waits out.
/// Why:
///   A step whose start gave no descriptor is named only by a number that may be reused,
/// so the ceiling cannot end it without risking another command's process (findings 1 and 4 of the third review).
/// A line that kept promising a stop would be the one thing the person is never told.
#[test]
fn a_step_that_cannot_be_stopped_is_said_to_be_waited_out() {
    let ceiling: Duration = Duration::from_secs(16_332);
    let stoppable: String = notice_line(false, Duration::from_secs(12), true, &[], ceiling, true);
    let stuck: String = notice_line(false, Duration::from_secs(12), true, &[], ceiling, false);
    assert!(stoppable.contains("it is stopped if it lasts 16332 seconds in all"), "{stoppable}");
    assert!(!stuck.contains("it is stopped if it lasts"), "{stuck}");
    assert!(stuck.contains("it cannot be stopped"), "{stuck}");
    assert!(stuck.contains("no descriptor was made for it when it started"), "{stuck}");
}
