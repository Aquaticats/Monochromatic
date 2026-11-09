//! What:
//!  The branch-worktree-only check:
//!  a new branch starts in its own linked worktree,
//!  not in the current
//!       one.
//! Why:
//!  Explicit creation is rejected from the arguments;
//!  a bare name is rejected only when Git would create a
//!       branch for it from the one remote that has it,
//!  which is the one fact this check asks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // checkBranchWorktree(facts, ['switch', 'topic'])
//! ```

/// The rule core and its text.
use crate::rule_branch_worktree::{
    BRANCH_CREATION_CODE, BranchWorktreeDecision, branch_creation_message, decide_branch_worktree,
};
/// The engine's outcomes,
///  the three endings every check shares and the facts interface.
use crate::policy_checks::{clean, fact_unavailable, rejected};
use crate::policy_engine::PolicyOutcome;
use crate::repository_facts::RepositoryFacts;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;

/// What:
///  The branch-worktree-only check.
/// Why:
///   Explicit creation is rejected from the arguments;
///  a bare name is rejected only
///       when Git would create a branch for it from the one remote that has it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkBranchWorktree(facts, args): Promise<PolicyOutcome>;
/// ```
pub fn check_branch_worktree(facts: &mut dyn RepositoryFacts, arguments: &[OsString]) -> PolicyOutcome {
    match decide_branch_worktree(arguments) {
        BranchWorktreeDecision::Pass => return clean(),
        BranchWorktreeDecision::Creates(command) => {
            return rejected(BRANCH_CREATION_CODE, branch_creation_message(command, None));
        }
        BranchWorktreeDecision::NeedsRemoteGuess { command, target } => {
            // `.as_os_str()` lends the owned name as a borrowed one.
            match facts.remote_guess_creates_branch(target.as_os_str()) {
                Err(message) => return fact_unavailable(message),
                Ok(false) => return clean(),
                Ok(true) => {
                    return rejected(
                        BRANCH_CREATION_CODE,
                        branch_creation_message(command, Some(target.as_os_str())),
                    );
                }
            }
        }
    }
}
