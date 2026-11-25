//! What:
//!  What one clone and its own snapshots retain,
//!  in named ZFS accounting fields:
//!  the clone dataset's `used`,
//!  `usedbydataset`,
//!  `usedbysnapshots` and `written`,
//!  each own snapshot's `used` and `written`,
//!  and the history the clone inherited from its origin.
//! Why:
//!  The design asks the report to "report snapshot retention using named ZFS
//!  accounting fields where measurable;
//!  do not sum shared blocks as if each clone owned them.
//!  Explain retained underlays in checkout snapshots"
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints";
//!  plan step 18,
//!  "What it builds").
//!  ZFS counts one block in every dataset that shares it,
//!  so a total across clones would claim space that exists once,
//!  and this line therefore prints one object's own fields per clause and
//!  computes no sum.
//!  A clone that was cloned from a checkout snapshot inherits that snapshot,
//!  which is retained under the checkout's own name and is not this clone's
//!  history,
//!  so the line says which names are the origin's.
//!  A reply that does not carry one of the fields is unknown and never zero:
//!  zero bytes and "nobody read it" are the two answers a person must never see
//!  swapped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkCloneRetention({ clone, dataset, snapshots, inherited }); // pass | unknown
//! ```

use super::check_outcome::{CheckOutcome, pass, unknown};
use super::zfs_checks::Reading;
use crate::worktree_clone_mount_handler::sweep_dataset_facts::PropertyValue;

/// The property naming the blocks a dataset or snapshot itself holds.
pub const USED_PROPERTY: &str = "used";

/// The property naming the blocks the dataset's own data holds.
pub const USED_BY_DATASET_PROPERTY: &str = "usedbydataset";

/// The property naming the blocks the dataset's snapshots hold.
pub const USED_BY_SNAPSHOTS_PROPERTY: &str = "usedbysnapshots";

/// The property naming the blocks written since the object's creation,
/// or since its last `zfs snapshot`-creating act.
pub const WRITTEN_PROPERTY: &str = "written";

/// What:
///  The accounting fields of one dataset or snapshot.
/// Why:
///  Each field is a separate property of one object's own reply,
///  so the four names travel together and a control writes the numbers it
///  judges.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AccountingFacts = { object; used; written; usedByDataset; usedBySnapshots };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountingFacts {
    /// The dataset or snapshot the reply described.
    pub object: String,
    /// The object's `used` value.
    pub used: PropertyValue,
    /// The object's `written` value.
    pub written: PropertyValue,
    /// The dataset's `usedbydataset` value,
    /// and the reply's own answer for a snapshot,
    /// which has none.
    pub used_by_dataset: PropertyValue,
    /// The dataset's `usedbysnapshots` value,
    /// and the reply's own answer for a snapshot,
    /// which has none.
    pub used_by_snapshots: PropertyValue,
}

/// What:
///  One clone's retention:
///  its dataset's fields,
///  each own snapshot's,
///  the names it inherited,
///  and the origin snapshot it was cloned from.
/// Why:
///  The clone's own history and the history it inherited are counted under the
///  same dataset name,
///  and only the origin's names tell a person which rows a removal of this
///  clone would destroy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneRetentionFacts = { clone; dataset: Reading<AccountingFacts>; snapshots: AccountingFacts[]; inherited: string[]; originSnapshot?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloneRetentionFacts {
    /// The clone dataset whose retention this is.
    pub clone: String,
    /// The clone dataset's own accounting fields.
    pub dataset: Reading<AccountingFacts>,
    /// One value per own snapshot of the clone,
    /// in name order.
    pub snapshots: Vec<AccountingFacts>,
    /// The snapshots the clone inherited from its origin,
    /// by name.
    pub inherited: Vec<String>,
    /// The origin snapshot the clone was made from,
    /// when the record names one.
    pub origin_snapshot: Option<String>,
}

/// What:
///  One field's number,
///  or the sentence that says why this line cannot report it.
/// Why:
///  A field that is absent, textual or unreadable is no measurement,
///  and printing it as zero would tell a person the object holds no blocks
///  when nobody read the number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = number(facts.used, 'used', object); // number | throws the sentence
/// ```
fn number(value: &PropertyValue, property: &str, object: &str) -> Result<u64, String> {
    return match value.number() {
        Some(held) => Ok(held),
        None => Err(format!("{object} carries no `{property}` number today ({})", value.describe(property))),
    };
}

/// What:
///  The clause one object's fields print.
/// Why:
///  A snapshot's clause names whether it is the clone's own or inherited,
///  because only the own ones a removal of this clone destroys.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clause = `\`clone@s1\` (own) used 4096, written 4096`;
/// ```
fn clause(facts: &AccountingFacts, own: bool) -> Result<String, String> {
    let used: u64 = number(&facts.used, USED_PROPERTY, facts.object.as_str())?;
    let written: u64 = number(&facts.written, WRITTEN_PROPERTY, facts.object.as_str())?;
    if own {
        return Ok(format!("`{}` (own) `{USED_PROPERTY}` {used}, `{WRITTEN_PROPERTY}` {written}", facts.object));
    }
    return Ok(format!("`{}` `{USED_PROPERTY}` {used}, `{WRITTEN_PROPERTY}` {written}", facts.object));
}

/// What:
///  What one clone retains,
///  in named fields,
///  with the origin's own history kept apart.
/// Why:
///  Retention is a report and not a judgement:
///  a number that is there is a fact,
///  and a field nobody could read is the only reason this line says nothing
///  rather than a number.
///  The line names the underlay a clone leaves in the checkout's snapshots,
///  so a person who counts the checkout's `usedbysnapshots` knows which of it a
///  clone caused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkCloneRetention(facts); // pass | unknown
/// ```
pub fn check_clone_retention(facts: &CloneRetentionFacts) -> CheckOutcome {
    let clone: &str = facts.clone.as_str();
    let dataset: &AccountingFacts = match &facts.dataset {
        Reading::Missing { why } => return pass(format!("the pool holds no dataset `{clone}` ({why}), so it retains no blocks and its snapshots are the clone snapshot line's report")),
        Reading::Unreadable { why } => return unknown(format!("the clone dataset `{clone}` could not be read ({why}), so what it retains is unproved")),
        Reading::Read(held) => held,
    };
    let mut clauses: Vec<String> = Vec::new();
    let used: u64 = match number(&dataset.used, USED_PROPERTY, dataset.object.as_str()) {
        Ok(value) => value,
        Err(said) => return unknown(said),
    };
    let written: u64 = match number(&dataset.written, WRITTEN_PROPERTY, dataset.object.as_str()) {
        Ok(value) => value,
        Err(said) => return unknown(said),
    };
    let used_by_dataset: u64 = match number(&dataset.used_by_dataset, USED_BY_DATASET_PROPERTY, dataset.object.as_str()) {
        Ok(value) => value,
        Err(said) => return unknown(said),
    };
    let used_by_snapshots: u64 = match number(&dataset.used_by_snapshots, USED_BY_SNAPSHOTS_PROPERTY, dataset.object.as_str()) {
        Ok(value) => value,
        Err(said) => return unknown(said),
    };
    clauses.push(format!(
        "the clone dataset `{clone}`: `{USED_PROPERTY}` {used}, `{USED_BY_DATASET_PROPERTY}` {used_by_dataset}, `{USED_BY_SNAPSHOTS_PROPERTY}` {used_by_snapshots}, `{WRITTEN_PROPERTY}` {written}"
    ));
    for snapshot in &facts.snapshots {
        match clause(snapshot, true) {
            Ok(text) => clauses.push(text),
            Err(said) => return unknown(said),
        }
    }
    if facts.snapshots.is_empty() {
        clauses.push(String::from("it holds no snapshot of its own"));
    }
    let underlay: String = match (&facts.origin_snapshot, facts.inherited.is_empty()) {
        (Some(snapshot), true) => format!("The origin snapshot `{snapshot}` it was cloned from is retained under the checkout's own name, and no field above counts that snapshot's blocks as this clone's."),
        (Some(snapshot), false) => format!(
            "The origin snapshot `{snapshot}` it was cloned from and the {} snapshot(s) it inherited ({}) are retained under the checkout's own name, and no field above counts their blocks as this clone's.",
            facts.inherited.len(),
            facts.inherited.join(", ")
        ),
        (None, true) => String::from("The record names no origin snapshot, so no underlay in the checkout's snapshots is this clone's."),
        (None, false) => format!("The {} snapshot(s) it inherited ({}) are retained under the checkout's own name.", facts.inherited.len(), facts.inherited.join(", ")),
    };
    return pass(format!(
        "{}; {underlay} ZFS counts one shared block in every dataset that holds it, so these are separate fields of one object's own reply and this report sums none of them.",
        clauses.join("; ")
    ));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_retention_checks_tests.rs"]
mod tests;
