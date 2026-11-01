//! Verifies Rust-owned state exposed through generated Qt bindings.

use super::{AppBridgeRust, QString};

/// Ensures default construction preserves QML's visible initial state.
#[test]
fn default_state_carries_the_qml_greeting() {
    let bridge = AppBridgeRust::default();

    assert_eq!(
        bridge.greeting,
        QString::from("Monochromatic file manager (Qt / cxx-qt)"),
    );
}
