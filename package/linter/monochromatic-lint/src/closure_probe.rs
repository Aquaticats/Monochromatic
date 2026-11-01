//! What:
//!  Check a package's prerequisites on this machine and collect what the compile needs added to its environment.
//! Why:
//!  A package whose prerequisites are missing is skipped with a note that names the missing piece,
//!  never reported as a failure;
//!  a package whose prerequisites are present gets the variables and arguments that its build needs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Readiness = { ready: PackageSetup } | { skipped: string };
//! function prepare(prerequisites: Prerequisite[], directory: Path, search: string): Readiness;
//! ```

/// Import the prerequisite kinds and the platform-specific preparations.
use crate::{
    closure_manifest::Prerequisite,
    closure_probe_platform::{android_setup, ide_setup},
};
/// Import the borrowed operating-system string type,
///  native paths and the process type that runs `pkg-config`.
use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

/// Import the execute-bit accessor of Unix permissions.
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// What:
///  What a ready package needs added to its `cargo` run.
/// Why:
///  The variables and arguments come from the fixed preparations of this program,
///  never from the manifest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PackageSetup = { environment: [string, string][]; arguments: string[] };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PackageSetup {
    /// Variables set for the `cargo dylint` process,
    ///  in addition to the inherited environment.
    pub environment: Vec<(String, String)>,
    /// Arguments appended after `--` for `cargo check`.
    pub arguments: Vec<String>,
}

/// What:
///  The result of checking one package's prerequisites.
/// Why:
///  A missing prerequisite is a normal outcome with an explanation,
///  so it is a variant and not an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Readiness = { kind: 'ready'; setup: PackageSetup } | { kind: 'skipped'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Readiness {
    /// Every prerequisite holds.
    Ready(
        /// What the package's run needs added.
        PackageSetup,
    ),
    /// A prerequisite is missing;
    ///  the text says which and how to provide it.
    Skipped(
        /// Which prerequisite is missing and how to provide it.
        String,
    ),
}

/// What:
///  Find a program in the directories of a `PATH`-style list.
/// Why:
///  Looking a name up never runs it;
///  the name has been validated as a plain name before it reaches here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findExecutable(name: string, search: string): Path | undefined;
/// ```
pub fn find_executable(name: &str, search: &OsStr) -> Option<PathBuf> {
    for directory in std::env::split_paths(search) {
        let candidate: PathBuf = directory.join(name);
        if is_executable_file(&candidate) {
            return Some(candidate);
        }
    }
    return None;
}

/// What:
///  True for a regular file that the owner can execute.
/// Why:
///  On Unix the execute bit decides;
///  elsewhere a file with the name is enough.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isExecutableFile(path: Path): boolean;
/// ```
#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    return match std::fs::metadata(path) {
        Ok(metadata) => metadata.is_file() && metadata.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    };
}

/// What:
///  True for a regular file with the given name.
/// Why:
///  A platform without execute bits has no cheaper test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isExecutableFile(path: Path): boolean;
/// ```
#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    return path.is_file();
}

/// What:
///  Check that every named `pkg-config` module is installed.
/// Why:
///  `pkg-config --exists` answers by status and prints nothing;
///  a machine without `pkg-config` has none of the modules either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkModules(modules: string[]): string | undefined; // reason to skip
/// ```
fn check_modules(modules: &[String]) -> Option<String> {
    for module in modules {
        let status = Command::new("pkg-config")
            .arg("--exists")
            .arg(module)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        match status {
            Ok(done) => {
                if !done.success() {
                    return Some(format!(
                        "the system library {module} is not installed (`pkg-config --exists {module}` failed); install its development files"
                    ));
                }
            }
            Err(error) => {
                return Some(format!(
                    "the system library {module} cannot be checked because pkg-config does not run ({error}); install pkg-config and the library's development files"
                ));
            }
        }
    }
    return None;
}

/// What:
///  Check that every named program is on the search path.
/// Why:
///  The compile of the package runs the programs;
///  this program only looks for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkExecutables(names: string[], search: string): string | undefined;
/// ```
fn check_executables(names: &[String], search: &OsStr) -> Option<String> {
    for name in names {
        if find_executable(name, search).is_none() {
            return Some(format!(
                "the program {name} is not on PATH; install it or add it to PATH"
            ));
        }
    }
    return None;
}

/// What:
///  Point `QMAKE` at a Qt 6 `qmake`.
/// Why:
///  The Qt build locates Qt by running `qmake`,
///  and a distribution may install only `qmake6`;
///  an existing `QMAKE` the machine owner set is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function qmakeSetup(search: string, current?: string): [string, string] | string; // variable or reason
/// ```
fn qmake_setup(search: &OsStr, current: Option<OsString>) -> Result<Vec<(String, String)>, String> {
    if let Some(existing) = current
        && Path::new(&existing).is_file()
    {
        return Ok(Vec::<(String, String)>::new());
    }
    for name in ["qmake6", "qmake"] {
        if let Some(found) = find_executable(name, search) {
            return Ok(vec![(
                String::from("QMAKE"),
                found.to_string_lossy().into_owned(),
            )]);
        }
    }
    return Err(String::from(
        "neither qmake6 nor qmake is on PATH and QMAKE names no file; install the Qt 6 development tools",
    ));
}

/// What:
///  Check every prerequisite of a package in manifest order and collect its setup.
/// Why:
///  The first missing prerequisite decides the skip,
///  because the later ones would only repeat the same advice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepare(prerequisites: Prerequisite[], directory: Path, cargo: Path, search: string): Readiness;
/// ```
pub fn prepare(
    prerequisites: &[Prerequisite],
    directory: &Path,
    cargo: &Path,
    search: &OsStr,
) -> Readiness {
    let mut setup: PackageSetup = PackageSetup::default();
    for prerequisite in prerequisites {
        let missing: Option<String> = match prerequisite {
            Prerequisite::PkgConfig(modules) => check_modules(modules.as_slice()),
            Prerequisite::Executables(names) => check_executables(names.as_slice(), search),
            Prerequisite::Qmake => match qmake_setup(search, std::env::var_os("QMAKE")) {
                Ok(variables) => {
                    setup.environment.extend(variables);
                    None
                }
                Err(reason) => Some(reason),
            },
            Prerequisite::IdeRuntimeArtifacts => match ide_setup(directory) {
                Ok(variables) => {
                    setup.environment.extend(variables);
                    None
                }
                Err(reason) => Some(reason),
            },
            Prerequisite::AndroidNdk => match android_setup(cargo) {
                Ok(prepared) => {
                    setup.environment.extend(prepared.environment);
                    setup.arguments.extend(prepared.arguments);
                    None
                }
                Err(reason) => Some(reason),
            },
        };
        if let Some(reason) = missing {
            return Readiness::Skipped(reason);
        }
    }
    return Readiness::Ready(setup);
}

/// Prerequisite probe controls stay outside release artifacts.
#[cfg(test)]
#[path = "closure_probe_tests.rs"]
mod tests;
