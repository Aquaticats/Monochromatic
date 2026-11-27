//! What:
//!  What one bounded query of real Git answered:
//!  the status the wait returned and the bytes the two pipes had given,
//!  what ended the query,
//!  and whether a descendant of the query still holds a pipe or its input.
//! Why:
//!  A control reads all four together,
//!  because a caller that reads only the output still has to learn that the output may be short
//!  and that the query may have been ended at its bound rather than by itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type QueryAnswer = { output: Output; ending: QueryEnding; heldOpen: boolean; inputHeld: boolean };
//! ```

/// What ended the query,
/// as the last status the wait returned showed it.
use super::query_ending::QueryEnding;
/// The status the wait returned,
/// and the bytes the two pipes had given.
use std::process::Output;

/// What one bounded query of real Git answered.
#[derive(Debug)]
pub(crate) struct QueryAnswer {
    /// The status the wait returned,
    /// and the bytes the two pipes had given by the end of the grace after the ending.
    ///  A descendant that holds a pipe can add bytes in that window,
    /// and `held_open` says when one did.
    pub output: Output,
    /// What ended the query,
    /// as the last status the wait returned showed it.
    pub ending: QueryEnding,
    /// Whether a pipe of the query was still held when the window for its end passed,
    /// as the kernel answered it,
    /// so a descendant of the query kept that end open and the output may be short of what the query wrote.
    pub held_open: bool,
    /// Whether the input was still being written when the window passed,
    /// so a descendant of the query kept its read end open and the query never read all of it.
    ///  No verdict rests on this figure,
    /// so the writer's own late waking can make it say so and mislead nothing.
    pub input_held: bool,
}
