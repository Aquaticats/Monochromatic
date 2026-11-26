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

/// A slot with a space in it,
/// so the quoting is shown rather than assumed.
const A_SLOT: &str = "/home/user/work/.claude/worktrees/a slot";

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
