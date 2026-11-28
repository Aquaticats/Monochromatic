//! What:
//!  The cancel command the removal's design keeps as the way back from a
//!  blocked removal,
//!  answered from the words the command itself takes.
//! Why:
//!  plan step 18 lists the kept cancel command among the lines `doctor`
//!  prints,
//!  and the answer is what a person reads beside a clone whose removal is
//!  blocked
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 18,
//!  "What it builds";
//!  `doc/decision/cli-git-zfs-step-17-forced-and-cancelled-removal-designs-accepted.md`).
//!  The command is built in `worktree_clone_removal_cancel/`,
//!  and that folder and this one already sit in one loop the layout allow list
//!  tolerates,
//!  so its word and its usage line arrive as values the wiring run reads from
//!  the command's own grammar rather than as an import
//!  (`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`;
//!  `AGENTS.md` rule LYL).
//!  A sentence written here instead would keep saying the command takes one
//!  argument after the command took two.
//!  The line says what the command does and not how a person reaches it:
//!  whether a management grammar admits the word is that grammar's own
//!  passage in `SPEC.md`,
//!  and this check has no fact about it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkCancelCommand(facts); // { kind: 'pass', detail: '… usage: git … cancel-worktree-removal <slot>' }
//! ```

use super::check_outcome::{
    CheckOutcome,
    pass,
};

/// What:
///  The cancel command this build carries,
///  as the command's own grammar states it:
///  the word a person types and the usage line its help prints.
/// Why:
///  the report must name the grammar the command parses,
///  and the only copy of that grammar is the command's own,
///  so the two words travel as values the wiring run reads there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = { word: 'cancel-worktree-removal', usageLine: 'usage: git monochromatic-git-policy-cli cancel-worktree-removal <slot>' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancelCommandFacts {
    /// The word a person types after the program's name.
    pub word: String,
    /// The usage line the command prints when it is asked for its help,
    /// which is the grammar it parses.
    pub usage_line: String,
}

/// What:
///  What the kept cancel command answers:
///  that it is built,
///  its usage line as the command itself prints it,
///  and what it proves and refuses.
/// Why:
///  the only fact this line reads is the built command,
///  so there is no state in which it could be unknown,
///  and a person beside a blocked removal learns the way back exists without
///  reading `SPEC.md`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkCancelCommand(facts); // pass, naming the usage line
/// ```
pub fn check_cancel_command(facts: &CancelCommandFacts) -> CheckOutcome {
    return pass(format!(
        "the cancel command is built: `{}`. `git monochromatic-git-policy-cli {} <slot>` proves the slot, the clone and the removal's records under the clone's removal claim, settles what the removal's destruction plan already destroyed, and asks the handler for one `mount` request that puts the clone back at its slot; it destroys nothing, forces no unmount, ends no holder, and refuses where the clone's head is gone or a destroy outcome is unknown",
        facts.usage_line, facts.word
    ));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "cancel_command_checks_tests.rs"]
mod tests;
