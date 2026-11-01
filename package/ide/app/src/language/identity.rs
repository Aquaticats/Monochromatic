//! Identities every language result carries,
//!  so a late result can be told from a current one.

/// What:
///  A small copyable record of two unsigned 64-bit counters.
///  `u64` never goes negative and
///       does not depend on pointer width (siblings:
///  `usize`,
///  `u32`,
///  `i64`).
/// Why:
///  `file` changes when another file is displayed and `revision` changes on every accepted
///      external reload;
///  together they name exactly one displayed text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DocumentStamp = { file: number; revision: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentStamp {
    /// File-open generation assigned by the interface thread.
    pub file: u64,
    /// Content revision of that file,
    ///  as `Document::revision` reports it.
    pub revision: u64,
}

/// What:
///  A server's configured name plus a number that is unique per started process.
///       `String` owns its text (sibling:
///  borrowed `&str`).
/// Why:
///  A restarted server keeps its name,
///  so the instance number is what keeps an old
///      process's late output apart from the new process's results.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ServerIdentity = { name: string; instance: number };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerIdentity {
    /// Name of the server definition,
    ///  for example `rust-analyzer`.
    pub name: String,
    /// Process generation;
    ///  zero marks a server that was never started.
    pub instance: u64,
}
