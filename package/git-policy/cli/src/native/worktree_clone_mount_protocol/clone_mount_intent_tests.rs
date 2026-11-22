//! What:
//!  Controls of the durable mount intent:
//!  the two accepted values,
//!  the malformed,
//!  oversized and inherited cases,
//!  the byte bound before parsing,
//!  and that every refusal names what was read and authorizes nothing.
//! Why:
//!  The design accepts exactly `v1:<32 hex digits>:mount` or
//!  `:unmount`,
//!  ignores inherited intent,
//!  and rejects a malformed intent before any filesystem operation on its
//!  behalf
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!  The property reply is the nested pair OpenZFS writes for each property,
//!  `value` and `source`,
//!  and a local property's source is the dataset's own name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(parseMountIntent('v1:00000000000000000000000000000000:mount')).toEqual({ operation: 'mount' });
//! ```

use super::{
    IntentReading, IntentRefusal, MAXIMUM_INTENT_BYTES, MountIntent, MountOperation, RequestNonce, intent_text, parse_mount_intent,
    read_intent,
};
use crate::worktree_clone_mount_protocol::clone_mount_property_names::{DESIRED_STATE_PROPERTY, NONCE_HEX_DIGITS};
use crate::worktree_clone_mount_protocol::local_user_property::PropertyRefusal;
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;

/// The clone's name every local property is set on.
const CLONE: &str = "zpcachyos/nosnap/home-user-Monochromatic/git-worktrees/one";

/// The parent the clone inherits a property from.
const PARENT: &str = "zpcachyos/nosnap/home-user-Monochromatic/git-worktrees";

/// What:
///  A property reply in the shape the kernel writes it:
///  the property name mapped to a nested list with `value` and `source`.
/// Why:
///   The intent reader must judge exactly this shape,
///  and a control that handed it a bare string would test a reply the kernel
///  never sends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const properties = (name, value, source) => new NvList([[name, { value, source }]]);
/// ```
fn properties_with(value: &str, source: Option<&str>) -> NvList {
    let mut inner: NvList = NvList::new();
    inner.add_str("value", value);
    if let Some(word) = source {
        inner.add_str("source", word);
    }
    let mut properties: NvList = NvList::new();
    properties.add_nvlist(DESIRED_STATE_PROPERTY, inner);
    return properties;
}

/// A nonce of all zeros,
/// the value these controls spell out.
fn zeros() -> String {
    return "0".repeat(NONCE_HEX_DIGITS);
}

/// Both designed values parse into the operation they name.
#[test]
fn the_two_designed_values_parse() {
    let nonce: String = zeros();
    for (word, operation) in [("mount", MountOperation::Mount), ("unmount", MountOperation::Unmount)] {
        let text: String = format!("v1:{nonce}:{word}");
        let intent: MountIntent = parse_mount_intent(text.as_str()).expect("the designed value parses");
        assert_eq!(intent.operation, operation);
        assert_eq!(intent.nonce.as_str(), nonce);
        assert_eq!(intent_text(intent.operation, &intent.nonce), text);
    }
}

/// A nonce of the wrong length or alphabet is refused as a nonce.
#[test]
fn a_malformed_nonce_is_refused() {
    let short: String = format!("v1:{}:mount", "0".repeat(NONCE_HEX_DIGITS - 1));
    assert!(matches!(parse_mount_intent(short.as_str()), Err(IntentRefusal::Nonce(_))));
    let uppercase: String = format!("v1:{}:mount", "A".repeat(NONCE_HEX_DIGITS));
    let refusal: IntentRefusal = parse_mount_intent(uppercase.as_str()).expect_err("uppercase is another value");
    assert!(matches!(refusal, IntentRefusal::Nonce(_)), "{refusal:?}");
    assert!(refusal.message().contains("nonce"), "{}", refusal.message());
}

/// An unknown version or operation,
/// and a value with the wrong number of fields,
/// are refused.
#[test]
fn a_wrong_version_operation_or_field_count_is_refused() {
    let nonce: String = zeros();
    assert!(matches!(parse_mount_intent(format!("v2:{nonce}:mount").as_str()), Err(IntentRefusal::WrongVersion { .. })));
    assert!(matches!(parse_mount_intent(format!("1:{nonce}:mount").as_str()), Err(IntentRefusal::WrongVersion { .. })));
    assert!(matches!(parse_mount_intent(format!("v1:{nonce}:attach").as_str()), Err(IntentRefusal::UnknownOperation { .. })));
    assert!(matches!(parse_mount_intent(format!("v1:{nonce}::mount").as_str()), Err(IntentRefusal::FieldCount { .. })));
    assert!(matches!(parse_mount_intent("v1:mount"), Err(IntentRefusal::FieldCount { .. })));
    assert!(matches!(parse_mount_intent("mount"), Err(IntentRefusal::FieldCount { .. })));
    assert!(matches!(parse_mount_intent(""), Err(IntentRefusal::FieldCount { .. })));
}

/// A value longer than the bound is refused as too long before any field is
/// split,
/// even when its first bytes would parse.
#[test]
fn an_oversized_value_is_refused_before_parsing() {
    let nonce: String = zeros();
    let one_over: String = format!("v1:{nonce}:unmount!");
    assert_eq!(one_over.len(), MAXIMUM_INTENT_BYTES + 1);
    let refusal: IntentRefusal = parse_mount_intent(one_over.as_str()).expect_err("one byte over the bound is refused");
    assert_eq!(refusal, IntentRefusal::TooLong { bytes: one_over.len(), bound: MAXIMUM_INTENT_BYTES });
    let huge: String = format!("v1:{nonce}:mount{}", "x".repeat(1_000_000));
    let huge_refusal: IntentRefusal = parse_mount_intent(huge.as_str()).expect_err("a huge value is refused");
    assert_eq!(huge_refusal, IntentRefusal::TooLong { bytes: huge.len(), bound: MAXIMUM_INTENT_BYTES });
    assert!(huge_refusal.message().contains(&MAXIMUM_INTENT_BYTES.to_string()));
}

/// A property whose source is the clone itself is a request.
#[test]
fn local_intent_is_a_request() {
    let nonce: String = zeros();
    let properties: NvList = properties_with(format!("v1:{nonce}:mount").as_str(), Some(CLONE));
    let reading: IntentReading = read_intent(Some(&properties), CLONE);
    let IntentReading::Request(intent) = reading else {
        panic!("a local property asks for its operation: {reading:?}");
    };
    assert_eq!(intent.operation, MountOperation::Mount);
    assert_eq!(intent.nonce.as_str(), nonce);
}

/// A property whose source is an ancestor is ignored,
/// never turned into a request.
#[test]
fn inherited_intent_is_ignored() {
    let nonce: String = zeros();
    let properties: NvList = properties_with(format!("v1:{nonce}:mount").as_str(), Some(PARENT));
    let reading: IntentReading = read_intent(Some(&properties), CLONE);
    assert_eq!(reading, IntentReading::Inherited { from: String::from(PARENT) });
    assert!(!reading.asks_for_work(), "an inherited intent authorizes nothing");
}

/// A property with no value or no source is refused rather than guessed,
/// and an absent property is absent.
#[test]
fn a_missing_value_or_source_is_refused() {
    let with_value_only: NvList = properties_with("v1:00000000000000000000000000000000:mount", None);
    assert!(matches!(read_intent(Some(&with_value_only), CLONE), IntentReading::Refused(IntentRefusal::Property(PropertyRefusal::MissingSource))));
    let mut no_value: NvList = NvList::new();
    no_value.add_str(DESIRED_STATE_PROPERTY, "v1:00000000000000000000000000000000:mount");
    let refusal: IntentReading = read_intent(Some(&no_value), CLONE);
    assert!(matches!(refusal, IntentReading::Refused(IntentRefusal::Property(PropertyRefusal::NotAPropertyPair { .. }))));
    assert_eq!(read_intent(None, CLONE), IntentReading::Absent);
    assert_eq!(read_intent(Some(&NvList::new()), CLONE), IntentReading::Absent);
}

/// A malformed local value is refused with the property's own words,
/// so its refusal can be reported without an operation happening.
#[test]
fn a_malformed_local_value_is_refused() {
    let properties: NvList = properties_with("v1:nothex:mount", Some(CLONE));
    let reading: IntentReading = read_intent(Some(&properties), CLONE);
    let IntentReading::Refused(refusal) = reading else {
        panic!("a malformed value is refused: {reading:?}");
    };
    assert!(refusal.message().contains(DESIRED_STATE_PROPERTY), "{}", refusal.message());
    assert!(!IntentReading::Absent.asks_for_work());
}

/// A fresh nonce's text carries no path separator,
/// so a receipt names a clone by its GUID and the nonce only as data.
#[test]
fn a_nonce_never_reaches_a_file_name() {
    let nonce: RequestNonce = RequestNonce::parse(zeros().as_str()).expect("the all-zero nonce is valid");
    let text: String = intent_text(MountOperation::Unmount, &nonce);
    assert!(!text.contains('/'), "{text}");
    assert!(!text.contains('.'), "{text}");
}
