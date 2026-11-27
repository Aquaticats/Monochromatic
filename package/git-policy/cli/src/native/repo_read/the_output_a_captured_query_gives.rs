//! What:
//!  What one captured Git query produced:
//!  its exit status and the exact bytes it wrote to each stream.
//! Why:
//!  The type is what a reader of a captured query hands back,
//!  and more than a dozen readings name it:
//!  while it lived in the runner's file (`git_metadata.rs`),
//!  the hook that answers a query in process,
//!  and the runner that starts Git instead,
//!  used each other in a loop (`AGENTS.md` rule `LYF`).
//!  Here it is a file the runner,
//!  the hook and every reader name,
//!  and which names nothing of the folder back.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type MetadataOutput = { success: boolean; code: number | null; stdout: Buffer; stderr: Buffer };
//! ```

/// What:
///  What one metadata query produced.
///       `Vec<u8>` is an owned byte list (`u8` is one byte);
///  sibling `String` would
///       require UTF-8.
/// Why:
///   Git prints paths as raw bytes,
///  and a failed query is an ordinary answer
///       ("not a repository"),
///  not an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MetadataOutput = { success: boolean; code: number | null; stdout: Buffer; stderr: Buffer };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataOutput {
    /// Whether Git exited with status zero.
    pub success: bool,
    /// Git's exit code,
    ///  or nothing when a signal ended it.
    ///  Some queries answer through
    /// their exit code alone (`git diff-index --quiet` exits 1 for "differs").
    pub code: Option<i32>,
    /// Exact bytes Git wrote to standard output,
    ///  including what it printed before failing.
    pub stdout: Vec<u8>,
    /// Exact bytes Git wrote to standard error.
    pub stderr: Vec<u8>,
}
