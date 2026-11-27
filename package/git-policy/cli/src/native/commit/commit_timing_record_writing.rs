//! What:
//!  The one append that writes a timing record to its file,
//!  best effort.
//! Why:
//!  The command's result never depends on the record:
//!  a file that cannot be written is only a debug line.
//!  The directory is never created and nothing is flushed to disk,
//!  so the record costs one `write`;
//!  the ledgers of this command's steps and tools are read at that moment,
//!  because their figures belong to the command that just ended and the next command starts them empty.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! recordTiming(timing, exitCode)
//! ```

use crate::commit_build_part_durations::drain_build_parts;
use crate::commit_index_builds::drain_index_builds;
use crate::commit_prepare_part_durations::drain_prepare_parts;
/// The record this file appends.
use crate::commit_timing::{
    CommitTiming,
    TAG,
    TIMING_FILE,
};
use crate::diagnostic_log::debug;
/// The bound the log generations are kept to,
/// and the append that never holds a lock.
use crate::log_generations_kept_to_a_bound::UNPINNED_LOG;
use crate::log_lines_appended_without_a_lock::append_line;
use crate::prepare_work_spans::drain_spans;
use crate::quiet_ledger::drain;
use crate::scratch_commit_step::drain_scratch_steps;
use crate::time_spent_outside_the_phases::outside_the_phases;
use crate::transaction_step_durations::drain_steps;
/// `PathBuf` is an owned filesystem path.
use std::path::PathBuf;

impl CommitTiming {
    /// What:
    ///  Appends the record to the repository's timing file,
    /// best effort.
    /// Why:
    ///   The command's result never depends on the record:
    /// a file that cannot be written is only a debug line.
    /// The directory
    ///  is never created,
    /// and nothing is flushed to disk,
    /// so the record costs one `write`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// recordTiming(timing, exitCode)
    /// ```
    pub fn finish(
        &self,
        exit_code: i32,
    ) {
        let Some(place) = self.place.as_ref() else {
            return;
        };
        let target: PathBuf = place.common_dir.join(TIMING_FILE);
        // The ledger holds what this command's Git steps and tools showed; the record takes it and the next command starts empty.
        let mut with_quiet: CommitTiming = self.clone();
        // A build no seam closed is the phase the command ended in, so its time is stated here rather than left to the remainder.
        with_quiet.close_an_open_build();
        // The stretch after the last phase is closed here, and the total is read after it, so it is a named phase too.
        with_quiet.close_the_closing();
        with_quiet.quiet = drain();
        (with_quiet.steps, with_quiet.steps_dropped) = drain_steps(self.began);
        with_quiet.index_builds = drain_index_builds();
        (with_quiet.prepare_spans, with_quiet.prepare_spans_dropped) = drain_spans();
        with_quiet.outside = outside_the_phases();
        (with_quiet.prepare_parts, with_quiet.prepare_parts_dropped) = drain_prepare_parts();
        (with_quiet.build_parts, with_quiet.build_parts_dropped) = drain_build_parts();
        with_quiet.scratch_steps = drain_scratch_steps();
        let line: String = with_quiet.line(self.began.elapsed(), exit_code);
        if let Err(error) = append_line(target.as_path(), line.as_bytes(), &UNPINNED_LOG) {
            debug(
                TAG,
                &format!(
                    "the timing record could not be written to {}: {error}",
                    target.display()
                ),
            );
        }
    }
}
