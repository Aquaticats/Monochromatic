//! What:
//!  Controls of the wrapper's request payloads and its gate:
//!  the intent property's one key and exact value,
//!  the nudge's one key and the nonce alone,
//!  and a refused gate making no request and opening no device.
//! Why:
//!  A request supplies no path,
//!  dataset name or mount flag,
//!  and the wrapper may open `/dev/zfs` only after the gate passes
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering";
//!  `doc/decision/cli-git-zfs-runtime-layout-checker.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(intentProperty(intent).get(DESIRED_STATE_PROPERTY)).toBe('v1:...:mount');
//! ```

use super::{
    CloneIntentWriter,
    IntentWriteRefusal,
    intent_property,
    nudge_property,
};
use crate::worktree_clone_mount_protocol::clone_mount_intent::{
    MountIntent,
    MountOperation,
};
use crate::worktree_clone_mount_protocol::clone_mount_property_names::{
    DESIRED_STATE_PROPERTY,
    SWEEP_NONCE_PROPERTY,
};
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::zfs_compatibility_gate::gate_verdict::{
    BeforeRequests,
    Verdict,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    CompatibilityRefusal,
    CompatibilityStep,
};

/// The clone every payload names.
const CLONE: &str = "zpcachyos/nosnap/clones/one";

/// The nonce the controls write.
fn nonce() -> RequestNonce {
    return RequestNonce::parse("0123456789abcdef0123456789abcdef")
        .expect("the control nonce is valid");
}

/// A gate that refuses at the device step without reading a device.
struct RefusingGate;

impl BeforeRequests for RefusingGate {
    fn verdict(&self) -> Verdict {
        return Verdict::Refused(CompatibilityRefusal {
            step: CompatibilityStep::Device,
            file: None,
            detail: String::from("the control's own refusal"),
        });
    }
}

/// The intent payload carries one key and exactly the designed value.
#[test]
fn the_intent_payload_is_one_property_with_the_designed_value() {
    for (operation, word) in [
        (MountOperation::Mount, "mount"),
        (MountOperation::Unmount, "unmount"),
    ] {
        let intent: MountIntent = MountIntent {
            operation,
            nonce: nonce(),
        };
        let payload = intent_property(&intent);
        assert_eq!(payload.iter().count(), 1);
        assert_eq!(
            payload.get_str(DESIRED_STATE_PROPERTY),
            Some(format!("v1:{}:{word}", nonce().as_str()).as_str())
        );
        assert_eq!(payload.get_str(SWEEP_NONCE_PROPERTY), None);
        assert!(
            !payload
                .get_str(DESIRED_STATE_PROPERTY)
                .unwrap_or_default()
                .contains('/')
        );
        assert!(
            !payload
                .get_str(DESIRED_STATE_PROPERTY)
                .unwrap_or_default()
                .contains("flag")
        );
    }
}

/// The nudge payload carries one key and the nonce alone,
/// so it names no operation.
#[test]
fn the_nudge_payload_is_the_nonce_alone() {
    let payload = nudge_property(&nonce());
    assert_eq!(payload.iter().count(), 1);
    assert_eq!(
        payload.get_str(SWEEP_NONCE_PROPERTY),
        Some(nonce().as_str())
    );
    assert_eq!(payload.get_str(DESIRED_STATE_PROPERTY), None);
    let value: &str = payload.get_str(SWEEP_NONCE_PROPERTY).unwrap_or_default();
    assert!(!value.contains("mount"), "{value}");
    assert!(!value.contains(':'), "{value}");
}

/// A refused gate answers before any device is opened,
/// so a machine without `/dev/zfs` still runs this refusal.
#[test]
fn a_refused_gate_opens_no_device() {
    let refusal: IntentWriteRefusal = CloneIntentWriter::open(&RefusingGate, CLONE)
        .err()
        .expect("the gate refuses");
    match &refusal {
        IntentWriteRefusal::Gate {
            line,
        } => assert!(line.contains("device"), "{line}"),
        other => panic!("the refusal is the gate's: {other:?}"),
    }
    let message: String = refusal.message();
    assert!(message.contains("no request was made"), "{message}");
    assert!(message.contains(DESIRED_STATE_PROPERTY), "{message}");
}
