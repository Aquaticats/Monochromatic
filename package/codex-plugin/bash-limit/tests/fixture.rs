//! What:
//! The disposable fixture's own guarantees:
//! the shared scratch root created under an injected home is owner-only,
//! and the native hook works from a fixture built there.
//! Why:
//! `AGENTS.md` (rule ATH) requires `${HOME}/temp/agent` to be created and then made `0700`,
//! because trust checks reject group and other bits.
//! Every other test calls the fixture in the process's real home,
//! where the directory already exists,
//! so only a fresh home shows the mode it is created with.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // fixture.integration.test.ts
//! const home = mktemp({ dir: targetTmp });
//! const parent = fixtureIn(home);
//! assert.equal(statSync(join(home, 'temp/agent')).mode & 0o777, 0o700);
//! ```

/// Import the shared fixture and native process helpers.
mod support;

/// Import filesystem operations, the I/O error type and child processes.
use std::{fs, io, process::Command};

/// Import the permission bits (`mode()`) and the `0o700` owner-only value.
use std::os::unix::fs::PermissionsExt;

/// Import the owned path type.
use std::path::{Path, PathBuf};

/// What:
/// `const OWNER_ONLY_MODE: u32 = 0o700;` is the mode rule ATH requires for the shared scratch root:
/// read, write and search for the owner,
/// nothing for the group and nothing for anyone else.
/// Why:
/// The assertion and the diagnostic both read the one value,
/// so the test cannot drift from the requirement it checks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const OWNER_ONLY_MODE = 0o700;
/// ```
const OWNER_ONLY_MODE: u32 = 0o700;

/// What:
/// `fn fresh_home() -> io::Result<PathBuf>` declares a helper that makes an empty, unused home directory under
/// Cargo's per-target temporary directory.
/// Why:
/// A test of the scratch root's creation needs a home where that directory does not exist yet,
/// and `mktemp` creates its own directory owner-only,
/// unlike a hand-made path under a shared parent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const freshHome = (): string => spawnSync('mktemp', ['--directory', join(targetTmp, 'fresh-home-XXXXXXXX')]).stdout.trim();
/// ```
fn fresh_home() -> io::Result<PathBuf> {
    let output = Command::new("mktemp")
        .arg("--directory")
        .arg(Path::new(env!("CARGO_TARGET_TMPDIR")).join("fresh-home-XXXXXXXX"))
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(String::from_utf8_lossy(&output.stderr).into_owned()));
    }
    let text = String::from_utf8(output.stdout).map_err(io::Error::other)?;
    return Ok(PathBuf::from(text.trim_end()));
}

/// What:
/// `fn mode_of(path: &Path) -> io::Result<u32>` declares a helper returning the permission bits of a path,
/// with the file-type bits masked off.
/// Why:
/// A raw `st_mode` carries the type as well,
/// and only the nine permission bits are the claim.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const modeOf = (path: string): number => statSync(path).mode & 0o777;
/// ```
fn mode_of(path: &Path) -> io::Result<u32> {
    return Ok(fs::metadata(path)?.permissions().mode() & 0o777);
}

/// A fresh injected home gets its shared scratch root created owner-only,
/// and the native hook initializes the fixture built there.
#[test]
fn a_fresh_home_gets_an_owner_only_scratch_root() -> io::Result<()> {
    let home = fresh_home()?;
    // The fixture creates `${home}/temp/agent` because the home holds nothing yet.
    let parent = support::fixture_in(&home)?;
    let scratch = home.join("temp").join("agent");

    assert_eq!(mode_of(&scratch)?, OWNER_ONLY_MODE, "{} is not owner-only", scratch.display());
    assert!(parent.starts_with(&scratch), "{} is outside {}", parent.display(), scratch.display());

    // The fixture built under that home is usable: the hook initializes it and allows a plain command.
    let path = parent.to_str().ok_or_else(|| { return io::Error::other("fixture path encoding"); })?;
    let initialized = support::invoke(&["--initialize", path], "")?;
    assert!(initialized.status.success(), "{initialized:?}");
    let allowed = support::invoke(&["--hook", path], &support::event("true")?)?;
    assert!(allowed.status.success(), "{allowed:?}");

    fs::remove_dir_all(&home)?;
    return Ok(());
}

/// The shared scratch root under the process's own home is owner-only as well,
/// which is the state every other test in this package inherits.
/// This holds whether the fixture created the directory or found it already there.
#[test]
fn the_scratch_root_under_the_process_home_is_owner_only() -> io::Result<()> {
    let parent = support::fixture()?;
    let home = std::env::var("HOME").map_err(io::Error::other)?;
    let scratch = Path::new(&home).join("temp").join("agent");

    assert_eq!(mode_of(&scratch)?, OWNER_ONLY_MODE, "{} is not owner-only", scratch.display());

    fs::remove_dir_all(parent)?;
    return Ok(());
}
