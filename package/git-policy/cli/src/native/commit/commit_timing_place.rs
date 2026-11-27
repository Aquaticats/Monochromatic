//! What:
//!  Where one timing record goes:
//!  the worktree's own Git directory,
//!  the common Git directory that holds the file,
//!  and the service instance the record names.
//! Why:
//!  The file collects the records of every worktree of one repository,
//!  so a record that cannot say which worktree and which instance it belongs to
//!  cannot be read beside the service's own lines.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Place = { gitDir: string; commonDir: string; instance: string };
//! ```

/// The one quote every JSON word of a record is written with.
use crate::json_record::quote;
/// `PathBuf` is an owned filesystem path.
use std::path::PathBuf;

/// Where the record goes and which service instance it belongs to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Place {
    /// The worktree's own Git directory.
    pub(crate) git_dir: PathBuf,
    /// The common Git directory,
    /// which holds the file.
    pub(crate) common_dir: PathBuf,
    /// The instance name that `journalctl --user --unit=cli-git@<instance>.service` takes.
    pub(crate) instance: String,
}

impl Place {
    /// The instance as a JSON string.
    pub(crate) fn instance_json(&self) -> String {
        return quote(self.instance.as_str());
    }

    /// The worktree's own Git directory as a JSON string.
    pub(crate) fn git_dir_json(&self) -> String {
        return quote(&self.git_dir.to_string_lossy());
    }
}
