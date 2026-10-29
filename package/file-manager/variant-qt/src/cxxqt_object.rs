//! What:
//!  the root Rust QObject bridged to QML for the Qt file-manager.
//! Why:
//!  cxx-qt exposes this as a QML-instantiable type.
//!  This minimal version proves the
//!      Rust <-> QML pipeline (one property,
//!  one invokable) and is the seam the directory
//!      model,
//!  inbound-drop handler,
//!  and outbound-drag source grow onto next.

/// What:
///  cxx-qt bridge declaring the `AppBridge` QObject and its QML-visible surface.
/// Why:
///  the `#[cxx_qt::bridge]` macro turns these declarations into a C++ QObject subclass
///      registered with the QML type system under the module URI from build.rs.
#[cxx_qt::bridge]
pub mod qobject {
    // Import cxx-qt-lib's QString: QML strings cross the boundary as QString, and this extern
    // block makes the type and its C++ header available to the generated shim. cxx-qt forbids
    // doc comments on an extern block itself (only namespace / auto_cxx_name / auto_rust_name
    // are allowed there), so this is a plain comment; the documented item is the alias inside.
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");

        /// What:
        ///  alias to cxx-qt-lib's QString.
        /// Why:
        ///  keeps the property and invokable signatures below readable.
        type QString = cxx_qt_lib::QString;
    }

    // The Rust-backed QObject and the methods QML may call on it. cxx-qt forbids doc comments
    // on the extern block itself, so the per-item docs below carry the explanation.
    extern "RustQt" {
        /// What:
        ///  QML-exposed QObject backed by `AppBridgeRust`.
        /// Why:
        ///  `#[qml_element]` registers it in the module so QML can instantiate it;
        ///  the
        ///      `greeting` QString property proves property binding round-trips to QML.
        #[qobject]
        #[qml_element]
        #[qproperty(QString, greeting)]
        #[namespace = "file_manager"]
        type AppBridge = super::AppBridgeRust;

        /// What:
        ///  receive a message from QML and log it in Rust.
        /// Why:
        ///  minimal end-to-end proof of the invokable bridge;
        ///  grows into real actions
        ///      (open,
        ///  navigate,
        ///  accept-drop) as the app fills in.
        #[qinvokable]
        #[cxx_name = "logFromQml"]
        fn log_from_qml(&self, message: &QString);
    }
}

use cxx_qt_lib::QString;

/// What:
///  backing Rust state for the `AppBridge` QObject.
/// Why:
///  holds the values cxx-qt exposes to QML as Q_PROPERTYs.
pub struct AppBridgeRust {
    /// What:
    ///  greeting string shown by the placeholder UI.
    /// Why:
    ///  demonstrates a QString Q_PROPERTY seeded from Rust and rendered in QML.
    greeting: QString,
}

/// What:
///  seed the initial `AppBridge` state.
/// Why:
///  gives QML a non-empty greeting binding without hardcoding it QML-side.
impl Default for AppBridgeRust {
    fn default() -> Self {
        return Self {
            greeting: QString::from("Monochromatic file manager (Qt / cxx-qt)"),
        }
    }
}

/// What:
///  the invokable behavior of the `AppBridge` QObject.
/// Why:
///  methods here are callable from QML through the bridge declared above.
impl qobject::AppBridge {
    /// What:
    ///  log a QML-provided message through the non-blocking tracing pipeline.
    /// Why:
    ///  proves a QML->Rust invokable call succeeds,
    ///  and routes it through the same
    ///      off-thread logging the whole app uses,
    ///  so it never blocks a frame.
    pub fn log_from_qml(&self, message: &QString) {
        tracing::info!(target: "app_bridge", message = %message, "logFromQml invoked from QML");
    }
}

/// Verifies Rust-owned state exposed through generated Qt bindings.
#[cfg(test)]
#[path = "cxxqt_object_tests.rs"]
mod tests;
