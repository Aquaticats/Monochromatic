//! What:
//!  Trigger spellings,
//!  each shipped policy's trigger set,
//!  and the ported lifecycles.
//! Why:
//!  A trigger missing from a policy's set silently skips that policy at one lifecycle
//!      point,
//!  and an unported lifecycle read as ported would run nothing and look clean.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(policyTriggers('require-root')).toEqual(['pre-forward', 'direct-check']);
//! ```

/// The table under test and the registry it is keyed by.
use super::{Trigger, policy_runs_on, policy_triggers, trigger_is_ported, trigger_name};
use crate::policy_registry::{POLICY_REGISTRY, PolicyId};

/// The five triggers in the order the incumbent declares them.
const ALL: [Trigger; 5] = [
    Trigger::PreForward,
    Trigger::PostCommit,
    Trigger::ManualPush,
    Trigger::DirectCheck,
    Trigger::DirectFix,
];

/// Each trigger prints the incumbent's spelling.
#[test]
fn triggers_print_their_wire_spelling() {
    let mut names: Vec<&str> = Vec::<&str>::new();
    for trigger in ALL {
        names.push(trigger_name(trigger));
    }
    assert_eq!(
        names,
        [
            "pre-forward",
            "post-commit",
            "manual-push",
            "direct-check",
            "direct-fix"
        ]
    );
}

/// Every shipped policy declares exactly the incumbent definition's triggers.
#[test]
fn every_policy_declares_the_incumbent_triggers() {
    let expected: [(PolicyId, &[Trigger]); 9] = [
        (
            PolicyId::RequireRoot,
            &[Trigger::PreForward, Trigger::DirectCheck],
        ),
        (PolicyId::LinkedWorktreeOnly, &[Trigger::PreForward]),
        (PolicyId::BranchWorktreeOnly, &[Trigger::PreForward]),
        (PolicyId::AddExplicit, &[Trigger::PreForward]),
        (PolicyId::FinalNewline, &ALL),
        (PolicyId::MarkdownAutofix, &ALL),
        (
            PolicyId::ForbiddenRootContext,
            &[Trigger::PreForward, Trigger::DirectCheck],
        ),
        (
            PolicyId::DependentVersionBump,
            &[
                Trigger::PreForward,
                Trigger::DirectCheck,
                Trigger::DirectFix,
            ],
        ),
        (
            PolicyId::ForbiddenStrings,
            &[
                Trigger::PreForward,
                Trigger::PostCommit,
                Trigger::ManualPush,
                Trigger::DirectCheck,
            ],
        ),
    ];
    assert_eq!(expected.len(), POLICY_REGISTRY.len());
    for (index, (id, triggers)) in expected.iter().enumerate() {
        assert_eq!(POLICY_REGISTRY[index].id, *id, "registry order");
        assert_eq!(policy_triggers(*id), *triggers, "{id:?}");
        for trigger in ALL {
            assert_eq!(
                policy_runs_on(*id, trigger),
                triggers.contains(&trigger),
                "{id:?} {trigger:?}"
            );
        }
    }
}

/// Every lifecycle is ported.
#[test]
fn every_lifecycle_is_ported() {
    let mut ported: Vec<Trigger> = Vec::<Trigger>::new();
    for trigger in ALL {
        if trigger_is_ported(trigger) {
            ported.push(trigger);
        }
    }
    assert_eq!(
        ported,
        [
            Trigger::PreForward,
            Trigger::PostCommit,
            Trigger::ManualPush,
            Trigger::DirectCheck,
            Trigger::DirectFix
        ]
    );
}
