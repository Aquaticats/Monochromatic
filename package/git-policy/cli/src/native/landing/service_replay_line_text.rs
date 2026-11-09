//! What:
//!  Small text steps of the replay of captured changes:
//! whether a line of Git's output is empty.
//! Why:
//!  `service_replay.rs` reads Git's output by lines,
//! and these steps depend on nothing in it.
//!  A path never passes through text here:
//! the replay keeps a repository path as the bytes Git listed it as,
//! so a name that is not text reaches the Git that writes the synthetic base unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! isNotEmptyLine(line)
//! ```

/// What:
///  Whether a line of Git's output is not empty.
/// Why:
///  The output ends in a line break,
/// so the last line of a split is empty and is no line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNotEmptyLine = (line: string) => line !== '';
/// ```
pub(crate) fn is_not_empty_line(line: &&str) -> bool {
    return !line.is_empty();
}
