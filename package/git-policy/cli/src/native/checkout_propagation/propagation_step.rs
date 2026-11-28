//! What:
//!  The private-propagation step:
//!  what every root run does before it decides about an attach,
//!  in the order the accepted design fixes,
//!  and the answer it files and says.
//! Why:
//!  A mount that replaces the checkout's mount comes up shared again,
//!  and a clone mounted under a shared checkout is copied into every other
//!  mount namespace,
//!  so the route checks the checkout and sets it private before it enumerates
//!  or attaches anything
//!  (`doc/decision/cli-git-zfs-checkout-propagation-repair.md`,
//!  "Who may set the propagation when no person is present").
//!  The step acts on a set that carries `MS_SHARED` and on nothing else,
//!  changes the mount only through the descriptor it verified,
//!  and records the instance it answered for,
//!  so `doctor` can compare the record with the live mount
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "The step that re-applies the setting").
//!  A refusal changes nothing,
//!  attaches nothing,
//!  files no receipt,
//!  and still writes the record,
//!  because the sentence a person needs is what the record carries.
//!  Nothing reaches this step in this release:
//!  its call at the head of `boot-reconcile` and of the default sweep is the
//!  wiring of plan step 19, which is not built,
//!  so the guest arm beside these files calls it itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = runTheStep(places, resolution, calls); // ok: { outcome, line } | refusal: { step, errno, message }
//! ```

use crate::checkout_propagation::checkout_mount_checks::{
    MountRefusal,
    check_the_facts,
    check_the_row,
};
use crate::checkout_propagation::mount_descriptor_read::DescriptorMount;
use crate::checkout_propagation::propagation_calls::{
    CallFailure,
    PropagationCalls,
};
use crate::checkout_propagation::propagation_record::PropagationRecord;
use crate::checkout_propagation::propagation_record_file::PropagationFile;
use crate::checkout_propagation::propagation_words::{
    PropagationOutcome,
    PropagationStep,
};
use crate::checkout_propagation::shared_mounts_beneath::{
    SharedMountsBeneath,
    shared_mounts_beneath,
};
use crate::subvolume::fs_mount_options::MountFacts;
use std::path::{
    Path,
    PathBuf,
};

/// The path the kernel names this boot by,
/// which the readiness record reads the same way.
const BOOT_ID_PATH: &str = "/proc/sys/kernel/random/boot_id";

/// What:
///  The checkout and the dataset its mount must serve,
///  as the caller resolved them,
///  or the refusal that resolution answered.
/// Why:
///  The step takes the answer its caller resolved and never reads the
///  configuration itself,
///  so a file that names neither dataset leaves the step working and the same
///  two refusals reach it
///  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
///  "With no configuration file";
///  the design's own choice V5 puts the step after that resolution).
///  A resolution that refused is still this step's refusal to file,
///  with the word `resolve`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CheckoutResolution = { kind: 'at'; checkout: string; dataset: string } | { kind: 'refused'; why: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckoutResolution {
    /// The checkout's path and the dataset its mount must serve.
    At {
        /// The checkout's path.
        checkout: PathBuf,
        /// The dataset's name.
        dataset: String,
    },
    /// The caller's own resolution refused,
    /// and the step files that refusal.
    Refused {
        /// The sentence the caller's refusal said.
        why: String,
    },
}

/// What:
///  Where one run of the step reads and writes.
/// Why:
///  The release run takes the installed record place and this boot's
///  identifier,
///  while a control hands in a fixture directory and a stated boot,
///  so the places are one value and no step hardcodes them twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PropagationPlaces = { record: PropagationFile; boot: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropagationPlaces {
    /// Where the record is published.
    pub record: PropagationFile,
    /// The boot identifier the record names.
    pub boot: String,
}

impl PropagationPlaces {
    /// What:
    ///  The installed places and this boot's own identifier.
    /// Why:
    ///  The handler runs from the installed unit,
    ///  and the record it writes must name the boot a reader compares it with.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const places = () => ({ record: PropagationFile.live(), boot: readFile('/proc/sys/kernel/random/boot_id').trim() });
    /// ```
    ///
    /// # Errors
    /// This boot's identifier could not be read.
    pub fn live() -> Result<Self, String> {
        let boot: String = std::fs::read_to_string(BOOT_ID_PATH).map_err(|error| {
            return format!(
                "this boot's identifier could not be read from {BOOT_ID_PATH} ({error}), so a propagation record would name no boot"
            );
        })?;
        return Ok(PropagationPlaces {
            record: PropagationFile::live(),
            boot: boot.trim().to_string(),
        });
    }
}

/// What:
///  What the step did,
///  and the line the mode says about it.
/// Why:
///  The mode's summary line is what an operator and a guest arm read,
///  so the step hands it back rather than printing it,
///  and it carries the same facts the record does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PropagationDone = { outcome: PropagationOutcome; line: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropagationDone {
    /// What the step did.
    pub outcome: PropagationOutcome,
    /// The line the mode says.
    pub line: String,
}

/// What:
///  Why the step refused.
/// Why:
///  A refusal is the act that refused,
///  the kernel's number where one answered,
///  and the sentence a person reads,
///  and the mode turns it into the handler's exit status of 2.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PropagationRefusal = { step: PropagationStep; errno?: number; message: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropagationRefusal {
    /// The act that refused.
    pub step: PropagationStep,
    /// The kernel's error number,
    /// where a call answered one.
    pub errno: Option<i32>,
    /// The sentence a person reads.
    pub message: String,
}

/// What:
///  Run the step:
///  resolve,
///  open,
///  read,
///  check,
///  read the table and what stands beneath,
///  set,
///  prove,
///  and file the answer.
/// Why:
///  The order is the design's,
///  the same descriptor carries the read,
///  the set and the read after it so the act names one instance,
///  and each refusal files the fields the run reached,
///  so nothing is changed and nothing is left unexplained
///  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
///  "Where it runs,
///  as whom,
///  and in what order").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = runTheStep(places, resolution, calls);
/// ```
///
/// # Errors
/// The checkout was not resolved,
/// a kernel act refused,
/// the mount was not the checkout's own instance,
/// the table held no row for it,
/// the read after the set did not prove it,
/// or the answer could not be filed.
pub fn run_the_step(
    places: &PropagationPlaces,
    resolution: &CheckoutResolution,
    calls: &dyn PropagationCalls,
) -> Result<PropagationDone, PropagationRefusal> {
    let (checkout, dataset): (PathBuf, String) = match resolution {
        CheckoutResolution::At {
            checkout,
            dataset,
        } => (checkout.clone(), dataset.clone()),
        CheckoutResolution::Refused {
            why,
        } => {
            let record: PropagationRecord = a_refusal(
                places,
                PropagationStep::Resolve,
                None,
                SharedMountsBeneath::Unread,
            );
            return Err(filed(places, record, why.clone()));
        }
    };
    let opened: std::fs::File = calls
        .open_the_checkout(checkout.as_path())
        .map_err(|failure| {
            return refused_call(places, PropagationStep::Open, &failure);
        })?;
    let before: DescriptorMount = calls.mount_of(&opened).map_err(|failure| {
        return refused_call(places, PropagationStep::Statmount, &failure);
    })?;
    let checked: Result<(), PropagationRefusal> =
        match check_the_facts(&before, checkout.as_path(), dataset.as_str()) {
            Some(refusal) => Err(refused_check(places, checkout.as_path(), &before, &refusal)),
            None => Ok(()),
        };
    checked?;
    let table: Vec<MountFacts> = calls.read_the_table().map_err(|failure| {
        return refused_call(places, PropagationStep::BeforeTable, &failure);
    })?;
    let row: &MountFacts = match check_the_row(&before, table.as_slice(), dataset.as_str()) {
        Ok(row) => row,
        Err(refusal) => {
            let record: PropagationRecord = a_refusal_with_facts(
                places,
                &before,
                checkout.as_path(),
                refusal.word,
                None,
                SharedMountsBeneath::Unread,
            );
            return Err(filed(places, record, refusal.why));
        }
    };
    let beneath: SharedMountsBeneath = shared_mounts_beneath(table.as_slice(), row.mount_id);
    if !before.propagation.carries_shared() {
        let outcome: PropagationOutcome = match before.propagation.is_private() {
            true => PropagationOutcome::AlreadyPrivate,
            false => PropagationOutcome::Left,
        };
        let record: PropagationRecord =
            an_answer(places, checkout.as_path(), &before, beneath, outcome);
        return finished(places, &record);
    }
    return set_then_prove(places, checkout.as_path(), &before, beneath, &opened, calls);
}

/// What:
///  Set the mount private through the descriptor the read verified,
///  prove the set on the same instance,
///  and file the answer.
/// Why:
///  The set is the one act that changes a mount,
///  and the read after it is what tells a set that worked from one that
///  answered success and left the set in place,
///  so both are one function and no caller reports a set it never proved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = setThenProve(places, checkout, before, beneath, opened, calls);
/// ```
fn set_then_prove(
    places: &PropagationPlaces,
    checkout: &Path,
    before: &DescriptorMount,
    beneath: SharedMountsBeneath,
    opened: &std::fs::File,
    calls: &dyn PropagationCalls,
) -> Result<PropagationDone, PropagationRefusal> {
    calls.make_private(opened).map_err(|failure| {
        return refused_with_facts(
            places,
            checkout,
            before,
            beneath.clone(),
            PropagationStep::Set,
            &failure,
        );
    })?;
    let after: DescriptorMount = calls.mount_of(opened).map_err(|failure| {
        return refused_with_facts(
            places,
            checkout,
            before,
            beneath.clone(),
            PropagationStep::VerifyAfter,
            &failure,
        );
    })?;
    let proved: bool = after.mount_id == before.mount_id
        && after.propagation.is_private()
        && after.peer_group.is_none()
        && after.master.is_none();
    if !proved {
        let record: PropagationRecord = PropagationRecord {
            after: Some(after.propagation),
            step: PropagationStep::VerifyAfter,
            ..an_answer(
                places,
                checkout,
                before,
                beneath,
                PropagationOutcome::Refused,
            )
        };
        return Err(filed(
            places,
            record,
            format!(
                "the set answered success and the mount reads {} with peer group {} and master {}, so the change was not proved",
                after.propagation.word(),
                after
                    .peer_group
                    .map_or(String::from("none"), |number| return number.to_string()),
                after
                    .master
                    .map_or(String::from("none"), |number| return number.to_string())
            ),
        ));
    }
    let record: PropagationRecord = PropagationRecord {
        after: Some(after.propagation),
        ..an_answer(
            places,
            checkout,
            before,
            beneath,
            PropagationOutcome::Applied,
        )
    };
    return finished(places, &record);
}

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
fn an_answer(
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
fn a_refusal(
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
fn a_refusal_with_facts(
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
fn refused_call(
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
fn refused_with_facts(
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
fn refused_check(
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
fn filed(
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
fn finished(
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

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "propagation_step_tests.rs"]
mod tests;
