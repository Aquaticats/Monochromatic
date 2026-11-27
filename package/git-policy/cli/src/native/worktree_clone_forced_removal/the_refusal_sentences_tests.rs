//! What:
//!  Controls for the printed refusal sentences:
//!  each names the slot as a shell word,
//!  each says what was not changed,
//!  the namespace sentence says leaving the folder is not enough,
//!  and the locked sentence gives the stop command only where the removal may
//!  not stop the unit itself.
//! Why:
//!  A person's next act is decided by these sentences,
//!  so each of the design's sentences must be whole.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(aLockedHolder(...)).toContain('systemctl --user stop');
//! ```

use super::{
    TheHolder,
    WhyNotStopped,
    a_holder_that_would_not_end,
    a_locked_holder,
    caller_holds_in_its_working_directory,
    the_callers_namespace_holds,
};
use crate::shown_so_it_cannot_rewrite_the_line::{
    TheWrappingAroundTheName,
    shown_so_it_cannot_rewrite_the_line,
};

/// A slot with a space in it,
/// so the quoting is shown rather than assumed.
const A_SLOT: &str = "/home/user/work/.claude/worktrees/a slot";

/// What:
///  A command name that would rewrite the sentence showing it,
///  one shape for each thing a name can do to a line:
///  the parenthesis that closes the wrapping,
///  a backslash,
///  an escape that moves the cursor and erases the line,
///  a line feed that starts a line this program never wrote,
///  a right-to-left override that reorders the words,
///  and a Unicode line separator.
/// Why:
///  A process names itself with these bytes,
///  so a sentence that prints the name as the kernel holds it is a sentence that
///  program wrote part of.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_NAME_THAT_WOULD_REWRITE_THE_LINE = 'x) \\ \u001b[2K\n\u202e\u2028';
/// ```
const A_NAME_THAT_WOULD_REWRITE_THE_LINE: &str = "x) \\ \u{1b}[2K\n\u{202e}\u{2028}";

/// What:
///  A name this program's own text is,
///  with no shape a line would act on.
/// Why:
///  The control compares the hostile name's sentence with this one's,
///  so each thing it asserts on is the difference the name makes and not a figure
///  the control wrote down.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_PLAIN_NAME = 'zsh';
/// ```
const A_PLAIN_NAME: &str = "zsh";

/// What:
///  The process number and session the two sentences below name.
/// Why:
///  A number the sentence prints is the same with either name,
///  so the expected line is built from these and from the shown name alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_PID = 4321;
/// const A_SESSION = 4300;
/// ```
const A_PID: i32 = 4321;
const A_SESSION: i32 = 4300;

#[test]
fn a_protected_working_directory_sentence_names_the_process_and_the_one_way_out() {
    let text: String = caller_holds_in_its_working_directory(A_SLOT, 4321, "zsh", 4300);
    assert!(text.starts_with("the clone at '/home/user/work/.claude/worktrees/a slot' is held by the removal's own protected set"), "the slot is one shell word, and it reads {text}");
    assert!(text.contains("process 4321 (zsh) holds it in its working directory, in session 4300"));
    assert!(text.contains("Nothing was moved, unmounted or destroyed: the clone, its snapshots and the Git registration are as they were, and the folder is still mounted."));
    assert!(text.contains("Leave the folder (end that job, or run the command from another shell) and run the removal again."));
}

#[test]
fn the_namespace_sentence_says_leaving_the_folder_is_not_enough() {
    let text: String = the_callers_namespace_holds(
        A_SLOT,
        "mnt:[4026532001]",
        "/home/user/work/one",
        77,
        4321,
        "zsh",
        4300,
    );
    assert!(text.contains(
        "mount namespace mnt:[4026532001] keeps a copy at '/home/user/work/one' (mount 77)"
    ));
    assert!(
        text.contains("Leaving the folder does not release a namespace's copy"),
        "and it reads {text}"
    );
    assert!(text.contains("Nothing was moved, unmounted or destroyed"));
}

/// What:
///  How many parentheses `text` writes as its own,
///  which is every `)` an escape does not carry.
/// Why:
///  A count of `)` counts the ones inside the escape too,
///  so a name holding `)` would look like the sentence wrote one more;
///  this walks the escapes and counts only the parentheses the sentence wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parenthesesTheTextWrites(text: string): number {
///   let count = 0;
///   for (let at = 0; at < text.length; at += 1) {
///     if (text[at] !== '\\') { if (text[at] === ')') count += 1; continue; }
///     at += text[at + 1] === 'u' ? text.indexOf('}', at) - at : 1;
///   }
///   return count;
/// }
/// ```
fn parentheses_the_text_writes(text: &str) -> usize {
    let characters: Vec<char> = text.chars().collect();
    let mut count: usize = 0;
    let mut at: usize = 0;
    while at < characters.len() {
        let character: char = characters[at];
        if character != '\\' {
            if character == ')' {
                count += 1;
            }
            at += 1;
            continue;
        }
        // What: where this escape ends, which is past its brace for `\u{..}` and two characters otherwise.
        // Why:  the characters inside an escape are the name's, not the sentence's own punctuation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // at += characters[at + 1] === 'u' ? characters.indexOf('}', at) - at + 1 : 2;
        // ```
        let escape_ends: usize = if characters.get(at + 1) == Some(&'u') {
            let closing: Option<usize> = characters
                .iter()
                .enumerate()
                .skip(at)
                .find(|(_, found)| return **found == '}')
                .map(|(found_at, _)| return found_at);
            closing.map_or(characters.len(), |found_at| return found_at + 1)
        } else {
            at + 2
        };
        at = escape_ends;
    }
    return count;
}

/// What:
///  Whether `hostile` shows `name` through the escape,
///  and holds nothing else the name brought:
///  no line of its own,
///  no control character,
///  no bidirectional control,
///  no raw line separator,
///  and no parenthesis that closes the wrapping early.
/// Why:
///  Every assertion is the difference from `plain`,
///  the same sentence with a name of this
///  program's own text,
///  so a control that passes says the hostile name changed nothing a person could act on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function onlyTheShownNameDiffers(plain: string, hostile: string, shown: string): void {
///   expect(hostile).toContain(`(${shown})`);
///   expect(hostile.split('\n').length).toBe(plain.split('\n').length);
///   expect(controlCharacters(hostile)).toBe(controlCharacters(plain));
///   expect(hostile).not.toContain('\u202e');
///   expect(hostile).not.toContain('\u2028');
///   expect(parenthesesTheTextWrites(hostile)).toBe(parenthesesTheTextWrites(plain));
/// }
/// ```
fn only_the_shown_name_differs(
    plain: &str,
    hostile: &str,
    shown: &str,
) {
    assert!(
        hostile.contains(format!("({shown})").as_str()),
        "the name is shown through the escape the sentence uses: {shown:?} in {hostile:?}"
    );
    assert_eq!(
        hostile.lines().count(),
        plain.lines().count(),
        "a line feed inside the name starts no line of the sentence: {hostile:?}"
    );
    assert_eq!(
        hostile
            .chars()
            .filter(|character| return character.is_control())
            .count(),
        plain
            .chars()
            .filter(|character| return character.is_control())
            .count(),
        "the only control characters are the sentence's own line ends: {hostile:?}"
    );
    assert!(
        !hostile.contains('\u{202e}') && !hostile.contains('\u{2028}'),
        "a bidirectional control and a line separator come back spelled out: {hostile:?}"
    );
    assert_eq!(
        parentheses_the_text_writes(hostile),
        parentheses_the_text_writes(plain),
        "a parenthesis inside the name cannot close the parentheses around it: {hostile:?}"
    );
}

/// The name in the sentence about a holder in the caller's working directory is escaped,
/// so it cannot close the sentence's parenthesis,
/// start a line,
/// or reorder or erase what a person reads.
#[test]
fn a_name_in_the_protected_set_sentence_cannot_rewrite_its_line() {
    let plain: String =
        caller_holds_in_its_working_directory(A_SLOT, A_PID, A_PLAIN_NAME, A_SESSION);
    let hostile: String = caller_holds_in_its_working_directory(
        A_SLOT,
        A_PID,
        A_NAME_THAT_WOULD_REWRITE_THE_LINE,
        A_SESSION,
    );
    let shown: String = shown_so_it_cannot_rewrite_the_line(
        A_NAME_THAT_WOULD_REWRITE_THE_LINE,
        TheWrappingAroundTheName::Parentheses,
    );
    assert!(
        hostile.contains(
            format!(
                "process {A_PID} ({shown}) holds it in its working directory, in session {A_SESSION}"
            )
            .as_str()
        ),
        "the sentence names the process and shows its name escaped, and it reads {hostile}"
    );
    only_the_shown_name_differs(plain.as_str(), hostile.as_str(), shown.as_str());
}

/// The name in the sentence about the caller's namespace is escaped for the same reason,
/// and the line that holds it is the sentence's own line.
#[test]
fn a_name_in_the_namespace_sentence_cannot_rewrite_its_line() {
    let plain: String = the_callers_namespace_holds(
        A_SLOT,
        "mnt:[4026532001]",
        "/home/user/work/one",
        77,
        A_PID,
        A_PLAIN_NAME,
        A_SESSION,
    );
    let hostile: String = the_callers_namespace_holds(
        A_SLOT,
        "mnt:[4026532001]",
        "/home/user/work/one",
        77,
        A_PID,
        A_NAME_THAT_WOULD_REWRITE_THE_LINE,
        A_SESSION,
    );
    let shown: String = shown_so_it_cannot_rewrite_the_line(
        A_NAME_THAT_WOULD_REWRITE_THE_LINE,
        TheWrappingAroundTheName::Parentheses,
    );
    assert!(
        hostile.contains(
            format!("process {A_PID} ({shown}) in session {A_SESSION} is the caller").as_str()
        ),
        "the sentence names the process and shows its name escaped, and it reads {hostile}"
    );
    only_the_shown_name_differs(plain.as_str(), hostile.as_str(), shown.as_str());
}

#[test]
fn an_unendable_sentence_carries_the_reason_and_what_was_left() {
    let text: String = a_holder_that_would_not_end(
        A_SLOT,
        "mount namespace mnt:[7] could not be entered.",
        "the parent of mnt:[9] is a slave and one directory was removed.",
    );
    assert!(text.starts_with("the removal of '/home/user/work/.claude/worktrees/a slot' was asked to end the holders and could not."));
    assert!(text.contains("could not be entered."));
    assert!(text.contains("one directory was removed."));
    assert!(
        text.ends_with("Nothing was destroyed after that point.\n"),
        "and it reads {text}"
    );
}

#[test]
fn a_locked_holder_that_is_not_the_account_s_unit_is_named_without_a_stop_command() {
    let holder: TheHolder<'_> = TheHolder {
        noun: "process",
        named: "4321 (bwrap)",
    };
    let text: String = a_locked_holder(
        A_SLOT,
        &holder,
        "mnt:[4026532360]",
        "/home/user/work/one",
        77,
        &WhyNotStopped::NotAUnitOfThisAccountsManager,
    );
    assert!(text.contains("because one of the holders has a user namespace of its own."));
    assert!(text.contains("holder process '4321 (bwrap)' in mount namespace mnt:[4026532360] keeps a copy at '/home/user/work/one' (mount 77)"));
    assert!(text.contains("the kernel locked every mount it received"));
    assert!(text.contains("it is not a service unit of this account's own user manager, so this removal may not stop it"));
    assert!(
        !text.contains("systemctl"),
        "a holder the removal may not stop gets no stop command, and it reads {text}"
    );
}

#[test]
fn a_locked_unit_whose_stop_failed_gets_its_own_command_and_state() {
    let holder: TheHolder<'_> = TheHolder {
        noun: "unit",
        named: "watcher.service",
    };
    let text: String = a_locked_holder(
        A_SLOT,
        &holder,
        "mnt:[3]",
        "/home/user/work/one",
        9,
        &WhyNotStopped::TheStopFailed {
            said: "the cgroup was still there when T_stop ended",
        },
    );
    assert!(text.contains("the manager's stop failed, or left its cgroup, or the unit restarted: the cgroup was still there when T_stop ended"));
    assert!(
        text.contains("`systemctl --user stop 'watcher.service'`"),
        "the exact command is named with the unit as one shell word, and it reads {text}"
    );
    assert!(text.ends_with('\n'));
}
