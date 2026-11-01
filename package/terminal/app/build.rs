//! Compiles the Slint terminal UI into generated Rust code.

// What:     `fn main()` is Cargo's build-script entry point. Cargo runs this
//           function before compiling the crate itself.
// Why:      The app needs generated Rust types for `ui/app.slint` before
//           `src/main.rs` can use `slint::include_modules!()`.
//
// In TS you'd write (pseudocode):
// ```ts
// generateUiBindings("ui/app.slint");
// ```
fn main() {
    // What:     `println!("cargo:rustc-link-arg-bin=...")` sends a Cargo
    //           build-script instruction to stdout. Cargo reads this line and
    //           passes `-Wl,-rpath,$ORIGIN/../lib/monochromatic-terminal` only
    //           to the `monochromatic-terminal` binary link step.
    // Why:      The crates.io `libghostty-vt-sys` package links Ghostty's VT core
    //           as `libghostty-vt.so.0`, so the installed binary needs a stable
    //           relative library lookup path beside `~/.local/bin`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // bundler.addRuntimeLibraryPath("$ORIGIN/../lib/monochromatic-terminal");
    // ```
    println!(
        "cargo:rustc-link-arg-bin=monochromatic-terminal=-Wl,-rpath,$ORIGIN/../lib/monochromatic-terminal",
    );
    // What:     `slint_build::compile(...)` invokes Slint's Rust build helper,
    //           and `.expect(...)` unwraps the success value or panics with the
    //           message when the markup compiler returns an error.
    // Why:      A broken `.slint` file should stop the build immediately.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // compileSlint("ui/app.slint");
    // ```
    slint_build::compile("ui/app.slint").expect("Slint UI compilation failed");
}
