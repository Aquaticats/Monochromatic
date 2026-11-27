//! What:
//! What an update is asked and what it answers:
//! the two procedures it can run (`UpdateKind`),
//! everything one run needs (`UpdateRequest`),
//! and what a finished,
//! refused or dry run reports (`UpdateReport`).
//! Why:
//! The two callers of an update (the CLI's `update` and dry run,
//! and the shadow's answer to `cargo update`)
//! pass one request value and read one report value,
//! so the shape of both lives in one file that no step of the work owns.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update_request_and_report.ts: type UpdateKind, interface UpdateRequest, interface UpdateReport
//! ```

/// What:
/// `use crate::tighten::Syncer;` imports `Syncer` from another module of this crate so this file can name it unqualified.
/// Why:
/// `UpdateRequest` holds one:
/// a `--breaking` update raises floors in the catalog and this value renders them into the manifests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Syncer } from 'crate/tighten';
/// ```
use crate::tighten::Syncer;

/// What:
/// `use crate::cargo_run::CargoRunner;` imports `CargoRunner` from another module of this crate so this file can name it unqualified.
/// Why:
/// `UpdateRequest` holds one as `&dyn CargoRunner`,
/// so a test can pass a fake cargo in place of the real one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CargoRunner } from 'crate/cargo_run';
/// ```
use crate::cargo_run::CargoRunner;

/// What:
/// `use crate::install::LockBaseFactory;` imports `LockBaseFactory` from another module of this crate so this file can name it unqualified.
/// Why:
/// `UpdateRequest` holds an optional one under `trustLockfile`,
/// because resolving the trust base may start git and so must wait until a check needs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockBaseFactory } from 'crate/install';
/// ```
use crate::install::LockBaseFactory;

/// What:
/// `use crate::lockfile::LockedVersion;` imports `LockedVersion` from another module of this crate so this file can name it unqualified.
/// Why:
/// `UpdateReport` lists the registry versions the update newly locked,
/// and each of them is one of these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockedVersion } from 'crate/lockfile';
/// ```
use crate::lockfile::LockedVersion;

/// What:
/// `use crate::registry::Registry;` imports `Registry` from another module of this crate so this file can name it unqualified.
/// Why:
/// `UpdateRequest` holds one as `&dyn Registry`:
/// the policy checks ask crates.io through it,
/// and a test passes a fake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Registry } from 'crate/registry';
/// ```
use crate::registry::Registry;

/// What:
/// `use monochromatic_dependency_catalog::Settings;` imports `Settings` from the external crate `monochromatic_dependency_catalog` so this file can name it unqualified.
/// Why:
/// `UpdateRequest` holds the release-age and trust settings that the update applies to every version it locks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Settings } from 'monochromatic_dependency_catalog';
/// ```
use monochromatic_dependency_catalog::Settings;

/// What:
/// `use std::path::{Path, PathBuf};` imports `Path` from the standard library so this file can name it unqualified.
/// Why:
/// `UpdateRequest` borrows the repository root and the catalog path,
/// which are `&Path` values;
/// the owned `PathBuf` of the same family is not named here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for a path: string
/// ```
use std::path::Path;

/// Which procedure the update runs.
pub enum UpdateKind<'a> {
    /// Re-resolve every listed crate root under the release-age cutoff (`pnpm update` with no names).
    Full,
    /// Run the user's `cargo update` arguments as given (`pnpm update <name>[@<version>]`).
    Targeted {
        /// The complete cargo arguments,
        /// starting with `update`.
        cargo_args: &'a [String],
    },
}

/// What an update needs.
pub struct UpdateRequest<'a> {
    /// The repository root.
    pub root: &'a Path,
    /// The crate roots to update,
    /// repo-relative.
    pub crate_dirs: &'a [String],
    /// Report without keeping anything.
    pub dry_run: bool,
    /// The current time in milliseconds.
    pub now: i64,
    /// The catalog settings.
    pub settings: &'a Settings,
    /// The catalog file,
    /// edited to prune stale excludes after a kept full update.
    pub catalog_path: &'a Path,
    /// The cargo to run.
    pub runner: &'a dyn CargoRunner,
    /// The registry for the policy checks.
    pub registry: &'a dyn Registry,
    /// Cargo options forwarded to every cargo call (`--offline`,
    /// `--config`,
    /// `-Z`,
    /// `--color`,
    /// `-q`,
    /// `-v`).
    pub forward: &'a [String],
    /// Renders the catalog into the manifests after `--breaking` raised floors.
    pub syncer: &'a dyn Syncer,
    /// Prune exclude selectors that no locked version needs.
    /// Only an update that covers every lockfile of the repository may
    /// say that nothing needs a selector;
    /// an update of some workspaces must leave the lists alone.
    pub prune_catalog: bool,
    /// Makes the reader of one crate's lockfile text at the trust base (empty when absent there);
    /// under `trustLockfile` the update checks every version the base did not hold,
    /// as an install does.
    /// A factory,
    /// because resolving the base may start git.
    pub lock_base: Option<LockBaseFactory<'a>>,
}

/// The outcome of an update.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UpdateReport {
    /// The crate roots updated.
    pub crate_dirs: Vec<String>,
    /// Registry versions newly locked (kept or restored).
    pub added: Vec<LockedVersion>,
    /// Findings that refused the update.
    pub problems: Vec<String>,
    /// Findings reported without refusing,
    /// including set-asides.
    pub warnings: Vec<String>,
    /// Roots resolved without the release-age cutoff (`minimumReleaseAgeStrict = false`).
    pub fell_back: Vec<String>,
    /// Whether the new lockfiles were kept.
    pub written: bool,
    /// Exclude selectors pruned from the catalog.
    pub pruned: Vec<String>,
}
