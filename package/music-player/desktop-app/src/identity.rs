//! Single source of truth for the application's platform identity strings.
//!
//! The app is named in three different identity systems,
//!  and the strings MUST
//! NOT drift,
//!  so they live here instead of scattered across modules:
//!
//! - Linux:
//!    `APP_ID` (`monochromatic.music-player`) is the Wayland app id,
//!    the
//!   X11 `WM_CLASS`,
//!    and the `<APP_ID>.desktop` basename.
//!    KDE matches the
//!   running window to the launcher entry by it,
//!    and
//!   `share/applications/monochromatic.music-player.desktop` is named after it,
//!   so changing it is a breaking change.
//!    Read only on Linux (launcher.rs).
//! - macOS:
//!    the bundle id is `MACOS_BUNDLE_IDENTIFIER`
//!   (`dev.monochromatic.musicplayer`).
//!    It actually lives in `macos/Info.plist`
//!   (XML cannot share a Rust constant);
//!    it is mirrored here so the value has
//!   one documented home and a test can assert the plist matches.
//! - All platforms:
//!    the per-user config directory is derived from the
//!   reverse-DNS triple (`CONFIG_QUALIFIER`,
//!    `CONFIG_ORGANIZATION`,
//!   `CONFIG_APPLICATION`) by the `directories` crate (session.rs).
//!    Changing any
//!   of the three moves every existing user's `session.json`,
//!    so they are fixed.
//!
//! The macOS bundle id and the config triple now unify to
//! `dev.monochromatic.musicplayer`;
//!  only the Wayland `APP_ID`
//! (`monochromatic.music-player`) differs,
//!  because it is the `.desktop` basename
//! and KDE `WM_CLASS`,
//!  where renaming is a breaking change.
//!  Collecting them here
//! makes that one intentional difference visible rather than looking like drift.

/// What:
///      `use std::path::PathBuf;` imports the OWNED filesystem-path type (heap-
///           allocated,
///  growable;
///  sibling:
///  the borrowed `&Path`).
/// Why:
///       `config_dir` below returns an owned `PathBuf` the callers extend with a
///           filename.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PathBuf = string;
/// ```
use std::path::PathBuf;

/// What:
///      `pub const APP_ID: &str = "monochromatic.music-player";`.
///  A module-
///           level immutable string constant.
///  `&str` is a borrowed string slice
///           (sibling:
///  the owned,
///  heap-allocated `String`);
///  a `const` `&str`
///           points at bytes baked into the binary,
///  so no allocation happens.
/// Why:
///       One name for the Wayland app id / WM_CLASS / `.desktop` basename,
///  so
///           the window,
///  the desktop file,
///  and KDE's launcher matching can never
///           disagree.
///  Declared on every platform (only Linux reads it);
///  a `pub`
///           constant unused inside this library is NOT dead code,
///  so non-Linux
///           builds stay warning-clean without a `#[cfg]`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const APP_ID = "monochromatic.music-player";
/// ```
pub const APP_ID: &str = "monochromatic.music-player";

/// What:
///      `pub const MACOS_BUNDLE_IDENTIFIER: &str = "dev.monochromatic.musicplayer";`.
///           Another `const` string slice (see `APP_ID` for the `&str` vs `String`
///           note).
/// Why:
///       The macOS `CFBundleIdentifier` is authored in `macos/Info.plist`,
///  but
///           recording it here gives the value a single documented home and lets a
///           unit test assert the plist still contains it,
///  catching silent drift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const MACOS_BUNDLE_IDENTIFIER = "dev.monochromatic.musicplayer";
/// ```
pub const MACOS_BUNDLE_IDENTIFIER: &str = "dev.monochromatic.musicplayer";

/// What:
///      `pub const CONFIG_QUALIFIER: &str = "dev";`.
///  The first of the three
///           reverse-DNS parts the `directories` crate's `ProjectDirs::from`
///           takes (qualifier,
///  organization,
///  application).
/// Why:
///       Pulling the literal out of session.rs means the config-dir identity
///           is named in one place next to the other identifiers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const CONFIG_QUALIFIER = "dev";
/// ```
pub const CONFIG_QUALIFIER: &str = "dev";

/// What:
///      `pub const CONFIG_ORGANIZATION: &str = "monochromatic";`.
///  The second
///           `ProjectDirs::from` part (the organization name).
/// Why:
///       Same single-source-of-truth reason as `CONFIG_QUALIFIER`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const CONFIG_ORGANIZATION = "monochromatic";
/// ```
pub const CONFIG_ORGANIZATION: &str = "monochromatic";

/// What:
///      `pub const CONFIG_APPLICATION: &str = "musicplayer";`.
///  The third
///           `ProjectDirs::from` part (the application name);
///  also the last
///           segment of the config directory path.
/// Why:
///       Same single-source-of-truth reason as `CONFIG_QUALIFIER`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const CONFIG_APPLICATION = "musicplayer";
/// ```
pub const CONFIG_APPLICATION: &str = "musicplayer";

/// What:
///      `pub(crate) fn config_dir() -> Option<PathBuf>`.
///  Resolve the per-user
///           config DIRECTORY (no filename) from the reverse-DNS triple via the
///           `directories` crate,
///  or `None` when the platform exposes no config home.
///           `pub(crate)` so session.rs and peakcache.rs share it.
/// Why:
///       The session file and the peak cache both live in this directory;
///  deriving
///           it once here keeps their parent path from drifting and removes the
///           duplicated `ProjectDirs::from` triple from both modules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function configDir(): string | null {
///   const dirs = projectDirs(CONFIG_QUALIFIER, CONFIG_ORGANIZATION, CONFIG_APPLICATION);
///   return dirs ? dirs.configDir : null;
/// }
/// ```
pub(crate) fn config_dir() -> Option<PathBuf> {
    // What:     `directories::ProjectDirs::from(CONFIG_QUALIFIER, CONFIG_ORGANIZATION,
    //           CONFIG_APPLICATION)`. Ask the `directories` crate for the standard
    //           per-app directories from this module's own reverse-DNS constants;
    //           returns `Option<ProjectDirs>`.
    // Why:      Respect the platform's config-dir convention from the single identity
    //           source.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const dirs = projectDirs(CONFIG_QUALIFIER, CONFIG_ORGANIZATION, CONFIG_APPLICATION);
    // ```
    return directories::ProjectDirs::from(CONFIG_QUALIFIER, CONFIG_ORGANIZATION, CONFIG_APPLICATION)
        // What:     `.map(|dirs| dirs.config_dir().to_path_buf())`. Runs only on `Some`.
        //           `dirs.config_dir()` borrows a `&Path` from the temporary `dirs`, so
        //           `.to_path_buf()` copies it into an owned `PathBuf` before `dirs`
        //           drops. Tail expression -> return.
        // Why:      Hand back an owned directory the callers can `.join(<filename>)`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return dirs ? dirs.configDir : null;
        // ```
        .map(|dirs| return dirs.config_dir().to_path_buf())
}

/// What:
///      `#[cfg(test)] #[path = "identity_tests.rs"] mod tests;`.
///  Declare the
///           test submodule,
///  sourced from the sibling `identity_tests.rs` file.
///           `#[cfg(test)]` compiles it only for test builds;
///  `#[path = "..."]`
///           points the module at the flat sibling file instead of the default
///           `identity/tests.rs` lookup.
/// Why:
///       Keep this file to the constants while the drift-guard tests live beside
///           it (sibling `*_tests.rs` files are exempt from the max-lines linter),
///           matching the convention in session.rs / peakcache.rs / etc.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // identity.unit.test.ts beside identity.ts; the test runner picks it up
/// ```
#[cfg(test)]
#[path = "identity_tests.rs"]
mod tests;
