//! What:
//!  When a sweep takes this boot's `first-attach` name and what it writes
//!  there:
//!  after the key check and before the walk,
//!  and the attach's outcome before the receipt.
//! Why:
//!  The start-probe design places the claim and the write in the handler's
//!  attach path,
//!  and plan step 8 is that path
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The first real attach's record",
//!  "Which plan steps carry the duty";
//!  `doc/decision/cli-git-zfs-start-probe.md`,
//!  Q7 and Q-b;
//!  the project chose on 2026-11-22 that step 8 builds the writer and the call
//!  in that design's claim order,
//!  `doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "The `first-attach` writer").
//!  The key check is not inside the claim:
//!  an unavailable key is a finding about the key rather than an attach
//!  outcome,
//!  so an attempt the key refuses takes no claim and writes nothing.
//!  The clone's type,
//!  GUID,
//!  origin,
//!  properties,
//!  slot derivation and intent parse all precede the walk,
//!  and none of them takes a claim either.
//!  Every refusal from the walk onward is written with the step that failed.
//!  The seam is what lets a control drive the sweep's order without writing
//!  under `/run`,
//!  and the release recorder is the one writer the guest and the boot unit run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claim = recorder.claim(); // this boot's name, or nothing when another process won it
//! recorder.record(claim, { kind: 'attached', mountId, flags, slot, clone, guid });
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::start_probe::first_attach_record::{FirstAttachClaim, FirstAttachOutcome, FirstAttachPlace, outcome_record};
use crate::start_probe::probe_fingerprint::{FingerprintPaths, ProbeFingerprint};

/// Module tag for diagnostics.
const TAG: &str = "sweepFirstAttach";

/// What:
///  The two calls the sweep makes on this boot's `first-attach` file.
/// Why:
///  The release path writes `/run`,
///  and a control must not,
///  so the sweep asks a seam and the control answers nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface FirstAttachRecorder { claim(); record(claim, outcome); }
/// ```
pub trait FirstAttachRecorder {
    /// Take this boot's name,
    /// or answer nothing when another process claimed it or the record cannot
    /// be written.
    fn claim(&self) -> Option<FirstAttachClaim>;

    /// Publish the attach's outcome over the claimed name.
    fn record(&self, claim: &FirstAttachClaim, outcome: &FirstAttachOutcome);
}

/// What:
///  The release recorder:
///  the fingerprinted started record and the outcome under `/run`.
/// Why:
///  The file sits beside the probe's record at the same directory and mode,
///  and the fingerprint is the same read the probe's record uses,
///  so one build's first attach is pinned to the kernel and module it ran on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recorder = LiveFirstAttach.live();
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveFirstAttach {
    /// The place the file lives.
    place: FirstAttachPlace,
}

impl LiveFirstAttach {
    /// The place the handler writes at boot.
    pub fn live() -> Self {
        return LiveFirstAttach { place: FirstAttachPlace::live() };
    }

    /// The place a control names,
    /// so the claim and the rename are proved without `/run`.
    pub fn at(place: FirstAttachPlace) -> Self {
        return LiveFirstAttach { place };
    }
}

impl FirstAttachRecorder for LiveFirstAttach {
    fn claim(&self) -> Option<FirstAttachClaim> {
        // A fingerprint this process cannot read is a machine fact no record can pin, and the probe's own read already
        // refuses the route then; a first attach that writes nothing is the smaller loss, and the handler's journal says
        // which fact was missing.
        let fingerprint: ProbeFingerprint = match ProbeFingerprint::read(&FingerprintPaths::live()) {
            Ok(fingerprint) => fingerprint,
            Err(refusal) => {
                warn(TAG, refusal.message().as_str());
                return None;
            }
        };
        let time: String = crate::iso_time::now_iso();
        return match self.place.claim(&fingerprint, time.as_str()) {
            Ok(claim) => {
                debug(TAG, format!("this process claimed this boot's first attach at {}", claim.path().display()).as_str());
                Some(claim)
            }
            Err(refusal) => {
                // Another process winning the name is the ordinary case of a second sweep, not a failure of this one.
                debug(TAG, refusal.message().as_str());
                None
            }
        };
    }

    fn record(&self, claim: &FirstAttachClaim, outcome: &FirstAttachOutcome) {
        let time: String = crate::iso_time::now_iso();
        let text: String = outcome_record(claim.fingerprint(), time.as_str(), outcome);
        if let Err(refusal) = self.place.record(claim, text.as_str()) {
            warn(TAG, refusal.message().as_str());
        }
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_first_attach_tests.rs"]
mod tests;
