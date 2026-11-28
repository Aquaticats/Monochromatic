//! What:
//!  The controls of the delegation facts:
//!  the account and the groups the kernel reports,
//!  the two datasets' grants gathered into one list,
//!  a reply the pool refused,
//!  and a second reply that failed after the first answered.
//! Why:
//!  The delegation check compares the account's grants with the nine the design
//!  scopes,
//!  and a read that failed would otherwise report the decided permissions as
//!  missing,
//!  so the check must answer unknown instead of a finding about a delegation
//!  nobody read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theDelegationFacts({ layout, pool, unread }).grants).toHaveLength(2);
//! expect(unread.map((r) => r.check)).toEqual(['delegation']);
//! ```

/// The pool a control scripts.
use super::super::a_scripted_pool_for_each_control::AScriptedPool;
/// The read a control judges.
use super::the_delegation_facts;
/// The check a failed read answers.
use crate::doctor_zfs_route::check_outcome::{
    Check,
    CheckReport,
};
/// Everything the delegation check reads,
/// one grant and who it names.
use crate::doctor_zfs_route::delegation_reply::{
    DelegationFacts,
    Grant,
    Inheritance,
    Who,
};
/// The names the design's layout fixes.
use crate::doctor_zfs_route::route_test_facts::{
    CHECKOUT_DATASET,
    CONTAINER_DATASET,
};
/// The account the add's own preflight reads,
/// which this report must name too.
use crate::worktree_clone_add_route::add_grants::this_account;
/// The three names a layout holds.
use crate::worktree_clone_mount_handler::sweep_configuration::SweepConfiguration;
/// The owned path a control's layout names.
use std::path::PathBuf;

/// One layout naming the design's datasets at a fixture path.
fn the_layout() -> SweepConfiguration {
    return SweepConfiguration {
        parent_dataset: String::from(CONTAINER_DATASET),
        checkout: PathBuf::from("/fixture/checkout"),
        checkout_dataset: String::from(CHECKOUT_DATASET),
    };
}

/// One grant of the design's shape at one dataset.
fn a_grant(carried_by: &str) -> Grant {
    return Grant {
        carried_by: String::from(carried_by),
        permission: String::from("snapshot"),
        who: Who::User(this_account()),
        inheritance: Inheritance::Local,
    };
}

/// What:
///  Both datasets' replies arrive in one list,
///  and the account and its groups are the ones the kernel reports.
/// Why:
///  The check reads every grant of the one list and compares them with the
///  account's own id,
///  so a grant left out of the list would pass for a delegation nobody has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(facts.grants.map((g) => g.carriedBy)).toEqual([CHECKOUT_DATASET, CONTAINER_DATASET]);
/// expect(facts.account).toBe(process.geteuid());
/// ```
#[test]
fn both_datasets_grants_and_the_account_arrive_together() {
    let mut pool: AScriptedPool = AScriptedPool::a_healthy_machine();
    pool.grants = vec![
        (
            String::from(CHECKOUT_DATASET),
            Ok(vec![a_grant(CHECKOUT_DATASET)]),
        ),
        (
            String::from(CONTAINER_DATASET),
            Ok(vec![a_grant(CONTAINER_DATASET)]),
        ),
    ];
    let mut unread: Vec<CheckReport> = Vec::new();
    let facts: DelegationFacts = the_delegation_facts(&the_layout(), &pool, &mut unread);
    assert_eq!(facts.account, this_account());
    assert_eq!(facts.checkout_dataset, CHECKOUT_DATASET);
    assert_eq!(facts.container_dataset, CONTAINER_DATASET);
    assert_eq!(
        facts
            .grants
            .iter()
            .map(|grant| return grant.carried_by.as_str())
            .collect::<Vec<&str>>(),
        vec![CHECKOUT_DATASET, CONTAINER_DATASET]
    );
    assert!(unread.is_empty(), "{unread:?}");
}

/// What:
///  A reply the pool refused answers the delegation check an unknown naming
///  the dataset,
///  and a second reply that failed answers a second one.
/// Why:
///  A finding about missing grants would send a person to `zfs allow` for a
///  delegation nobody could read,
///  and the report must say which dataset answered nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(unread.map((r) => r.check)).toEqual(['delegation']);
/// ```
#[test]
fn a_reply_the_pool_refused_answers_an_unknown() {
    let mut pool: AScriptedPool = AScriptedPool::a_healthy_machine();
    pool.grants = vec![
        (
            String::from(CHECKOUT_DATASET),
            Ok(vec![a_grant(CHECKOUT_DATASET)]),
        ),
        (
            String::from(CONTAINER_DATASET),
            Err(format!(
                "{CONTAINER_DATASET}'s delegation could not be read (EIO)"
            )),
        ),
    ];
    let mut unread: Vec<CheckReport> = Vec::new();
    let facts: DelegationFacts = the_delegation_facts(&the_layout(), &pool, &mut unread);
    assert_eq!(facts.grants.len(), 1);
    let lines: Vec<String> = unread.iter().map(|report| return report.line()).collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(
        unread
            .iter()
            .map(|report| return report.check)
            .collect::<Vec<Check>>(),
        vec![Check::Delegation]
    );
    assert!(lines[0].contains(CONTAINER_DATASET), "{}", lines[0]);
}
