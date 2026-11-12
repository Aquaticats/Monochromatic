//! What:
//!  Git LFS support:
//!  pre-push planning,
//!  pointer scanning and bounded uploads.
//! Why:
//!  LFS is an optional feature with its own vocabulary.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // lfs/index.ts: one `export * from './file'` per module below.
//! ```

/// When a push needs the Git LFS upload,
/// and the texts the parent prints around it.
pub mod lfs_pre_push;

/// What the Git LFS upload sends:
/// the remote and the refs,
/// read from the push and the configuration Git would read.
pub mod lfs_upload_plan;

/// The same plan worked out with every wait on the runtime,
/// beside the plan itself so that neither file passes the package's length limit.
pub mod lfs_upload_plan_awaiting;

/// The configuration the Git LFS upload plan reads,
/// and the default remote derived from it.
pub mod lfs_upload_settings;

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

/// The configuration a push carries on its command line,
/// as the variables the upload child reads,
/// beside the run of the upload so that each file passes the package's length limit.
pub mod lfs_upload_child_configuration;

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
