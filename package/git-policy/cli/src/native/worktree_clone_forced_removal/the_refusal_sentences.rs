//! What:
//!  The sentences a forced removal prints,
//!  other than the service-manager one:
//!  a holder of the caller's own protected set,
//!  a namespace of that set,
//!  a holder the act could not end or a namespace its reach refused,
//!  and a locked holder.
//! Why:
//!  The design fixes each text whole,
//!  and a person reads the sentence rather than a word,
//!  so each is built here with its values filled and every path quoted as a
//!  shell word
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The refusal,
//! and what it leaves").
//!  Each says what was and was not changed,
//!  because a person's next act depends on whether the clone,
//!  its snapshots and the Git registration are as they were.
//!  A command's name is another program's text,
//!  and it is escaped here and nowhere earlier:
//!  the record keeps the kernel's own bytes,
//!  so the sentence that shows them is where they enter a line
//!  (`AGENTS.md` rule `SYB`,
//!  and `source_guard/every_sentence_that_names_a_process_shows_it_tests.rs`,
//!  which holds both of these sentences to it).
//!  The service-manager sentence is
//!  `started_by_the_service_manager::the_refusal_text`'s,
//!  which the wrapper prints before any request is written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! console.error(callerHoldsInItsWorkingDirectory({ slot, pid, command, session }));
//! ```

use crate::shown_so_it_cannot_rewrite_the_line::{
    TheWrappingAroundTheName,
    shown_so_it_cannot_rewrite_the_line,
};
use crate::worktree_clone_forced_removal::the_acts_journal_lines::quoted;

/// What:
///  The process or unit a locked holder's sentence names.
/// Why:
///  A locked holder may be a task or a unit the account's own manager runs,
///  and the two are spelled differently in the sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheHolder = { kind: 'a process'; value: string } | { kind: 'the unit'; value: string };
/// ```
pub struct TheHolder<'a> {
    /// The noun the sentence uses for it.
    pub noun: &'a str,
    /// What names it,
    /// such as a process's number and command or a unit's name.
    pub named: &'a str,
}

/// What:
///  Why a locked holder was not stopped,
///  in the sentence's own words.
/// Why:
///  A holder that is not the account's own unit and one whose stop failed are
///  two different next acts for a person,
///  and the design gives each its own clause.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhyNotStopped = { kind: 'not-a-unit' } | { kind: 'the-stop-failed'; said: string };
/// ```
pub enum WhyNotStopped<'a> {
    /// It is not a service unit of this account's own user manager,
    /// so this removal may not stop it.
    NotAUnitOfThisAccountsManager,
    /// It is that unit and the manager's stop failed,
    /// left its cgroup or came back,
    /// with what the read answered.
    TheStopFailed {
        /// What the manager or the wait answered.
        said: &'a str,
    },
}

/// What:
///  The sentence for a holder in the caller's protected set that holds the
///  clone through its working directory.
/// Why:
///  It names the process,
///  its session and the reason the removal never ends it,
///  and then the one act that frees the folder.
///  The command's name is shown through the escape for the parenthesis the
///  sentence puts around it,
///  so no name a process chose closes that parenthesis,
///  starts a line or moves the cursor.
pub fn caller_holds_in_its_working_directory(
    slot: &str,
    pid: i32,
    command: &str,
    session: i32,
) -> String {
    // What: the command's name as this line may show it: one parenthesis wrapping, escaped.
    // Why:  the name is the one field of the sentence another program wrote,
    //  and the escape is the last step before the name reaches the person.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const shownCommand = shownSoItCannotRewriteTheLine(command, TheWrappingAroundTheName.Parentheses);
    // ```
    let shown_command: String =
        shown_so_it_cannot_rewrite_the_line(command, TheWrappingAroundTheName::Parentheses);
    return format!(
        "the clone at {} is held by the removal's own protected set, and a removal never ends it.\n\
         process {pid} ({shown_command}) holds it in its working directory, in session {session}, and that process is the caller, or is in the caller's session or its ancestor chain.\n\
         Nothing was moved, unmounted or destroyed: the clone, its snapshots and the Git registration are as they were, and the folder is still mounted.\n\
         Leave the folder (end that job, or run the command from another shell) and run the removal again.\n",
        quoted(slot)
    );
}

/// What:
///  The sentence for a mount namespace of the caller's protected set.
/// Why:
///  Leaving the folder does not release a namespace's copy,
///  which is the one thing a person would otherwise try first.
///  The command's name is shown through the escape for the parenthesis the
///  sentence puts around it,
///  so no name a process chose closes it,
///  starts a line or moves the cursor.
pub fn the_callers_namespace_holds(
    slot: &str,
    namespace: &str,
    mount_point: &str,
    mount_id: u64,
    pid: i32,
    command: &str,
    session: i32,
) -> String {
    // What: the command's name as this line may show it: one parenthesis wrapping, escaped.
    // Why:  the escape is the last step before the name reaches the person,
    //  so the parenthesis the sentence writes around it is the one that closes it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const shownCommand = shownSoItCannotRewriteTheLine(command, TheWrappingAroundTheName.Parentheses);
    // ```
    let shown_command: String =
        shown_so_it_cannot_rewrite_the_line(command, TheWrappingAroundTheName::Parentheses);
    return format!(
        "the clone at {} is held by the removal's own protected set, and a removal never ends it.\n\
         mount namespace {namespace} keeps a copy at {} (mount {mount_id}), and process {pid} ({shown_command}) in session {session} is the caller, or is in the caller's session or its ancestor chain.\n\
         Leaving the folder does not release a namespace's copy: end that session or its unit, or run the removal from a session that never entered the clone.\n\
         Nothing was moved, unmounted or destroyed: the clone, its snapshots and the Git registration are as they were.\n",
        quoted(slot),
        quoted(mount_point)
    );
}

/// What:
///  The sentence for a holder the act could not end,
///  or a namespace its reach refused.
/// Why:
///  The reason line,
///  what was ended before the refusal and what was left must each be there,
///  or a person cannot tell what the removal already changed.
pub fn a_holder_that_would_not_end(
    slot: &str,
    reason: &str,
    what_was_left: &str,
) -> String {
    return format!(
        "the removal of {} was asked to end the holders and could not.\n\
         {reason}\n\
         {what_was_left}\n\
         Nothing was destroyed after that point.\n",
        quoted(slot)
    );
}

/// What:
///  The sentence for a locked holder the removal would not stop.
/// Why:
///  It says the kernel locked every mount the namespace received,
///  why neither way reaches the copy,
///  and,
///  for a unit of the account's own manager,
///  the exact command that stops it.
pub fn a_locked_holder(
    slot: &str,
    holder: &TheHolder<'_>,
    namespace: &str,
    mount_point: &str,
    mount_id: u64,
    why: &WhyNotStopped<'_>,
) -> String {
    let reason: String = match why {
        WhyNotStopped::NotAUnitOfThisAccountsManager => String::from(
            "it is not a service unit of this account's own user manager, so this removal may not stop it",
        ),
        WhyNotStopped::TheStopFailed {
            said,
        } => {
            format!(
                "it is that unit and the manager's stop failed, or left its cgroup, or the unit restarted: {said}"
            )
        }
    };
    let remedy: String = match (why, holder) {
        (
            WhyNotStopped::TheStopFailed {
                ..
            },
            named,
        ) => {
            format!(
                "stop it with `systemctl --user stop {}` and run the removal again.\n",
                quoted(named.named)
            )
        }
        (WhyNotStopped::NotAUnitOfThisAccountsManager, _holder) => String::new(),
    };
    return format!(
        "the removal of {} was refused before anything was ended, because one of the holders has a user namespace of its own.\n\
         holder {} {} in mount namespace {namespace} keeps a copy at {} (mount {mount_id}).\n\
         That namespace is owned by a user namespace other than this machine's first one, so the kernel locked every mount it received: this removal cannot unmount the copy, and moving it is refused too.\n\
         {reason}.\n\
         Nothing was moved, unmounted or destroyed: the clone, its snapshots, the Git registration and that holder are as they were.\n\
         {remedy}",
        quoted(slot),
        holder.noun,
        quoted(holder.named),
        quoted(mount_point)
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_refusal_sentences_tests.rs"]
mod tests;
