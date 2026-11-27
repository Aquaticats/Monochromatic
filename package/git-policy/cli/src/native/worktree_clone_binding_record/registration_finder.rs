//! What:
//!  Which `worktrees/<name>` entries belong to one slot's add,
//!  which row of the design's states each entry is,
//!  and what the resume does with it.
//! Why:
//!  A kill inside Git's own `worktree add` can leave anything from a bare directory to a complete registration,
//!  and the resume must clear what Git's failure path left before it runs Git again,
//!  or Git names the new registration `<name>1` and the leftover stays locked
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "A resume after a kill between Git's add and the registered write").
//!  The finder never trusts a name alone:
//!  an entry is this add's by the recomposed lock sentence,
//!  by a `gitdir` back-link naming `<slot>/.git`,
//!  or by the name when the slot stands empty,
//!  and a name another slot's binding owns is left alone.
//!  It also never runs `git worktree prune`,
//!  which is repository-wide and would remove another slot's half-made registration.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const finding = findRegistrations({ entries, slot, nonce, ownedByOtherSlots });
//! // { decision: { kind: 'adopt' | 'clear' | 'refuse' | 'rerun' | 'leave'; ... } }
//! ```

use std::path::PathBuf;

/// How many candidate names a slot's finder considers,
/// which is the slot's own name and Git's first few counters.
/// Git's counter rises one at a time,
/// and a name two counters above the slot's
/// needs two entries at the slot's own name to exist already,
/// which the finder's refusals would have stopped before.
pub const MOST_CANDIDATES: usize = 8;

/// What one entry of `<common>/worktrees` holds,
/// as the finder reads it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntryFacts {
    /// The entry's name below `worktrees`.
    pub name: String,
    /// The bytes of its `locked` file,
    /// when it holds one.
    pub locked: Option<Vec<u8>>,
    /// The path its `gitdir` file names,
    /// the ending newline trimmed and a relative path resolved.
    pub gitdir: Option<PathBuf>,
    /// The bytes of `<slot>/.git`,
    /// when the slot holds that file.
    pub slot_git_pointer: Option<Vec<u8>>,
    /// Whether the entry holds a `commondir` file.
    pub commondir: bool,
    /// Whether the entry holds a `HEAD` file.
    pub head: bool,
    /// How many entries the slot holds besides `.git`.
    pub slot_other_entries: usize,
}

/// What:
///  Which state of the design's table one entry is.
/// Why:
///  One row per state Git's source can leave gives every kill point one action,
///  and the resume's recipe and adoption are chosen from the row rather than from a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Row = 'bare' | 'lock-only' | 'gitdir-only' | 'pointer-only' | 'commondir' | 'complete' | 'foreign-lock' | 'other-slot' | 'other-path' | 'not-a-candidate';
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EntryRow {
    /// Row 1:
    /// a bare directory,
    /// which a kill before Git wrote `locked` leaves.
    Bare,
    /// Row 2:
    /// `locked` only,
    /// holding the sentence,
    /// a covering prefix or nothing yet.
    LockOnly,
    /// Row 3:
    /// `locked` and `gitdir`,
    /// the slot still empty.
    GitdirOnly,
    /// Row 4:
    /// the slot's `.git` stands and names this registration.
    PointerOnly,
    /// Row 5:
    /// `commondir` stands too.
    Commondir,
    /// Row 6:
    /// a complete registration,
    /// which the add adopts.
    ///
    /// What its lock is stands outside this table:
    /// a complete entry's lock is classified into the states S1 to S4 by
    /// `worktree_clone_managed_lock`,
    /// and a missing one is written again by the claim holder there,
    /// so the same lock never has two readings.
    Complete,
    /// A lock whose words are neither this add's sentence nor a covering prefix of it.
    ForeignLock,
    /// At one of this slot's candidate names,
    /// but its `gitdir` names another path.
    AnotherSlotsRegistration,
    /// A candidate name another slot's binding owns.
    OwnedByAnotherSlot,
    /// A name Git could not derive from this slot.
    NotACandidate,
}

/// What:
///  The name Git derives from a slot's name,
///  through Git's own refname component sanitiser (`refs.c`).
/// Why:
///  Git names a worktree's administrative directory after the destination's last component
///  and sanitises that component as a refname,
///  so the finder must derive the same candidates or it misses the entry Git made.
///  It collapses `..`,
///  replaces a forbidden byte and a leading dot with `-`,
///  turns the brace of `@{` into `-` so `a@{b` becomes `a@-b`,
///  strips a trailing `.lock` repeatedly,
///  and does not strip a trailing dot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sanitized = [...name].reduce(...).replace(/(\.lock)+$/, '');
/// ```
pub fn git_sanitized_name(name: &str) -> String {
    let mut out: String = String::with_capacity(name.len());
    let bytes: &[u8] = name.as_bytes();
    let mut index: usize = 0;
    while index < bytes.len() {
        let byte: u8 = bytes[index];
        if byte == b'@' && bytes.get(index + 1) == Some(&b'{') {
            out.push('@');
            out.push('-');
            index += 2;
            continue;
        }
        if byte == b'.' {
            let mut run: usize = 0;
            while bytes.get(index + run) == Some(&b'.') {
                run += 1;
            }
            if run >= 2 {
                out.push('-');
                index += run;
                continue;
            }
            if out.is_empty() {
                out.push('-');
                index += 1;
                continue;
            }
        }
        if forbidden_refname_byte(byte) {
            out.push('-');
            index += 1;
            continue;
        }
        out.push(char::from(byte));
        index += 1;
    }
    loop {
        if !out.ends_with(".lock") {
            break;
        }
        out.truncate(out.len() - ".lock".len());
    }
    return out;
}

/// Whether one byte may not stand in a refname (`check_refname_character`).
fn forbidden_refname_byte(byte: u8) -> bool {
    return byte < 0x20
        || byte == 0x7f
        || matches!(byte, b' ' | b'~' | b'^' | b':' | b'?' | b'*' | b'[' | b'\\');
}

/// What:
///  The names Git could have made for one slot:
///  the sanitised name,
///  then Git's decimal counters.
/// Why:
///  A stale registration makes Git name the new one `<name>1`,
///  so the finder must reach the named entry and every counter git may have appended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const candidates = [base, ...Array.from({length: MOST_CANDIDATES - 1}, (_, i) => `${base}${i + 1}`)];
/// ```
pub fn candidate_names(slot: &str) -> Vec<String> {
    let base: String = git_sanitized_name(slot);
    let mut names: Vec<String> = Vec::with_capacity(MOST_CANDIDATES);
    names.push(base.clone());
    for counter in 1..MOST_CANDIDATES {
        names.push(format!("{base}{counter}"));
    }
    return names;
}

/// Whether one slot's binding owns `name`,
/// which is any name that slot's own candidates hold.
pub fn owned_by_another_slot(
    name: &str,
    slot: &str,
    other_slots: &[String],
) -> bool {
    return other_slots.iter().any(|other| {
        return other != slot
            && candidate_names(other)
                .iter()
                .any(|candidate| return candidate == name);
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "registration_finder_tests.rs"]
mod tests;
