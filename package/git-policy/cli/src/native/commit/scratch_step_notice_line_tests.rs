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
    let first: String = notice_line(false, Duration::from_secs(12), true, &[], ceiling);
    assert!(first.starts_with("monochromatic-git-policy-cli: still waiting for the `git commit` this commit runs in its scratch repository, 12 seconds so far;"), "{first}");
    assert!(first.contains("stopped if it lasts 1260 seconds") && first.contains("Hint: it is still running"), "{first}");
    assert!(!first.contains("no time limit"), "{first}");
    assert!(!first.contains("signs"), "{first}");
    let repeat: String = notice_line(true, Duration::from_secs(22), false, &[(7, String::from("ssh-keygen"))], ceiling);
    assert!(!repeat.contains("Hint"), "{repeat}");
    assert!(repeat.contains("signs the commit because `commit.gpgsign` or `-S` asks for it") && repeat.contains("`ssh-keygen` (process 7) runs below it"), "{repeat}");
    assert!(first.ends_with('\n') && repeat.ends_with('\n'));
}
