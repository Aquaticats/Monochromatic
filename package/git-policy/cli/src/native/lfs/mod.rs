//! What:
//!  Git LFS support:
//!  pre-push planning,
//!  pointer scanning and bounded uploads.
//! Why:
//!  LFS is an optional feature with its own vocabulary.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `lfs_command_line_configuration.rs`:
//!   the configuration a push carries on its command line,
//!   read by the upload child and by the switch that turns the upload off.
//! - `lfs_output_mask.rs`:
//!   hides the credentials inside URLs in the text relayed from `git-lfs`.
//! - `lfs_pointer.rs`:
//!   whether a blob is a Git LFS pointer,
//!   decided as `git-lfs` decides it.
//! - `lfs_pointer_scan.rs`:
//!   the LFS objects a push introduces,
//!   found in process from the commits to be pushed.
//! - `lfs_pre_push.rs`:
//!   when a push needs the Git LFS upload,
//!   and the texts the parent prints around it.
//! - `lfs_program_search.rs`:
//!   the folders a push's children find their programs in,
//!   where that list came from,
//!   and the refusal that names both when `git-lfs` is not in it.
//! - `lfs_push_precheck.rs`:
//!   the check that `git-lfs` can start,
//!   made before a push that is preceded by the upload.
//! - `lfs_upload_explanations.rs`:
//!   the words a person reads about an upload,
//!   beside the run of it so that each file passes the package's length limit.
//! - `lfs_upload_settings.rs`:
//!   the configuration the Git LFS upload plan reads,
//!   and the default remote derived from it.
//! - `lfs_destination_confirms_tracking_refs.rs`:
//!   whether this clone's tracking refs for a remote describe the server a push goes to,
//!   asked of the destination itself.
//! - `lfs_upload_plan_arguments.rs`:
//!   what the arguments of a push and the configuration decide about an upload,
//!   before either face of the plan asks Git anything.
//! - `lfs_upload_refspecs.rs`:
//!   the refs one refspec names,
//!   which both faces of the plan expand,
//!   beside the plan itself so that neither file passes the package's length limit.
//! - `lfs_upload_plan.rs`:
//!   what the Git LFS upload sends:
//!   the remote and the refs,
//!   read from the push and the configuration Git would read.
//! - `lfs_object_plan.rs`:
//!   the upload plan narrowed to the objects the push introduces,
//!   or to nothing.
//! - `lfs_upload.rs`:
//!   the parent's bounded or interactive run of the Git LFS upload.
//! - `lfs_interactive.rs`:
//!   the Git LFS step before a push the caller typed:
//!   pre-check,
//!   plan and upload with the caller's streams.
//! - `lfs_upload_plan_awaiting.rs`:
//!   the same plan worked out with every wait on the runtime,
//!   beside the plan itself so that neither file passes the package's length limit.
//! - `lfs_bounded_push.rs`:
//!   the auto-push's bounded `git push`,
//!   preceded by the upload,
//!   its pre-check and its explanation.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // lfs/index.ts: one `export * from './file'` per module below.
//! ```

/// The configuration a push carries on its command line,
/// read by the upload child and by the switch that turns the upload off.
pub mod lfs_command_line_configuration;

/// When a push needs the Git LFS upload,
/// and the texts the parent prints around it.
pub mod lfs_pre_push;

/// The folders a push's children find their programs in,
/// where that list came from,
/// and the refusal that names both when `git-lfs` is not in it.
pub mod lfs_program_search;

/// What the arguments of a push and the configuration decide about an upload,
/// before either face of the plan asks Git anything.
pub mod lfs_upload_plan_arguments;

/// What the Git LFS upload sends:
/// the remote and the refs,
/// read from the push and the configuration Git would read.
pub mod lfs_upload_plan;

/// The same plan worked out with every wait on the runtime,
/// beside the plan itself so that neither file passes the package's length limit.
pub mod lfs_upload_plan_awaiting;

/// The refs one refspec names,
/// which both faces of the plan expand,
/// beside the plan itself so that neither file passes the package's length limit.
pub mod lfs_upload_refspecs;

/// The configuration the Git LFS upload plan reads,
/// and the default remote derived from it.
pub mod lfs_upload_settings;

/// Whether this clone's tracking refs for a remote describe the server a push goes to,
/// asked of the destination itself.
pub mod lfs_destination_confirms_tracking_refs;

/// Hides the credentials inside URLs in the text relayed from `git-lfs`.
pub mod lfs_output_mask;

/// Whether a blob is a Git LFS pointer,
/// decided as `git-lfs` decides it.
pub mod lfs_pointer;

/// The LFS objects a push introduces,
/// found in process from the commits to be pushed.
#[cfg(unix)]
pub mod lfs_pointer_scan;

/// The upload plan narrowed to the objects the push introduces,
/// or to nothing.
pub mod lfs_object_plan;

/// The parent's bounded or interactive run of the Git LFS upload.
pub mod lfs_upload;

/// The words a person reads about an upload,
/// beside the run of it so that each file passes the package's length limit.
pub mod lfs_upload_explanations;

/// The Git LFS step before a push the caller typed:
/// pre-check,
/// plan and upload with the caller's streams.
pub mod lfs_interactive;

/// The check that `git-lfs` can start,
/// made before a push that is preceded by the upload.
pub mod lfs_push_precheck;

/// The auto-push's bounded `git push`,
/// preceded by the upload,
/// its pre-check and its explanation.
pub mod lfs_bounded_push;
