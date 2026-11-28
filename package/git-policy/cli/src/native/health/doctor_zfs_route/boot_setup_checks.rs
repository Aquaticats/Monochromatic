//! What:
//!  The boot and shutdown units plan step 19 installs,
//!  judged against what stands on the machine and against the datasets
//!  measured under the checkout:
//!  every file this build's plan writes,
//!  the provider the gate orders itself on,
//!  the key-loading unit when the machine has one,
//!  the two time bounds,
//!  the drop-ins beside the gate,
//!  and `canmount` on every dataset whose mountpoint lies inside the
//!  checkout.
//! Why:
//!  the owner's answer V7 asks `doctor` to print the whole boot setup:
//!  the provider,
//!  whether a key unit exists and matches the unit's key lines,
//!  every dataset under the checkout not set `canmount=noauto`,
//!  the time bounds,
//!  the drop-ins,
//!  and the installed file's identity
//!  (`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`,
//!  V7;
//!  plan step 19,
//!  "What it builds").
//!  A file nobody installed runs nothing at boot,
//!  a file whose text an older release or a hand edit wrote runs lines this
//!  build did not decide,
//!  and a dataset left `canmount=on` is mounted by `zfs mount -a` before the
//!  gate proved the checkout,
//!  which is what V4 set `noauto` to prevent.
//!  The lines are read from the gate's own text,
//!  so a gate that lost its provider line or its key lines is a finding
//!  rather than a sentence printed from a decision nobody installed.
//!  Nothing here writes a file,
//!  starts a unit or changes a property:
//!  a setup run or the owner does that,
//!  and this line prints the correction
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 19,
//!  "What it builds").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkServiceUnit(facts); // pass | finding('provider'|'key-unit'|'bounds'|'installed-file'|'nested-canmount') | unknown
//! ```

use super::boot_setup_facts::{
    BootSetupFacts,
    FileAtInstallPath,
    PlannedFileReading,
};
use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
use super::zfs_checks_limits::{
    CANMOUNT_NOAUTO,
    CANMOUNT_PROPERTY,
    canmount_word,
};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::PropertyValue;

/// The field a finding about the provider's ordering lines names.
const FIELD_PROVIDER: &str = "provider";

/// The field a finding about the gate's key lines names.
const FIELD_KEY_UNIT: &str = "key-unit";

/// The field a finding about one of the two bounds names.
const FIELD_BOUNDS: &str = "bounds";

/// The field a finding about one installed file names.
const FIELD_INSTALLED_FILE: &str = "installed-file";

/// The field a finding about one dataset's `canmount` names.
const FIELD_NESTED_CANMOUNT: &str = "nested-canmount";

/// The sentence every file finding ends with,
/// because the repair is the same for a missing file and for a stale one.
const INSTALL_THE_FILE: &str = "the correction is the file this build's plan writes there, which a setup run installs from the same plan, and this check writes nothing";

/// The line a provider is wanted by.
const WANTS_PREFIX: &str = "Wants=";

/// The line a unit starts after.
const AFTER_PREFIX: &str = "After=";

/// The line a unit bounds its start by.
const START_BOUND_PREFIX: &str = "TimeoutStartSec=";

/// The line a unit bounds its stop by.
const STOP_BOUND_PREFIX: &str = "TimeoutStopSec=";

/// The line a key-loading unit is required by.
const REQUIRES_PREFIX: &str = "Requires=";

/// What:
///  What the boot and shutdown units answer:
///  a pass naming every decided line and where it stands,
///  a finding naming each line,
///  file or dataset that disagrees with the decision,
///  or unknown where a file could not be read.
/// Why:
///  a person reading the report needs the provider's name,
///  the bounds and the drop-in paths to compare with their own notes,
///  and a finding that named only "a file differs" would send them to diff a
///  folder of units.
///  A finding outranks an unknown here,
///  as the clone and key checks do,
///  because a file nobody installed is a fact whatever a second path answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkServiceUnit(facts);
/// ```
pub fn check_service_unit(facts: &BootSetupFacts) -> CheckOutcome {
    let mut findings: Vec<(&'static str, String)> = Vec::new();
    let mut unreadable: Vec<String> = Vec::new();
    let gate_text: &str = facts.gate.planned_text.as_str();
    let provider: Option<String> = first_word_after(gate_text, WANTS_PREFIX);
    let after: Option<String> = words_after(gate_text, AFTER_PREFIX);
    provider_findings(provider.as_deref(), after.as_deref(), &mut findings);
    key_findings(facts.key_unit.as_deref(), gate_text, &mut findings);
    bound_findings(gate_text, &mut findings);
    for file in std::iter::once(&facts.gate).chain(facts.others.iter()) {
        file_findings(file, &mut findings, &mut unreadable);
    }
    for setting in &facts.nested {
        let dataset: &str = setting.dataset.as_str();
        match canmount_word_for(&setting.canmount) {
            None => unreadable.push(format!(
                "the `{CANMOUNT_PROPERTY}` value of `{dataset}` ({})",
                setting.canmount.describe(CANMOUNT_PROPERTY)
            )),
            Some(word) => {
                if setting.canmount.number() != Some(CANMOUNT_NOAUTO) {
                    findings.push((
                        FIELD_NESTED_CANMOUNT,
                        format!(
                            "`{dataset}` is `{CANMOUNT_PROPERTY}={word}`, and every dataset whose mountpoint lies inside the checkout must be `noauto` so `zfs mount -a` cannot mount it before the gate proved the checkout; correction: `sudo zfs set {CANMOUNT_PROPERTY}=noauto {dataset}`, whose unit this build's plan already writes"
                        ),
                    ));
                }
            }
        }
    }
    if let Some((field, first)) = findings.first() {
        let mut detail: String = first.clone();
        for (_, other) in findings.iter().skip(1) {
            detail.push_str("; ");
            detail.push_str(other);
        }
        return finding(*field, detail);
    }
    if !unreadable.is_empty() {
        return unknown(format!(
            "the boot setup's answer is unproved because {} could not be read",
            unreadable.join("; ")
        ));
    }
    return pass(pass_detail(facts, provider.as_deref(), after.as_deref()));
}

/// The findings the gate's provider lines answer,
/// which is the ordering V2 decided.
fn provider_findings(
    provider: Option<&str>,
    after: Option<&str>,
    findings: &mut Vec<(&'static str, String)>,
) {
    let Some(wants) = provider else {
        findings.push((
            FIELD_PROVIDER,
            format!(
                "the gate this build's plan writes carries no `{WANTS_PREFIX}` line, so it wants no provider and nothing makes the boot mount the checkout before the gate runs"
            ),
        ));
        return;
    };
    let Some(after_words) = after else {
        findings.push((
            FIELD_PROVIDER,
            format!(
                "the gate this build's plan writes carries no `{AFTER_PREFIX}` line, so it starts beside its provider rather than after it"
            ),
        ));
        return;
    };
    if !after_words
        .split_whitespace()
        .any(|word| return word == wants)
    {
        findings.push((
            FIELD_PROVIDER,
            format!(
                "the gate wants `{wants}` and its `{AFTER_PREFIX}` line orders it after `{after_words}`, which does not name that unit"
            ),
        ));
    }
}

/// The findings the gate's key lines answer,
/// which is V7's "whether a key unit exists and matches the unit's key lines".
fn key_findings(
    key_unit: Option<&str>,
    gate_text: &str,
    findings: &mut Vec<(&'static str, String)>,
) {
    let Some(key) = key_unit else {
        return;
    };
    let requires: Option<String> = words_after(gate_text, REQUIRES_PREFIX);
    let after: Option<String> = words_after(gate_text, AFTER_PREFIX);
    let required: bool = requires
        .as_deref()
        .is_some_and(|words| return words.split_whitespace().any(|word| return word == key));
    let ordered: bool = after
        .as_deref()
        .is_some_and(|words| return words.split_whitespace().any(|word| return word == key));
    if !required || !ordered {
        findings.push((
            FIELD_KEY_UNIT,
            format!(
                "this machine's key-loading unit is `{key}` and the gate this build's plan writes does not `{REQUIRES_PREFIX}` it and `{AFTER_PREFIX}` it, so the gate can run before the key is loaded and read a dataset nobody can open; correction: install the gate text this build's plan writes for a machine with that unit"
            ),
        ));
    }
}

/// The findings the two bounds answer,
/// which are the owner's V8b numbers until step 22 measures the workload.
fn bound_findings(
    gate_text: &str,
    findings: &mut Vec<(&'static str, String)>,
) {
    for prefix in [START_BOUND_PREFIX, STOP_BOUND_PREFIX] {
        if first_word_after(gate_text, prefix).is_none() {
            findings.push((
                FIELD_BOUNDS,
                format!(
                    "the gate this build's plan writes carries no `{prefix}` line, so a provider that stops answering holds the boot for systemd's default rather than the bound the owner set"
                ),
            ));
        }
    }
}

/// The findings one planned file's state answers:
/// absent,
/// not a regular file,
/// or holding text other than this build's.
/// A path nobody could read is kept for the unknown answer.
fn file_findings(
    file: &PlannedFileReading,
    findings: &mut Vec<(&'static str, String)>,
    unreadable: &mut Vec<String>,
) {
    let path: String = file.path.display().to_string();
    match &file.at_the_path {
        FileAtInstallPath::Unreadable {
            why,
        } => unreadable.push(format!("{path} ({why})")),
        FileAtInstallPath::Absent => findings.push((
            FIELD_INSTALLED_FILE,
            format!("{path} is not installed, so nothing runs there at boot; {INSTALL_THE_FILE}"),
        )),
        FileAtInstallPath::WrongKind {
            found,
        } => findings.push((
            FIELD_INSTALLED_FILE,
            format!(
                "{path} is {found} rather than the regular file this build's plan writes, so systemd reads no text there; {INSTALL_THE_FILE}"
            ),
        )),
        FileAtInstallPath::Content(text) => {
            if let Some((line, wanted, stands)) = first_difference(file.planned_text.as_str(), text.as_str()) {
                findings.push((
                    FIELD_INSTALLED_FILE,
                    format!(
                        "{path} differs from the text this build writes for it at line {line}: `{stands}` stands where this build writes `{wanted}`, so an older release or a hand edit wrote it; {INSTALL_THE_FILE}"
                    ),
                ));
            }
        }
    }
}

/// What the pass says:
/// the provider,
/// the key unit,
/// both bounds and every file the plan writes beside the gate,
/// so a person compares them with their own notes.
fn pass_detail(
    facts: &BootSetupFacts,
    provider: Option<&str>,
    after: Option<&str>,
) -> String {
    let provider_words: &str = provider.unwrap_or("no provider");
    let after_words: &str = after.unwrap_or("no `After=` line");
    let key: String = match &facts.key_unit {
        Some(name) => format!(
            "the key-loading unit `{name}` exists and the gate requires it and starts after it"
        ),
        None => String::from(
            "this machine has no key-loading unit for the checkout's encryption root, so the gate names none",
        ),
    };
    let start: String = first_word_after(facts.gate.planned_text.as_str(), START_BOUND_PREFIX)
        .unwrap_or_else(|| return String::from("no bound"));
    let stop: String = first_word_after(facts.gate.planned_text.as_str(), STOP_BOUND_PREFIX)
        .unwrap_or_else(|| return String::from("no bound"));
    let others: Vec<String> = facts
        .others
        .iter()
        .map(|file| return file.path.display().to_string())
        .collect();
    return format!(
        "the gate `{}` stands as this build writes it, with `{WANTS_PREFIX}{provider_words}` and `{AFTER_PREFIX}{after_words}`, its start bounded at `{START_BOUND_PREFIX}{start}` and its stop at `{STOP_BOUND_PREFIX}{stop}`; {key}; the {} other file(s) this build's plan writes stand unchanged ({}); and {} dataset(s) under the checkout are `{CANMOUNT_PROPERTY}=noauto`",
        facts.gate.path.display(),
        facts.others.len(),
        others.join(", "),
        facts.nested.len()
    );
}

/// What:
///  Every word a unit text carries after one prefix,
///  from every line that starts with it,
///  or nothing when no line does.
/// Why:
///  systemd appends a repeated directive's words,
///  and this build's gate carries one `After=` line for the key-loading unit
///  and another for the provider,
///  so a reader that stopped at the first line would judge the provider
///  against the key unit's line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = wordsAfter(text, 'After='); // 'key.service zfs-mount.service zfs-import.target'
/// ```
fn words_after(
    text: &str,
    prefix: &str,
) -> Option<String> {
    let tails: Vec<&str> = text
        .lines()
        .filter(|line| return line.starts_with(prefix))
        .map(|line| return line[prefix.len()..].trim())
        .collect();
    if tails.is_empty() {
        return None;
    }
    return Some(tails.join(" "));
}

/// The first word one line of a unit text carries after a prefix.
fn first_word_after(
    text: &str,
    prefix: &str,
) -> Option<String> {
    let words: String = words_after(text, prefix)?;
    let first: &str = words.split_whitespace().next()?;
    return Some(String::from(first));
}

/// The first line two texts disagree on,
/// with its number and both lines.
fn first_difference(
    expected: &str,
    found: &str,
) -> Option<(usize, String, String)> {
    let wanted: Vec<&str> = expected.lines().collect();
    let stands: Vec<&str> = found.lines().collect();
    let count: usize = wanted.len().max(stands.len());
    for index in 0..count {
        let a: &str = wanted.get(index).copied().unwrap_or_default();
        let b: &str = stands.get(index).copied().unwrap_or_default();
        if a != b {
            return Some((index + 1, String::from(a), String::from(b)));
        }
    }
    return None;
}

/// The word OpenZFS prints for one `canmount` value,
/// or nothing where no value was read.
fn canmount_word_for(value: &PropertyValue) -> Option<&'static str> {
    let number: u64 = value.number()?;
    return Some(canmount_word(number));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "boot_setup_checks_tests.rs"]
mod tests;
