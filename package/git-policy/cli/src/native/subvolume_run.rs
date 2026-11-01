//! What:
//!  The state one migration command carries between its steps: the platform, the paths, the journal, the progress
//!  sink, the list of files this user may not read, and the helper that runs one tree pass and reports it.
//! Why:
//!  The migrate, rollback and remove-old commands share it, so each pass is timed, counted and reported the same way
//!  and the unreadable-file list is written the same way after every pass (a kill between passes loses nothing).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class Run { platform; layout; journal; interrupt; say(line); async pass(label, source, destination, options) { ... } }
//! ```

use super::subvolume_error::{Interrupt, MigrateError};
use super::subvolume_journal::Journal;
use super::subvolume_platform::Platform;
use super::subvolume_preflight::Layout;
use super::subvolume_sync::{SyncOptions, SyncStats, sync_tree};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// The path, relative to a checkout, of the lock the migration holds during its last pass.
pub const INDEX_LOCK: &str = ".git/index.lock";

/// What:
///  One command's working state.
///  `&'a mut dyn FnMut(&str)` is a borrowed progress callback that receives one line at a time.
/// Why:
///   See the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Run = { platform: Platform; layout: Layout; journal: Journal; interrupt: Interrupt; say: (line: string) => void; unreadable: Set<string> };
/// ```
pub struct Run<'a> {
    /// The operating-system operations.
    pub platform: &'a dyn Platform,
    /// Every path of the migration.
    pub layout: &'a Layout,
    /// The journal.
    pub journal: Journal,
    /// The simulated kill of the controls; never in release use.
    pub interrupt: &'a Interrupt,
    /// The progress sink.
    pub say: &'a mut dyn FnMut(&str),
    /// Relative paths of files this user may not read, which stay in the old tree.
    pub unreadable: BTreeSet<PathBuf>,
}

impl Run<'_> {
    /// What:
    ///  Start the state: load the journal and the unreadable list left by an earlier run.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const run = await Run.open(platform, layout, interrupt, say);
    /// ```
    pub fn open<'a>(
        platform: &'a dyn Platform,
        layout: &'a Layout,
        interrupt: &'a Interrupt,
        say: &'a mut dyn FnMut(&str),
    ) -> Result<Run<'a>, MigrateError> {
        let journal: Journal = Journal::load(layout.journal_path.as_path())?;
        let unreadable: BTreeSet<PathBuf> = match std::fs::read_to_string(&layout.unreadable_path) {
            Ok(text) => text.lines().filter(|line: &&str| { return !line.is_empty(); }).map(PathBuf::from).collect(),
            Err(_) => BTreeSet::new(),
        };
        return Ok(Run { platform, layout, journal, interrupt, say, unreadable });
    }

    /// Send one line to the progress sink.
    pub fn line(&mut self, text: &str) {
        (self.say)(text);
    }

    /// What:
    ///  Run one tree pass, report its counts and time, and record the unreadable files it found.
    /// Why:
    ///   The list is written after every pass, so a kill between passes keeps it; the report names the count and the file.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const stats = await run.pass('copy', source, destination, { delete: false, updateOnly: false });
    /// ```
    pub fn pass(&mut self, label: &str, source: &Path, destination: &Path, delete: bool, update_only: bool) -> Result<SyncStats, MigrateError> {
        let excluded: [PathBuf; 1] = [PathBuf::from(INDEX_LOCK)];
        let started: Instant = Instant::now();
        let stats: SyncStats = sync_tree(self.platform, source, destination, SyncOptions { delete, update_only, excluded: &excluded }, self.interrupt)?;
        self.line(
            format!(
                "{label}: {} entries, {} files copied ({} cloned), {} hard-linked, {} symlinks, {} unchanged, {} deleted, {} vanished, {:.2} s",
                stats.entries, stats.files_copied, stats.cloned, stats.hard_linked, stats.symlinks, stats.unchanged, stats.deleted, stats.vanished,
                started.elapsed().as_secs_f64()
            )
            .as_str(),
        );
        for path in &stats.special_skipped {
            self.line(format!("{label}: not recreated (a socket or device node): {}", path.display()).as_str());
        }
        if stats.losses.owner > 0 || stats.losses.extended_attributes > 0 {
            self.line(format!("{label}: could not set the owner of {} entries and {} extended attributes (not running as the owner or as root)", stats.losses.owner, stats.losses.extended_attributes).as_str());
        }
        if !stats.unreadable.is_empty() {
            self.unreadable.extend(stats.unreadable.iter().cloned());
            self.write_unreadable()?;
            self.line(format!("{label}: {} file(s) this user may not read stay in the old tree; list: {}", stats.unreadable.len(), self.layout.unreadable_path.display()).as_str());
        }
        return Ok(stats);
    }

    /// Write the unreadable-file list beside the checkout.
    fn write_unreadable(&self) -> Result<(), MigrateError> {
        let mut text: String = String::new();
        for path in &self.unreadable {
            text.push_str(path.to_string_lossy().as_ref());
            text.push('\n');
        }
        return std::fs::write(&self.layout.unreadable_path, text).map_err(|error: std::io::Error| {
            return MigrateError::io("write the list of unreadable files", self.layout.unreadable_path.as_path(), &error);
        });
    }

    /// Stop here when the control asked for a simulated kill at this point.
    pub fn checkpoint(&self, point: &str) -> Result<(), MigrateError> {
        return self.interrupt.check(point);
    }
}
