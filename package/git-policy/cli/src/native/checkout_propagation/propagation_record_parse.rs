//! What:
//!  The propagation record's parser:
//!  why a text is not a record this route wrote,
//!  and the reading of every field the writer fills.
//! Why:
//!  The writer proves its own parser reads the text back before a byte is
//!  written,
//!  and a reader compares a record with the live mount and never acts on it,
//!  so the grammar and the refusal it answers are one subject.
//!  It sits beside the record rather than in the folder that publishes it,
//!  because the parser uses the record's own words and the publisher uses both.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = parseRecord(text); // throws PropagationParseRefusal
//! ```

use crate::checkout_propagation::mount_propagation_set::MountPropagation;
use crate::checkout_propagation::propagation_record::PropagationRecord;
use crate::checkout_propagation::propagation_words::{
    BENEATH_PATH_KEY,
    BENEATH_UNREAD,
    MAXIMUM_BENEATH_PATH_BYTES,
    MAXIMUM_BOOT_BYTES,
    MAXIMUM_CHECKOUT_PATH_BYTES,
    MAXIMUM_DATASET_BYTES,
    MAXIMUM_ERRNO_BYTES,
    MAXIMUM_NUMBER_BYTES,
    MAXIMUM_SET_WORD_BYTES,
    MAXIMUM_STEP_BYTES,
    MAXIMUM_TIME_BYTES,
    NONE_WORD,
    PROPAGATION_KIND,
    PROPAGATION_VERSION,
    PropagationOutcome,
    PropagationStep,
    SHARED_MOUNTS_NAMED,
};
use crate::checkout_propagation::shared_mounts_beneath::SharedMountsBeneath;

/// What:
///  Why a text is not a record this route wrote.
/// Why:
///  Each deviation sends a reader somewhere else:
///  a missing key is a half-written file,
///  a wrong version is another build's file,
///  and an unknown word is a record whose writer this build cannot judge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PropagationParseRefusal = { kind: 'not-a-record' | 'version' | 'kind' | 'key' | 'too-long' | 'word' | 'number' | 'order'; what: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropagationParseRefusal {
    /// The kind of deviation.
    pub kind: &'static str,
    /// The line or the key that deviates,
    /// in the words a person reads.
    pub what: String,
}

impl PropagationParseRefusal {
    /// The sentence a person reads.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const message = (refusal: PropagationParseRefusal): string => `the propagation record is not one this route wrote: ${refusal.what}`;
    /// ```
    pub fn message(&self) -> String {
        return format!(
            "the propagation record is not one this route wrote ({kind}: {what})",
            kind = self.kind,
            what = self.what
        );
    }
}

/// What:
///  Read one record's text back,
///  refusing every shape this build does not write.
/// Why:
///  A reader compares the record with the live mount and never acts on it,
///  but a record the writer of another build wrote must still be refused,
///  so a malformed field is an answer rather than a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = parseRecord(text); // throws PropagationParseRefusal
/// ```
///
/// # Errors
/// The text is not this kind of record,
/// carries a version or a word this build does not know,
/// misses a key,
/// repeats one,
/// or holds a field longer than its bound.
pub fn parse_record(text: &str) -> Result<PropagationRecord, PropagationParseRefusal> {
    let mut lines: std::str::Lines<'_> = text.lines();
    let Some(first) = lines.next() else {
        return Err(PropagationParseRefusal {
            kind: "not-a-record",
            what: String::from("the text holds no line at all"),
        });
    };
    if first != PROPAGATION_VERSION {
        return Err(PropagationParseRefusal {
            kind: "version",
            what: format!(
                "the first line is `{first}` and this build writes `{PROPAGATION_VERSION}`"
            ),
        });
    }
    let mut fields: Vec<(String, String)> = Vec::new();
    for line in lines {
        let Some((key, value)) = line.split_once(' ') else {
            return Err(PropagationParseRefusal {
                kind: "not-a-record",
                what: format!("the line `{line}` holds no key and value"),
            });
        };
        fields.push((String::from(key), String::from(value)));
    }
    let kind: &str = value_of(&fields, "kind")?;
    if kind != PROPAGATION_KIND {
        return Err(PropagationParseRefusal {
            kind: "kind",
            what: format!(
                "the record's kind is `{kind}` and this route writes `{PROPAGATION_KIND}`"
            ),
        });
    }
    let record: PropagationRecord = PropagationRecord {
        boot: bounded(&fields, "boot", MAXIMUM_BOOT_BYTES)?,
        mount_id: optional_number(&fields, "mount-id")?,
        mount_id_old: optional_number(&fields, "mount-id-old")?,
        mount_namespace_id: optional_number(&fields, "mount-ns-id")?,
        checkout: optional_text(&fields, "checkout", MAXIMUM_CHECKOUT_PATH_BYTES)?,
        source: optional_text(&fields, "source", MAXIMUM_DATASET_BYTES)?,
        before: optional_set(&fields, "before")?,
        after: optional_set(&fields, "after")?,
        master: optional_number(&fields, "master")?,
        beneath: read_beneath(&fields)?,
        outcome: read_outcome(&fields)?,
        step: read_step(&fields)?,
        errno: optional_errno(&fields)?,
        time: bounded(&fields, "time", MAXIMUM_TIME_BYTES)?,
    };
    return Ok(record);
}

/// The value of one key,
/// which the text must hold exactly once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const valueOf = (fields: [string, string][], key: string): string => ...; // throws when absent or repeated
/// ```
fn value_of<'fields>(
    fields: &'fields [(String, String)],
    key: &str,
) -> Result<&'fields str, PropagationParseRefusal> {
    let mut found: Option<&str> = None;
    for (name, value) in fields {
        if name != key {
            continue;
        }
        if found.is_some() {
            return Err(PropagationParseRefusal {
                kind: "key",
                what: format!("the key `{key}` is written twice"),
            });
        }
        found = Some(value.as_str());
    }
    return match found {
        Some(value) => Ok(value),
        None => Err(PropagationParseRefusal {
            kind: "key",
            what: format!("the record carries no `{key}` line"),
        }),
    };
}

/// One bounded text field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bounded = (fields, key, bound): string => { const v = valueOf(fields, key); if (v.length > bound) throw ...; return v; };
/// ```
fn bounded(
    fields: &[(String, String)],
    key: &str,
    bound: usize,
) -> Result<String, PropagationParseRefusal> {
    let value: &str = value_of(fields, key)?;
    if value.len() > bound {
        return Err(PropagationParseRefusal {
            kind: "too-long",
            what: format!(
                "the `{key}` field is {} bytes and its bound is {bound}",
                value.len()
            ),
        });
    }
    return Ok(String::from(value));
}

/// One optional text field,
/// whose `none` word means the run never reached it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const optionalText = (fields, key, bound): string | undefined => { const v = bounded(fields, key, bound); return v === 'none' ? undefined : v; };
/// ```
fn optional_text(
    fields: &[(String, String)],
    key: &str,
    bound: usize,
) -> Result<Option<String>, PropagationParseRefusal> {
    let value: String = bounded(fields, key, bound)?;
    return match value.as_str() {
        NONE_WORD => Ok(None),
        _ => Ok(Some(value)),
    };
}

/// One bounded number field,
/// whose `none` word means the run never reached it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const optionalNumber = (fields, key): number | undefined => { const v = bounded(fields, key, 20); return v === 'none' ? undefined : Number(v); };
/// ```
fn optional_number(
    fields: &[(String, String)],
    key: &str,
) -> Result<Option<u64>, PropagationParseRefusal> {
    let value: String = bounded(fields, key, MAXIMUM_NUMBER_BYTES)?;
    if value == NONE_WORD {
        return Ok(None);
    }
    let Ok(number) = value.parse::<u64>() else {
        return Err(PropagationParseRefusal {
            kind: "number",
            what: format!("the `{key}` field is `{value}`, which is no number"),
        });
    };
    return Ok(Some(number));
}

/// The error number,
/// whose bound is the three digits a small error number takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const optionalErrno = (fields): number | undefined => { const v = bounded(fields, 'errno', 3); ... };
/// ```
fn optional_errno(fields: &[(String, String)]) -> Result<Option<i32>, PropagationParseRefusal> {
    let value: String = bounded(fields, "errno", MAXIMUM_ERRNO_BYTES)?;
    if value == NONE_WORD {
        return Ok(None);
    }
    let Ok(number) = value.parse::<i32>() else {
        return Err(PropagationParseRefusal {
            kind: "number",
            what: format!("the `errno` field is `{value}`, which is no error number"),
        });
    };
    return Ok(Some(number));
}

/// One optional propagation set,
/// whose word must be one of the five this build writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const optionalSet = (fields, key) => { const v = bounded(fields, key, 13); return v === 'none' ? undefined : MountPropagation.fromWord(v); };
/// ```
fn optional_set(
    fields: &[(String, String)],
    key: &str,
) -> Result<Option<MountPropagation>, PropagationParseRefusal> {
    let value: String = bounded(fields, key, MAXIMUM_SET_WORD_BYTES)?;
    if value == NONE_WORD {
        return Ok(None);
    }
    let Some(set) = MountPropagation::from_word(value.as_str()) else {
        return Err(PropagationParseRefusal {
            kind: "word",
            what: format!(
                "the `{key}` field holds `{value}`, which is no propagation set this build writes"
            ),
        });
    };
    return Ok(Some(set));
}

/// The outcome word,
/// which must be one of the four.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readOutcome = (fields) => PropagationOutcome.fromWord(valueOf(fields, 'outcome'));
/// ```
fn read_outcome(
    fields: &[(String, String)]
) -> Result<PropagationOutcome, PropagationParseRefusal> {
    let value: &str = value_of(fields, "outcome")?;
    let Some(outcome) = PropagationOutcome::from_word(value) else {
        return Err(PropagationParseRefusal {
            kind: "word",
            what: format!(
                "the `outcome` field holds `{value}`, which is no outcome this build writes"
            ),
        });
    };
    return Ok(outcome);
}

/// The step word,
/// which must be one of this build's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readStep = (fields) => PropagationStep.fromWord(valueOf(fields, 'step'));
/// ```
fn read_step(fields: &[(String, String)]) -> Result<PropagationStep, PropagationParseRefusal> {
    let value: String = bounded(fields, "step", MAXIMUM_STEP_BYTES)?;
    let Some(step) = PropagationStep::from_word(value.as_str()) else {
        return Err(PropagationParseRefusal {
            kind: "word",
            what: format!("the `step` field holds `{value}`, which is no step this build writes"),
        });
    };
    return Ok(step);
}

/// The count of shared mounts beneath the checkout and the paths it names,
/// which the record holds as one count followed by its path lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readBeneath = (fields) => { const count = valueOf(fields, 'beneath'); if (count === 'unread') return { kind: 'unread' }; const paths = fields.filter(([k]) => k === 'beneath-path').map(([, v]) => v); ... };
/// ```
fn read_beneath(
    fields: &[(String, String)]
) -> Result<SharedMountsBeneath, PropagationParseRefusal> {
    let value: String = bounded(fields, "beneath", MAXIMUM_NUMBER_BYTES)?;
    if value == BENEATH_UNREAD {
        return Ok(SharedMountsBeneath::Unread);
    }
    let Ok(count) = value.parse::<usize>() else {
        return Err(PropagationParseRefusal {
            kind: "number",
            what: format!("the `beneath` field is `{value}`, which is no count"),
        });
    };
    let mut paths: Vec<String> = Vec::new();
    for (key, text) in fields {
        if key != BENEATH_PATH_KEY {
            continue;
        }
        if text.len() > MAXIMUM_BENEATH_PATH_BYTES {
            return Err(PropagationParseRefusal {
                kind: "too-long",
                what: format!(
                    "a `{BENEATH_PATH_KEY}` line is {} bytes and its bound is {MAXIMUM_BENEATH_PATH_BYTES}",
                    text.len()
                ),
            });
        }
        if paths.len() == SHARED_MOUNTS_NAMED {
            return Err(PropagationParseRefusal {
                kind: "key",
                what: format!(
                    "the record names more than {SHARED_MOUNTS_NAMED} shared mounts beneath the checkout"
                ),
            });
        }
        paths.push(String::from(text));
    }
    if count < paths.len() {
        return Err(PropagationParseRefusal {
            kind: "number",
            what: format!(
                "the record counts {count} shared mounts beneath the checkout and names {}, so its count is wrong",
                paths.len()
            ),
        });
    }
    return Ok(SharedMountsBeneath::Counted {
        count,
        paths,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "propagation_record_parse_tests.rs"]
mod tests;
