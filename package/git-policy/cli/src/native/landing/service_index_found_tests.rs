//! Controls for the field that names the processes a scan saw holding a lock open.
//! Why:
//!  The field is written into a line a person reads,
//!  and the name in it is a name another program chose (`/proc/<pid>/comm`),
//!  so a name that would end the parentheses around it,
//!  or that would move the cursor or start a line, must come back spelled out.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(openByText({ holders: [{ pid: 42, command: 'x) FORGED\u001b[2K' }] })).toBe('open_by=[42 (x\\) FORGED\\u{1b}[2K)]');
//! ```

use super::*;

/// A name that would end the parentheses it is shown in,
/// and a second whose comma would read as another entry of the same list,
/// both come back spelled out,
/// so the list holds the processes the scan found and nothing else.
#[test]
fn a_name_that_would_end_the_parentheses_cannot_rewrite_the_open_by_list() {
    let seen: HolderEvidence = HolderEvidence {
        method: "proc-fd",
        holders: vec![
            Holder {
                pid: 42,
                command: Some(String::from("x) FORGED\u{1b}[2K")),
            },
            Holder {
                pid: 43,
                command: Some(String::from("evil, 9 (git")),
            },
        ],
        partial: Vec::new(),
        unresolvable: Vec::new(),
    };
    let text: String = open_by_text(Some(&seen)).expect("two holders are named");
    assert_eq!(
        text,
        "open_by=[42 (x\\) FORGED\\u{1b}[2K), 43 (evil, 9 \\(git)]"
    );
    assert_eq!(
        text.matches('\n').count(),
        0,
        "a list of names stays one line: {text:?}"
    );
}

/// A holder with no readable name is named by its number alone,
/// and an empty scan names nothing at all.
#[test]
fn a_holder_without_a_name_is_named_by_its_number() {
    let seen: HolderEvidence = HolderEvidence {
        method: "proc-fd",
        holders: vec![Holder {
            pid: 42,
            command: None,
        }],
        partial: Vec::new(),
        unresolvable: Vec::new(),
    };
    assert_eq!(open_by_text(Some(&seen)), Some(String::from("open_by=[42]")));
    let none: HolderEvidence = HolderEvidence {
        method: "proc-fd",
        holders: Vec::new(),
        partial: Vec::new(),
        unresolvable: Vec::new(),
    };
    assert_eq!(open_by_text(Some(&none)), None);
    assert_eq!(open_by_text(None), None);
}
