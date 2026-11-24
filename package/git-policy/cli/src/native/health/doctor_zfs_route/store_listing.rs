//! What:
//!  The slot identity store's folder listing as plan step 18 reports it:
//!  the entry count beside the records,
//!  a record filed under a name its key does not match,
//!  a foreign entry,
//!  the two known staging forms with their count and age,
//!  and a dot entry that matches neither form.
//! Why:
//!  The per-slot store gave up the one read that saw every record,
//!  so the content half of the old guard is `doctor`'s on demand and the
//!  listing half is this step's
//!  (`doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
//!  "The other readers";
//!  `doc/decision/cli-git-zfs-slot-identity-store-format.md`,
//!  "Who removes a killed writer's staging file").
//!  A staging name proves a writer was killed between its bytes and its link,
//!  and a third dot form would prove a writer this design never accounted for,
//!  which is why the two known forms and every other dot entry are told apart.
//!  The listing is a fact handed in,
//!  so a control writes the names and ages it chose and reads no store.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = { entries: [{ name: '.parent', modifiedSecondsAgo: 12 }, ...], parent, slots };
//! const outcome = check_store_entries(facts); // pass | finding('record'|'name') | unknown
//! ```

use super::check_outcome::{CheckOutcome, finding, pass, unknown};
use crate::worktree_clone_mount::{PARENT_RECORD_NAME, is_known_staging_name, slot_name_refusal};

/// What:
///  One name the store folder lists,
///  with how long ago its entry last changed where that could be read.
/// Why:
///  A staging entry's age is what tells a person whether a writer died a
///  minute or a month ago,
///  and a name whose age could not be read must not pass for a fresh one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ListedEntry = { name: string; modifiedSecondsAgo?: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListedEntry {
    /// The name as the folder listed it.
    pub name: String,
    /// The entry's age in whole seconds,
    /// or nothing when its own metadata could not be read.
    pub modified_seconds_ago: Option<u64>,
}

/// What:
///  What one record file answered when it was read as its own name.
/// Why:
///  A record that parses and carries its name's key is the ordinary state,
///  a record carrying another key is the misfiled one this step reports,
///  and a record this account could not read proves nothing either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordFacts = { kind: 'read'; recorded: string } | { kind: 'misfiled'; recorded: string } | { kind: 'unreadable'; why: string } | { kind: 'absent' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordFacts {
    /// The record carries the key its file's name belongs to.
    Read {
        /// The key the record carries,
        /// which is the slot's name or `.parent`.
        recorded: String,
    },
    /// The record carries another key than its file's name.
    Misfiled {
        /// The key the record carries.
        recorded: String,
    },
    /// The record could not be read.
    Unreadable {
        /// What the read answered.
        why: String,
    },
    /// No file is filed under the name the listing held.
    Absent,
}

/// What:
///  One slot record the listing held,
///  by name and by what its read answered.
/// Why:
///  The name and the record travel together,
///  so a misfiled record can be reported with both the file it sits under and
///  the key it carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SlotRecordFacts = { name: string; facts: RecordFacts };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlotRecordFacts {
    /// The name the record is filed under.
    pub name: String,
    /// What the read answered.
    pub facts: RecordFacts,
}

/// What:
///  The store folder's listing and every record in it,
///  from one visit.
/// Why:
///  One listing is what the design says `doctor` makes,
///  and the three checks it feeds read the same instant of the folder,
///  so no check can judge a set another listing would have changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StoreListingFacts = { entries: ListedEntry[]; parent: RecordFacts; slots: SlotRecordFacts[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoreListingFacts {
    /// Every name the folder listed,
    /// in the order the folder gave them.
    pub entries: Vec<ListedEntry>,
    /// What `.parent` answered.
    pub parent: RecordFacts,
    /// Each listed slot name and its record.
    pub slots: Vec<SlotRecordFacts>,
}

/// What:
///  The names the listing holds that are the two known staging forms.
/// Why:
///  A staging name is never read as a record,
///  so the only question about one is how many there are and how long they
///  have stood,
///  which is what the step removes and reports.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const staging = entries.filter((entry) => isKnownStagingName(entry.name));
/// ```
pub fn known_staging_entries(facts: &StoreListingFacts) -> Vec<&ListedEntry> {
    return facts.entries.iter().filter(|entry| return is_known_staging_name(&entry.name)).collect();
}

/// What:
///  Whether a listed name is a dot entry this layout does not account for.
/// Why:
///  `.parent` and the two staging forms are the only dot names a writer of
///  this repository makes,
///  so a third shape means a writer nobody designed for and must be reported
///  rather than removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unknownDot = (name) => name.startsWith('.') && name !== '.parent' && !isKnownStagingName(name);
/// ```
pub fn is_unknown_dot_entry(name: &str) -> bool {
    return name.starts_with('.') && name != PARENT_RECORD_NAME && !is_known_staging_name(name);
}

/// What:
///  Whether a listed name is neither a record this layout names nor a staging
///  entry.
/// Why:
///  The slot grammar is the store's own rule,
///  and a name it refuses could be a slot whose record a later read would miss,
///  so the listing reports it rather than treating it as a slot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const foreign = (name) => name !== '.parent' && !isKnownStagingName(name) && !name.startsWith('.') && slotNameRefusal(name) !== undefined;
/// ```
pub fn is_foreign_entry(name: &str) -> bool {
    if name == PARENT_RECORD_NAME || name.starts_with('.') || is_known_staging_name(name) {
        return false;
    }
    return slot_name_refusal(name).is_some();
}

/// How one age reads in a sentence.
fn age_text(age: Option<u64>) -> String {
    return match age {
        Some(seconds) => format!("{seconds}s old"),
        None => String::from("age unreadable"),
    };
}

/// How many records the listing holds:
/// the parent's where it is there,
/// and each slot's.
fn record_count(facts: &StoreListingFacts) -> usize {
    let parent: usize = usize::from(!matches!(facts.parent, RecordFacts::Absent));
    return parent + facts.slots.len();
}

/// What:
///  The folder's entry count beside its records,
///  every record filed under a name its key does not match,
///  and every foreign entry.
/// Why:
///  A misfiled record is the one state only a reader of every record finds,
///  and a foreign name could be a slot a later read would miss,
///  so both are findings while a record this account could not read leaves the
///  answer unknown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_store_entries(facts); // pass | finding('record'|'name') | unknown
/// ```
pub fn check_store_entries(facts: &StoreListingFacts) -> CheckOutcome {
    let mut misfiled: Vec<String> = Vec::new();
    for slot in &facts.slots {
        if let RecordFacts::Misfiled { recorded } = &slot.facts {
            misfiled.push(format!("`{}` carries the key `{recorded}`", slot.name));
        }
    }
    if let RecordFacts::Misfiled { recorded } = &facts.parent {
        misfiled.push(format!("`{PARENT_RECORD_NAME}` carries the key `{recorded}`"));
    }
    let foreign: Vec<String> = facts.entries.iter().filter(|entry| return is_foreign_entry(&entry.name)).map(|entry| return format!("`{}`", entry.name)).collect();
    if !misfiled.is_empty() {
        return finding("record", format!("the store holds a record under a name its key does not match: {}", misfiled.join("; ")));
    }
    if !foreign.is_empty() {
        return finding("name", format!("the store holds {} contraindicated name(s): {}; the slot grammar refuses each, so a slot under one would be missed by a later read", foreign.len(), foreign.join(", ")));
    }
    let mut unreadable: Vec<String> = Vec::new();
    for slot in &facts.slots {
        if let RecordFacts::Unreadable { why } = &slot.facts {
            unreadable.push(format!("`{}` ({why})", slot.name));
        }
    }
    if let RecordFacts::Unreadable { why } = &facts.parent {
        unreadable.push(format!("`{PARENT_RECORD_NAME}` ({why})"));
    }
    if !unreadable.is_empty() {
        return unknown(format!("the store holds {} record(s) this account could not read: {}, so the set is unproved", unreadable.len(), unreadable.join(", ")));
    }
    return pass(format!("the store lists {} entries: {} record(s) ({} slot(s) and {})", facts.entries.len(), record_count(facts), facts.slots.len(), if matches!(facts.parent, RecordFacts::Absent) { "no parent record" } else { "the parent's record" }));
}

/// What:
///  The two staging forms a killed writer left,
///  with their count and age.
/// Why:
///  No reader reads a staging name,
///  so a leftover changes no read and no mount,
///  but it proves a writer died mid-publication and this step is where a
///  machine without an adoption still learns of it before removing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_staging_forms(facts); // pass | finding('staging')
/// ```
pub fn check_staging_forms(facts: &StoreListingFacts) -> CheckOutcome {
    let staging: Vec<&ListedEntry> = known_staging_entries(facts);
    if staging.is_empty() {
        return pass(String::from("no known staging form is in the store"));
    }
    let described: Vec<String> = staging.iter().map(|entry| return format!("`{}` ({})", entry.name, age_text(entry.modified_seconds_ago))).collect();
    return finding("staging", format!("{} known staging form(s) left by a killed writer: {}; no reader reads one, and this step removes them", staging.len(), described.join(", ")));
}

/// What:
///  Every dot entry that matches neither staging form,
///  which no writer of this repository makes.
/// Why:
///  Removing an unknown name would destroy a stranger's file,
///  so the step reports the name and leaves it where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_unknown_dot_entries(facts); // pass | finding('name')
/// ```
pub fn check_unknown_dot_entries(facts: &StoreListingFacts) -> CheckOutcome {
    let unknown_dots: Vec<String> = facts.entries.iter().filter(|entry| return is_unknown_dot_entry(&entry.name)).map(|entry| return format!("`{}` ({})", entry.name, age_text(entry.modified_seconds_ago))).collect();
    if unknown_dots.is_empty() {
        return pass(String::from("every dot entry is `.parent` or one of the two known staging forms"));
    }
    return finding("name", format!("{} dot entry(s) match neither known staging form and no writer of this repository makes them: {}; they are reported and left in place", unknown_dots.len(), unknown_dots.join(", ")));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "store_listing_tests.rs"]
mod tests;
