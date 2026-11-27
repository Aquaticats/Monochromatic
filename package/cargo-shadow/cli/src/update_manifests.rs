//! What:
//! The manifests a `--breaking` update may rewrite:
//! `workspace_manifests` lists them before cargo runs,
//! from `cargo metadata --no-deps`,
//! and `sync_recording` renders the raised catalog floors into them while recording every manifest the sync touches.
//! Why:
//! Cargo's breaking update rewrites the requirements in `Cargo.toml` as well as the lockfile,
//! so the manifests are part of the transaction:
//! they are listed before the first mutation,
//! and a sync that fails halfway leaves nothing behind.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update_manifests.ts: workspaceManifests(request, crateDir), syncRecording(request, edits)
//! ```

/// What:
/// `use crate::cargo_run::CargoOutcome;` imports `CargoOutcome` from another module of this crate so this file can name it unqualified.
/// Why:
/// The manifest list comes from one cargo call,
/// whose exit status and output are this value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CargoOutcome } from 'crate/cargo_run';
/// ```
use crate::cargo_run::CargoOutcome;
/// What:
/// `use crate::manifest_edit::ManifestEdit;` imports the record of one manifest's text.
/// Why:
/// The sync records each manifest it rewrites as one of these,
/// so what it wrote halfway goes back with the transaction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ManifestEdit } from 'crate/manifest_edit';
/// ```
use crate::manifest_edit::ManifestEdit;
/// What:
/// `use crate::stamp::manifests_from_metadata;` imports the manifest list of a `cargo metadata` answer.
/// Why:
/// It turns the text of that answer into the workspace's manifest paths without a second cargo call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { manifests_from_metadata } from 'crate/stamp';
/// ```
use crate::stamp::manifests_from_metadata;
/// What:
/// `use crate::update_error::{UpdateError, other};` imports the update's error type and the conversion of a plain message into it.
/// Why:
/// A `cargo metadata` that fails is a cargo failure of the update,
/// and recording a manifest reports the catch-all update error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { UpdateError, other } from 'crate/update_error';
/// ```
use crate::update_error::{
    UpdateError,
    other,
};
/// What:
/// `use crate::update_recorded_work::SyncChain;` imports the chain of manifest recordings and the sync itself.
/// Why:
/// The sync runs inside one nest of recordings per crate root,
/// so what it writes is part of the transaction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { SyncChain } from 'crate/update_recorded_work';
/// ```
use crate::update_recorded_work::SyncChain;
/// What:
/// `use crate::update_request_and_report::UpdateRequest;` imports `UpdateRequest` from another module of this crate so this file can name it unqualified.
/// Why:
/// Both helpers read the crate roots,
/// the forwarded cargo options,
/// the runner and the syncer out of the request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { UpdateRequest } from 'crate/update_request_and_report';
/// ```
use crate::update_request_and_report::UpdateRequest;
/// What:
/// `use std::cell::RefCell;` imports `RefCell`,
/// a box whose contents can be changed through a shared reference;
/// the check that nothing else holds it happens at run time instead of at compile time.
/// Why:
/// The sync records each manifest it rewrites into the caller's list,
/// and it holds only a shared borrow of that list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects are shared mutable references by default; a RefCell is what Rust needs to get that.
/// ```
use std::cell::RefCell;
/// What:
/// `use std::io;` imports the standard library's input/output module so this file can name it unqualified.
/// Why:
/// Recording a manifest before the sync rewrites it can itself fail,
/// and that failure is an `io::Error` rather than the sync's own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // an I/O failure is an ordinary thrown Error in TS
/// ```
use std::io;
/// What:
/// `use std::path::PathBuf;` imports `PathBuf` from the standard library so this file can name it unqualified.
/// Why:
/// The manifest paths are produced by this file and outlive the call that parsed cargo's answer,
/// so they are owned paths;
/// the sibling `&Path` is a borrowed view that would dangle here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for a path: string
/// ```
use std::path::PathBuf;

/// What:
/// `fn workspace_manifests(request, crate_dir) -> Result<Vec<PathBuf>, UpdateError>` lists the manifests of the Cargo workspace
/// at a crate root,
/// from `cargo metadata --no-deps`.
/// Why:
/// The `--breaking` transaction has to know every file cargo may rewrite before cargo runs;
/// if the list cannot be made,
/// nothing has been mutated yet and the update stops.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function workspaceManifests(request, crateDir): Promise<string[]>  // throws when metadata fails
/// ```
pub(crate) fn workspace_manifests(
    request: &UpdateRequest,
    crate_dir: &str,
) -> Result<Vec<PathBuf>, UpdateError> {
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `let mut`: a binding that may change.
    // Typed binding: `args: Vec<String>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let mut args: Vec<String> = vec![
        "metadata".to_string(),
        "--no-deps".to_string(),
        "--format-version".to_string(),
        "1".to_string(),
    ];
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    args.extend(request.forward.iter().cloned());
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&request.root.join`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `outcome: CargoOutcome`.
    let outcome: CargoOutcome = request
        .runner
        .run(&request.root.join(crate_dir), &args)
        .map_err(other)?;
    if !outcome.ok {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(UpdateError::Cargo {
            crate_dir: crate_dir.to_string(),
            command: format!("cargo {}", args.join(" ")),
            output: outcome.output,
        });
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Borrow `&outcome.stdout`: lends the value without copying or giving it away.
    return Ok(manifests_from_metadata(&outcome.stdout));
}

/// What:
/// `fn sync_recording(request, edits) -> io::Result<Result<String, String>>` runs the manifest sync with every crate root's manifests
/// recorded,
/// so what a sync that fails halfway wrote goes back with the transaction.
/// Why:
/// Review finding (2c 11):
/// the sync rewrites manifests,
/// and a partial rewrite was not part of the restore.
/// The outer `io::Result` is a failure to record;
/// the inner `Result` is the sync's own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function syncRecording(request: UpdateRequest, edits: ManifestEdit[]): Result<string, string>  // throws if recording fails
/// ```
pub(crate) fn sync_recording(
    request: &UpdateRequest,
    edits: &RefCell<Vec<ManifestEdit>>,
) -> io::Result<Result<String, String>> {
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `let mut`: a binding that may change.
    // Typed binding: `run: Box<dyn FnOnce() -> io::Result<Result<...`.
    // What:     `&dyn Trait` is a borrowed value of any type that implements the trait (a TS interface type); the call goes through a lookup table at run time.
    // Why:      The caller may pass the real implementation or a test fake.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function f(registry: Registry) { ... }  // an interface type
    // ```
    // Constructor `Box::new(`: an empty owned collection or a boxed value.
    let mut chain: SyncChain = SyncChain::Sync {
        request,
    };
    for crate_dir in request.crate_dirs {
        chain = SyncChain::Around {
            crate_dir,
            edits,
            inner: Box::new(chain),
        };
    }
    return crate::file_owner::RecordedWork::run(chain);
}
