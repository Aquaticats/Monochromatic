//! What:
//!  What the files of the service's journal are called:
//! the extension of an intent,
//! of a finished batch,
//! of a file being written,
//! of a batch number taken,
//! of a scratch reference's record,
//! of an installed index update,
//! of the index a batch started from,
//! and of a batch's copy of the conclusion state;
//! and the path one batch's file of any of those has.
//! Why:
//!  Every reader of the journal asks about a file by its extension (`service_journal.rs`),
//! so the names sit in one file and no two of them drift apart.
//! A batch's files are all named by its number and one of these extensions,
//! which is what lets a claim,
//! an intent and a finished record be found as one batch.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! batchPath(dir, 7, OPEN_EXTENSION) // '/journal/7.intent'
//! ```

use std::path::{
    Path,
    PathBuf,
};

/// Extension of an intent whose batch may be unfinished.
pub const OPEN_EXTENSION: &str = "intent";

/// Extension of a finished batch,
/// kept so its request identities are remembered.
pub const DONE_EXTENSION: &str = "done";

/// Extension of a file being written;
/// never read as an intent.
pub const TEMPORARY_EXTENSION: &str = "tmp";

/// Extension of the empty file that holds a batch number for the service landing it,
/// from the moment the number is taken until
///  the landing ends;
/// never read as an intent.
pub const CLAIM_EXTENSION: &str = "claim";

/// Extension of the record a landing writes before it imports,
/// naming the worktree whose scratch reference the import is about to
///  create (`service_scratch_ref_marker_of_a_batch.rs`);
/// never read as an intent.
pub const SCRATCH_EXTENSION: &str = "scratch";

/// The extension of the record that a batch's index update is installed.
pub const INDEXED_EXTENSION: &str = "indexed";

/// The extension of the record of the live index file a batch started from.
pub const START_EXTENSION: &str = "start";

/// The extension of the folder that holds one batch's copy of the conclusion state it removes
/// (`conclusion_state_copy.rs`),
/// which is a copy of what the landing compares and never the state itself.
pub const CONCLUSION_COPY_EXTENSION: &str = "cleanup";

/// The path of a batch's file with an extension.
pub fn batch_path(
    dir: &Path,
    seq: u64,
    extension: &str,
) -> PathBuf {
    return dir.join(format!("{seq}.{extension}"));
}
