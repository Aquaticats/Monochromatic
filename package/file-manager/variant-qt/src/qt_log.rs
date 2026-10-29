//! Route Qt's own logging into the non-blocking tracing sink.
//!
//! Qt's default message handler is synchronous on the emitting thread.
//!  The C++ shim in
//! `qt_log.cpp` installs a `qInstallMessageHandler` that forwards every Qt message here,
//!  where
//! it becomes a tracing event and rides the same off-thread NonBlocking writer as the rest of
//! the app,
//!  so Qt-internal logs cannot block the UI/render thread.

/// Imports the C string and integer FFI types used by the Qt-to-tracing callback.
use std::ffi::{CStr, c_char, c_int};

// C++ shim in qt_log.cpp; extern block itself carries no docs (cxx/rustc forbid it).
unsafe extern "C" {
    /// Installs the process-wide Qt message handler that forwards into `fmqt_forward_log`.
    fn fmqt_install_qt_message_handler();
}

/// What:
///  install the Qt-to-tracing message-handler bridge.
/// Why:
///  call once at startup,
///  after the tracing subscriber is set up,
///  so Qt's own
///      qDebug/qCDebug/qWarning flow off-thread through tracing instead of Qt's synchronous
///      default handler.
pub fn install() {
    // SAFETY: registers a process-wide Qt handler; the C++ side only passes borrowed,
    // nul-terminated C strings into `fmqt_forward_log` below.
    unsafe { fmqt_install_qt_message_handler() };
}

/// What:
///  C-ABI callback the Qt handler invokes for each Qt log message.
/// Why:
///  maps QtMsgType (0 debug,
///  1 warning,
///  2 critical,
///  3 fatal,
///  4 info) to a tracing level
///      and emits the event under target "qt" with the Qt logging category as a field.
///
/// # Safety
/// `category_ptr` and `message_ptr` must each be null or a valid nul-terminated C string that
/// outlives this call.
///  The Qt handler passes borrowed `QByteArray::constData` and the Qt
/// category pointer,
///  which satisfy this.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fmqt_forward_log(
    kind: c_int,
    category_ptr: *const c_char,
    message_ptr: *const c_char,
) {
    // SAFETY: the C++ handler passes nul-terminated UTF-8 (from QByteArray::constData) and the
    // Qt category pointer or a fallback literal; both outlive this synchronous call.
    // The decoded strings take the plain names; the pointer parameters keep `_ptr` so
    // neither decoded binding shadows its input.
    let category = if category_ptr.is_null() {
        "qt"
    } else {
        unsafe { CStr::from_ptr(category_ptr) }.to_str().unwrap_or("qt")
    };
    let message = if message_ptr.is_null() {
        ""
    } else {
        unsafe { CStr::from_ptr(message_ptr) }
            .to_str()
            .unwrap_or("<non-utf8 qt message>")
    };
    match kind {
        0 => tracing::debug!(target: "qt", category, "{message}"),
        4 => tracing::info!(target: "qt", category, "{message}"),
        1 => tracing::warn!(target: "qt", category, "{message}"),
        _ => tracing::error!(target: "qt", category, "{message}"),
    }
}
