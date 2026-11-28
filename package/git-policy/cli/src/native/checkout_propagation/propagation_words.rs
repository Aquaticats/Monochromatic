//! What:
//!  The propagation record's own names:
//!  where the file lives,
//!  the version and the kind it carries,
//!  its keys in the order the file holds them,
//!  the bound of every field the writer fills,
//!  the largest record a reader accepts,
//!  the four outcome words and the words the `step` field carries.
//! Why:
//!  One record answers for one checkout,
//!  and a reader compares it with the live instance,
//!  so the file's places,
//!  its words and its bounds are one subject and sit in one file
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "The propagation record",
//!  which states every field and every bound).
//!  The two mount numbers,
//!  the set before and after and the mounts found beneath are what a person
//!  reads,
//!  so a field bound typed in beside another field's writer can fall below it
//!  and make a record the reader refuses,
//!  which is why the file bound is derived from the field bounds rather than
//!  invented (rule `LQM`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const recordPath = '/run/git-worktree-clone-mount-propagation/result';
//! const maxBytes = KEYS.reduce((sum, key) => sum + key.length + 1, 0) + FIELD_BOUNDS.reduce((a, b) => a + b, 0);
//! ```

use crate::checkout_propagation::mount_propagation_set::LONGEST_WORD;

/// Where the record's directory is,
/// which `/run` empties at every boot.
pub const PROPAGATION_DIRECTORY: &str = "/run/git-worktree-clone-mount-propagation";

/// The record's file name inside that directory.
pub const PROPAGATION_NAME: &str = "result";

/// The directory's mode,
/// which every account may read and only root may write.
pub const PROPAGATION_DIRECTORY_MODE: u32 = 0o755;

/// The record's mode,
/// which every reader this route names may read.
pub const PROPAGATION_MODE: u32 = 0o644;

/// The record's format version.
pub const PROPAGATION_VERSION: &str = "v1";

/// The record's kind,
/// which tells it from the readiness record and a receipt.
pub const PROPAGATION_KIND: &str = "checkout-propagation";

/// The key each of the shared mounts found beneath the checkout is written
/// under,
/// which follows the `beneath` count and repeats once per path.
pub const BENEATH_PATH_KEY: &str = "beneath-path";

/// How many shared mounts beneath the checkout the record names,
/// which is the writer's bound the project accepted (choice V7):
/// this host has one nested dataset,
/// so four leave room for three more and the count carries whatever lies
/// beyond.
pub const SHARED_MOUNTS_NAMED: usize = 4;

/// The word `beneath` carries when the run refused before it read the table,
/// which a reader must tell from a table that was read and held none.
pub const BENEATH_UNREAD: &str = "unread";

/// The word a field carries when the run refused before it reached that field.
pub const NONE_WORD: &str = "none";

/// The bound the kernel's own boot identifier uses,
/// a canonical UUID's 36 characters.
pub const MAXIMUM_BOOT_BYTES: usize = 36;

/// The bound one mount number, one namespace number, one master number or one
/// count may use,
/// which is twenty decimal digits of a 64-bit number.
pub const MAXIMUM_NUMBER_BYTES: usize = 20;

/// The bound one checkout path may use,
/// which is the kernel's `PATH_MAX` without its terminator,
/// and the readiness record's own bound.
pub const MAXIMUM_CHECKOUT_PATH_BYTES: usize = 4_095;

/// The bound one ZFS dataset name may use,
/// as the readiness record's own name bound does.
pub const MAXIMUM_DATASET_BYTES: usize = 255;

/// The bound one `beneath-path` line may use:
/// a path the kernel escapes at its worst four bytes each,
/// which is what keeps a path from breaking the record's line (rule `SYB`).
pub const MAXIMUM_BENEATH_PATH_BYTES: usize = MAXIMUM_CHECKOUT_PATH_BYTES * 4;

/// The bound the longest set word uses.
pub const MAXIMUM_SET_WORD_BYTES: usize = LONGEST_WORD.len();

/// The bound one outcome word may use,
/// which is `already-private` and its terminator.
pub const MAXIMUM_OUTCOME_BYTES: usize = 15;

/// The bound one step word may use:
/// `before-table` and `verify-after` are the longest and take twelve,
/// and the design's own bound leaves room for one this build adds.
pub const MAXIMUM_STEP_BYTES: usize = 32;

/// The bound one error number may use:
/// three decimal digits,
/// and the word a refusal leaves where no kernel call answered,
/// which is longer than the digits.
pub const MAXIMUM_ERRNO_BYTES: usize = match NONE_WORD.len() {
    length if length > 3 => length,
    _ => 3,
};

/// The bound one time may use:
/// the date,
/// the second,
/// nine fraction digits and the offset.
pub const MAXIMUM_TIME_BYTES: usize = 35;

/// What:
///  The largest record a reader accepts,
///  from the bound of every field the writer fills and the line each field is
///  written on.
/// Why:
///  A file bound typed in beside field bounds can fall below them,
///  and then a record the writer can build is one the reader refuses,
///  so a refusal nobody can read is what stands after the step ran.
///  Derived this way every record the writer builds fits,
///  and `publish` proves its own reader reads the text back before a byte is
///  written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAXIMUM = KEYS.join('\n').length + SUM_OF_FIELD_BOUNDS + NAMED_PATHS * (BENEATH_PATH_KEY + 1 + PATH_BOUND) + 1;
/// ```
pub const MAXIMUM_PROPAGATION_BYTES: usize = PROPAGATION_VERSION.len()
    + 1
    + "kind".len()
    + 1
    + PROPAGATION_KIND.len()
    + 1
    + "boot".len()
    + 1
    + MAXIMUM_BOOT_BYTES
    + 1
    + "mount-id".len()
    + 1
    + MAXIMUM_NUMBER_BYTES
    + 1
    + "mount-id-old".len()
    + 1
    + MAXIMUM_NUMBER_BYTES
    + 1
    + "mount-ns-id".len()
    + 1
    + MAXIMUM_NUMBER_BYTES
    + 1
    + "checkout".len()
    + 1
    + MAXIMUM_CHECKOUT_PATH_BYTES
    + 1
    + "source".len()
    + 1
    + MAXIMUM_DATASET_BYTES
    + 1
    + "before".len()
    + 1
    + MAXIMUM_SET_WORD_BYTES
    + 1
    + "after".len()
    + 1
    + MAXIMUM_SET_WORD_BYTES
    + 1
    + "master".len()
    + 1
    + MAXIMUM_NUMBER_BYTES
    + 1
    + "beneath".len()
    + 1
    + MAXIMUM_NUMBER_BYTES
    + 1
    + "outcome".len()
    + 1
    + MAXIMUM_OUTCOME_BYTES
    + 1
    + "step".len()
    + 1
    + MAXIMUM_STEP_BYTES
    + 1
    + "errno".len()
    + 1
    + MAXIMUM_ERRNO_BYTES
    + 1
    + "time".len()
    + 1
    + MAXIMUM_TIME_BYTES
    + 1
    + SHARED_MOUNTS_NAMED * (BENEATH_PATH_KEY.len() + 1 + MAXIMUM_BENEATH_PATH_BYTES + 1);

/// What:
///  What the step did with the checkout's mount,
///  in the four words the project accepted (choice V2).
/// Why:
///  A reader acts on the outcome:
///  `applied` means the set changed,
///  `already-private` and `left` mean the mount was already as the route
///  wants it or was left alone,
///  and `refused` means nothing was changed,
///  so the four are one type rather than one word among many.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PropagationOutcome = 'applied' | 'already-private' | 'left' | 'refused';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PropagationOutcome {
    /// The step set the mount private and proved it.
    Applied,
    /// The mount received and passed on nothing, so nothing was set.
    AlreadyPrivate,
    /// The mount was in a state the step reports and does not change.
    Left,
    /// The step refused, and nothing was changed.
    Refused,
}

impl PropagationOutcome {
    /// The word the record carries.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const word = (outcome: PropagationOutcome): string => outcome;
    /// ```
    pub fn word(self) -> &'static str {
        return match self {
            PropagationOutcome::Applied => "applied",
            PropagationOutcome::AlreadyPrivate => "already-private",
            PropagationOutcome::Left => "left",
            PropagationOutcome::Refused => "refused",
        };
    }

    /// Read one outcome word.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const fromWord = (word: string): PropagationOutcome | undefined => ...;
    /// ```
    pub fn from_word(word: &str) -> Option<Self> {
        return match word {
            "applied" => Some(PropagationOutcome::Applied),
            "already-private" => Some(PropagationOutcome::AlreadyPrivate),
            "left" => Some(PropagationOutcome::Left),
            "refused" => Some(PropagationOutcome::Refused),
            _ => {
                // A word this build does not know is another build's record, so a reader must refuse it.
                return None;
            }
        };
    }
}

/// What:
///  Which act of the step a refusal came from,
///  as the record's `step` field carries it.
/// Why:
///  A person who reads `refused` needs the act that refused rather than a
///  sentence to search the journal for,
///  and the design names one word per act
///  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
///  "The propagation record").
///  `record` is this build's own word for a step whose answer could not be
///  filed:
///  the design lists no such refusal,
///  and a run that cannot write its answer must still say which act refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PropagationStep = 'none' | 'resolve' | 'open' | 'statmount' | 'point' | 'type' | 'source' | 'namespace' | 'before-table' | 'set' | 'verify-after' | 'record';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PropagationStep {
    /// No act refused.
    NoStep,
    /// The checkout and its dataset were not resolved.
    Resolve,
    /// The checkout path could not be opened.
    Open,
    /// The descriptor's mount could not be read.
    Statmount,
    /// The mount's point is not the checkout path.
    Point,
    /// The mount's filesystem type is not `zfs`.
    Type,
    /// The mount's source is not the resolved checkout dataset.
    Source,
    /// The mount has no namespace, so it is a detached instance.
    Namespace,
    /// The mount table the record's row comes from could not be read,
    /// or holds no row describing this mount.
    BeforeTable,
    /// The kernel refused the set.
    Set,
    /// The read after the set did not prove it.
    VerifyAfter,
    /// The step's own answer could not be filed.
    Record,
}

impl PropagationStep {
    /// The word the record carries.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const word = (step: PropagationStep): string => step === 'NoStep' ? 'none' : step.toLowerCase();
    /// ```
    pub fn word(self) -> &'static str {
        return match self {
            PropagationStep::NoStep => NONE_WORD,
            PropagationStep::Resolve => "resolve",
            PropagationStep::Open => "open",
            PropagationStep::Statmount => "statmount",
            PropagationStep::Point => "point",
            PropagationStep::Type => "type",
            PropagationStep::Source => "source",
            PropagationStep::Namespace => "namespace",
            PropagationStep::BeforeTable => "before-table",
            PropagationStep::Set => "set",
            PropagationStep::VerifyAfter => "verify-after",
            PropagationStep::Record => "record",
        };
    }

    /// Read one step word.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const fromWord = (word: string): PropagationStep | undefined => ...;
    /// ```
    pub fn from_word(word: &str) -> Option<Self> {
        return match word {
            NONE_WORD => Some(PropagationStep::NoStep),
            "resolve" => Some(PropagationStep::Resolve),
            "open" => Some(PropagationStep::Open),
            "statmount" => Some(PropagationStep::Statmount),
            "point" => Some(PropagationStep::Point),
            "type" => Some(PropagationStep::Type),
            "source" => Some(PropagationStep::Source),
            "namespace" => Some(PropagationStep::Namespace),
            "before-table" => Some(PropagationStep::BeforeTable),
            "set" => Some(PropagationStep::Set),
            "verify-after" => Some(PropagationStep::VerifyAfter),
            "record" => Some(PropagationStep::Record),
            _ => {
                // A word this build does not know is another build's record, so a reader must refuse it.
                return None;
            }
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "propagation_words_tests.rs"]
mod tests;
