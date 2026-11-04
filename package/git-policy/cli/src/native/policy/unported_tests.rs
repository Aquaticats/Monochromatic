//! What:
//!  The exact notice of every refusal,
//!  and the mapping from the engine's answer.
//! Why:
//!  The notice is the only thing telling a person why their command did not run;
//!  a
//!      wrong or empty subject would send them looking in the wrong place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(unportedNotice({ kind: 'commit-transaction' }, 'commit')).toBe('cli-git: the commit transaction ...\n');
//! ```

/// The refusal list,
///  its notice and the engine answer it is built from.
use super::{Unported, unported_from_unavailable, unported_notice};
use crate::policy_engine::Unavailable;
use crate::policy_registry::PolicyId;
use crate::policy_trigger::Trigger;
use std::path::PathBuf;

/// The sentence around every subject.
fn notice(subject: &str, command: &str) -> String {
    return format!(
        "cli-git: {subject} is not implemented in this native development executable, so git {command} was not run. Run it with the installed cli-git.\n"
    );
}

/// Each reason has its own subject,
///  and the command is named as typed.
#[test]
fn every_reason_names_what_is_missing() {
    for (what, command, subject) in [
        (
            Unported::TransactionRecovery(PathBuf::from("/r/.git/cli-git-transactions")),
            "add",
            "recovering or waiting for the commit transactions recorded in /r/.git/cli-git-transactions",
        ),
        (
            Unported::Lifecycle(Trigger::ManualPush),
            "push",
            "the manual-push policy lifecycle",
        ),
        (
            Unported::Lifecycle(Trigger::PostCommit),
            "commit",
            "the post-commit policy lifecycle",
        ),
        (
            Unported::PolicyNeeds {
                policy: PolicyId::FinalNewline,
                needs: "predicting what git add would stage",
            },
            "add",
            "predicting what git add would stage, which the policy final-newline needs,",
        ),
        (
            Unported::PolicyNeeds {
                policy: PolicyId::ForbiddenStrings,
                needs: "reading the selected files",
            },
            "cli-git check",
            "reading the selected files, which the policy security/forbidden-strings needs,",
        ),
    ] {
        assert_eq!(
            unported_notice(&what, command),
            notice(subject, command),
            "{what:?}"
        );
    }
}

/// The engine's two answers map to the two refusals that carry the same facts.
#[test]
fn engine_answers_keep_their_facts() {
    assert_eq!(
        unported_from_unavailable(Unavailable::Lifecycle(Trigger::ManualPush)),
        Unported::Lifecycle(Trigger::ManualPush)
    );
    assert_eq!(
        unported_from_unavailable(Unavailable::Policy {
            policy: PolicyId::MarkdownAutofix,
            needs: "candidate content",
        }),
        Unported::PolicyNeeds {
            policy: PolicyId::MarkdownAutofix,
            needs: "candidate content",
        }
    );
}
