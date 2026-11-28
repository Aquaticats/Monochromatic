//! What:
//!  What one clone and its own snapshots retain,
//!  in named ZFS accounting fields:
//!  each object's `used`,
//!  `referenced`,
//!  `logicalreferenced`,
//!  and its `written` where the reply carries one,
//!  with the history the clone inherited kept apart.
//! Why:
//!  The design asks the report to "report snapshot retention using named ZFS
//!  accounting fields where measurable;
//!  do not sum shared blocks as if each clone owned them.
//!  Explain retained underlays in checkout snapshots"
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints";
//!  plan step 18,
//!  "What it builds").
//!  The four fields are the ones the object-stats reply carries for one object:
//!  `dsl_dataset_stats` adds `used`,
//!  `referenced`,
//!  `logicalreferenced` and,
//!  only once the object has a previous snapshot,
//!  `written`
//!  (`module/zfs/dsl_dataset.c` of `zfs-2.4.4`,
//!  `dsl_dataset_stats`).
//!  `usedbydataset`,
//!  `usedbysnapshots` and `logicalused` are not in that reply:
//!  `zfs get` computes them in userland,
//!  so a line that printed them as read facts would be printing arithmetic this
//!  route never made.
//!  ZFS counts one shared block in every dataset that holds it,
//!  so the line prints one object's own fields per clause and computes no sum.
//!  A field that is absent or not a number makes the line unknown and never
//!  zero,
//!  because zero bytes and "nobody read it" must never be swapped;
//!  `written` is the one exception the kernel states itself,
//!  and the line says so where the reply leaves it out.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkCloneRetention({ clone, dataset, snapshots, inherited }); // pass | unknown
//! ```

use super::check_outcome::{
    CheckOutcome,
    pass,
    unknown,
};
use super::zfs_checks::Reading;
use crate::worktree_clone_mount_handler::sweep_dataset_facts::PropertyValue;

/// The property naming the space one object and its descendants hold.
pub const USED_PROPERTY: &str = "used";

/// The property naming the space one object's own data holds.
pub const REFERENCED_PROPERTY: &str = "referenced";

/// The property naming the space one object's own data holds before
/// compression.
pub const LOGICAL_REFERENCED_PROPERTY: &str = "logicalreferenced";

/// The property naming the space written since the object's previous
/// snapshot,
/// which the kernel reports only for an object that has one.
pub const WRITTEN_PROPERTY: &str = "written";

/// What:
///  The accounting fields of one dataset or snapshot,
///  as its own object-stats reply carries them.
/// Why:
///  Each field is a separate property of one object's reply,
///  so the four names travel together and a control writes the numbers it
///  judges.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AccountingFacts = { object; used; referenced; logicalReferenced; written };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountingFacts {
    /// The dataset or snapshot the reply described.
    pub object: String,
    /// The object's `used` value.
    pub used: PropertyValue,
    /// The object's `referenced` value.
    pub referenced: PropertyValue,
    /// The object's `logicalreferenced` value.
    pub logical_referenced: PropertyValue,
    /// The object's `written` value,
    /// which the kernel adds only when the object has a previous snapshot.
    pub written: PropertyValue,
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
///  A field that is absent,
/// textual or unreadable is no measurement,
///  and printing it as zero would tell a person the object holds no blocks
///  when nobody read the number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = number(facts.used, 'used', object); // number | the sentence
/// ```
fn number(
    value: &PropertyValue,
    property: &str,
    object: &str,
) -> Result<u64, String> {
    return match value.number() {
        Some(held) => Ok(held),
        None => Err(format!(
            "{object} carries no `{property}` number in its reply ({})",
            value.describe(property)
        )),
    };
}

/// What:
///  The clause one object's fields print,
///  with `written` named as the kernel's own omission when it is absent.
/// Why:
///  An absent `written` is a state the kernel defines rather than a read that
///  failed,
///  so saying which it is keeps the rest of the numbers usable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clause = '`clone@s1` (own) `used` 4096, `referenced` 0, `logicalreferenced` 0, `written` 4096';
/// ```
fn clause(
    facts: &AccountingFacts,
    own: bool,
) -> Result<String, String> {
    let used: u64 = number(&facts.used, USED_PROPERTY, facts.object.as_str())?;
    let referenced: u64 = number(
        &facts.referenced,
        REFERENCED_PROPERTY,
        facts.object.as_str(),
    )?;
    let logical: u64 = number(
        &facts.logical_referenced,
        LOGICAL_REFERENCED_PROPERTY,
        facts.object.as_str(),
    )?;
    let written: String = match &facts.written {
        PropertyValue::Absent => format!(
            "`{WRITTEN_PROPERTY}` is not in the reply, which OpenZFS leaves out until the object has a previous snapshot"
        ),
        other => format!(
            "`{WRITTEN_PROPERTY}` {}",
            number(other, WRITTEN_PROPERTY, facts.object.as_str())?
        ),
    };
    let kind: &str = if own { " (own)" } else { "" };
    return Ok(format!(
        "`{}`{kind} `{USED_PROPERTY}` {used}, `{REFERENCED_PROPERTY}` {referenced}, `{LOGICAL_REFERENCED_PROPERTY}` {logical}, {written}",
        facts.object
    ));
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
///  so a person who counts the checkout's own space knows which of it a clone
///  caused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkCloneRetention(facts); // pass | unknown
/// ```
pub fn check_clone_retention(facts: &CloneRetentionFacts) -> CheckOutcome {
    let clone: &str = facts.clone.as_str();
    let dataset: &AccountingFacts = match &facts.dataset {
        Reading::Missing {
            why,
        } => {
            return pass(format!(
                "the pool holds no dataset `{clone}` ({why}), so it retains no blocks and its snapshots are the clone snapshot line's report"
            ));
        }
        Reading::Unreadable {
            why,
        } => {
            return unknown(format!(
                "the clone dataset `{clone}` could not be read ({why}), so what it retains is unproved"
            ));
        }
        Reading::Read(held) => held,
    };
    let mut clauses: Vec<String> = Vec::new();
    match clause(dataset, false) {
        Ok(text) => clauses.push(format!("the clone dataset {text}")),
        Err(said) => return unknown(said),
    }
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
        (Some(snapshot), true) => format!(
            "The origin snapshot `{snapshot}` it was cloned from is retained under the checkout's own name, and no field above counts that snapshot's blocks as this clone's."
        ),
        (Some(snapshot), false) => format!(
            "The origin snapshot `{snapshot}` it was cloned from and the {} snapshot(s) it inherited ({}) are retained under the checkout's own name, and no field above counts their blocks as this clone's.",
            facts.inherited.len(),
            facts.inherited.join(", ")
        ),
        (None, true) => String::from(
            "The record names no origin snapshot, so no underlay in the checkout's snapshots is this clone's.",
        ),
        (None, false) => format!(
            "The {} snapshot(s) it inherited ({}) are retained under the checkout's own name.",
            facts.inherited.len(),
            facts.inherited.join(", ")
        ),
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
