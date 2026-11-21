//! What:
//!  Controls of the sweep nudge:
//!  the read value,
//!  an inherited value,
//!  a malformed one,
//!  and the fact that an intent's text in this property is still not an
//!  instruction.
//! Why:
//!  The account owns this property and may write anything,
//!  so the handler must be able to show that no value of it becomes a mount
//!  request
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(readSweepNonce(withNudge('0'.repeat(32)), clone).kind).toBe('nudge');
//! ```

use super::{NudgeReading, NudgeRefusal, parse_sweep_nonce, read_sweep_nonce, sweep_nonce_text};
use crate::worktree_clone_mount_protocol::clone_mount_property_names::{NONCE_HEX_DIGITS, SWEEP_NONCE_PROPERTY};
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;

/// The clone the property is read from.
const CLONE: &str = "zpcachyos/nosnap/clones/one";

/// The ancestor a value can be inherited from.
const PARENT: &str = "zpcachyos/nosnap/clones";

/// A property reply with one sweep-nonce value and source.
fn properties_with(value: &str, source: &str) -> NvList {
    let mut pair: NvList = NvList::new();
    pair.add_str("value", value);
    pair.add_str("source", source);
    let mut properties: NvList = NvList::new();
    properties.add_nvlist(SWEEP_NONCE_PROPERTY, pair);
    return properties;
}

/// A valid value is a nudge whose text is the nonce alone.
#[test]
fn a_local_nonce_is_a_nudge_and_carries_no_operation() {
    let digits: String = String::from("abcdef0123456789").repeat(2);
    let properties: NvList = properties_with(digits.as_str(), CLONE);
    let reading: NudgeReading = read_sweep_nonce(Some(&properties), CLONE);
    let NudgeReading::Nudge(nonce) = reading else {
        panic!("a local valid value is a nudge: {reading:?}");
    };
    assert_eq!(nonce.as_str(), digits);
    let text: String = sweep_nonce_text(&nonce);
    assert_eq!(text, digits);
    assert!(!text.contains("mount"), "a nudge names no operation: {text}");
    assert!(!text.contains(':'), "a nudge has no fields: {text}");
}

/// A value inherited from an ancestor nudges nothing.
#[test]
fn an_inherited_nonce_is_ignored() {
    let properties: NvList = properties_with("0".repeat(NONCE_HEX_DIGITS).as_str(), PARENT);
    assert_eq!(
        read_sweep_nonce(Some(&properties), CLONE),
        NudgeReading::Inherited { from: String::from(PARENT) }
    );
}

/// An intent's whole text written into this property is refused as a nudge,
/// and the refusal does not become a request.
#[test]
fn an_intent_text_in_the_nudge_property_is_refused() {
    let intent: String = format!("v1:{}:mount", "0".repeat(NONCE_HEX_DIGITS));
    let properties: NvList = properties_with(intent.as_str(), CLONE);
    let reading: NudgeReading = read_sweep_nonce(Some(&properties), CLONE);
    let NudgeReading::Refused(refusal) = reading else {
        panic!("the intent text is not a nudge: {reading:?}");
    };
    assert!(matches!(refusal, NudgeRefusal::Nonce(_)), "{refusal:?}");
    assert!(refusal.message().contains("never a mount instruction"), "{}", refusal.message());
}

/// A short value, an empty value and a value over the nonce's length are each
/// refused before any digit is read.
#[test]
fn every_other_length_is_refused() {
    for length in [0, 1, NONCE_HEX_DIGITS - 1, NONCE_HEX_DIGITS + 1, 64] {
        let text: String = "0".repeat(length);
        let refusal = parse_sweep_nonce(text.as_str()).expect_err("only one length is a nudge");
        let NudgeRefusal::Nonce(cause) = refusal else {
            panic!("the cause is the nonce");
        };
        assert_eq!(cause, crate::worktree_clone_mount_protocol::request_nonce::NonceRefusal::Length { found: length });
    }
}

/// A property that cannot be read at all is refused rather than silent.
#[test]
fn an_unreadable_property_is_refused() {
    let mut properties: NvList = NvList::new();
    properties.add_str(SWEEP_NONCE_PROPERTY, "0".repeat(NONCE_HEX_DIGITS).as_str());
    let reading: NudgeReading = read_sweep_nonce(Some(&properties), CLONE);
    assert!(matches!(reading, NudgeReading::Refused(NudgeRefusal::Property(_))), "{reading:?}");
    let nonce: RequestNonce = RequestNonce::parse("0".repeat(NONCE_HEX_DIGITS).as_str()).expect("the all-zero nonce is valid");
    assert_eq!(sweep_nonce_text(&nonce).len(), NONCE_HEX_DIGITS);
}
