//! What:
//!  The records the step files:
//!  the answer a run reached,
//!  the refusal a call or a check answered,
//!  the sentence a refusal says,
//!  and the one line a mode says about either.
//! Why:
//!  Every outcome shares the fields it fills and every refusal names the
//!  instance the run reached,
//!  so the records are built in one place and the acts ask for one by name
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "The propagation record",
//!  and "What a failure reports").
//!  A refusal files no receipt and still writes the record,
//!  so the sentence a person reads and the record a reader compares are built
//!  from the same values.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = anAnswer(places, checkout, before, beneath, 'applied');
//! const refusal = filed(places, record, why);
//! ```

use crate::checkout_propagation::checkout_mount_checks::MountRefusal;
use crate::checkout_propagation::mount_descriptor_read::DescriptorMount;
use crate::checkout_propagation::propagation_calls::CallFailure;
use crate::checkout_propagation::propagation_record::PropagationRecord;
use crate::checkout_propagation::propagation_step_names::{
    PropagationDone,
    PropagationPlaces,
    PropagationRefusal,
};
use crate::checkout_propagation::propagation_words::{
    PropagationOutcome,
    PropagationStep,
};
use crate::checkout_propagation::shared_mounts_beneath::SharedMountsBeneath;
use std::path::Path;

/// What:
///  The record one answer or refusal carries,
///  with the set before and after it as the run left them.
/// Why:
///  Every answer but a refusal shares these fields,
///  so they are built once and a caller that changes only one field says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = anAnswer(places, checkout, before, beneath, 'applied');
/// ```
pub(crate) fn an_answer(
    places: &PropagationPlaces,
    checkout: &Path,
    before: &DescriptorMount,
    beneath: SharedMountsBeneath,
    outcome: PropagationOutcome,
) -> PropagationRecord {
    return PropagationRecord {
        boot: places.boot.clone(),
        mount_id: Some(before.mount_id),
        mount_id_old: Some(before.mount_id_old),
        mount_namespace_id: before.namespace_id,
        checkout: Some(checkout.display().to_string()),
        source: before.source.clone(),
        before: Some(before.propagation),
        after: Some(before.propagation),
        master: before.master,
        beneath,
        outcome,
        step: PropagationStep::NoStep,
        errno: None,
        time: crate::iso_time::now_iso(),
    };
}

/// What:
///  The record one refusal files before any mount was read,
///  with `none` for every field the run never reached.
/// Why:
///  A reader must tell "the step never looked" from "nothing was found",
///  so a refusal writes `unread` for the mounts beneath and `none` for the
///  fields it did not read,
///  and `doctor` prints exactly that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = aRefusal(places, 'open', 2, { kind: 'unread' });
/// ```
pub(crate) fn a_refusal(
    places: &PropagationPlaces,
    step: PropagationStep,
    errno: Option<i32>,
    beneath: SharedMountsBeneath,
) -> PropagationRecord {
    return PropagationRecord {
        boot: places.boot.clone(),
        mount_id: None,
        mount_id_old: None,
        mount_namespace_id: None,
        checkout: None,
        source: None,
        before: None,
        after: None,
        master: None,
        beneath,
        outcome: PropagationOutcome::Refused,
        step,
        errno,
        time: crate::iso_time::now_iso(),
    };
}

/// What:
///  The record one refusal files once the mount was read,
///  naming the instance and the set it carried.
/// Why:
///  The facts a run reached are what a person needs to see why it refused,
///  and the instance's two numbers are what `doctor` compares with the live
///  mount,
///  so the fields already read are kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = aRefusalWithFacts(places, facts, checkout, 'type', undefined, beneath);
/// ```
pub(crate) fn a_refusal_with_facts(
    places: &PropagationPlaces,
    facts: &DescriptorMount,
    checkout: &Path,
    step: &str,
    errno: Option<i32>,
    beneath: SharedMountsBeneath,
) -> PropagationRecord {
    let word: PropagationStep =
        PropagationStep::from_word(step).unwrap_or(PropagationStep::BeforeTable);
    return PropagationRecord {
        boot: places.boot.clone(),
        mount_id: Some(facts.mount_id),
        mount_id_old: Some(facts.mount_id_old),
        mount_namespace_id: facts.namespace_id,
        checkout: Some(checkout.display().to_string()),
        source: facts.source.clone(),
        before: Some(facts.propagation),
        after: None,
        master: facts.master,
        beneath,
        outcome: PropagationOutcome::Refused,
        step: word,
        errno,
        time: crate::iso_time::now_iso(),
    };
}

/// What:
///  One call's refusal,
///  filed with no facts at all.
/// Why:
///  An act that refused before the mount was read has nothing to name,
///  and the kernel's number and the refusal's own sentence are what a person
///  acts on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = refusedCall(places, 'open', failure);
/// ```
pub(crate) fn refused_call(
    places: &PropagationPlaces,
    step: PropagationStep,
    failure: &CallFailure,
) -> PropagationRefusal {
    let record: PropagationRecord =
        a_refusal(places, step, failure.errno(), SharedMountsBeneath::Unread);
    return filed(places, record, failure.message());
}

/// What:
///  One call's refusal,
///  filed with the facts the run had read.
/// Why:
///  A set or a read after it refuses on an instance the run verified,
///  so the record names it and the count of mounts beneath it stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = refusedWithFacts(places, checkout, from, beneath, 'set', failure);
/// ```
pub(crate) fn refused_with_facts(
    places: &PropagationPlaces,
    checkout: &Path,
    from: &DescriptorMount,
    beneath: SharedMountsBeneath,
    step: PropagationStep,
    failure: &CallFailure,
) -> PropagationRefusal {
    let record: PropagationRecord = a_refusal_with_facts(
        places,
        from,
        checkout,
        step.word(),
        failure.errno(),
        beneath,
    );
    return filed(places, record, failure.message());
}

/// What:
///  One check's refusal,
///  filed with the facts the run had read.
/// Why:
///  A check refuses after the mount was read,
///  so its record names the instance and the set and says which check
///  disagreed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = refusedCheck(places, checkout, facts, { word: 'type', why: '...' });
/// ```
pub(crate) fn refused_check(
    places: &PropagationPlaces,
    checkout: &Path,
    facts: &DescriptorMount,
    check: &MountRefusal,
) -> PropagationRefusal {
    let record: PropagationRecord = a_refusal_with_facts(
        places,
        facts,
        checkout,
        check.word,
        None,
        SharedMountsBeneath::Unread,
    );
    return filed(places, record, check.why.clone());
}

/// What:
///  File one refusal's record and answer the refusal the mode reports.
/// Why:
///  A refusal must leave its sentence where a person reads it,
///  and a run that cannot file its answer says so with this route's own words
///  rather than reporting a refusal nobody can look up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return filed(places, record, why); // throws PropagationRefusal
/// ```
pub(crate) fn filed(
    places: &PropagationPlaces,
    record: PropagationRecord,
    why: String,
) -> PropagationRefusal {
    let step: PropagationStep = record.step;
    let errno: Option<i32> = record.errno;
    let message: String = format!(
        "checkout propagation: {step} refused: {why}. {place}, and this run changed nothing under the checkout",
        step = step.word(),
        why = why,
        place = where_the_run_stands(&record)
    );
    return match places.record.publish(&record) {
        Ok(()) => PropagationRefusal {
            step,
            errno,
            message,
        },
        Err(failure) => PropagationRefusal {
            step: PropagationStep::Record,
            errno: None,
            message: format!(
                "checkout propagation: {message}; and the refusal's record could not be filed either ({})",
                failure.message()
            ),
        },
    };
}

/// What:
///  The instance and the counts one refusal's sentence names.
/// Why:
///  A person reading the line must know which mount the refusal was about and
///  what stood beneath it,
///  and the record holds the same facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = (record) => `mount ${record.mountId} (table ${record.mountIdOld}) of ${record.checkout}, ${record.beneath.count} shared beneath`;
/// ```
fn where_the_run_stands(record: &PropagationRecord) -> String {
    let beneath: String = match record.beneath.word().as_str() {
        "unread" => String::from("the mounts beneath it were never read"),
        count => format!(
            "{count} shared mounts stand beneath it ({})",
            paths_phrase(&record.beneath)
        ),
    };
    return format!(
        "mount {mount} (table {table}) of {checkout}; {beneath}",
        mount = record
            .mount_id
            .map_or(String::from("none"), |number| return number.to_string()),
        table = record
            .mount_id_old
            .map_or(String::from("none"), |number| return number.to_string()),
        checkout = record
            .checkout
            .as_deref()
            .unwrap_or("a checkout the run never resolved"),
        beneath = beneath
    );
}

/// What:
///  File one answer and hand the mode its line.
/// Why:
///  Every answer ends here,
///  so the record is written once and the line is built from the same record
///  the file holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return finished(places, record); // { outcome, line }
/// ```
pub(crate) fn finished(
    places: &PropagationPlaces,
    record: &PropagationRecord,
) -> Result<PropagationDone, PropagationRefusal> {
    places.record.publish(record).map_err(|failure| {
        return PropagationRefusal {
            step: PropagationStep::Record,
            errno: None,
            message: format!(
                "checkout propagation: {outcome} was reached and its record could not be filed ({reason}), so nothing outside this run knows what it did",
                outcome = record.outcome.word(),
                reason = failure.message()
            ),
        };
    })?;
    return Ok(PropagationDone {
        outcome: record.outcome,
        line: summary_line(record),
    });
}

/// What:
///  The one line the mode says about the step,
///  carrying the same facts the record does.
/// Why:
///  An operator reads the journal and a guest arm reads this line,
///  so the outcome,
///  the instance's two numbers,
///  the set before and after,
///  the cost a set detached and the mounts found beneath are said once in
///  words rather than looked up in the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = `checkout propagation: ${outcome} for ${checkout} mount ${id} (table ${old}, namespace ${ns}): ${before} to ${after}; ...`;
/// ```
fn summary_line(record: &PropagationRecord) -> String {
    let set: String = match (record.before, record.after) {
        (Some(before), Some(after)) => format!("{} to {}", before.word(), after.word()),
        (Some(before), None) => String::from(before.word()),
        (None, _) => String::from("no set, because the mount was never read"),
    };
    let master: String = match record.master {
        Some(number) => format!(
            "; the mount was slave to master {number} and no longer receives that master's mounts"
        ),
        None => String::new(),
    };
    let beneath: String = match record.beneath.word().as_str() {
        "unread" => String::from("; the mounts beneath it were never read"),
        count => format!(
            "; {count} shared mounts beneath it, {}",
            paths_phrase(&record.beneath)
        ),
    };
    return format!(
        "checkout propagation: {outcome} for {checkout} mount {mount} (table {table}, namespace {namespace}): {set}{master}{beneath}",
        outcome = record.outcome.word(),
        checkout = record
            .checkout
            .as_deref()
            .unwrap_or("a checkout the run never resolved"),
        mount = record
            .mount_id
            .map_or(String::from("none"), |number| return number.to_string()),
        table = record
            .mount_id_old
            .map_or(String::from("none"), |number| return number.to_string()),
        namespace = record
            .mount_namespace_id
            .map_or(String::from("not reported"), |number| return number
                .to_string()),
        set = set,
        master = master,
        beneath = beneath
    );
}

/// The paths a run named beneath the checkout,
/// or the words that say it named none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathsPhrase = (beneath: SharedMountsBeneath): string => beneath.paths().length === 0 ? 'none named' : beneath.paths().join(', ');
/// ```
fn paths_phrase(beneath: &SharedMountsBeneath) -> String {
    let paths: &[String] = beneath.paths();
    if paths.is_empty() {
        return String::from("none named");
    }
    return paths.join(", ");
}
