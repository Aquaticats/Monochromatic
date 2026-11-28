//! What:
//!  Every word the forced arm writes or answers:
//!  the seven refusals the receipt's failed step and the report name,
//!  the two clone-local properties,
//!  the scan file's and the progress file's suffixes,
//!  the reserved prefix of the destination directory,
//!  the receipt's bound on a failed-step word,
//!  and the stage words the progress file carries.
//! Why:
//!  The design fixes each word exactly and one list keeps the writer,
//!  the report and the controls spelling the same bytes
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The refusal,
//!  and what it leaves",
//!  "The act's mechanism",
//!  and "The callers and its protected set").
//!  A receipt's `failedStep` and `errno` words are bounded at 64 bytes and the
//!  scan file and the progress file sit beside the receipt in the same
//!  root-owned folder,
//!  which is why their suffixes live here rather than beside
//!  their readers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type RefusalWord = 'caller-protected-holds' | 'holder-unendable' | ...;
//! const CALLER_PROPERTY = 'org.monochromatic:git-worktree-clone-removal-caller';
//! ```

/// The clone's local property carrying who asked for the forced removal.
/// Value:
/// `v1:<nonce>:<pid>:<starttime>:<parent-pid>:<parent-starttime>:<boot>`.
pub const CALLER_PROPERTY: &str = "org.monochromatic:git-worktree-clone-removal-caller";

/// The clone's local property that turns the same request into a forced one.
/// Value:
/// `v1:<nonce>`.
/// Why:
///  The design names "the force property" in the write order and not its name
///  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
///  "The time between the scan and the ending",
///  and V3).
///  This build chooses a name of its own and the report says the design left it open,
///  so a later edit can rename it in one place with the wrapping step's own grammar.
pub const FORCE_PROPERTY: &str = "org.monochromatic:git-worktree-clone-removal-force";

/// The suffix of the file root publishes the holder scan's own text in,
/// beside the receipt,
/// named only by the dataset GUID.
pub const SCAN_FILE_SUFFIX: &str = ".holder-scan";

/// The suffix of the file the handler advances while its act lives.
pub const PROGRESS_FILE_SUFFIX: &str = ".removal-progress";

/// The prefix of the directory way 7 makes in the checkout and removes after the unmount.
/// A later run's sweep and the cancel command remove one that carries it and this clone's GUID.
pub const DESTINATION_DIRECTORY_PREFIX: &str = ".git-worktree-clone-removal-";

/// The most bytes the receipt's own grammar allows a failed step and an errno.
/// Why:
///  A refusal word travels in that field,
/// so a word longer than this bound would
///  be cut by the receipt and read back as another word.
pub const FAILED_STEP_MAXIMUM_BYTES: usize = 64;

/// What:
///  One word a forced removal answers with,
///  which is the receipt's failed step and the report's first line.
/// Why:
///  A person,
///  `doctor` and the sibling design each read the word,
///  and the design fixes them:
///  `caller-protected-holds` for a holder in the set this arm never ends,
///  `holder-unendable` for one it could not end or a namespace its reach
///  refused,
///  `holder-unseen` for an `EBUSY` after an empty search,
///  `holder-blind` for a read it could not make,
///  `holder-locked` for a copy the kernel locked and this arm may not stop,
///  `holder-structural` for a change the kernel refused in the middle of the
///  structural stage,
///  and `caller-gone` for a wrapper that ended before the act did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RefusalWord = 'caller-protected-holds' | 'holder-unendable' | 'holder-unseen' | 'holder-blind' | 'holder-locked' | 'holder-structural' | 'caller-gone';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefusalWord {
    /// A holder is the command's caller,
    /// in its session,
    /// or above it.
    CallerProtectedHolds,
    /// A holder this arm could not end,
    /// or a namespace its reach refused.
    HolderUnendable,
    /// An unmount inside a holder's namespace answered `EBUSY` after the matcher found no holder.
    HolderUnseen,
    /// A read the preflight could not make.
    HolderBlind,
    /// A copy whose mount namespace is owned by a user namespace of its own.
    HolderLocked,
    /// A mount change the kernel refused in the middle of the structural stage.
    HolderStructural,
    /// The wrapper that asked ended before the act did.
    CallerGone,
}

impl RefusalWord {
    /// The word the receipt's failed step carries and the report prints.
    pub fn word(self) -> &'static str {
        return match self {
            RefusalWord::CallerProtectedHolds => "caller-protected-holds",
            RefusalWord::HolderUnendable => "holder-unendable",
            RefusalWord::HolderUnseen => "holder-unseen",
            RefusalWord::HolderBlind => "holder-blind",
            RefusalWord::HolderLocked => "holder-locked",
            RefusalWord::HolderStructural => "holder-structural",
            RefusalWord::CallerGone => "caller-gone",
        };
    }

    /// The refusal a stored word names,
    /// or nothing for a word this grammar does not carry.
    pub fn parse(word: &str) -> Option<RefusalWord> {
        return match word {
            "caller-protected-holds" => Some(RefusalWord::CallerProtectedHolds),
            "holder-unendable" => Some(RefusalWord::HolderUnendable),
            "holder-unseen" => Some(RefusalWord::HolderUnseen),
            "holder-blind" => Some(RefusalWord::HolderBlind),
            "holder-locked" => Some(RefusalWord::HolderLocked),
            "holder-structural" => Some(RefusalWord::HolderStructural),
            "caller-gone" => Some(RefusalWord::CallerGone),
            _ => None,
        };
    }
}

/// What:
///  How far the act got,
///  as the progress file's stage word names it.
/// Why:
///  The wrapper reads the stage to tell a stalled act from a slow one,
///  and the counter beside it is what actually says the handler lives,
///  so the words are a vocabulary rather than a promise about what happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Stage = 'preflight' | 'structural' | 'signals' | 'unmount' | 'done';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    /// The read-only preflight over every holder namespace.
    Preflight,
    /// The structural stage's mount changes and signals' own directory.
    Structural,
    /// The signal stage's `SIGTERM`,
    /// wait and `SIGKILL`.
    Signals,
    /// The unmount stage.
    Unmount,
    /// The act ended and filed its scan and receipt.
    Done,
}

impl Stage {
    /// The word the progress file carries.
    pub fn word(self) -> &'static str {
        return match self {
            Stage::Preflight => "preflight",
            Stage::Structural => "structural",
            Stage::Signals => "signals",
            Stage::Unmount => "unmount",
            Stage::Done => "done",
        };
    }

    /// What:
    ///  The stage one word names,
    ///  which is how a reader turns a stored word back into the vocabulary.
    /// Why:
    ///  The progress file carries the word and a reader must refuse a word this
    ///  build does not write rather than guess a stage.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const stage = Stage.parseWord('signals'); // undefined for another word
    /// ```
    pub fn parse_word(word: &str) -> Option<Stage> {
        return match word {
            "preflight" => Some(Stage::Preflight),
            "structural" => Some(Stage::Structural),
            "signals" => Some(Stage::Signals),
            "unmount" => Some(Stage::Unmount),
            "done" => Some(Stage::Done),
            _ => None,
        };
    }

    /// What:
    ///  The stage one order value names,
    ///  which a counter's own word reads back.
    /// Why:
    ///  The heartbeat publishes the stage the act last set,
    ///  and it holds that word as a number,
    ///  so exactly one place turns the number back into the vocabulary.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const stage = Stage.ofOrder(2); // 'signals'
    /// ```
    pub fn of_order(order: u8) -> Stage {
        return match order {
            1 => Stage::Structural,
            2 => Stage::Signals,
            3 => Stage::Unmount,
            4 => Stage::Done,
            _ => Stage::Preflight,
        };
    }

    /// The stage an earlier stage may advance to.
    /// Why:
    ///  The act's order is fixed and its words say which stage it is in,
    ///  so a read that went backwards names a handler whose file was rewritten.
    pub fn order(self) -> u8 {
        return match self {
            Stage::Preflight => 0,
            Stage::Structural => 1,
            Stage::Signals => 2,
            Stage::Unmount => 3,
            Stage::Done => 4,
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "forced_removal_words_tests.rs"]
mod tests;
