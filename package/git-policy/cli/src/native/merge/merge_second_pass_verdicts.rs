//! What:
//!  What the word-level second pass accepts and what it leaves:
//!  the size and byte limits Git and this pass merge a file under,
//!  the modes a plain file may have,
//!  the reason a conflicted path stays as Git left it,
//!  and what became of each path.
//! Why:
//!  Every caller of the pass decides with these same values,
//!  and a caller that reads a limit must read the one the pass applies,
//!  so they live in one file that names each measurement and each rule it comes from.
//!  A file is merged only when all three stages exist with one plain mode and are small text;
//!  anything else is left to the person exactly as Git left it,
//!  with the reason.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Left = 'missing-stage' | 'not-a-plain-file' | 'binary' | 'too-large' | { driver: string } | { overlap: number } | 'unknown-attributes';
//! type Verdict = { merged: Merged } | { left: { path, why: Left } };
//! const why = unfit(bytes); // 'too-large' | 'binary' | undefined
//! ```

use crate::merge_second_pass_conflicts::{
    Conflict,
    Stage,
};
use crate::word_merge::MergedPart;
/// The number of leading bytes in which Git looks for a NUL byte to call a file binary:
/// `FIRST_FEW_BYTES` in Git 2.56.0's
///  `xdiff-interface.c` line 197,
/// used by `buffer_is_binary` and by the merge driver choice in `merge-ll.c` lines 120 to 122.
const BINARY_PROBE_BYTES: usize = 8000;

/// The size above which Git's own merge treats a file as binary and does not merge it by text:
/// `MAX_XDIFF_SIZE`,
/// 1023 MiB,
/// in
///  `xdiff-interface.h` line 14,
/// tested in `merge-ll.c` lines 117 to 119.
const GIT_MAX_XDIFF_SIZE: usize = 1024 * 1024 * 1023;

/// The size above which this pass leaves a file to the person,
/// although Git would still merge it by text.
/// Git's value cannot
///  bound the cost of this pass:
/// measured on three texts that share no line,
/// one 1 MiB of lines took 0.08 s and 45 MB,
/// 8 MiB took
///  0.73 s and 352 MB,
/// 32 MiB took 4.12 s and 1320 MB (release build,
/// Myers diff,
/// one core),
/// so a file at Git's limit would take
///  minutes and tens of gigabytes.
/// 8 MiB keeps the worst case under a second.
const PASS_COST_LIMIT_BYTES: usize = 8 * 1024 * 1024;

/// The largest blob the second pass re-merges:
/// the smaller of Git's own limit and the one measured above.
pub(super) const LARGEST_BLOB_BYTES: usize = if PASS_COST_LIMIT_BYTES < GIT_MAX_XDIFF_SIZE {
    PASS_COST_LIMIT_BYTES
} else {
    GIT_MAX_XDIFF_SIZE
};

/// The largest stage the pass reads,
/// in the bytes a caller's answer ceiling counts.
/// Why:
///   The pass asks `git cat-file --batch` for the three stages of every candidate and refuses a file past this size,
/// so an answer larger than one
///  of these per name asked for is one the pass would refuse after reading it.
pub const LARGEST_STAGE_BYTES: u64 = LARGEST_BLOB_BYTES as u64;

/// The modes of a plain file,
/// with and without the executable bit.
const PLAIN_FILE_MODES: &[u32] = &[0o100_644, 0o100_755];
/// Why a conflicted path is left to the person.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Left {
    /// A stage is missing:
    /// both sides added the path,
    /// or one side deleted it.
    MissingStage,
    /// The stages differ in mode,
    /// or the file is a link,
    /// a submodule or not a plain file.
    NotAPlainFile,
    /// A stage holds a NUL byte near its start.
    Binary,
    /// A stage is larger than the pass takes.
    TooLarge,
    /// The `merge` attribute names a driver,
    /// `binary` or is unset,
    /// so the merge belongs to that setting.
    Driver(String),
    /// At least one line region holds two different changes of the same words,
    /// or a word merge of it would write text no side wrote in that shape.
    Overlap(usize),
    /// A `.gitattributes` of the merge is itself conflicted,
    /// so no `merge` setting can be read for the path.
    UnknownAttributes,
}

/// Where the pass reads the `merge` attribute of the paths it may merge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributeSource<'a> {
    /// Whatever the invocation itself names:
    /// the worktree of a merge a person runs,
    /// or the tree that `--attr-source`,
    /// `GIT_ATTR_SOURCE` or `attr.tree` names.
    Invocation,
    /// The tree the merge produced,
    /// which carries a `merge` rule the merge itself introduces.
    Tree(&'a str),
    /// A `.gitattributes` of the merge is itself conflicted,
    /// so no rule of it can be trusted and nothing may be merged.
    Unknown,
}

/// A conflicted file that the word pass merged completely.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Merged {
    /// The path as bytes.
    pub path: Vec<u8>,
    /// The mode all three stages share.
    pub mode: u32,
    /// The merged bytes.
    pub bytes: Vec<u8>,
    /// The line regions that needed the word pass.
    pub parts: Vec<MergedPart>,
    /// The three stages the file was merged from,
    /// kept to put the conflict back if a later step fails.
    pub stages: Conflict,
}

/// What became of one conflicted path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// Merged completely.
    Merged(Box<Merged>),
    /// Left as Git left it.
    Left {
        /// The path as bytes.
        path: Vec<u8>,
        /// Why.
        why: Left,
    },
}

/// What:
///  Whether the bytes are text the pass may merge:
/// no NUL byte near the start,
/// and not larger than the limit.
/// Why:
///   Git's own binary test looks at the first bytes only;
/// a file it would call binary is never a candidate for a
///  line-and-word merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verdict = (bytes) => bytes.length > LARGEST ? 'too-large' : bytes.subarray(0, PROBE).includes(0) ? 'binary' : 'text';
/// ```
pub(crate) fn unfit(bytes: &[u8]) -> Option<Left> {
    if bytes.len() > LARGEST_BLOB_BYTES {
        return Some(Left::TooLarge);
    }
    if bytes[..bytes.len().min(BINARY_PROBE_BYTES)].contains(&0) {
        return Some(Left::Binary);
    }
    return None;
}

/// The mode all three stages share,
/// when it is a plain file's.
pub(super) fn shared_plain_mode(stages: [&Stage; 3]) -> Option<u32> {
    let mode: u32 = stages[0].mode;
    for stage in stages {
        if stage.mode != mode {
            return None;
        }
    }
    if PLAIN_FILE_MODES.contains(&mode) {
        return Some(mode);
    }
    return None;
}

/// What:
///  Whether a conflicted path is a `.gitattributes` file.
/// Why:
///   A `.gitattributes` names the rules of its own folder and of everything below it,
/// so one a merge left conflicted can change any rule that bears on a candidate,
/// and a copy of it that holds Git's markers is no side's rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAttributesPath = (path: string) => path === '.gitattributes' || path.endsWith('/.gitattributes');
/// ```
pub fn is_attributes_path(path: &[u8]) -> bool {
    return path == b".gitattributes" || path.ends_with(b"/.gitattributes");
}
