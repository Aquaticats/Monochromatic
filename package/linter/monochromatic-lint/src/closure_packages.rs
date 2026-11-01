//! What:
//!  Group the Rust files that select the closure check by the Cargo package that owns them.
//! Why:
//!  The lint runs once per package,
//! not once per file,
//!  and a package is checked when at least one of its files selects the rule.
//!  Fuzz and bench crates are exempt by directory name,
//!  and a file that no `Cargo.toml` owns cannot be compiled,
//! so it is not checked.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ClosurePackage = { manifest: Path; library: Path; configuration: string };
//! function groupPackages(plans: FilePlan[]): { packages: ClosurePackage[]; notes: string[] };
//! ```

/// Import the manifest discovery,
///  the plan data and the path normalization.
use crate::{
    markdown_lfs_target::lexical_normal,
    run_paths::{Language, display_name},
    run_plan::{FilePlan, RootRules},
    rust_workspace::discover_manifest,
};
/// Import an ordered map and native paths.
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// What:
///  One Cargo package to check.
/// Why:
///  The manifest locates the package,
///  the library path comes from the rule's option,
///  and the configuration name labels a failure that concerns the option.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClosurePackage = { manifest: Path; library: Path; configuration: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosurePackage {
    /// The package's `Cargo.toml`.
    pub manifest: PathBuf,
    /// The Dylint library directory,
    ///  resolved against the directory of the configuration file that named it.
    pub library: PathBuf,
    /// The display name of the configuration file that selected the rule.
    pub configuration: String,
}

/// What:
///  The packages of a run and the debug notes about files that were left out.
/// Why:
///  Leaving a file out is normal and explained only on request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Grouping = { packages: ClosurePackage[]; notes: string[] };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Grouping {
    /// One entry per package,
    ///  ordered by manifest path.
    pub packages: Vec<ClosurePackage>,
    /// Why a file selected by the rule is in no package.
    pub notes: Vec<String>,
}

/// What:
///  True for a directory of a fuzz or bench crate.
/// Why:
///  Sidecar crates are exempt from the check by the repository owner's decision;
///  the suffix is the repository's naming convention for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isSidecar(directory: Path): boolean { return /\.(fuzz|bench)$/.test(basename(directory)); }
/// ```
pub fn is_sidecar_crate(directory: &Path) -> bool {
    let Some(name) = directory.file_name() else {
        return false;
    };
    let text = name.to_string_lossy();
    return text.ends_with(".fuzz") || text.ends_with(".bench");
}

/// What:
///  Group the plans that select the rule by their owning package.
/// Why:
///  Each package is linted once,
///  with the library of the first of its files that names one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function groupPackages(plans: FilePlan[], cwd: Path): Grouping;
/// ```
pub fn group_packages(plans: &[FilePlan], cwd: &Path) -> Grouping {
    let mut by_manifest: BTreeMap<PathBuf, ClosurePackage> = BTreeMap::new();
    let mut notes: Vec<String> = Vec::<String>::new();
    for plan in plans {
        if plan.language != Language::Rust {
            continue;
        }
        let RootRules::Rust { settings } = &plan.root else {
            continue;
        };
        let Some(selected) = &settings.avoidable_closure else {
            continue;
        };
        let manifest: PathBuf = match discover_manifest(&plan.absolute) {
            Ok(found) => found,
            Err(error) => {
                notes.push(format!(
                    "{}: not checked by rust/avoidable-closure: {error}",
                    plan.display
                ));
                continue;
            }
        };
        let Some(directory) = manifest.parent() else {
            continue;
        };
        if is_sidecar_crate(directory) {
            notes.push(format!(
                "{}: not checked by rust/avoidable-closure: {} is a fuzz or bench crate",
                plan.display,
                display_name(directory, cwd)
            ));
            continue;
        }
        if by_manifest.contains_key(&manifest) {
            continue;
        }
        let library: PathBuf = lexical_normal(&plan.config.base.join(&selected.library));
        by_manifest.insert(
            manifest.clone(),
            ClosurePackage {
                manifest,
                library,
                configuration: display_name(&plan.config.path, cwd),
            },
        );
    }
    return Grouping {
        packages: by_manifest.into_values().collect::<Vec<ClosurePackage>>(),
        notes,
    };
}

/// Grouping controls stay outside release artifacts.
#[cfg(test)]
#[path = "closure_packages_tests.rs"]
mod tests;
