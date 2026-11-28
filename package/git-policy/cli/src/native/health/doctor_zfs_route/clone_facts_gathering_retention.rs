//! What:
//!  What one clone and its own snapshots retain,
//!  in the pool's own named fields,
//!  with the history the clone inherited kept apart.
//! Why:
//!  The split between a clone's own snapshots and the history it inherited is
//!  the removal's own rule,
//!  read from the live creation group,
//!  so a snapshot counted as the clone's own here is one the removal destroys
//!  as its own
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints";
//!  plan step 18,
//!  "What it builds").
//!  An accounting field nobody read makes the line say so,
//!  never zero,
//!  because zero bytes and "nobody read it" must never be swapped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const retention = retentionOf(reads, clone, record, snapshots, inventory);
//! ```

/// Every read one clone's facts come from.
use super::clone_facts_reads::CloneFactsReads;
/// One object's accounting fields,
/// and what one clone and its snapshots retain.
use super::clone_retention_checks::{
    AccountingFacts,
    CloneRetentionFacts,
};
/// The clone's own snapshots beside the plan's states.
use super::clone_snapshot_checks::CloneSnapshotFacts;
/// What a read of one object answered.
use super::zfs_checks::Reading;
/// The record one add wrote.
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
/// The property value one accounting field holds.
use crate::worktree_clone_mount_handler::sweep_dataset_facts::PropertyValue;
/// The snapshots and bookmarks one clone holds.
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveInventory;

/// What:
///  What one clone and its own snapshots retain,
///  in the pool's own named fields.
/// Why:
///  The split between a clone's own snapshots and the history it inherited is
///  the removal's own rule,
///  read from the live creation group,
///  so a snapshot counted as the clone's own here is one the removal destroys
///  as its own.
pub(crate) fn retention_of(
    reads: &dyn CloneFactsReads,
    clone: &str,
    record: Option<&BindingRecord>,
    snapshots: &CloneSnapshotFacts,
    inventory: &Reading<LiveInventory>,
) -> CloneRetentionFacts {
    // A slot whose record reads nothing names no dataset, so the pool is not
    // asked about a name that is a slot's, and the line says why instead.
    let dataset: Reading<AccountingFacts> = match record {
        Some(_) => reads.accounting(clone),
        None => Reading::Unreadable {
            why: format!(
                "the binding record filed under this slot names no clone dataset this report can read, so nothing retains blocks of `{clone}` that this line proves"
            ),
        },
    };
    let mut own: Vec<AccountingFacts> = Vec::new();
    let mut inherited: Vec<String> = Vec::new();
    if let Reading::Read(held) = inventory {
        for object in held.snapshots() {
            if object.createtxg > snapshots.clone_createtxg {
                own.push(the_accounting_or_its_refusal(
                    reads,
                    object.full_name().as_str(),
                ));
            } else {
                inherited.push(object.full_name());
            }
        }
    }
    return CloneRetentionFacts {
        clone: String::from(clone),
        dataset,
        snapshots: own,
        inherited,
        origin_snapshot: record.map(|held| return held.origin_snapshot.clone()),
    };
}

/// One snapshot's accounting,
/// or the four fields of a reply this account could not read,
/// so the retention line says why it states no number.
fn the_accounting_or_its_refusal(
    reads: &dyn CloneFactsReads,
    object: &str,
) -> AccountingFacts {
    let reading: Reading<AccountingFacts> = reads.accounting(object);
    return match reading {
        Reading::Read(held) => held,
        Reading::Missing {
            why,
        }
        | Reading::Unreadable {
            why,
        } => AccountingFacts {
            object: String::from(object),
            used: PropertyValue::Unreadable {
                why: why.clone(),
            },
            referenced: PropertyValue::Unreadable {
                why: why.clone(),
            },
            logical_referenced: PropertyValue::Unreadable {
                why: why.clone(),
            },
            written: PropertyValue::Unreadable {
                why,
            },
        },
    };
}
