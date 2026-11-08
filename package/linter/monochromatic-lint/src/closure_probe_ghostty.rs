//! What:
//!  The prerequisite that hands `libghostty-vt-sys` its offline inputs:
//!  the Ghostty source tree and the Zig package directory that the terminal's fetch task prepared.
//! Why:
//!  The crate's build script otherwise clones Ghostty and lets `zig build` download Zig packages while Cargo compiles,
//!  which `cargo fetch` cannot prefetch and which no checksum of this repository covers.
//!  The check reads only what the fetch task verified and left under `target`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function ghosttySetup(directory: Path): [string, string][]; // throws string (reason to skip)
//! ```

/// Import native paths.
use std::path::{Path, PathBuf};

/// What:
///  The variable by which `libghostty-vt-sys` takes a prepared Ghostty source tree instead of cloning one.
/// Why:
///  Its build script reads exactly this name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GHOSTTY_SOURCE_VARIABLE = 'GHOSTTY_SOURCE_DIR';
/// ```
pub const GHOSTTY_SOURCE_VARIABLE: &str = "GHOSTTY_SOURCE_DIR";

/// What:
///  The variable by which `libghostty-vt-sys` hands Zig a directory of prepared packages and forbids fetching.
/// Why:
///  Its build script passes the directory as `zig build --system`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GHOSTTY_ZIG_SYSTEM_VARIABLE = 'GHOSTTY_ZIG_SYSTEM_DIR';
/// ```
pub const GHOSTTY_ZIG_SYSTEM_VARIABLE: &str = "GHOSTTY_ZIG_SYSTEM_DIR";

/// What:
///  Prepare the libghostty build:
///  require the fetched inputs and name them in the two variables the crate reads.
/// Why:
///  Fetching needs the network and a checksum step,
///  which a linter does not start,
///  so absent or unfinished inputs skip the package and name the task that makes them.
///  The ready marker is written last by that task,
///  so a fetch that stopped halfway is never taken for a finished one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ghosttySetup(directory: Path): [string, string][]; // throws string
/// ```
pub fn ghostty_setup(directory: &Path) -> Result<Vec<(String, String)>, String> {
    let root: PathBuf = directory.join("target").join("ghostty-offline");
    let marker: PathBuf = root.join("ready");
    let source: PathBuf = root.join("source");
    let packages: PathBuf = root.join("packages");
    if !marker.is_file() || !source.join("build.zig").is_file() || !packages.is_dir() {
        return Err(format!(
            "the offline inputs of libghostty-vt-sys are not prepared ({} must exist and hold a Ghostty source tree with build.zig and the Zig packages); run `mise run //package/terminal/app:fetch:ghostty-offline-inputs` first",
            marker.display()
        ));
    }
    return Ok(vec![
        (
            String::from(GHOSTTY_SOURCE_VARIABLE),
            source.to_string_lossy().into_owned(),
        ),
        (
            String::from(GHOSTTY_ZIG_SYSTEM_VARIABLE),
            packages.to_string_lossy().into_owned(),
        ),
    ]);
}

/// Offline-input preparation controls stay outside release artifacts.
#[cfg(test)]
#[path = "closure_probe_ghostty_tests.rs"]
mod tests;
