//! What:
//!  Controls of the protocol's fixed names and its one bound:
//!  both property names,
//!  the version and operation words,
//!  and the byte bound the longest intent needs.
//! Why:
//!  A moved character in either property is a protocol that reads nothing,
//!  and a bound smaller than the longest accepted value would refuse a
//!  well-formed intent.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(MAXIMUM_INTENT_BYTES).toBe('v1:00000000000000000000000000000000:unmount'.length);
//! ```

use super::{
    DESIRED_STATE_PROPERTY,
    FIELD_SEPARATOR,
    MAXIMUM_INTENT_BYTES,
    MOUNT_WORD,
    NONCE_HEX_DIGITS,
    PROPERTY_SOURCE_KEY,
    PROPERTY_VALUE_KEY,
    SWEEP_NONCE_PROPERTY,
    UNMOUNT_WORD,
    VALUE_VERSION,
};

/// The two property names are the ones the design fixed,
/// and they sit in the route's own namespace.
#[test]
fn the_properties_are_the_designed_names() {
    assert_eq!(
        DESIRED_STATE_PROPERTY,
        "org.monochromatic:git-worktree-clone-mount-desired-state"
    );
    assert_eq!(
        SWEEP_NONCE_PROPERTY,
        "org.monochromatic:git-worktree-clone-mount-sweep-nonce"
    );
    assert!(DESIRED_STATE_PROPERTY.starts_with("org.monochromatic:"));
    assert!(SWEEP_NONCE_PROPERTY.starts_with("org.monochromatic:"));
}

/// The longest intent the grammar admits is exactly the bound,
/// and the mount form is shorter than it.
#[test]
fn the_bound_fits_the_longest_intent_and_no_more() {
    let nonce: String = "0".repeat(NONCE_HEX_DIGITS);
    let unmount: String =
        format!("{VALUE_VERSION}{FIELD_SEPARATOR}{nonce}{FIELD_SEPARATOR}{UNMOUNT_WORD}");
    let mount: String =
        format!("{VALUE_VERSION}{FIELD_SEPARATOR}{nonce}{FIELD_SEPARATOR}{MOUNT_WORD}");
    assert_eq!(unmount.len(), MAXIMUM_INTENT_BYTES);
    assert!(mount.len() < MAXIMUM_INTENT_BYTES);
}

/// The reply keys are OpenZFS's own two words,
/// which the kernel writes for each property.
#[test]
fn the_reply_keys_are_the_kernels() {
    assert_eq!(PROPERTY_VALUE_KEY, "value");
    assert_eq!(PROPERTY_SOURCE_KEY, "source");
}
