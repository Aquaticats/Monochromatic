//! Controls for the two clone-local properties:
//!  a written value reads back as the same identity,
//!  a value of another version, of the wrong field count, of a bad number or boot and one too long are refused,
//!  and the force word carries the nonce alone.

use crate::worktree_clone_forced_removal::caller_property::*;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

/// One identity a control writes and reads.
fn an_identity() -> CallerIdentity {
    return CallerIdentity {
        nonce: RequestNonce::parse("0123456789abcdef0123456789abcdef")
            .expect("32 lowercase hex digits"),
        pid: 4242,
        started: 8123,
        parent: 4000,
        parent_started: 8000,
        boot: String::from("6f0f1a2b-3c4d-4e5f-8a9b-0c1d2e3f4a5b"),
    };
}

/// A written value reads back as the identity it was written from.
#[test]
fn a_written_value_reads_back_as_the_same_identity() {
    let identity: CallerIdentity = an_identity();
    let value: String = caller_property_value(&identity);
    assert_eq!(
        value,
        "v1:0123456789abcdef0123456789abcdef:4242:8123:4000:8000:6f0f1a2b-3c4d-4e5f-8a9b-0c1d2e3f4a5b"
    );
    assert!(value.len() <= MAXIMUM_CALLER_PROPERTY_BYTES);
    assert_eq!(parse_caller_property(value.as_str()), Ok(identity));
}

/// Every fault has its own refusal,
/// and a value longer than the grammar admits is refused before it is split.
#[test]
fn each_fault_is_its_own_refusal() {
    assert!(matches!(
        parse_caller_property("v2:1:2:3:4:5:x"),
        Err(CallerPropertyRefusal::Version { .. })
    ));
    assert_eq!(
        parse_caller_property("v1:0123456789abcdef0123456789abcdef:4242"),
        Err(CallerPropertyRefusal::Fields {
            found: 3
        })
    );
    assert!(matches!(
        parse_caller_property(
            "v1:0123456789abcdef0123456789abcdef:pid:8123:4000:8000:6f0f1a2b-3c4d-4e5f-8a9b-0c1d2e3f4a5b"
        ),
        Err(CallerPropertyRefusal::Number {
            field: "pid"
        })
    ));
    assert!(matches!(
        parse_caller_property("v1:0123456789abcdef0123456789abcdef:4242:8123:4000:8000:not-a-boot"),
        Err(CallerPropertyRefusal::Boot)
    ));
    assert!(matches!(
        parse_caller_property(
            "v1:not-a-nonce:4242:8123:4000:8000:6f0f1a2b-3c4d-4e5f-8a9b-0c1d2e3f4a5b"
        ),
        Err(CallerPropertyRefusal::Nonce(_))
    ));
    let too_long: String = "v".repeat(MAXIMUM_CALLER_PROPERTY_BYTES + 1);
    assert_eq!(
        parse_caller_property(too_long.as_str()),
        Err(CallerPropertyRefusal::TooLong {
            found: MAXIMUM_CALLER_PROPERTY_BYTES + 1
        })
    );
}

/// The force word is the version word and the nonce,
/// and it reads back as the same nonce.
#[test]
fn the_force_word_carries_the_nonce_alone() {
    let nonce: RequestNonce =
        RequestNonce::parse("fedcba9876543210fedcba9876543210").expect("a nonce");
    let value: String = force_property_value(&nonce);
    assert_eq!(value, "v1:fedcba9876543210fedcba9876543210");
    assert_eq!(parse_force_property(value.as_str()), Ok(nonce));
    assert!(matches!(
        parse_force_property("v1:one:two"),
        Err(ForceWordRefusal::Fields {
            found: 3
        })
    ));
    assert!(matches!(
        parse_force_property("v1:fedcba9876543210fedcba9876543210:extra"),
        Err(ForceWordRefusal::TooLong { .. })
    ));
    assert!(matches!(
        parse_force_property("v2:fedcba9876543210fedcba9876543210"),
        Err(ForceWordRefusal::Version { .. })
    ));
    assert!(matches!(
        parse_force_property("v1:not-a-nonce"),
        Err(ForceWordRefusal::Nonce(_))
    ));
    let too_long: String = "v".repeat(MAXIMUM_FORCE_PROPERTY_BYTES + 1);
    assert!(matches!(
        parse_force_property(too_long.as_str()),
        Err(ForceWordRefusal::TooLong { .. })
    ));
}

/// The boot shape is the kernel's own,
/// so a text of another shape proves nothing.
#[test]
fn the_boot_shape_is_the_kernels() {
    assert!(is_boot_id("6f0f1a2b-3c4d-4e5f-8a9b-0c1d2e3f4a5b"));
    assert!(!is_boot_id("6f0f1a2b3c4d4e5f8a9b0c1d2e3f4a5b"));
    assert!(!is_boot_id("6F0F1A2B-3C4D-4E5F-8A9B-0C1D2E3F4A5B"));
    assert!(!is_boot_id("6f0f1a2b-3c4d-4e5f-8a9b-0c1d2e3f4a5"));
    assert!(!is_boot_id(""));
    assert_eq!(BOOT_ID_BYTES, 36);
}
