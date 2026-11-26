//! What:
//!  What the add's resume does with one `worktrees` entry,
//!  and with a listing that holds none for its slot.
//! Why:
//!  Every state Git's own failure path can leave gets one action and one order,
//!  so a resume never guesses and never runs a repository-wide prune
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "A resume after a kill between Git's add and the registered write").
//!  A complete registration is adopted,
//!  an incomplete one is cleared with the recipe,
//!  an entry this add did not write is left alone,
//!  and an incomplete one whose lock holds words this add did not write refuses.
//!  The lock's own state of a complete registration is not a row here:
//!  `worktree_clone_managed_lock` classifies it into S1 to S4 and the claim holder
//!  acts on that answer,
//!  so the add's resume,
//!  `doctor` and the removal read one order
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "The states,
//! the checks and the words",
//!  and "Where the code goes").
//!  Two entries that both attribute to this add are a refusal,
//!  because only a copied sentence could make two.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const finding = findRegistrations({ entries, slot, slotPath, nonce, otherSlots });
//! ```

use super::lock_words::{
    LockStanding,
    is_a_prefix_of_the_sentence,
    lock_standing_of,
};
use super::registration_finder::{
    EntryFacts,
    EntryRow,
    candidate_names,
    owned_by_another_slot,
};
use crate::worktree_clone_mount_protocol::RequestNonce;
use std::path::Path;

/// What:
///  What the resume does with one entry.
/// Why:
///  One answer per state keeps the recipe,
///  the adoption and the refusals in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Decision = { kind: 'adopt' } | { kind: 'clear'; row: Row } | { kind: 'refuse'; row: Row; sentence: string } | { kind: 'leave'; row: Row };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EntryDecision {
    /// Adopt the complete registration and write the recorded fields.
    ///
    /// Why:
    ///  The five facts prove the registration and nothing else is the finder's to answer:
    ///  the lock it holds is classified by `worktree_clone_managed_lock`,
    ///  which the claim holder asks before it writes the record.
    Adopt,
    /// Clear the incomplete entry with the recipe.
    Clear {
        /// The row the entry answered.
        row: EntryRow,
    },
    /// Refuse and change nothing.
    Refuse {
        /// The row,
        /// where one names the state.
        row: EntryRow,
        /// The sentence a person reads.
        sentence: String,
    },
    /// Leave the entry,
    /// naming why.
    Leave {
        /// The row that left it.
        row: EntryRow,
    },
}

/// What:
///  What one listing of `worktrees` says about this add.
/// Why:
///  The resume needs one answer rather than a walk it repeats,
///  and the names it read are what a refusal prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Finding = { decision: Decision; namesSeen: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    /// What the resume does.
    pub decision: ListingDecision,
    /// The names the listing showed.
    pub names_seen: Vec<String>,
}

/// What a whole listing decides,
/// which includes the two states no entry holds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ListingDecision {
    /// One entry decides it.
    Entry(EntryDecision),
    /// No entry for this slot and the slot holds no `.git`:
    /// re-run Git's add.
    RerunTheGitAdd,
    /// No entry for this slot and the slot holds a `.git`:
    /// nothing proves this add wrote it.
    Refuse {
        /// The sentence a person reads.
        sentence: String,
    },
}

/// The `.git` path inside the slot,
/// which a registration's `gitdir` must name.
fn slot_pointer_path(slot_path: &Path) -> std::path::PathBuf {
    return slot_path.join(".git");
}

/// Whether one entry attributes itself to this add,
/// which is what the two-match refusal counts.
fn attributes_itself(decision: &EntryDecision) -> bool {
    return matches!(
        decision,
        EntryDecision::Adopt | EntryDecision::Clear { .. } | EntryDecision::Refuse { .. }
    );
}

/// What:
///  Decide one entry.
/// Why:
///  The order is the finder's whole rule:
///  ownership exclusion first,
///  then the `gitdir` back-link,
///  then the five facts that make an entry complete,
///  and then the lock's words for an incomplete one.
///  An empty or too-short lock attributes nothing,
///  so a row whose other names pick it is cleared as that row says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decision = decide(entry, slot, slotPath, nonce, otherSlots);
/// ```
pub fn decide(
    entry: &EntryFacts,
    slot: &str,
    slot_path: &Path,
    nonce: &RequestNonce,
    other_slots: &[String],
) -> EntryDecision {
    if !candidate_names(slot)
        .iter()
        .any(|candidate| return candidate == &entry.name)
    {
        return EntryDecision::Leave {
            row: EntryRow::NotACandidate,
        };
    }
    if owned_by_another_slot(entry.name.as_str(), slot, other_slots) {
        return EntryDecision::Leave {
            row: EntryRow::OwnedByAnotherSlot,
        };
    }
    if entry
        .gitdir
        .as_deref()
        .is_some_and(|gitdir| return gitdir != slot_pointer_path(slot_path))
    {
        return EntryDecision::Leave {
            row: EntryRow::AnotherSlotsRegistration,
        };
    }
    let pointer_stands: bool = entry
        .slot_git_pointer
        .as_deref()
        .is_some_and(|bytes| return !bytes.is_empty());
    if entry.gitdir.is_some() && entry.commondir && entry.head && pointer_stands {
        return EntryDecision::Adopt;
    }
    let foreign: bool = entry.locked.as_deref().is_some_and(|bytes| {
        return lock_standing_of(bytes, nonce) == LockStanding::Foreign
            && !is_a_prefix_of_the_sentence(bytes, nonce);
    });
    if foreign {
        let first_line: String = entry
            .locked
            .as_deref()
            .map(|bytes| {
                return String::from_utf8_lossy(bytes)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string();
            })
            .unwrap_or_default();
        return EntryDecision::Refuse {
            row: EntryRow::ForeignLock,
            sentence: format!(
                "the registration at `worktrees/{}` is this slot's and its lock holds words this add did not write (`{first_line}`), so a person's lock stands on it. Nothing was removed or rewritten. Ways forward: run `git worktree unlock {slot}` if the worktree is to be resumed, or remove it with `git worktree remove {slot}`.",
                entry.name
            ),
        };
    }
    if entry.gitdir.is_none() && entry.slot_git_pointer.is_none() && entry.slot_other_entries == 0 {
        return match entry.locked.is_some() {
            true => EntryDecision::Clear {
                row: EntryRow::LockOnly,
            },
            false => EntryDecision::Clear {
                row: EntryRow::Bare,
            },
        };
    }
    if entry.gitdir.is_some() && entry.slot_git_pointer.is_none() {
        return EntryDecision::Clear {
            row: EntryRow::GitdirOnly,
        };
    }
    if pointer_stands && !entry.commondir {
        return EntryDecision::Clear {
            row: EntryRow::PointerOnly,
        };
    }
    if entry.commondir && !entry.head {
        return EntryDecision::Clear {
            row: EntryRow::Commondir,
        };
    }
    return EntryDecision::Refuse {
        row: EntryRow::NotACandidate,
        sentence: format!(
            "the entry at `worktrees/{}` answers none of the states this add's resume knows, and nothing here proves it wrote the files that stand. Nothing was removed. Ways forward: inspect the entry and the slot, then remove by hand what is not a worktree.",
            entry.name
        ),
    };
}

/// What:
///  Decide one whole listing for one slot.
/// Why:
///  The resume needs one answer,
///  and the two states no entry holds,
///  an empty slot and a slot with nothing that proves who wrote its `.git`,
///  are decided here rather than inside one entry's row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const finding = findRegistrations({ entries, slot, slotPath, nonce, otherSlots, slotPointer });
/// ```
pub fn find_registrations(
    entries: &[EntryFacts],
    slot: &str,
    slot_path: &Path,
    nonce: &RequestNonce,
    other_slots: &[String],
    slot_has_a_git_pointer: bool,
) -> Finding {
    let names_seen: Vec<String> = entries
        .iter()
        .map(|entry| return entry.name.clone())
        .collect();
    let mut attributing: Vec<(String, EntryDecision)> = Vec::new();
    for entry in entries {
        let decision: EntryDecision = decide(entry, slot, slot_path, nonce, other_slots);
        if attributes_itself(&decision) {
            attributing.push((entry.name.clone(), decision));
        }
    }
    if attributing.len() > 1 {
        let names: Vec<String> = attributing
            .iter()
            .map(|(name, _)| return name.clone())
            .collect();
        return Finding {
            decision: ListingDecision::Refuse {
                sentence: format!(
                    "two registrations attribute themselves to this add ({}), and only a copy of the lock sentence could make two. Nothing was removed or rewritten. Ways forward: inspect both entries and remove by hand the one that is not this add's.",
                    names.join("`, `")
                ),
            },
            names_seen,
        };
    }
    if let Some((_, decision)) = attributing.pop() {
        return Finding {
            decision: ListingDecision::Entry(decision),
            names_seen,
        };
    }
    if !slot_has_a_git_pointer {
        return Finding {
            decision: ListingDecision::RerunTheGitAdd,
            names_seen,
        };
    }
    return Finding {
        decision: ListingDecision::Refuse {
            sentence: format!(
                "no registration names this slot's `.git`, and the slot holds one, so nothing proves this add wrote that pointer. Nothing was removed. Ways forward: inspect the slot's `.git` and the registrations, then either run `git worktree remove {slot}` or restore the registration the pointer names."
            ),
        },
        names_seen,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "registration_resume_tests.rs"]
mod tests;
