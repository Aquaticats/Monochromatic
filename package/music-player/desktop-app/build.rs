//! Compiles the Slint UI markup into generated Rust at build time,
//!  writes the license notices the binary embeds (`src/licenses.rs`),
//!  and on
//! Windows embeds the application icon into the .exe.

/// What:
///     The text a build embeds when no notice file was given.
/// Why:
///      Development builds must not need cargo-about;
/// the text says how to get the real notices.
const NOTICES_PLACEHOLDER: &str = "This build of Music Player carries no third-party license notices.
Release builds embed them: the release workflow writes them with
`mise run //package/music-player/desktop-app:notices` and builds with
MUSIC_PLAYER_THIRD_PARTY_LICENSES set to that file (target/THIRD-PARTY-LICENSES.txt).
Every release also ships them as a separate file next to the programs.
";

/// What:
///     Writes `OUT_DIR/third_party_licenses.txt`:
/// a copy of the file named by
///           `MUSIC_PLAYER_THIRD_PARTY_LICENSES`,
/// or the placeholder when the variable is unset or empty.
///           With `MUSIC_PLAYER_REQUIRE_THIRD_PARTY_LICENSES=true` a missing variable stops the build.
/// Why:
///      The release binaries must carry the notices (the single Windows executable has nothing beside it),
///           while a development build stays free of the cargo-about step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = process.env.MUSIC_PLAYER_THIRD_PARTY_LICENSES;
/// if (!source && process.env.MUSIC_PLAYER_REQUIRE_THIRD_PARTY_LICENSES === "true") throw new Error("...");
/// writeFile(`${OUT_DIR}/third_party_licenses.txt`, source ? readFile(source) : PLACEHOLDER);
/// ```
fn write_license_notices() {
    println!("cargo:rerun-if-env-changed=MUSIC_PLAYER_THIRD_PARTY_LICENSES");
    println!("cargo:rerun-if-env-changed=MUSIC_PLAYER_REQUIRE_THIRD_PARTY_LICENSES");
    let out_dir = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR for build scripts"));
    let destination = out_dir.join("third_party_licenses.txt");
    let required = std::env::var("MUSIC_PLAYER_REQUIRE_THIRD_PARTY_LICENSES").as_deref() == Ok("true");
    let source = std::env::var_os("MUSIC_PLAYER_THIRD_PARTY_LICENSES").unwrap_or_default();
    if source.is_empty() {
        assert!(
            !required,
            "MUSIC_PLAYER_REQUIRE_THIRD_PARTY_LICENSES is true, but MUSIC_PLAYER_THIRD_PARTY_LICENSES names no notice \
             file. Write it with `mise run //package/music-player/desktop-app:notices` and set \
             MUSIC_PLAYER_THIRD_PARTY_LICENSES to the absolute path of target/THIRD-PARTY-LICENSES.txt, or unset \
             MUSIC_PLAYER_REQUIRE_THIRD_PARTY_LICENSES for a development build without notices."
        );
        std::fs::write(&destination, NOTICES_PLACEHOLDER).expect("writing the placeholder notices into OUT_DIR");
        return;
    }
    let source_path = std::path::PathBuf::from(source);
    println!("cargo:rerun-if-changed={}", source_path.display());
    let text = match std::fs::read_to_string(&source_path) {
        Ok(text) => text,
        Err(error) => panic!(
            "MUSIC_PLAYER_THIRD_PARTY_LICENSES names {}, which cannot be read as UTF-8 text: {error}. Write it with \
             `mise run //package/music-player/desktop-app:notices`.",
            source_path.display()
        ),
    };
    // A notice file always lists Rust crates; anything else is the wrong file.
    assert!(
        text.contains("Rust crates under"),
        "{} lists no Rust crate, so it is not a notice file written by the notices task.",
        source_path.display()
    );
    std::fs::write(&destination, text).expect("writing the license notices into OUT_DIR");
}

fn main() {
    slint_build::compile("ui/app.slint").expect("Slint UI compilation failed");

    // What:     Write the license notices `src/licenses.rs` embeds.
    // Why:      See `write_license_notices`.
    write_license_notices();

    // What:     `#[cfg(windows)]` is a conditional-compilation attribute. It
    //           tells the Rust compiler to keep the statement directly below it
    //           ONLY when this build script is compiled for a Windows host; on
    //           Linux and macOS the statement is physically removed before
    //           compilation, so nothing runs there. This tests the HOST that
    //           runs the build script, not the target. The shipped Windows
    //           binaries are built natively (host == target), so the icon is
    //           embedded. A cross-compile of the GNU target from Linux (the
    //           `build:windows-gnu` mise task in a container) is a proof of the
    //           toolchain only: there the host is Linux, this statement is
    //           removed, and the resulting .exe carries no icon.
    // Why:      Embedding an icon resource into an executable is a Windows-only
    //           concept. The macOS icon lives in the .app bundle and Linux uses
    //           the .desktop file's `Icon=` name, so neither needs this.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (process.platform === "win32") embedWindowsIcon();
    // ```
    #[cfg(windows)]
    embed_windows_icon();
}

// What:     A plain function declaration named `embed_windows_icon`, taking no
//           arguments and returning nothing (Rust's `()`, i.e. void). The
//           `#[cfg(windows)]` attribute means the whole function only exists in a
//           Windows build, mirroring its call site; non-Windows builds never see
//           it and never pull in the winresource crate (itself gated to
//           cfg(windows) in Cargo.toml).
// Why:      Keeps the Windows-only logic out of `main` and out of every other
//           platform's compile entirely.
//
// In TS you'd write (pseudocode):
// ```ts
// function embedWindowsIcon(): void { ... }
// ```
#[cfg(windows)]
fn embed_windows_icon() {
    // What:     `winresource::WindowsResource::new()` calls the associated
    //           function `new` on the `WindowsResource` type from the
    //           `winresource` crate, producing a fresh builder value.
    //           `let mut resource` binds it to a MUTABLE local (`mut` is needed
    //           because the next line changes it; a bare `let` would be
    //           read-only).
    // Why:      The builder accumulates what to embed (here, the icon) and then
    //           compiles it into a `.res` object file the linker bakes into the
    //           .exe.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const resource = new WindowsResource();
    // ```
    let mut resource = winresource::WindowsResource::new();

    // What:     `.set_icon(...)` is a method on the builder recording the path to
    //           the .ico to embed. The argument is the committed icon produced by
    //           the `gen:icons` mise task.
    // Why:      Tells winresource which icon the .exe should carry, shown by
    //           Explorer, the taskbar, and Alt-Tab on Windows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // resource.setIcon("asset/music-player.ico");
    // ```
    resource.set_icon("asset/music-player.ico");

    // What:     `.compile()` runs the Windows resource compiler (rc.exe from the
    //           Windows SDK for the MSVC targets, `windres` from MinGW-w64 binutils
    //           for the GNU target; winresource picks by the target environment) to turn the recorded resources into
    //           a `.res` file and tells Cargo to link it into the binary. It
    //           returns a `Result` (Rust's ok-or-error value); `.expect("...")`
    //           unwraps the success case and PANICS with the given message if it
    //           is an error, failing the build loudly.
    // Why:      A missing or broken resource compiler should stop the build with a
    //           clear message rather than silently shipping an icon-less .exe.
    // Gotcha:   `.expect` on an error aborts the whole build script; there is no
    //           silent fallback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // resource.compile(); // throws on failure
    // ```
    resource
        .compile()
        .expect("embedding the Windows icon into the .exe failed");
}
