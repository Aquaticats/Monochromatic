//! What:
//!  One timing record as the single line of JSON it is written as.
//! Why:
//!  One `write` call per line keeps lines of concurrent commits from interleaving,
//!  and a branch name with a quote or a newline is escaped,
//!  so the line stays one line whatever the worktree is called.
//!  Every figure the record holds is a field of that line,
//!  and the phase sums it states are read from the record itself,
//!  so a reader of the file never has to guess which stretch a number covers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! JSON.stringify(record) + '\n'
//! ```

/// The stretches of the build phase,
/// the preparation's own work,
/// the index builds,
/// the scratch step's facts,
/// the quiet ledger,
/// the steps and the stop reason,
/// each written as its own JSON field.
use crate::commit_build_part_durations::build_parts_json;
use crate::commit_index_builds::index_builds_json;
use crate::commit_prepare_part_durations::prepare_parts_json;
use crate::commit_stop_reason::stop_json;
/// The record this line is written from.
use crate::commit_timing::{
    CommitTiming,
    TIMING_SCHEMA_VERSION,
};
use crate::commit_timing_json_words::{
    landing_json,
    null_word,
};
use crate::commit_timing_place::Place;
use crate::iso_time::now_iso;
/// The quote and the string-array word the line's names are written with.
use crate::json_record::{
    quote,
    string_array,
};
use crate::prepare_work_spans::spans_json;
use crate::quiet_ledger::{
    longest_healthy,
    object_json,
};
use crate::scratch_commit_step::scratch_steps_json;
/// The release this record names.
use crate::service_protocol::RELEASE;
use crate::transaction_step_durations::steps_json;
/// `Duration` is an elapsed stretch of time from the monotonic clock.
use std::time::Duration;

impl CommitTiming {
    /// What:
    ///  The record as one line of JSON ending in a newline.
    /// Why:
    ///   One `write` call per line keeps lines of concurrent commits from interleaving;
    /// a branch name with a quote or a
    ///  newline is escaped by `quote`,
    /// so the line stays one line.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// JSON.stringify(record) + '\n'
    /// ```
    pub fn line(
        &self,
        total: Duration,
        exit_code: i32,
    ) -> String {
        let place: Option<&Place> = self.place.as_ref();
        let branch: String = self.reference.as_deref().map_or_else(null_word, quote);
        let answered_by: String = self
            .submit
            .reach
            .answered_by
            .as_deref()
            .map_or_else(null_word, quote);
        let request: String = self.request.as_deref().map_or_else(null_word, quote);
        let level: &str = if self.outcome == "stopped" {
            "warn"
        } else {
            "info"
        };
        let positions: Vec<String> = self.submit.positions.iter().map(usize::to_string).collect();
        return format!(
            "{{\"schemaVersion\":{TIMING_SCHEMA_VERSION},\"time\":{},\"level\":\"{level}\",\"startMillis\":{},\"pid\":{},\"release\":{},\"instance\":{},\"gitDir\":{},\
             \"requestId\":{request},\"branch\":{branch},\"outcome\":{},\"exitCode\":{exit_code},\"stop\":{},\"totalMs\":{},\
             \"routeMs\":{},\"prepareMs\":{},\"attempts\":{},\"prepareSpans\":{},\"prepareSpansDropped\":{},\"indexBuilds\":{},\"scratchSteps\":{},\
             \"buildMs\":{},\"buildParts\":{},\"buildPartsDropped\":{},\"provisionMs\":{},\
             \"submitMs\":{},\"sends\":{},\"reachMs\":{},\"connectTries\":{},\"refusedConnects\":{},\"startWaitMs\":{},\
             \"updateRounds\":{},\"releasesSeen\":{},\"answeredByRelease\":{answered_by},\
             \"answerWaitMs\":{},\"positions\":[{}],\"waitingNotices\":{},\"lockHolderKind\":{},\"reconnects\":{},\
             \"afterLandingMs\":{},\"gateMs\":{},\"handoffMs\":{},\"closingMs\":{},\"otherMs\":{},\
             \"scratchRemovalMs\":{},\"removalsThatDidNotFinish\":{},\"stderrMs\":{},\
             \"prepareParts\":{},\"preparePartsDropped\":{},\
             \"longestQuietMs\":{},\"quietMs\":{},\"steps\":{},\"stepsDropped\":{},\"landing\":{}}}\n",
            quote(now_iso().as_str()),
            self.start_millis,
            std::process::id(),
            quote(RELEASE),
            place.map_or_else(null_word, Place::instance_json),
            place.map_or_else(null_word, Place::git_dir_json),
            quote(self.outcome),
            stop_json(self.stop.as_ref()),
            total.as_millis(),
            self.route.as_millis(),
            self.prepare.as_millis(),
            self.attempts,
            spans_json(self.prepare_spans.as_slice()),
            self.prepare_spans_dropped,
            index_builds_json(self.index_builds.as_slice()),
            scratch_steps_json(self.scratch_steps.as_slice()),
            self.build.as_millis(),
            build_parts_json(self.build_parts.as_slice()),
            self.build_parts_dropped,
            self.provision.as_millis(),
            self.submit.total.as_millis(),
            self.submit.sends,
            self.submit.reach.elapsed.as_millis(),
            self.submit.reach.tries,
            self.submit.reach.refused,
            self.submit.reach.start_waited.as_millis(),
            self.submit.reach.retires_sent,
            string_array(self.submit.reach.releases_seen.as_slice()),
            self.submit.answer_wait.as_millis(),
            positions.join(","),
            self.submit.waiting_notices,
            self.submit.holder_kind.map_or_else(null_word, quote),
            self.submit.reconnects,
            self.after_landing.as_millis(),
            self.gate.as_millis(),
            self.handoff.as_millis(),
            self.closing.as_millis(),
            self.other(total).as_millis(),
            self.outside.scratch_removal.as_millis(),
            self.outside.removals_that_did_not_finish,
            self.outside.standard_error.as_millis(),
            prepare_parts_json(self.prepare_parts.as_slice()),
            self.prepare_parts_dropped,
            longest_healthy(self.quiet.as_slice()),
            object_json(self.quiet.as_slice()),
            steps_json(self.steps.as_slice()),
            self.steps_dropped,
            landing_json(self.submit.landing.as_ref()),
        );
    }
}
