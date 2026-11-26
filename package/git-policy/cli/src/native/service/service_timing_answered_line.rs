//! What:
//!  The one line the service writes per answered request:
//!  its identity,
//!  what became of it,
//!  and every figure of the wait,
//!  the landing,
//!  the residual reads,
//!  the syncs,
//!  the quiet gaps and the Git steps.
//! Why:
//!   One line per request is what makes a slow commit explicable after the fact:
//!   the queue,
//! another program's hold on the index,
//!   the landing and a service being replaced each have their own figure,
//!   and the line names the repository it is about,
//!   because one journal covers every worktree of the machine.
//!  Text values are written in Rust's quoted form,
//!  which has no raw newline or unescaped quote,
//!  so the line stays one line whatever a client sent.
//!  The figures it prints are the shape in `service_timing_records.rs`,
//!  and the lock wait's words are `service_timing_wait_line.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! `answered request=${JSON.stringify(id)} outcome=${outcome} queued_ms=${queued} ...`
//! ```
use crate::quiet_ledger::longest_healthy;
use crate::service_durable_syncs::{
    syncs_held,
    syncs_waited_count,
    syncs_word,
};
use crate::service_timing_records::Answered;
use crate::service_timing_wait_line::lock_notes_fields;
use crate::transaction_step_durations::steps_word;
use std::time::Duration;

/// What:
///  One quiet gap as a word:
/// `name=milliseconds` with three decimals.
/// Why:
///  `quiet_steps` joins the words of all the gaps with semicolons.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const quietStepWord = ([step, gap]: [string, number]) => `${step}=${(gap / 1000).toFixed(3)}`;
/// ```
fn quiet_step_word(figure: &(String, Duration)) -> String {
    let (step, gap) = figure;
    return format!("{step}={:.3}", gap.as_micros() as f64 / 1000.0);
}

/// The text of the quiet gaps in the answered line:
/// `name=milliseconds` joined by semicolons.
fn quiet_steps(figures: &[(String, Duration)]) -> String {
    let words: Vec<String> = figures.iter().map(quiet_step_word).collect();
    return words.join(";");
}

/// The word for a named holder,
/// in the quoted form.
fn holder_field(who: &String) -> String {
    return format!(" lock_holder={who:?}");
}

/// What:
///  One answered request as a line of `key=value` words,
/// without the prefix.
/// Why:
///   Text values are written in Rust's quoted form,
/// which has no raw newline or unescaped quote,
/// so the line stays one line
///  whatever a client sent;
/// the numbers are whole milliseconds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `answered request=${JSON.stringify(id)} outcome=${outcome} queued_ms=${queued} ...`
/// ```
pub fn answered_line(answered: &Answered<'_>) -> String {
    let mut holder: String = answered
        .landing
        .lock_holder
        .as_ref()
        .map_or_else(String::new, holder_field);
    holder.push_str(lock_notes_fields(&answered.landing.lock_notes).as_str());
    return format!(
        "answered request={:?} branch={:?} outcome={} batch={} repository={:?} size={} position={} queued_behind={} read_ms={} queued_ms={} settle_ms={} \
         lock_wait_ms={}{holder} lock_hold_ms={} reads_ms={} objects_ms={} journal_ms={} ref_ms={} index_update_ms={} attempts={} landing_ms={} \
         reads_config_ms={} reads_branch_ms={} reads_parents_ms={} reads_changed_ms={} reads_content_ms={} \
         reads_witness_ms={} reads_conclusion_ms={} reads_owed_ms={} reads_follow_ms={} \
         syncs_waited={} syncs_queued={} sync_held_ms={} syncs={:?} \
         answer_ms={} total_ms={} quiet_ms={} quiet_steps={:?} steps={:?} steps_dropped={}",
        answered.request,
        answered.reference,
        answered.outcome,
        answered.batch,
        answered.repository,
        answered.size,
        answered.position,
        answered.queued_behind,
        answered.arrival.read.as_millis(),
        answered.queued.as_millis(),
        answered.settle.as_millis(),
        answered.landing.lock_wait.as_millis(),
        answered.landing.hold.as_millis(),
        answered.landing.reads(answered.land_total).as_millis(),
        answered.landing.objects.as_millis(),
        answered.landing.journal.as_millis(),
        answered.landing.ref_update.as_millis(),
        answered.landing.index_update.as_millis(),
        answered.landing.attempts,
        answered.land_total.as_millis(),
        answered.landing.reads_split.config.as_millis(),
        answered.landing.reads_split.branch.as_millis(),
        answered.landing.reads_split.parents.as_millis(),
        answered.landing.reads_split.changed.as_millis(),
        answered.landing.reads_split.content.as_millis(),
        answered.landing.reads_split.witness.as_millis(),
        answered.landing.reads_split.conclusion.as_millis(),
        answered.landing.reads_split.owed.as_millis(),
        answered.landing.reads_split.follow.as_millis(),
        syncs_waited_count(answered.landing.syncs.as_slice()),
        answered.landing.syncs.len() - syncs_waited_count(answered.landing.syncs.as_slice()),
        syncs_held(answered.landing.syncs.as_slice()).as_millis(),
        syncs_word(answered.landing.syncs.as_slice()),
        answered.answer.as_millis(),
        answered.total.as_millis(),
        longest_healthy(answered.quiet),
        quiet_steps(answered.quiet),
        steps_word(answered.steps),
        answered.steps_dropped,
    );
}
