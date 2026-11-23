//! What:
//!  The two names the probe takes from the root-owned handler configuration:
//!  the dataset whose `ready` snapshot is the source,
//!  and the checkout the walk opens read-only.
//! Why:
//!  The configuration is the only place the source comes from
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The source:
//! the probe dataset"),
//!  and a configuration that is absent,
//!  unreadable or names nothing the parse accepts is a boot whose route stays
//!  blocked rather than a probe failure
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  Q4).
//!  The project answered on 2026-11-21 that both sit inside one `startProbe`
//!  object,
//!  as `dataset` and `checkout`,
//!  so the root-owned configuration holds one object rather than two flat
//!  keys.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const source = sourceFromConfiguration('{"startProbe":{"dataset":"pool/ds"}}');
//! ```

use super::probe_source_name::{NameRefusal, ProbeSource, check_dataset_name};
use crate::jsonc_member_text::{MemberReading, text_of_member};
use std::path::{Path, PathBuf};

/// The one object the project answered holds both names.
pub const START_PROBE_KEY: &str = "startProbe";

/// The key inside `startProbe` that names the probe dataset,
/// beside the pool and dataset names the configuration pins.
pub const DATASET_KEY: &str = "dataset";

/// The key inside `startProbe` that names the checkout the walk opens
/// read-only.
pub const CHECKOUT_KEY: &str = "checkout";

/// What:
///  Why the configured name is a name the kernel would refuse,
///  or why the configuration could not supply one.
/// Why:
///   Every cause is a different thing for a person to look at,
///  and the record carries a closed word for each so a reader needs no prose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SourceRefusal = { kind: 'configuration-missing' | 'configuration-unreadable' | 'configuration-unparsable' | 'name-absent' | 'name-not-text' | 'name-refused' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceRefusal {
    /// The configuration file is not there.
    ConfigurationMissing {
        /// The path that was read.
        path: PathBuf,
    },
    /// The configuration file could not be read.
    ConfigurationUnreadable {
        /// The path that was read.
        path: PathBuf,
        /// What the read answered.
        reason: String,
    },
    /// The configuration file is not one JSONC document.
    ConfigurationUnparsable {
        /// The path that was read.
        path: PathBuf,
        /// What the parse answered.
        reason: String,
    },
    /// The configuration has no key naming a dataset.
    NameAbsent {
        /// The path that was read.
        path: PathBuf,
    },
    /// The key's value is not text.
    NameNotText {
        /// The path that was read.
        path: PathBuf,
    },
    /// The value is text the name rule refuses.
    NameRefused {
        /// The name as written.
        name: String,
        /// Why it is refused.
        why: NameRefusal,
    },
}

impl SourceRefusal {
    /// The word the record carries for this cause.
    pub fn word(&self) -> &'static str {
        return match self {
            SourceRefusal::ConfigurationMissing { .. } => "configuration-missing",
            SourceRefusal::ConfigurationUnreadable { .. } => "configuration-unreadable",
            SourceRefusal::ConfigurationUnparsable { .. } => "configuration-unparsable",
            SourceRefusal::NameAbsent { .. } => "name-absent",
            SourceRefusal::NameNotText { .. } => "name-not-text",
            SourceRefusal::NameRefused { .. } => "name-refused",
        };
    }

    /// The refusal in words,
    /// naming the path,
    /// the key and what to change.
    pub fn message(&self) -> String {
        let key: String = format!("{START_PROBE_KEY}.{DATASET_KEY}");
        return match self {
            SourceRefusal::ConfigurationMissing { path } => format!(
                "the probe's source is not configured: {} is not there. Nothing was attached, and every managed attach stays on the reflink or ordinary Git route until the configuration names the probe dataset. Ways forward: create the file with `{key}` naming the dataset, which `doctor` prints a command for.",
                path.display()
            ),
            SourceRefusal::ConfigurationUnreadable { path, reason } => format!(
                "the probe's source is not configured: {} could not be read ({reason}). Nothing was attached. Ways forward: make the file readable by root and run the probe again.",
                path.display()
            ),
            SourceRefusal::ConfigurationUnparsable { path, reason } => format!(
                "the probe's source is not configured: {} is not one JSONC document ({reason}). Nothing was attached. Ways forward: fix the file so it parses, then run the probe again.",
                path.display()
            ),
            SourceRefusal::NameAbsent { path } => format!(
                "the probe's source is not configured: {} has no `{key}` key. Nothing was attached. Ways forward: add `{key}` with the dataset name `doctor` prints.",
                path.display()
            ),
            SourceRefusal::NameNotText { path } => format!(
                "the probe's source is not configured: `{key}` in {} is not one text value. Nothing was attached. Ways forward: give it one quoted dataset name.",
                path.display()
            ),
            SourceRefusal::NameRefused { name, why } => format!(
                "the probe's source is not configured: `{key}` names `{name}`, which is not a dataset name the kernel takes ({}). Nothing was attached. Ways forward: give `{key}` the dataset name, with no snapshot, bookmark or path separator a name may not hold.",
                why.message()
            ),
        };
    }
}

impl CheckoutRefusal {
    /// The reason word the record carries.
    pub fn word(&self) -> &'static str {
        return match self {
            CheckoutRefusal::Unreadable => "configuration-missing",
            CheckoutRefusal::Unparsable => "configuration-unparsable",
            CheckoutRefusal::AbsentOrNotText => "no-checkout-path",
            CheckoutRefusal::NotAbsolute => "name-refused",
        };
    }
}

/// What:
///  Why the configuration names no checkout path the probe can open.
/// Why:
///   The checkout-root step opens one folder read-only,
///  and a field no step has written yet is a step that did not run rather than
///  a probe failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CheckoutRefusal = 'configuration-missing' | 'configuration-unparsable' | 'no-checkout-path' | 'name-refused';
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckoutRefusal {
    /// The configuration file could not be read.
    Unreadable,
    /// The file is not one JSONC document.
    Unparsable,
    /// The file has no checkout key,
    /// or its value is not text.
    AbsentOrNotText,
    /// The value is text that is not an absolute path.
    NotAbsolute,
}

/// What:
///  Read one configuration document's text value at one key inside
///  `startProbe`.
/// Why:
///  Both keys are one JSONC member,
///  and the parse and the key lookup are the repository's own single reader of
///  that file,
///  so a missing file and a missing key cannot be worded twice and the two
///  readers of the configuration cannot drift
///  (`package/git-policy/cli/src/native/foundation/jsonc_member_text.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = textAtKey(document, 'dataset'); // undefined when absent or not text
/// ```
fn text_at_key(configuration: &str, key: &str) -> Result<Option<String>, ()> {
    return match text_of_member(configuration, START_PROBE_KEY, key) {
        MemberReading::Text(text) => Ok(Some(text)),
        MemberReading::Absent | MemberReading::NotText => Ok(None),
        MemberReading::DocumentNotJsonc => Err(()),
    };
}

/// What:
///  The source the configuration names,
///  read from the document's text and checked.
/// Why:
///   The worker reads no property and no directory at start,
///  so this is the whole of the probe's knowledge about its source before the
///  attach.
///
/// # Errors
/// The document is not JSONC,
///  has no key,
///  or holds text the name rule refuses.
pub fn source_from_configuration(configuration: &str, path: &Path) -> Result<ProbeSource, SourceRefusal> {
    let text: Option<String> = match text_at_key(configuration, DATASET_KEY) {
        Ok(text) => text,
        Err(()) => return Err(SourceRefusal::ConfigurationUnparsable { path: path.to_path_buf(), reason: String::from("the document did not parse") }),
    };
    let Some(name) = text else {
        return Err(SourceRefusal::NameAbsent { path: path.to_path_buf() });
    };
    if let Err(why) = check_dataset_name(name.as_str()) {
        return Err(SourceRefusal::NameRefused { name, why });
    }
    return Ok(ProbeSource { dataset: name });
}

/// What:
///  The checkout path the same configuration names,
///  checked as an absolute path and nothing else.
/// Why:
///  The probe opens this folder read-only and never reads an entry of it,
///  so the only rule it can apply is that the path is absolute:
///  a relative path would be resolved against the worker's own directory.
///
/// # Errors
/// The file is not JSONC,
///  has no checkout key,
///  holds no text there,
///  or holds a path that is not absolute.
pub fn checkout_from_configuration(configuration: &str, _path: &Path) -> Result<PathBuf, CheckoutRefusal> {
    let text: Option<String> = match text_at_key(configuration, CHECKOUT_KEY) {
        Ok(text) => text,
        Err(()) => return Err(CheckoutRefusal::Unparsable),
    };
    let Some(checkout_text) = text else {
        return Err(CheckoutRefusal::AbsentOrNotText);
    };
    let checkout: PathBuf = PathBuf::from(checkout_text);
    return if checkout.is_absolute() { Ok(checkout) } else { Err(CheckoutRefusal::NotAbsolute) };
}

/// What:
///  Read the configuration file and answer the source and the checkout path it
///  names.
/// Why:
///   The file is read once,
///  so both keys come from the same bytes and a file changed between the two
///  reads cannot make them disagree.
///
/// # Errors
/// The file is absent or unreadable.
pub fn read_configuration(path: &Path) -> Result<String, SourceRefusal> {
    if !path.exists() {
        return Err(SourceRefusal::ConfigurationMissing { path: path.to_path_buf() });
    }
    return match std::fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(error) => Err(SourceRefusal::ConfigurationUnreadable { path: path.to_path_buf(), reason: error.to_string() }),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_configuration_tests.rs"]
mod tests;
