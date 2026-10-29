//! Binary entry point:
//!  a thin wrapper over the `file_manager` library `run`.

/// What:
///  imports the library crate's `run` entry point.
/// Why:
///  all startup and UI logic lives in the library so the domain modules unit-test without
///      the binary;
///  the bin exists only to hand the process to `run`.
use file_manager::run;

/// What:
///  process entry point;
///  delegate to the library and return its GTK exit code.
/// Why:
///  keeps the binary minimal (the library owns startup);
///  GTK's `ExitCode` propagates out.
fn main() -> gtk4::glib::ExitCode {
    return run()
}
