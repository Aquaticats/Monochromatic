//! What:
//!  Where the copies of discarded files live:
//!  the one name below a repository's common Git directory.
//! Why:
//!  The copy that is kept,
//!  the pass that prunes the folder and the doctor's report
//!  all name this directory,
//!  and while one of those three files held the name the other two reached it
//!  through that file,
//!  a loop between files.
//!  This file uses nothing of the folder,
//!  so each of them may use it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const SAVED_DIRECTORY = 'monochromatic-git-policy-cli/discarded';
//! ```

/// Where the copies live,
/// below the common Git directory,
/// beside `monochromatic-git-policy-cli-audit.jsonl`.
pub const SAVED_DIRECTORY: &str = "monochromatic-git-policy-cli/discarded";
