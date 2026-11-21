//! What:
//!  The words a caller and `doctor` print for a verdict:
//! the compatible line naming
//!  the module,
//! its release string,
//! the target and the files,
//! and the refusal naming the
//!  step,
//! the file,
//! the detail and what stays disabled.
//! Why:
//!  The gate's facts and the gate's words are kept apart,
//! so the wording is tested on
//!  built facts and a caller prints the same thing about the same refusal.
//!  A refusal is the process's answer and not a failure of one command:
//! the handler answers
//!  status 2 with it,
//! so the shared line says what stays disabled and leaves what a
//!  particular program does next to that program,
//! which is where a mode's own sentence belongs.
//!  The release sentence is reporting and never a rule:
//! it is printed only when the
//!  string differs from the text the tested module reported,
//! `2.4.4-1`,
//! and it says
//!  a different text was read,
//! not a different version.
//!  A refusal offers no bypass:
//! there is no flag and no environment that turns a
//!  refusal into a pass.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const words = verdict.compatible ? compatibleLine(verdict.facts) : refusalLine(verdict.refusal);
//! ```

use super::gate_verdict::{Facts, Verdict};
use monochromatic_zfs_request_to_openzfs_kernel_device::CompatibilityRefusal as Refusal;

/// The release text the module the design tested reported,
/// which the release sentence compares against.
pub const TESTED_MODULE_VERSION: &str = "2.4.4-1";

/// The sentence that says the release string is not the tested text,
/// or nothing when it is.
pub fn release_sentence(version: Option<&str>) -> Option<String> {
    let Some(read): Option<&str> = version else {
        return Some(format!(
            "the loaded module reports no release text, and the module this repository tested reports {TESTED_MODULE_VERSION}: nothing compares a release text with a rule, so this is a fact a person reads and not a refusal"
        ));
    };
    if read == TESTED_MODULE_VERSION {
        return None;
    }
    return Some(format!(
        "the loaded module reports the release text {read}, and the module this repository tested reports {TESTED_MODULE_VERSION}: a different text was read, not a different version, and nothing compares that text with a rule"
    ));
}

/// The line a compatible verdict prints.
pub fn compatible_line(facts: &Facts) -> String {
    let mut words: String = format!(
        "the start-time ZFS compatibility gate is compatible: target {}, module release text {}, kernel type information {} bytes (header {}), module type information {} bytes (header {}), so the loaded module's declared layout and request numbers match this build and raw requests may be made",
        facts.target,
        facts.version.as_deref().unwrap_or("absent"),
        facts.base_btf_bytes,
        facts.header_bytes.0,
        facts.module_btf_bytes,
        facts.header_bytes.1
    );
    if let Some(sentence) = release_sentence(facts.version.as_deref()) {
        words.push_str("; ");
        words.push_str(sentence.as_str());
    }
    if !facts.absent.is_empty() {
        words.push_str("; ");
        words.push_str(format!(
            "this module carries no {}, so the reload check has fewer terms to compare and doctor says the check is weaker there",
            facts.absent.join(", ")
        ).as_str());
    }
    return words;
}

/// The words a refusal prints,
/// with what stays disabled and that nothing was changed.
///
/// The line names no mode and no command's status:
/// the handler appends its own sentence saying no mode was entered,
/// and the wrapper's refusal,
/// when its route lands,
/// appends its own.
pub fn refusal_line(refusal: &Refusal) -> String {
    return format!(
        "the start-time ZFS compatibility gate refused: {refusal}. Every raw request, reads included, stays disabled until a module whose declared layout and request numbers match this build is loaded, and registrations and durable recovery state stay as they were. No flag and no environment turns this refusal into a pass."
    );
}

/// The line one verdict prints.
pub fn verdict_line(verdict: &Verdict) -> String {
    return match verdict {
        Verdict::Compatible(facts) => compatible_line(facts),
        Verdict::Refused(refusal) => refusal_line(refusal),
    };
}
