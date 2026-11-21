//! What:
//!  The wrapper's side of the durable request:
//!  the property payload one intent is written as,
//!  the payload one nudge is written as,
//!  and the two requests behind the start gate that send them on the clone.
//! Why:
//!  The design keeps the request channel to the clone's own user properties
//!  and says the wrapper writes the intent and the nudge
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!  A request supplies no path,
//!  dataset name or mount flags:
//!  the property's value is the whole grammar.
//!  No raw request leaves this process before the compatibility gate passes,
//!  so the wrapper's first ZFS route obeys the same gate the handler does
//!  (`doc/decision/cli-git-zfs-runtime-layout-checker.md`,
//!  "Any version,
//! layout decides").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const writer = CloneIntentWriter.open(gate, clone); writer.request(intent); writer.nudge(nonce);
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_intent::{MountIntent, intent_text};
use crate::worktree_clone_mount_protocol::clone_mount_property_names::{DESIRED_STATE_PROPERTY, SWEEP_NONCE_PROPERTY};
use crate::worktree_clone_mount_protocol::clone_mount_sweep_nonce::sweep_nonce_text;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::zfs_compatibility_gate::gate_verdict::{BeforeRequests, Verdict};
use crate::zfs_compatibility_gate::gate_words::verdict_line;
use monochromatic_zfs_request_to_openzfs_kernel_device::{DeviceError, NvList, ZfsDevice};

/// What:
///  Why the wrapper could not write the clone's intent or nudge.
/// Why:
///   A caller must be able to tell a refused gate,
///  a refused device and a refused write apart,
///  because each has a different way forward,
///  and none of them may be read as a filed request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IntentWriteRefusal = { kind: 'gate' | 'device' | 'write'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IntentWriteRefusal {
    /// The compatibility gate refused,
    /// with the line it prints.
    Gate {
        /// The gate's line.
        line: String,
    },
    /// The device could not be opened.
    Device {
        /// What the call answered.
        reason: String,
    },
    /// The request was refused or its outcome is unknown.
    Write {
        /// What the call answered.
        reason: String,
    },
}

impl IntentWriteRefusal {
    /// The refusal in words,
    /// naming what refused and the way forward.
    pub fn message(&self) -> String {
        let reason: String = match self {
            IntentWriteRefusal::Gate { line } => format!("{line}\nthe compatibility gate refused, so no request was made"),
            IntentWriteRefusal::Device { reason } => format!("the ZFS device could not be opened ({reason}), so no request was made"),
            IntentWriteRefusal::Write { reason } => format!(
                "the property request was refused or its outcome is unknown ({reason}), so this build does not report it as filed"
            ),
        };
        return format!(
            "{reason}. Nothing was mounted or unmounted by this invocation. Ways forward: read the handler's journal (`journalctl -u reconcile-git-worktree-clone-mounts.service`); if the outcome is unknown, inspect the clone's `{DESIRED_STATE_PROPERTY}` property before any retry, because this route never sends the same write twice blindly."
        );
    }
}

/// What:
///  The property list one intent is written as:
///  the desired-state property carrying exactly the designed value.
/// Why:
///   The kernel takes a property list,
///  and keeping the payload here means every writer sends the same keys and
///  no path,
///  dataset name or flag travels with a request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const payload = intent_property(intent); // { 'org.monochromatic:...desired-state': 'v1:...:mount' }
/// ```
pub fn intent_property(intent: &MountIntent) -> NvList {
    let mut properties: NvList = NvList::new();
    properties.add_str(DESIRED_STATE_PROPERTY, intent_text(intent.operation, &intent.nonce));
    return properties;
}

/// What:
///  The property list one nudge is written as:
///  the sweep-nonce property carrying the nonce alone.
/// Why:
///   A nudge is not an instruction,
///  so its payload cannot carry an operation even if a caller composed one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const payload = nudge_property(nonce); // { 'org.monochromatic:...sweep-nonce': '...' }
/// ```
pub fn nudge_property(nonce: &RequestNonce) -> NvList {
    let mut properties: NvList = NvList::new();
    properties.add_str(SWEEP_NONCE_PROPERTY, sweep_nonce_text(nonce));
    return properties;
}

/// What:
///  One clone's intent writer:
///  the device opened once behind the gate,
///  and the clone every write names.
/// Why:
///  The wrapper's first request route must run the gate before the device is
///  opened,
///  and the clone is named once so a write can never land on another dataset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CloneIntentWriter { static open(gate, clone) {...} request(intent) {...} nudge(nonce) {...} }
/// ```
pub struct CloneIntentWriter {
    /// The opened device.
    device: ZfsDevice,
    /// The clone every write names.
    clone: String,
}

impl CloneIntentWriter {
    /// Run the gate,
    /// then open the device.
    ///
    /// # Errors
    /// The gate refused,
    /// or the device could not be opened.
    pub fn open(gate: &dyn BeforeRequests, clone: &str) -> Result<Self, IntentWriteRefusal> {
        let verdict: Verdict = gate.verdict();
        if let Verdict::Refused(_) = verdict {
            return Err(IntentWriteRefusal::Gate { line: verdict_line(&verdict) });
        }
        let device: ZfsDevice = ZfsDevice::open().map_err(device_refusal)?;
        return Ok(CloneIntentWriter { device, clone: String::from(clone) });
    }

    /// Write the durable intent on the clone.
    ///
    /// # Errors
    /// The kernel refused the write or its outcome is unknown.
    pub fn request(&self, intent: &MountIntent) -> Result<(), IntentWriteRefusal> {
        return self.write(&intent_property(intent));
    }

    /// Change the clone's sweep nonce.
    ///
    /// # Errors
    /// The kernel refused the write or its outcome is unknown.
    pub fn nudge(&self, nonce: &RequestNonce) -> Result<(), IntentWriteRefusal> {
        return self.write(&nudge_property(nonce));
    }

    /// Send one property list to the clone.
    fn write(&self, properties: &NvList) -> Result<(), IntentWriteRefusal> {
        let answer = self.device.set_prop(self.clone.as_str(), properties).map_err(write_refusal)?;
        drop(answer);
        return Ok(());
    }
}

/// Describe a device that could not be opened.
fn device_refusal(error: DeviceError) -> IntentWriteRefusal {
    return IntentWriteRefusal::Device { reason: error.to_string() };
}

/// Describe a write the kernel refused or answered unclearly.
fn write_refusal(error: DeviceError) -> IntentWriteRefusal {
    return IntentWriteRefusal::Write { reason: error.to_string() };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_mount_intent_write_tests.rs"]
mod tests;
