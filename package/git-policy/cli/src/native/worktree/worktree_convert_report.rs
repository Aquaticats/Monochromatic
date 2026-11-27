//! What:
//!  What the person is told as the conversions finish,
//! in input order,
//! and how many of each kind there were.
//! Why:
//!  The runner calls the report on the calling thread only,
//! one event at a time,
//! so the counts need no lock,
//! and the summary the run ends with is counted from them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class ConversionReport implements OrderedEvents<Result<Verdict>> { refused = 0; converted = 0; failed = 0; }
//! ```

use crate::parallel_runner::{
    Event,
    OrderedEvents,
};
use crate::subvolume_error::MigrateError;
use crate::worktree_convert::Verdict;
use std::path::PathBuf;

/// What:
///  What the person is told as the conversions finish,
/// in input order,
/// and how many of each kind there were.
/// Why:
///   The runner calls the report on the calling thread only,
/// one event at a time,
/// so the counts need no lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ConversionReport implements OrderedEvents<Result<Verdict>> { refused = 0; converted = 0; failed = 0; }
/// ```
pub(crate) struct ConversionReport<'a> {
    /// Where the lines go.
    pub(crate) say: &'a mut dyn FnMut(&str),

    /// The worktrees,
    /// in the order their output is reported.
    pub(crate) list: &'a [PathBuf],

    /// Whether the run only lists what it would do.
    pub(crate) dry_run: bool,

    /// How many worktrees were refused before anything changed,
    /// which is what ends a run of named worktrees with status 1.
    pub(crate) refused: usize,

    /// How many worktrees needed nothing because the path is a subvolume already.
    pub(crate) already_subvolume: usize,

    /// How many unfinished conversions the run undid because it was asked to (`--abandon`).
    pub(crate) undone: usize,

    /// How many worktrees were converted or completed.
    pub(crate) converted: usize,

    /// How many worktrees a dry run would convert or complete.
    pub(crate) would_convert: usize,

    /// How many conversions failed.
    pub(crate) failed: usize,
}

impl OrderedEvents<Result<Verdict, MigrateError>> for ConversionReport<'_> {
    fn on_event(
        &mut self,
        event: Event<'_, Result<Verdict, MigrateError>>,
    ) {
        match event {
            Event::Begin(index) => (self.say)(format!("{}", self.list[index].display()).as_str()),
            Event::Line(_index, text) => (self.say)(text),
            Event::End(_index, Ok(Verdict::Converted)) => {
                self.converted += 1;
                (self.say)("  converted into a subvolume");
            }
            Event::End(_index, Ok(Verdict::Resumed)) => {
                self.converted += 1;
                (self.say)(if self.dry_run {
                    "  an unfinished conversion would be completed"
                } else {
                    "  the unfinished conversion was completed"
                });
            }
            Event::End(_index, Ok(Verdict::WouldConvert)) => {
                self.would_convert += 1;
                (self.say)("  would convert");
            }
            Event::End(_index, Ok(Verdict::AlreadySubvolume)) => {
                self.already_subvolume += 1;
                (self.say)("  already a subvolume, skipped");
            }
            Event::End(_index, Ok(Verdict::Abandoned)) => {
                self.undone += 1;
                (self.say)("  the unfinished conversion was undone");
            }
            Event::End(_index, Ok(Verdict::Skipped(reasons))) => {
                self.refused += 1;
                for reason in reasons {
                    (self.say)(format!("  skipped: {reason}").as_str());
                }
            }
            Event::End(_index, Err(error)) => {
                self.failed += 1;
                (self.say)(format!("  failed: {error}").as_str());
                // A conversion whose step never started has no journal to finish, so only a step that ran may say it has one.
                if !matches!(error, MigrateError::Runtime(_)) {
                    (self.say)(
                        "  The conversion is journaled. Run the same command again to finish it, or add --abandon to undo it before its swap.",
                    );
                }
            }
        }
    }
}
