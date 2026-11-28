//! What:
//!  What the shutdown pass answers:
//!  the boot it ran in,
//!  the readiness record it withdrew,
//!  one ending per mount it reached,
//!  and the lines a person or the unit's journal reads.
//! Why:
//!  Every mode ends in the same two answers to an operator or a unit:
//!  a line saying what happened,
//!  or a refusal saying why nothing happened
//!  (`crate::worktree_clone_mount_handler::handler_mode_dispatch`).
//!  A pass over many mounts must answer each mount rather than a total,
//!  and a clone that refused must not stop the ones after it,
//!  so one value carries every ending and the summary counts them
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 19,
//!  "Acceptance tests").
//!  Nothing here writes:
//!  the lines are the pass's whole record,
//!  because the pass changes no durable intent and files no receipt.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = { boot, withdrawal, verifiedNothing, reached };
//! outcome.lines(); // ['shutdown-unmount: …', 'shutdown-unmount: unmounted …']
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_readiness_file_withdraw::{
    RecordStood,
    Withdrawal,
};
use std::path::PathBuf;

/// What:
///  One namespace that holds a mount of the clone the pass tried to unmount.
/// Why:
///   A busy unmount names its holder,
///  and a holder is a namespace and the task whose table was read,
///  not a process that comes and goes
///  (`crate::clone_holder_scan::namespace_holder::NamespaceHolder`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Holder = { namespace: string; task: string; mounts: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Holder {
    /// The kernel's own namespace identity,
    /// such as `mnt:[4026532001]`.
    pub(crate) namespace: String,
    /// The live task whose table of that namespace was read.
    pub(crate) task: PathBuf,
    /// How many mounts of the clone that namespace holds.
    pub(crate) mounts: usize,
}

/// What:
///  What one holder scan found,
///  or why no scan could be made.
/// Why:
///   A busy unmount names the holders it found,
///  and a scan that could not run is its own answer:
///  reading a failed scan as a machine with no holder would tell a person the
///  clone is clear when nothing was read
///  (`crate::clone_holder_scan::holder_scan::HolderScan`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Holdings = { holders: Holder[]; incompleteReads: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Holdings {
    /// Every namespace other than this process's own that holds a mount of the
    ///  clone,
    ///  in the scan's order.
    pub(crate) holders: Vec<Holder>,
    /// How many mounts of the clone stand in this process's own mount
    ///  namespace,
    ///  which is where an unmount from here would act.
    pub(crate) own_mounts: usize,
    /// How many reads the scan could not complete.
    pub(crate) incomplete_reads: usize,
}

/// What:
///  What one mount's turn did.
/// Why:
///   A person reads one line per mount,
///  and a mount that left is not the same answer as a mount a task holds,
///  so each ending is its own word and its own sentence
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Boot and shutdown unit":
/// "report which clone holds the mount,
///  and never force or detach it").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountEnding = { kind: 'unmounted'; mountId } | { kind: 'held'; said; holders } | { kind: 'left'; said };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MountEnding {
    /// The kernel unmounted the clone and the mount is gone.
    Unmounted {
        /// The mount's kernel number,
        /// which the row and the descriptor both named.
        mount_id: u64,
    },
    /// The kernel refused the unmount because something holds the mount,
    /// and the mount is exactly where it was.
    Held {
        /// The mount's kernel number.
        mount_id: u64,
        /// What the kernel answered.
        said: String,
        /// What the holder scan found,
        /// or why no scan could be made.
        holdings: Result<Holdings, String>,
    },
    /// The pass did not unmount this mount,
    /// and says why.
    Left {
        /// Why the mount stands.
        said: String,
    },
}

/// What:
///  One slot the pass reached and what its turn did.
/// Why:
///   The slot names the place a person goes to look,
///  and the clone names the dataset whose mount it was,
///  so one line says both without a second read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReachedMount = { slot: string; clone: string; ending: MountEnding };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReachedMount {
    /// The slot's name,
    /// which is the folder's last name under `.claude/worktrees`.
    pub(crate) slot: String,
    /// The clone dataset the mount's source named.
    pub(crate) clone: String,
    /// Where the mount's whole path is.
    pub(crate) slot_path: PathBuf,
    /// What the turn did with it.
    pub(crate) ending: MountEnding,
}

/// What:
///  One whole shutdown pass,
///  as the answer a person or a unit reads.
/// Why:
///   The pass withdraws readiness before it decides anything else,
///  so what stood under the record's name belongs to the answer whatever the
///  pass then did;
///  and a boot whose record proved nothing has an answer of its own rather
///  than an empty list a reader would take for a machine with no clones
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Boot and shutdown unit").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PassOutcome = { boot; withdrawal; verifiedNothing; reached };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PassOutcome {
    /// The boot the pass ran in.
    pub(crate) boot: String,
    /// What stood under the readiness record's name,
    /// and the proof that nothing answers there now.
    pub(crate) withdrawal: Withdrawal,
    /// Why no mount was unmounted,
    /// when this boot's record proved nothing the pass may act on.
    pub(crate) verified_nothing: Option<String>,
    /// Every mount the pass reached,
    /// in mount order.
    pub(crate) reached: Vec<ReachedMount>,
}

impl PassOutcome {
    /// How many mounts the kernel took down.
    pub(crate) fn unmounted(&self) -> usize {
        return self
            .reached
            .iter()
            .filter(|mount| return matches!(mount.ending, MountEnding::Unmounted { .. }))
            .count();
    }

    /// How many mounts something holds.
    pub(crate) fn held(&self) -> usize {
        return self
            .reached
            .iter()
            .filter(|mount| return matches!(mount.ending, MountEnding::Held { .. }))
            .count();
    }

    /// How many mounts the pass left standing for another reason.
    pub(crate) fn left(&self) -> usize {
        return self
            .reached
            .iter()
            .filter(|mount| return matches!(mount.ending, MountEnding::Left { .. }))
            .count();
    }

    /// What the readiness record's name held when the pass withdrew it,
    /// as the summary says it.
    fn readiness_word(&self) -> String {
        return match &self.withdrawal.stood {
            RecordStood::Nothing => String::from("this boot had published no readiness record"),
            RecordStood::Record(record) => format!(
                "readiness for boot {} was withdrawn before any mount was touched",
                record.boot
            ),
            RecordStood::Unreadable {
                said,
            } => format!(
                "a readiness record this route's reader refuses was removed before any mount was touched ({said})"
            ),
        };
    }

    /// What:
    ///  Every line this pass answers,
    ///  the summary first and one line per mount after it.
    /// Why:
    ///   An operator greps the summary and reads a mount's own line where the
    ///  summary says something was left,
    ///  and the mode writes these lines to the journal it already writes to
    ///  (`crate::worktree_clone_mount_handler::handler_mode_dispatch`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// outcome.lines(); // one summary line, then one line per reached mount
    /// ```
    pub fn lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        lines.push(self.summary());
        for mount in &self.reached {
            lines.push(one_mount_line(mount));
        }
        return lines;
    }

    /// The one line that counts the whole pass.
    ///
    /// Why:
    ///  A pass with nothing to do says so rather than printing nothing,
    ///  because an empty answer and a pass that stopped read the same in a
    ///  journal.
    fn summary(&self) -> String {
        let readiness: String = self.readiness_word();
        if let Some(said) = &self.verified_nothing {
            return format!(
                "shutdown-unmount in boot {}: {readiness}; {said}, so no mount was unmounted and no durable intent was changed",
                self.boot
            );
        }
        return format!(
            "shutdown-unmount in boot {}: {readiness}; {} mount(s) of this boot's clone container stood at a slot, {} unmounted, {} held, {} left, and no durable intent was changed",
            self.boot,
            self.reached.len(),
            self.unmounted(),
            self.held(),
            self.left()
        );
    }
}

/// What:
///  One mount's line.
/// Why:
///   Each ending names the slot,
///  the clone and what happened,
///  so a person reads one line rather than the summary and a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// oneMountLine({ slot: 'one', clone: 'tank/…', ending: { kind: 'unmounted', mountId: 41 } });
/// ```
fn one_mount_line(mount: &ReachedMount) -> String {
    let place: String = format!(
        "slot `{}` at {} of `{}`",
        mount.slot,
        mount.slot_path.display(),
        mount.clone
    );
    return match &mount.ending {
        MountEnding::Unmounted {
            mount_id,
        } => format!("shutdown-unmount: unmounted {place}, which was mount {mount_id}"),
        MountEnding::Held {
            mount_id,
            said,
            holdings,
        } => format!(
            "shutdown-unmount: held {place}, which is mount {mount_id} and stays exactly as it was; the kernel answered `{said}`, and {}. Nothing was forced and nothing was detached",
            holdings_sentence(holdings)
        ),
        MountEnding::Left {
            said,
        } => format!("shutdown-unmount: left {place} as it was: {said}"),
    };
}

/// What:
///  What the holder scan found,
///  as one held mount's line says it.
/// Why:
///   The namespaces that hold the clone and the reads that could not be
///  completed are two facts a person acts on differently,
///  and a scan that could not be made is a third
///  (`crate::clone_holder_scan::holder_scan::HolderScan`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdingsSentence({ holders, incompleteReads }); // 'the all-namespace scan named … (2 read(s) it could not complete)'
/// ```
fn holdings_sentence(holdings: &Result<Holdings, String>) -> String {
    let found: &Holdings = match holdings {
        Ok(found) => found,
        Err(said) => return said.clone(),
    };
    if found.holders.is_empty() && found.own_mounts == 0 {
        return format!(
            "the all-namespace scan named no namespace holding a mount of it, and {} read(s) of it could not be completed",
            found.incomplete_reads
        );
    }
    if found.holders.is_empty() {
        return format!(
            "the all-namespace scan found {} mount(s) of it in this process's own mount namespace and no other namespace holding one, and {} read(s) of it could not be completed",
            found.own_mounts, found.incomplete_reads
        );
    }
    let named: Vec<String> = found
        .holders
        .iter()
        .map(|holder| return holder_line(holder))
        .collect();
    return format!(
        "the all-namespace scan found {own} mount(s) of it in this process's own mount namespace and named {list} ({count} read(s) it could not complete)",
        own = found.own_mounts,
        list = named.join("; "),
        count = found.incomplete_reads
    );
}

/// What:
///  One holder as a line names it.
/// Why:
/// The namespace is what holds the mount and the task is the evidence a live
///  task was there,
///  so both travel into the line
///  (`crate::clone_holder_scan::namespace_holder::NamespaceHolder`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holderLine({ namespace: 'mnt:[4026532001]', task: '/proc/1234', mounts: 1 });
/// ```
fn holder_line(holder: &Holder) -> String {
    return format!(
        "`{}` read through {} with {} mount(s) of it",
        holder.namespace,
        holder.task.display(),
        holder.mounts
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "shutdown_unmount_answer_tests.rs"]
mod tests;
