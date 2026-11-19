//! Compiles the picker markup into Rust bindings before Cargo builds the app.

/// What: Cargo runs this entry point before compiling the app.
/// Why: The app needs typed access to the picker window and its callbacks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function main(): void { compileSlint("ui/picker.slint"); }
/// ```
fn main() {
    // What: Compile the markup and stop the build if the compiler returns an error.
    // Why: The app requires a valid picker for both launches and diagnostics.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // compileSlint("ui/picker.slint"); // throws on failure
    // ```
    slint_build::compile("ui/picker.slint").expect("compile browser picker");
}
