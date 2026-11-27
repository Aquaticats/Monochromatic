//! Controls for the sentence that names a live lock holder and its way out.
//! Why:
//!  A process sets its own name to any bytes it likes (`/proc/<pid>/comm`),
//!  and that sentence is how a person finds which program holds the lock,
//!  so a name that would end the parentheses it is shown in,
//!  or that would move the cursor or start a line, must come back spelled out.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(holderWayOut({ pid: 2147483647, command: 'x) FORGED\u001b[2K' })).toContain('(x\\) FORGED\\u{1b}[2K)');
//! ```

use super::*;

/// A process number this host cannot have:
/// Linux caps process numbers far below the largest a signed 32-bit number holds,
/// so the kernel answers that no such process runs.
const A_NUMBER_NO_PROCESS_HAS: i64 = 2_147_483_647;

/// A name that would end the parentheses it is shown in comes back spelled out,
/// so the rest of it cannot read as this program's own words,
/// and the sentence still names the process and the way to end it.
#[test]
fn a_name_that_would_end_the_parentheses_cannot_rewrite_the_holders_line() {
    let name: &str = "x) FORGED\u{1b}[2K";
    let line: String = holder_way_out(A_NUMBER_NO_PROCESS_HAS, Some(name));
    assert_eq!(
        line,
        format!(
            "The holder is PID {A_NUMBER_NO_PROCESS_HAS} (x\\) FORGED\\u{{1b}}[2K), start time unknown, it has just ended; \
             if it is stuck, `kill {A_NUMBER_NO_PROCESS_HAS}` ends it and Git removes its lock, and a commit still waiting then lands.\n"
        )
    );
    assert_eq!(
        line.matches('\n').count(),
        1,
        "a line break inside the name starts no line of its own: {line:?}"
    );
}

/// A backtick inside the name is not the wrapping of this sentence,
/// so it is shown as the program spelled it,
/// and a name of printable ASCII costs the reader nothing.
#[test]
fn a_backtick_and_an_ordinary_name_are_shown_as_they_are() {
    assert!(
        holder_way_out(A_NUMBER_NO_PROCESS_HAS, Some("a`b"))
            .contains("(a`b), start time unknown"),
        "the backtick of the name belongs to the name"
    );
    assert!(
        holder_way_out(A_NUMBER_NO_PROCESS_HAS, Some("git-lfs")).contains("(git-lfs), start time unknown"),
        "an ordinary name is shown unchanged"
    );
}

/// Nothing to call the program is said in this program's own words,
/// so the sentence still leads a person to the way out.
#[test]
fn a_holder_with_no_name_still_gets_the_way_out() {
    let line: String = holder_way_out(A_NUMBER_NO_PROCESS_HAS, None);
    assert!(
        line.starts_with(format!("The holder is PID {A_NUMBER_NO_PROCESS_HAS} (program unknown), ").as_str()),
        "{line}"
    );
    assert!(line.contains("`kill "), "{line}");
}
