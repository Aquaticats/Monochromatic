//! What:
//!  The two prerequisites that prepare a build instead of only checking the machine:
//!  the IDE's embedded-artifact layout and the Android cross-compilation variables.
//! Why:
//!  Both change how `cargo check` compiles the package.
//!  They run fixed steps of this program (links inside the package's `target` directory,
//!  one `cargo ndk-env` query) and read nothing executable from a manifest.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function ideSetup(directory: Path): [string, string][]; // throws string (reason to skip)
//! function androidSetup(cargo: Path): PackageSetup;      // throws string (reason to skip)
//! ```

/// Import the setup shape that the Android preparation fills.
use crate::closure_probe::PackageSetup;
/// Import native paths and the process types that query `cargo ndk-env`.
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

/// What:
///  The Rust target triple that the Android check compiles for.
/// Why:
///  The crate imports Android-only APIs,
///  so it cannot compile for the host;
///  `arm64-v8a` is the device ABI and the one the NDK variables are requested for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ANDROID_TRIPLE = 'aarch64-linux-android';
/// ```
const ANDROID_TRIPLE: &str = "aarch64-linux-android";

/// What:
///  The directory below the package's `target` that holds the build intermediates of this check.
/// Why:
///  The IDE's build script climbs from `OUT_DIR` to find its embedded artifacts,
///  and the shared `build-dir` setting of the machine moves intermediates elsewhere,
///  so the check keeps them in a directory whose layout it controls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BUILD_DIRECTORY = 'target/dylint';
/// ```
const BUILD_DIRECTORY: &str = "dylint";

/// What:
///  Create one symbolic link,
///  replacing a link left by an earlier run.
/// Why:
///  The build directory receives links to the artifacts that the container tasks wrote under `target`,
///  not copies,
/// so a refreshed artifact is seen at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function link(location: Path, destination: string): void; // throws string
/// ```
#[cfg(unix)]
fn link(location: &Path, destination: &Path) -> Result<(), String> {
    if let Some(parent) = location.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        return Err(format!("cannot create {}: {error}", parent.display()));
    }
    if location.symlink_metadata().is_ok()
        && let Err(error) = std::fs::remove_file(location)
    {
        return Err(format!("cannot replace {}: {error}", location.display()));
    }
    return match std::os::unix::fs::symlink(destination, location) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("cannot link {}: {error}", location.display())),
    };
}

/// What:
///  Report that links are not made on this platform.
/// Why:
///  The IDE check is a Linux development gate;
///  a platform without the link call skips the package with a reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function link(location: Path, destination: string): void; // always throws
/// ```
#[cfg(not(unix))]
fn link(location: &Path, _destination: &Path) -> Result<(), String> {
    return Err(format!(
        "cannot link {}: symbolic links are not made on this platform",
        location.display()
    ));
}

/// What:
///  Prepare the IDE build:
///  require the artifacts,
/// link them into the check's build directory and select that directory.
/// Why:
///  The IDE embeds `target/debug/runtime` and `target/crate-licenses.json` at build time.
///  Preparing them needs containers and network,
///  so a missing artifact skips the package and names the tasks that make it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ideSetup(directory: Path): [string, string][]; // throws string
/// ```
pub fn ide_setup(directory: &Path) -> Result<Vec<(String, String)>, String> {
    let runtime: PathBuf = directory.join("target").join("debug").join("runtime");
    let licenses: PathBuf = directory.join("target").join("crate-licenses.json");
    if !runtime.join("manifest.json").is_file() || !licenses.is_file() {
        return Err(format!(
            "the embedded artifacts are not prepared ({} and {} are required); run the runtime and notices tasks of the package first",
            runtime.join("manifest.json").display(),
            licenses.display()
        ));
    }
    let build: PathBuf = directory.join("target").join(BUILD_DIRECTORY);
    link(
        &build.join("debug").join("build").join("runtime"),
        Path::new("../../../debug/runtime"),
    )?;
    link(
        &build.join("debug").join("crate-licenses.json"),
        Path::new("../../crate-licenses.json"),
    )?;
    return Ok(vec![(
        String::from("CARGO_BUILD_BUILD_DIR"),
        build.to_string_lossy().into_owned(),
    )]);
}

/// What:
///  Read `export NAME="value"` lines into variable pairs.
/// Why:
///  `cargo ndk-env` prints a shell fragment;
///  only lines of exactly that shape are taken,
///  and the value is used as data,
/// never run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseExports(text: string): [string, string][];
/// ```
pub fn parse_exports(text: &str) -> Vec<(String, String)> {
    let mut variables: Vec<(String, String)> = Vec::<(String, String)>::new();
    for line in text.lines() {
        let Some(assignment) = line.strip_prefix("export ") else {
            continue;
        };
        let Some((name, quoted)) = assignment.split_once('=') else {
            continue;
        };
        let Some(unquoted) = quoted.strip_prefix('"') else {
            continue;
        };
        let Some(value) = unquoted.strip_suffix('"') else {
            continue;
        };
        if name.is_empty() || !name.chars().all(is_variable_character) {
            continue;
        }
        variables.push((String::from(name), String::from(value)));
    }
    return variables;
}

/// What:
///  True for a character of a portable variable name.
/// Why:
///  A line whose name has any other character is not an assignment this program accepts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isVariableCharacter(character: string): boolean { return /[A-Za-z0-9_]/.test(character); }
/// ```
fn is_variable_character(character: char) -> bool {
    return character.is_ascii_alphanumeric() || character == '_';
}

/// What:
///  The NDK installation directory that a cargo-ndk sysroot path lies in.
/// Why:
///  CMake-based build scripts (the Opus binding) read `ANDROID_NDK_HOME`,
///  while `cargo ndk-env` prints only paths below `<ndk>/toolchains/llvm/prebuilt/<host>/sysroot`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ndkHomeOf(sysroot: string): string | undefined { return sysroot.split('/toolchains/')[0]; }
/// ```
pub fn ndk_home_of(sysroot: &str) -> Option<String> {
    let (home, _below) = sysroot.split_once("/toolchains/")?;
    if home.is_empty() {
        return None;
    }
    return Some(String::from(home));
}

/// What:
///  Prepare the Android cross-compilation:
///  the NDK variables from `cargo ndk-env` and the target argument.
/// Why:
///  The crate's build scripts need the NDK compilers,
///  and a machine without `cargo-ndk` or the NDK skips the package.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function androidSetup(cargo: Path): PackageSetup; // throws string
/// ```
pub fn android_setup(cargo: &Path) -> Result<PackageSetup, String> {
    let output = match Command::new(cargo)
        .args(["ndk-env", "--target", "arm64-v8a"])
        .stdin(Stdio::null())
        .output()
    {
        Ok(done) => done,
        Err(error) => {
            return Err(format!(
                "the Android NDK variables cannot be read because cargo does not run ({error}); install cargo-ndk"
            ));
        }
    };
    if !output.status.success() {
        return Err(format!(
            "`cargo ndk-env --target arm64-v8a` failed ({}); install cargo-ndk and the Android NDK: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let mut variables: Vec<(String, String)> =
        parse_exports(String::from_utf8_lossy(&output.stdout).as_ref());
    // A value the machine owner exported is kept; otherwise it is derived from the sysroot that cargo-ndk found.
    if std::env::var_os("ANDROID_NDK_HOME").is_none() {
        let mut derived: Option<String> = None;
        for (name, value) in &variables {
            if name == "CARGO_NDK_SYSROOT_PATH" {
                derived = ndk_home_of(value);
            }
        }
        if let Some(home) = derived {
            variables.push((String::from("ANDROID_NDK_HOME"), home));
        }
    }
    if variables.is_empty() {
        return Err(String::from(
            "`cargo ndk-env --target arm64-v8a` printed no variable; install the Android NDK",
        ));
    }
    return Ok(PackageSetup {
        environment: variables,
        arguments: vec![String::from("--target"), String::from(ANDROID_TRIPLE)],
    });
}

/// Platform preparation controls stay outside release artifacts.
#[cfg(test)]
#[path = "closure_probe_platform_tests.rs"]
mod tests;
