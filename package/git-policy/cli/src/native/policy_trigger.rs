//! What:
//!  The lifecycle points at which policies run,
//!  which shipped policy runs at which
//!       point,
//!  and which points this executable can serve.
//! Why:
//!  A policy runs only for the triggers it declares.
//!  A trigger whose lifecycle is not
//!      ported must be answered as "unavailable",
//!  never as an empty result that would read
//!      as "checked and clean".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type PolicyTrigger = 'pre-forward' | 'post-commit' | 'manual-push' | 'direct-check' | 'direct-fix';
//! ```

/// What:
///  `use` brings a name from a sibling file into this file;
///  `super::` means "the
///       parent module",
///  where every sibling file of this crate is declared.
/// Why:
///   The trigger table is keyed by the registry's typed policy identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PolicyId } from './policy_registry.ts';
/// ```
use super::policy_registry::PolicyId;

/// What:
///  One lifecycle point.
///  An `enum` is a closed set of named alternatives.
///       `#[derive(...)]` asks the compiler to generate copying (`Clone`,
///  `Copy`),
///  debug
///       printing (`Debug`) and `==` (`Eq`,
///  `PartialEq`).
/// Why:
///   The set is fixed by the event format,
///  where each event names its trigger.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PolicyTrigger = 'pre-forward' | 'post-commit' | 'manual-push' | 'direct-check' | 'direct-fix';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Trigger {
    /// Before a wrapped Git command is forwarded.
    PreForward,
    /// After a commit landed,
    ///  against the landed commit.
    PostCommit,
    /// Before a manual `git push`,
    ///  against newly published content.
    ManualPush,
    /// `git cli-git check`.
    DirectCheck,
    /// `git cli-git fix`.
    DirectFix,
}

/// What:
///  The wire spelling of a trigger.
///  `&'static str` is text baked into the program
///       for its whole run (sibling `String` would be an owned copy).
/// Why:
///   Events print exactly the spellings the incumbent emitted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function triggerName(trigger: PolicyTrigger): string { return trigger; }
/// ```
pub fn trigger_name(trigger: Trigger) -> &'static str {
    // `match` picks one arm per variant; the compiler refuses the code if one is forgotten.
    match trigger {
        Trigger::PreForward => return "pre-forward",
        Trigger::PostCommit => return "post-commit",
        Trigger::ManualPush => return "manual-push",
        Trigger::DirectCheck => return "direct-check",
        Trigger::DirectFix => return "direct-fix",
    }
}

/// What:
///  Every trigger.
///  `&[Trigger]` is a borrowed,
///  read-only list (siblings:
///  `Vec<T>`,
///       owned and growable;
///  `[T; N]`,
///  fixed length in the type).
/// Why:
///   Two shipped policies check content at every lifecycle point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const EVERY_TRIGGER = ['pre-forward', 'post-commit', 'manual-push', 'direct-check', 'direct-fix'] as const;
/// ```
const EVERY_TRIGGER: &[Trigger] = &[
    Trigger::PreForward,
    Trigger::PostCommit,
    Trigger::ManualPush,
    Trigger::DirectCheck,
    Trigger::DirectFix,
];

/// What:
///  The triggers a shipped policy declares,
///  copied from the incumbent definitions.
/// Why:
///   The engine skips a policy at every other lifecycle point,
///  exactly as the
///       incumbent's stage did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function policyTriggers(id: PolicyId): readonly PolicyTrigger[];
/// ```
pub fn policy_triggers(id: PolicyId) -> &'static [Trigger] {
    // `|` joins patterns that share one arm; `&[...]` is a list baked into the program.
    match id {
        PolicyId::RequireRoot | PolicyId::ForbiddenRootContext => {
            return &[Trigger::PreForward, Trigger::DirectCheck];
        }
        PolicyId::LinkedWorktreeOnly | PolicyId::BranchWorktreeOnly | PolicyId::AddExplicit => {
            return &[Trigger::PreForward];
        }
        PolicyId::FinalNewline | PolicyId::MarkdownAutofix => return EVERY_TRIGGER,
        PolicyId::DependentVersionBump => {
            return &[
                Trigger::PreForward,
                Trigger::DirectCheck,
                Trigger::DirectFix,
            ];
        }
        PolicyId::ForbiddenStrings => {
            return &[
                Trigger::PreForward,
                Trigger::PostCommit,
                Trigger::ManualPush,
                Trigger::DirectCheck,
            ];
        }
    }
}

/// What:
///  Whether a policy declares a trigger.
///  `bool` is true or false.
/// Why:
///   The stage asks this once per policy before anything else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runs = policy.triggers.includes(trigger);
/// ```
pub fn policy_runs_on(id: PolicyId, trigger: Trigger) -> bool {
    // `.contains(&trigger)` borrows the trigger for the lookup.
    return policy_triggers(id).contains(&trigger);
}

/// What:
///  Whether this executable implements the lifecycle a trigger belongs to.
/// Why:
///  Every lifecycle is ported: the exhaustive match below answers true for each variant, and a new
///  variant fails to compile until it is answered here.
///  The engine still checks this before running a lifecycle, so a future unported variant gets a typed
///  "unavailable" instead of running zero policies and looking clean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ported = (trigger: Trigger): boolean => true;
/// ```
pub fn trigger_is_ported(trigger: Trigger) -> bool {
    match trigger {
        Trigger::PreForward
        | Trigger::PostCommit
        | Trigger::ManualPush
        | Trigger::DirectCheck
        | Trigger::DirectFix => {
            return true;
        }
    }
}

/// Trigger sets,
///  spellings and the ported set stay out of the release executable.
#[cfg(test)]
#[path = "policy_trigger_tests.rs"]
mod tests;
