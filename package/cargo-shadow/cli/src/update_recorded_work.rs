//! What:
//! The work values of the update:
//! the cargo command of one crate root,
//! the same command with the manifests of the root recorded,
//! and the chain of manifest recordings around the `--breaking` sync.
//! Why:
//! `run_recording` and `run_recording_manifests` take their work as a value (`RecordedWork`),
//! and these types hold what the closures that stood here captured.
//! Split out of `update.rs`,
//! which runs `cargo-shadow update`,
//! to keep that file under the code-line limit.
//! The items are re-exported from `update`,
//! so their paths there are unchanged.

/// The request,
/// the error and the cargo runner of the update that the work values use.
use crate::update::{UpdateError, UpdateRequest, run_or_fail};
/// Imports `RefCell`,
/// a list or value that can be changed through a shared reference;
/// it holds the record of the lockfiles this run wrote.
use std::cell::RefCell;


/// What:
/// `use crate::cargo_run::{CargoOutcome, CargoRunner};` imports `CargoOutcome`,
/// `CargoRunner` from another module of this crate so this file can name them unqualified.
/// Why:
/// `CargoOutcome` is used by `run_or_fail`,
/// `resolve_crate`;
/// `CargoRunner` is used by `UpdateRequest`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CargoOutcome, CargoRunner } from 'crate/cargo_run';
/// ```
use crate::cargo_run::{CargoOutcome};

/// Imports the manifest transaction that `--breaking` needs:
/// cargo rewrites `Cargo.toml` as well as the lockfile.
use crate::manifest_edit::{ManifestEdit, run_recording_manifests};
/// Imports the file-system error module (the result type of the recorded calls).
use std::io;

/// What:
/// `enum SyncChain<'a>` is the work of the `--breaking` sync:
/// `Sync` runs the syncer,
/// and `Around` runs the work inside it with the manifests of one crate root recorded.
/// Why:
/// The sync runs inside one recording per crate root,
/// the innermost first,
/// and an enum replaces the boxed closures that were nested for it.
pub(crate) enum SyncChain<'a> {
    /// The innermost work:
    /// the syncer itself.
    Sync {
        /// The update request whose syncer runs.
        request: &'a UpdateRequest<'a>,
    },
    /// Work wrapped in the recording of one crate root's manifests.
    Around {
        /// The crate root whose manifests are recorded.
        crate_dir: &'a str,
        /// The manifest edits the recording attributes.
        edits: &'a RefCell<Vec<ManifestEdit>>,
        /// The work that runs inside the recording.
        inner: Box<SyncChain<'a>>,
    },
}

/// What:
/// The implementation of `RecordedWork` for `SyncChain`.
/// Why:
/// `run_recording_manifests` runs it between its before and after observations.
impl crate::file_owner::RecordedWork<io::Result<Result<String, String>>> for SyncChain<'_> {
    /// What:
    /// Runs the syncer,
    /// or the inner work inside the recording of one crate root's manifests.
    /// Why:
    /// The result of the recording is flattened into the sync's own result.
    fn run(self) -> io::Result<Result<String, String>> {
        match self {
            SyncChain::Sync { request } => return Ok(request.syncer.sync(request.root)),
            SyncChain::Around { crate_dir, edits, inner } => return run_recording_manifests(crate_dir, edits, *inner).and_then(std::convert::identity),
        }
    }
}

/// What:
/// `pub(crate) struct CargoRun<'a> { request, crate_dir, args }` is the work of running one cargo command in a crate root and failing with the crate named when it exits non-zero.
/// Why:
/// `run_recording` takes the work as a value,
/// and this holds what the closure that stood here captured.
pub(crate) struct CargoRun<'a> {
    /// The update request.
    pub(crate) request: &'a UpdateRequest<'a>,
    /// The crate root cargo runs in.
    pub(crate) crate_dir: &'a str,
    /// The arguments of the cargo command.
    pub(crate) args: &'a [String],
}

/// What:
/// The implementation of `RecordedWork` for `CargoRun`.
/// Why:
/// `run_recording` runs it between its before and after observations.
impl crate::file_owner::RecordedWork<Result<CargoOutcome, UpdateError>> for CargoRun<'_> {
    /// What:
    /// Runs the cargo command.
    /// Why:
    /// The result is the one the closure returned.
    fn run(self) -> Result<CargoOutcome, UpdateError> {
        return run_or_fail(self.request, self.crate_dir, self.args);
    }
}

/// What:
/// `struct ManifestsThenCargo<'a>` is the work of running one cargo command with the manifests of its crate root recorded.
/// Why:
/// `run_recording` takes the work as a value,
/// and this holds what the closure that stood here captured.
pub(crate) struct ManifestsThenCargo<'a> {
    /// The cargo command to run.
    pub(crate) cargo: CargoRun<'a>,
    /// The manifest edits the recording attributes.
    pub(crate) edits: &'a RefCell<Vec<ManifestEdit>>,
}

/// What:
/// The implementation of `RecordedWork` for `ManifestsThenCargo`.
/// Why:
/// `run_recording` runs it between its before and after observations.
impl crate::file_owner::RecordedWork<io::Result<Result<CargoOutcome, UpdateError>>> for ManifestsThenCargo<'_> {
    /// What:
    /// Runs the cargo command inside the recording of the manifests.
    /// Why:
    /// The result is the one the closure returned.
    fn run(self) -> io::Result<Result<CargoOutcome, UpdateError>> {
        return run_recording_manifests(self.cargo.crate_dir, self.edits, self.cargo);
    }
}
