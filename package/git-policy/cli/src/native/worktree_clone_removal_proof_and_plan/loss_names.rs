//! What:
//!  The list a removal prints before it requests the unmount:
//!  the slot,
//!  the clone dataset,
//!  and the exact snapshot names to be lost.
//! Why:
//!  The design requires a proven set to "print the slot,
//! dataset and exact snapshot names to be
//!  lost"
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//! "Removal and recovery",
//! step 2),
//!  and the project decided the printed report is the only trace of a destruction that outlives the
//!  removal beyond its records
//!  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
//!  "What trace of a destruction outlives the removal").
//!  The bookmarks are named beside the snapshots because the head destroy takes the clone's
//!  bookmarks with it:
//!  a person who is not told cannot ask about them before the command runs.
//!  Nothing here is a record:
//!  the plan holds the tuples and no bookmark entry.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! console.log(printNamesToBeLost({ slot, dataset, snapshots, bookmarks }));
//! ```

/// What:
///  Every name one removal is about to lose.
/// Why:
///  One value means the line a person reads and the plan the removal writes cannot be about
///  different objects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NamesToBeLost = { slot; dataset; snapshots: string[]; bookmarks: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamesToBeLost {
    /// The slot's name.
    pub slot: String,
    /// The clone's full dataset name.
    pub dataset: String,
    /// Every snapshot of the clone's own history,
    /// by full name,
    /// in the order the plan records them.
    pub snapshots: Vec<String>,
    /// Every bookmark the clone holds,
    /// by full name.
    pub bookmarks: Vec<String>,
}

impl NamesToBeLost {
    /// What:
    ///  The lines a person reads,
    ///  one name each.
    /// Why:
    ///  A name per line is what a person can compare with `zfs list`,
    ///  and the first line says which slot and dataset the whole list is about.
    ///  The last lines say what cannot be restored and what the removal does not touch.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const lines = names.lines();
    /// ```
    pub fn lines(&self) -> Vec<String> {
        let mut lines: Vec<String> =
            Vec::with_capacity(self.snapshots.len() + self.bookmarks.len() + 4);
        lines.push(format!(
            "removing the ZFS worktree clone at `{}` (dataset `{}`) loses:",
            self.slot, self.dataset
        ));
        for snapshot in &self.snapshots {
            lines.push(format!("  snapshot {snapshot}"));
        }
        for bookmark in &self.bookmarks {
            lines.push(format!(
                "  bookmark {bookmark}, which goes with the clone's head and cannot be restored"
            ));
        }
        lines.push(format!(
            "{} snapshot(s) and {} bookmark(s) above cannot be restored by this wrapper, and the main checkout keeps every snapshot it holds.",
            self.snapshots.len(),
            self.bookmarks.len()
        ));
        return lines;
    }

    /// The lines as one block a caller prints as it stands.
    pub fn text(&self) -> String {
        return self.lines().join("\n");
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "loss_names_tests.rs"]
mod tests;
