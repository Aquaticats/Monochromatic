//! Unit tests for the accessibility answer formatting.

use super::{format_tree, role_name, state_names, Node};

/// The named bits decode from the two-word state vector.
#[test]
fn state_bits_decode_across_words() {
    assert_eq!(state_names(&[0, 0]), Vec::<&str>::new());
    assert_eq!(state_names(&[(1 << 11) | (1 << 12), 0]), vec!["focusable", "focused"]);
    assert_eq!(state_names(&[(1 << 8) | (1 << 30), 0]), vec!["enabled", "visible"]);
    assert_eq!(state_names(&[0, 1 << 1]), Vec::<&str>::new());
    assert_eq!(state_names(&[1 << 4]), vec!["checked"]);
}

/// A node record carries depth,
///  role,
///  name,
///  states,
///  and bounds,
///  with text percent-encoded.
#[test]
fn records_encode_text_and_carry_bounds() {
    let nodes = [
        Node { depth: 0, role: "application".to_string(), name: "My App".to_string(), states: vec![], bounds: None },
        Node { depth: 1, role: "push button".to_string(), name: "Play; now".to_string(), states: vec!["enabled", "focusable"], bounds: Some((10, 20, 30, 40)) },
    ];
    assert_eq!(
        format_tree(&nodes),
        "2 depth=0 role=application name=My%20App states= bounds=; \
         depth=1 role=push%20button name=Play%3B%20now states=enabled,focusable bounds=10,20,30,40",
    );
    assert_eq!(format_tree(&[]), "0");
}

/// Role numbers map to the AT-SPI role names,
///  and an unknown number still prints.
#[test]
fn roles_have_names() {
    assert_eq!(role_name(0), "invalid");
    assert_eq!(role_name(7), "check box");
    assert_eq!(role_name(23), "frame");
    assert_eq!(role_name(29), "label");
    assert_eq!(role_name(43), "push button");
    assert_eq!(role_name(61), "text");
    assert_eq!(role_name(75), "application");
    assert_eq!(role_name(79), "entry");
    assert_eq!(role_name(9999), "role-9999");
}
