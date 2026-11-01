//! What:
//!  The work of the installed cli-git that this executable does not do yet,
//!  as one
//!       typed list,
//!  and the notice printed when a command needs any of it.
//! Why:
//!  A command that needs unported work must stop with exit status 2 and say what is
//!      missing.
//!  Forwarding it instead would run the command without the protection the
//!      installed wrapper gives it,
//!  and nobody would see that it happened.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // process.stderr.write(unportedNotice({ kind: 'commit-transaction' }, 'commit')); process.exitCode = 2;
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  where every sibling file of this crate is declared.
/// Why:
///   A refusal can name a policy,
///  a lifecycle trigger,
///  or an engine "unavailable" answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { triggerName } from './policy_trigger.ts';
/// ```
use super::policy_engine::Unavailable;
use super::policy_registry::{PolicyId, policy_descriptor};
use super::policy_trigger::{Trigger, trigger_name};
/// What:
///  `PathBuf` is an owned filesystem path of raw bytes.
///  Sibling the reader might
///       expect:
///  `String`,
///  which must be valid UTF-8.
/// Why:
///   A refusal about leftover state names the directory that holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::path::PathBuf;

/// What:
///  One piece of unported work.
///  An `enum` is a closed set of named alternatives;
///  some
///       carry a value.
///  `#[derive(...)]` asks the compiler to generate copying,
///  debug
///       printing and `==`.
/// Why:
///   Every refusal of this executable is one of these,
///  so the list of what stops a
///       command is in one place and a test can name the exact reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Unported =
///   | { kind: 'transaction-recovery'; directory: string }
///   | { kind: 'lifecycle'; trigger: PolicyTrigger }
///   | { kind: 'policy-needs'; policy: PolicyId; needs: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Unported {
    /// Recovering,
    ///  or waiting for,
    ///  the commit transactions recorded in this directory.
    TransactionRecovery(PathBuf),
    /// The policy lifecycle of this trigger.
    Lifecycle(Trigger),
    /// Something one policy needs before it can be evaluated.
    PolicyNeeds {
        /// The policy that could not be evaluated.
        policy: PolicyId,
        /// What it needs,
        ///  in words.
        needs: &'static str,
    },
}

/// What:
///  Turn the engine's "unavailable" answer into the refusal it stands for.
/// Why:
///   The engine reports what it could not evaluate;
///  the wrapper refuses with one
///       vocabulary whether the gap was found by the engine or before it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unportedFromUnavailable(unavailable: Unavailable): Unported;
/// ```
pub fn unported_from_unavailable(unavailable: Unavailable) -> Unported {
    // `match` picks one arm per variant and binds the fields each one carries.
    match unavailable {
        Unavailable::Lifecycle(trigger) => return Unported::Lifecycle(trigger),
        Unavailable::Policy { policy, needs } => return Unported::PolicyNeeds { policy, needs },
    }
}

/// What:
///  Name the missing work as the subject of the sentence "... is not implemented".
///       `&Unported` borrows the reason;
///  `String` is owned UTF-8 text.
/// Why:
///   The person reading the notice must learn which protection is missing,
///  in plain
///       words,
///  and what they can do when there is a way to avoid needing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unportedSubject(what: Unported): string;
/// ```
fn unported_subject(what: &Unported) -> String {
    match what {
        // `format!` builds owned text; `.display()` shows a path, replacing bytes that are not UTF-8.
        Unported::TransactionRecovery(directory) => {
            return format!(
                "recovering or waiting for the commit transactions recorded in {}",
                directory.display()
            );
        }
        // `*trigger` copies the small trigger value out of the borrowed reason.
        Unported::Lifecycle(trigger) => {
            return format!("the {} policy lifecycle", trigger_name(*trigger));
        }
        Unported::PolicyNeeds { policy, needs } => {
            return format!(
                "{needs}, which the policy {} needs,",
                policy_descriptor(*policy).name
            );
        }
    }
}

/// What:
///  The complete,
///  line-terminated notice for a command stopped by unported work.
///       `&str` borrows the command words as the caller typed them.
/// Why:
///   One sentence shape for every refusal:
///  what is missing,
///  that the command did not
///       run,
///  and where the complete implementation is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unportedNotice(what: Unported, command: string): string;
/// ```
pub fn unported_notice(what: &Unported, command: &str) -> String {
    return format!(
        "cli-git: {} is not implemented in this native development executable, so git \
         {command} was not run. Run it with the installed cli-git.\n",
        unported_subject(what)
    );
}

/// The wording of every refusal stays out of the release executable.
#[cfg(test)]
#[path = "unported_tests.rs"]
mod tests;
