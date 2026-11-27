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
    assert!(
        first.contains("stopped if it lasts 1260 seconds")
            && first.contains("Hint: it is still running"),
        "{first}"
    );
    assert!(!first.contains("no time limit"), "{first}");
    assert!(!first.contains("signs"), "{first}");
    let repeat: String = notice_line(
        true,
        Duration::from_secs(22),
        false,
        &[(7, String::from("ssh-keygen"))],
        ceiling,
    );
    assert!(!repeat.contains("Hint"), "{repeat}");
    assert!(
        repeat.contains("signs the commit because `commit.gpgsign` or `-S` asks for it")
            && repeat.contains("`ssh-keygen` (process 7) runs below it"),
        "{repeat}"
    );
    assert!(first.ends_with('\n') && repeat.ends_with('\n'));
}

/// What:
///  A program below the step is shown between backticks,
///  and its name is a name another program chose (`/proc/<pid>/stat`).
/// Why:
///  A name holding a backtick would end the backticks that show it and speak the rest of the
///  line as its own,
///  and one holding an escape or a line break would move the cursor or start a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(noticeLine({ below: [[7, 'x` FORGED\u001b[2K']] })).toContain('`x\\` FORGED\\u{1b}[2K` (process 7) runs below it');
/// ```
#[test]
fn a_name_that_would_end_the_backticks_cannot_rewrite_the_notice() {
    let ceiling: Duration = Duration::from_secs(1260);
    let line: String = notice_line(
        false,
        Duration::from_secs(12),
        true,
        &[(7, String::from("x` FORGED\u{1b}[2K\nsecond line"))],
        ceiling,
    );
    assert!(
        line.contains("`x\\` FORGED\\u{1b}[2K\\nsecond line` (process 7) runs below it"),
        "{line:?}"
    );
    assert_eq!(
        line.matches('\n').count(),
        1,
        "the only line break of the notice is the one it ends with: {line:?}"
    );
    assert_eq!(
        line.matches("x`").count(),
        0,
        "the backtick of the name is shown behind a backslash, not as one: {line:?}"
    );
}
