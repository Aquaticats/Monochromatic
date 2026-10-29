//! Thin binary wrapper over the library `run`.

/// What:
///  start the sticky-variant file manager and exit with GTK's code.
/// Why:
///  the bin stays a one-liner so the library remains the single entry point.
fn main() -> gtk4::glib::ExitCode {
    return file_manager_gtk_sticky::run()
}
