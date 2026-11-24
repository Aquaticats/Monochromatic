//! What:
//!  The report of one damaged link:
//!  what was found,
//!  which copy of the pointer was read,
//!  what is kept,
//!  what a repair can and cannot restore,
//!  and the one next safe command.
//! Why:
//!  A damaged link with surviving administrative data can be put right by
//!  `git worktree repair <slot>` from the main checkout,
//!  while a registration Git already pruned loses HEAD,
//!  the index and the worktree's lock,
//!  and nothing rebuilds those from the pointer alone
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  plan step 12).
//!  A person needs that difference before they act,
//!  so every class says what happened,
//!  what is kept,
//!  what is restorable and what is lost,
//!  and the one command that is safe next.
//!  The report changes nothing:
//!  the restore and the guard before a prune act on their own arms.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = reportOf(linkFactsOf('one', record, cloneMount)); // lines() names the slot, the state and the next command
//! ```

use crate::worktree_clone_prune_pointer::link_facts::LinkFacts;
use crate::worktree_clone_prune_pointer::pointer_state::PointerState;

/// What:
///  Which of the four states one link is in.
/// Why:
///  The registration's survival decides what a repair can do,
///  and the record's bytes decide whether the wrapper can compare the pointer
///  at all,
///  so the four answers are the ones whose words and next command differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinkRepair = { kind: 'intact' } | { kind: 'damaged' } | { kind: 'unchecked' } | { kind: 'pruned'; backup } | { kind: 'unprovable'; reason };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkRepair {
    /// The registration stands,
    /// names this slot,
    /// and the pointer at the slot is the bytes the add recorded.
    Intact,
    /// The registration stands and names this slot,
    /// and the pointer is absent or holds other bytes than the record's.
    Damaged,
    /// The registration stands and names this slot,
    /// and the record keeps no pointer bytes,
    /// so the pointer was not compared.
    Unchecked,
    /// The registration is gone,
    /// and the record's backup is present or absent.
    Pruned {
        /// Whether the record keeps the pointer bytes.
        backup: bool,
    },
    /// Nothing here proves which of these the link is.
    Unprovable {
        /// What refused the reading.
        reason: String,
    },
}

impl LinkRepair {
    /// The state in the words a report prints.
    pub fn word(&self) -> &'static str {
        return match self {
            LinkRepair::Intact => "intact",
            LinkRepair::Damaged => "damaged, and the registration stands",
            LinkRepair::Unchecked => "not compared, and the registration stands",
            LinkRepair::Pruned { .. } => "pruned, and the registration is gone",
            LinkRepair::Unprovable { .. } => "cannot be proved",
        };
    }
}

/// What:
///  One link's report,
///  ready to print.
/// Why:
///  A caller prints one value,
///  so the state,
///  the evidence and the next command cannot come from two
///  readings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinkRepairReport = { slot: string; state: string; lines: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkRepairReport {
    /// The slot's name.
    pub slot: String,
    /// The state in words.
    pub state: &'static str,
    /// The evidence,
    /// the copy read,
    /// what is kept,
    /// what is restorable or lost,
    /// and the next safe command.
    pub lines: Vec<String>,
}

impl LinkRepairReport {
    /// The report as the lines a command prints.
    ///
    /// Why:
    ///  One shape for every reader,
    ///  and every line names the field it carries.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const lines = report.lines(); // ['slot `one`', 'link: intact', 'evidence: ...', ...]
    /// ```
    pub fn lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = vec![format!("slot `{}`", self.slot), format!("link: {}", self.state)];
        lines.extend(self.lines.iter().cloned());
        return lines;
    }
}

/// What:
///  The state one link is in,
///  in the order the readings prove it.
/// Why:
///  The registration is what a repair uses,
///  the pointer is what a prune reads,
///  and a reading that refused is named rather than folded into either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = classOf(facts);
/// ```
pub fn class_of(facts: &LinkFacts) -> LinkRepair {
    if facts.registration.is_none() {
        return LinkRepair::Unprovable { reason: String::from("the record names no Git registration yet, so there is no link to repair") };
    }
    if !facts.registration_stands {
        return LinkRepair::Pruned { backup: facts.backup };
    }
    match facts.registration_names_the_slot {
        Some(true) => {}
        Some(false) => {
            return LinkRepair::Unprovable {
                reason: String::from("the registration's `gitdir` names another path than this slot's `.git`, so a repair here would rebuild the wrong link"),
            };
        }
        None => {
            return LinkRepair::Unprovable { reason: String::from("the registration's `gitdir` could not be read, so nothing proves which worktree it belongs to") };
        }
    }
    return match &facts.pointer {
        PointerState::Intact => LinkRepair::Intact,
        PointerState::Missing | PointerState::Differs { .. } => LinkRepair::Damaged,
        PointerState::Present { .. } => LinkRepair::Unchecked,
        PointerState::Unprovable { reason } => LinkRepair::Unprovable { reason: reason.clone() },
    };
}

/// What:
///  The sentence that names which pointer copy the check read.
/// Why:
///  The design keeps the underlay pointer and the clone's own pointer,
///  and a person must know which of the two a report is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = `copy: ${facts.site.sentence()}`;
/// ```
fn copy_line(facts: &LinkFacts) -> String {
    return format!("copy: {}", facts.site.sentence());
}

/// What:
///  What happened,
///  in the facts the class came from.
/// Why:
///  The state's word alone does not say what stands and what does not,
///  and a person compares the report with `ls` and `git worktree list`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = evidenceLine('damaged', facts);
/// ```
fn evidence_line(state: &LinkRepair, facts: &LinkFacts) -> String {
    let pointer: String = match &facts.pointer {
        PointerState::Intact => String::from("is the bytes the add recorded"),
        PointerState::Missing => String::from("is absent"),
        PointerState::Differs { found } => format!("holds {found} bytes and not the ones the add recorded"),
        PointerState::Present { found } => format!("holds {found} bytes and the record keeps none to compare them with"),
        PointerState::Unprovable { reason } => format!("cannot be compared ({reason})"),
    };
    let registration: String = facts.registration.as_ref().map_or_else(|| return String::from("the record names none"), |path| return path.display().to_string());
    return match state {
        LinkRepair::Intact => format!("evidence: the pointer at `{}` {pointer}, and the registration at `{registration}` stands", facts.slot_path.join(".git").display()),
        LinkRepair::Damaged => format!("evidence: the pointer at `{}` {pointer}, and the registration at `{registration}` stands", facts.slot_path.join(".git").display()),
        LinkRepair::Unchecked => format!(
            "evidence: the record keeps no pointer bytes, and the registration at `{registration}` stands and names this slot, so the pointer at `{}` was not compared",
            facts.slot_path.join(".git").display()
        ),
        LinkRepair::Pruned { .. } => format!("evidence: the registration the record names at `{registration}` is gone, so Git pruned this worktree's administrative entry"),
        LinkRepair::Unprovable { reason } => format!("evidence: {reason}"),
    };
}

/// What:
///  What is kept,
///  what a repair restores,
///  what is lost,
///  and the one next safe command.
/// Why:
///  Each class answers the question a person has when they read it,
///  and the pruned arm never suggests that a rebuilt pointer file brings back
///  what only the registration held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rest = ['kept: ...', 'restorable: ...', 'next safe command: ...'];
/// ```
fn kept_and_restorable(state: &LinkRepair, facts: &LinkFacts) -> Vec<String> {
    let registration: String = facts.registration.as_ref().map_or_else(|| return String::from("the record names none"), |path| return path.display().to_string());
    let repair_command: String = format!("`git worktree repair {}`", facts.slot_path.display());
    let next_after_repair: String = format!("next safe command: {repair_command}, then `git worktree list`");
    return match state {
        LinkRepair::Intact => vec![
            format!("kept: the registration at `{registration}`, with HEAD, the index and the worktree's lock, and the clone dataset `{}`", facts.clone_dataset),
            String::from("restorable: nothing needs restoring, and `git worktree prune` keeps this registration because the pointer stands"),
            String::from("next safe command: nothing here; run Git in the worktree while the clone is mounted, or the removal to be rid of it"),
        ],
        LinkRepair::Damaged => vec![
            format!("kept: the registration at `{registration}`, with HEAD, the index and the worktree's lock, and the clone dataset `{}`", facts.clone_dataset),
            String::from("kept: the record keeps the exact pointer bytes the add wrote, and this wrapper writes them back before a `git worktree prune` it forwards"),
            format!(
                "restorable: the pointer file, which {repair_command} rebuilds from the registration; Git 2.56.1 exits 1 while rebuilding a missing pointer, so a second run of it exits 0. Not restorable this way: a registration Git already pruned"
            ),
            next_after_repair,
        ],
        LinkRepair::Unchecked => vec![
            format!("kept: the registration at `{registration}`, with HEAD, the index and the worktree's lock, and the clone dataset `{}`", facts.clone_dataset),
            format!(
                "restorable: a missing or changed pointer, which {repair_command} rebuilds from the registration's own `gitdir`; Git 2.56.1 exits 1 while rebuilding a missing pointer, so a second run of it exits 0"
            ),
            String::from("unknown: whether the pointer needs that repair, because the record keeps no bytes to compare it with"),
            String::from("next safe command: nothing while the worktree works; if Git reports the slot's `.git` unreadable, run the repair command above"),
        ],
        LinkRepair::Pruned { backup } => {
            let bytes: String = if *backup {
                String::from("lost: the registration's bytes are not the loss; the record's pointer bytes are kept and are what a re-registration is proven against")
            } else {
                String::from("lost: the pointer bytes as well, because the record keeps none")
            };
            vec![
                format!("kept: the clone dataset `{}` with the worktree's checked-out files, and the slot's own folder", facts.clone_dataset),
                format!("lost: the registration at `{registration}`, which held HEAD, the index, the worktree's per-worktree refs and the managed lock"),
                bytes,
                String::from("what a repair cannot do: `git worktree repair` rebuilds a link from a registration and cannot rebuild a registration, and a pointer restored from the record would point at nothing"),
                String::from("next safe command: decide explicitly whether to re-register at a commit you name or to remove the clone dataset; this wrapper never guesses a branch or a path, and nothing was changed here"),
            ]
        }
        LinkRepair::Unprovable { .. } => vec![
            String::from("kept: nothing was changed here"),
            String::from("next safe command: read what the evidence names, then run the command again"),
        ],
    };
}

/// What:
///  The report for one link's facts.
/// Why:
///  The class and the lines come from one set of facts,
///  so a report cannot print one class with another class's next command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = linkReportOf(facts);
/// ```
pub fn link_report_of(facts: &LinkFacts) -> LinkRepairReport {
    let state: LinkRepair = class_of(facts);
    let mut lines: Vec<String> = vec![evidence_line(&state, facts), copy_line(facts)];
    lines.extend(kept_and_restorable(&state, facts));
    return LinkRepairReport { slot: facts.slot.clone(), state: state.word(), lines };
}

/// What:
///  The lines one link's facts print,
///  for a caller that already holds the facts.
/// Why:
///  The prune guard's refusal reads the facts it refused on,
///  so it asks for the same lines rather than gathering them a second time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = linkReportLines(facts);
/// ```
pub fn link_report_lines(facts: &LinkFacts) -> Vec<String> {
    return link_report_of(facts).lines();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "repair_report_tests.rs"]
mod tests;
