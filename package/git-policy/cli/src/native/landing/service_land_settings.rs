//! What:
//!  What a landing starts with:
//!  how many times its batch may be built again,
//! and what it reads once before the first build.
//! Why:
//!  The body rebuilds a batch whose branch moved under it,
//!  so the count and the settings it reads belong beside each other rather than in the entry points.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const LANDING_ATTEMPTS = 3; const settings = { importFsync, plainHistory };
//! ```

/// How many times a batch is built again when the branch moved under it before the failure is reported.
pub const LANDING_ATTEMPTS: usize = 3;

/// What the landing reads once before it starts and hands to its body.
pub(crate) struct Settings<'a> {
    /// The `core.fsync` value for the one import,
    /// when the repository's own would flush a pack that is thrown away or leave a kept one unflushed.
    pub(crate) import_fsync: Option<&'a str>,
    /// Whether Git shows the history the commit objects say,
    /// so the object store may answer the checks of a commit Git built.
    pub(crate) plain_history: bool,
}
