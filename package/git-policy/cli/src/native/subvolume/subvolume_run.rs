//! What:
//!  The state one migration command carries between its steps:
//! the platform,
//! the paths,
//! the journal,
//! the progress
//!  sink,
//! the list of files this user may not read,
//! and the helper that runs one tree pass and reports it.
//! Why:
//!  The migrate,
//! rollback and remove-old commands share it,
//! so each pass is timed,
//! counted and reported the same way
//!  and the unreadable-file list is written the same way after every pass (a kill between passes loses nothing).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class Run { platform; layout; journal; interrupt; say(line); async pass(label, source, destination, options) { ... } }
//! ```

use crate::subvolume_error::{Interrupt, MigrateError};
use crate::subvolume_file_write_steps::write_beside_then_rename;
use crate::subvolume_journal::Journal;
use crate::subvolume_lock::relative_lock_paths;
use crate::subvolume_platform::Platform;
use crate::subvolume_preflight::Layout;
use crate::subvolume_pass_report::what_a_pass_could_not_carry;
use crate::subvolume_sync::{SyncOptions, SyncStats, sync_tree};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// The path,
/// relative to a checkout,
/// of the lock the migration holds during its last pass.
pub const INDEX_LOCK: &str = ".git/index.lock";

/// What:
///  The paths recorded in the plain file at `path`,
/// read line by line from a handle opened without waiting.
/// Why:
///   Both cleanup modes call `Run::open` before they do anything,
/// and the writer has no bound on how many paths a list
///  holds,
/// so a whole-file cap here was a bound the writer did not share:
/// the migration could write a safeguard it
///  then refused to reopen,
/// which blocked the rollback and the removal until a person edited the file.
/// Reading
///  line by line holds the same paths the run needs and not the record's bytes twice,
/// and a named pipe or
///  device at the name is refused by the plain-file open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readUnreadableList(path: string): Promise<Set<string>>;
/// ```
fn read_unreadable_list(path: &Path) -> std::io::Result<BTreeSet<PathBuf>> {
    let file: std::fs::File = crate::plain_file::open_plain_file(path)?;
    let mut reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(file);
    let mut listed: BTreeSet<PathBuf> = BTreeSet::new();
    let mut line: Vec<u8> = Vec::new();
    loop {
        line.clear();
        let read: usize = std::io::BufRead::read_until(&mut reader, b'\n', &mut line)?;
        if read == 0 {
            break;
        }
        if line.last() == Some(&b'\n') {
            line.pop();
        }
        if line.is_empty() {
            continue;
        }
        listed.insert(PathBuf::from(String::from_utf8_lossy(line.as_slice()).as_ref()));
    }
    return Ok(listed);
}

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
    /// The simulated kill of the controls;
    /// never in release use.
    pub interrupt: &'a Interrupt,
    /// The progress sink.
    pub say: &'a mut dyn FnMut(&str),
    /// Relative paths of files this user may not read,
    /// which stay in the old tree.
    pub unreadable: BTreeSet<PathBuf>,
    /// Relative paths of entries the old tree has,
    /// the checkout lacks,
    /// and that are older than the catch-up,
    /// so carrying them would undo a deletion made in the checkout.
    pub left_out: BTreeSet<PathBuf>,
    /// Relative paths of entries the old tree has where the checkout holds another kind of entry,
    /// which the carry kept as the newer one,
    /// so nothing from the old tree reached the checkout at that path.
    pub conflicts: BTreeSet<PathBuf>,
}

impl Run<'_> {
    /// What:
    ///  Start the state:
    /// load the journal and the unreadable list left by an earlier run.
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
        let unreadable: BTreeSet<PathBuf> = match read_unreadable_list(layout.unreadable_path.as_path()) {
            Ok(listed) => listed,
            // Only a missing list means "no unreadable files were recorded"; a list that exists and cannot be read is
            // the safeguard failing, and counting it as empty would let the deletion unlink files that exist only there.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => BTreeSet::new(),
            Err(error) => return Err(MigrateError::io("read the list of unreadable files", layout.unreadable_path.as_path(), &error)),
        };
        return Ok(Run { platform, layout, journal, interrupt, say, unreadable, left_out: BTreeSet::new(), conflicts: BTreeSet::new() });
    }

    /// Send one line to the progress sink.
    pub fn line(&mut self, text: &str) {
        (self.say)(text);
    }

    /// What:
    ///  Run one tree pass,
    /// report its counts and time,
    /// and record the unreadable files it found.
    /// Why:
    ///   The list is written after every pass,
    /// so a kill between passes keeps it;
    /// the report names the count and the file.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const stats = await run.pass('copy', source, destination, { delete: false, updateOnly: false });
    /// ```
    pub fn pass(&mut self, label: &str, source: &Path, destination: &Path, delete: bool, update_only: bool) -> Result<SyncStats, MigrateError> {
        let stats: SyncStats = self.pass_unrecorded(label, source, destination, delete, update_only)?;
        self.record_unreadable(label, &stats)?;
        return Ok(stats);
    }

    /// Record the unreadable files a pass found in the list beside the checkout,
    /// and say so.
    fn record_unreadable(&mut self, label: &str, stats: &SyncStats) -> Result<(), MigrateError> {
        if !stats.unreadable.is_empty() {
            self.unreadable.extend(stats.unreadable.iter().cloned());
            self.write_unreadable()?;
            self.line(format!("{label}: {} file(s) this user may not read stay in the old tree; list: {}", stats.unreadable.len(), self.layout.unreadable_path.display()).as_str());
        }
        return Ok(());
    }

    /// What:
    ///  Run one tree pass and report its counts and time,
    /// without recording the unreadable files it found.
    /// Why:
    ///   The rollback's carry-back pass reads the new root,
    /// whose unreadable files are not the old tree's:
    /// they must
    ///  not enter the list that guards the deletion of the old tree,
    /// and the rollback decides about them itself.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const stats = await run.passUnrecorded('carry back', source, destination, { delete: false, updateOnly: true });
    /// ```
    pub fn pass_unrecorded(&mut self, label: &str, source: &Path, destination: &Path, delete: bool, update_only: bool) -> Result<SyncStats, MigrateError> {
        return self.pass_core(label, source, destination, SyncOptions { delete, update_only, excluded: &[], created_since: None });
    }

    /// What:
    ///  Run the update-only pass that carries what was written to the old tree after the catch-up:
    /// files newer than
    ///  their copy,
    /// and files the new root lacks that are no older than `since` (seconds after the epoch).
    /// Why:
    ///   Programs that kept a descriptor in the old tree can write there until the person stops them,
    /// and the deletion
    ///  of the old tree must not be the end of those writes.
    /// A file the new root lacks that is older than the catch-up
    ///  was deleted there on purpose and stays deleted.
    /// Unreadable files found are recorded like any other pass.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const stats = await run.passLate('late writes', oldTree, root, catchupSeconds);
    /// ```
    pub fn pass_late(&mut self, label: &str, source: &Path, destination: &Path, since: i64) -> Result<SyncStats, MigrateError> {
        let stats: SyncStats = self.pass_core(label, source, destination, SyncOptions { delete: false, update_only: true, excluded: &[], created_since: Some(since) })?;
        self.record_unreadable(label, &stats)?;
        self.record_left_out(&stats)?;
        for path in &stats.copied_sample {
            self.line(format!("{label}: carried {}", path.display()).as_str());
        }
        return Ok(stats);
    }

    /// The pass itself:
    /// the lock files of both trees are added to the excluded paths,
    /// and the counts are reported.
    fn pass_core(&mut self, label: &str, source: &Path, destination: &Path, requested: SyncOptions<'_>) -> Result<SyncStats, MigrateError> {
        let mut excluded: Vec<PathBuf> = relative_lock_paths(source);
        excluded.extend(relative_lock_paths(destination));
        excluded.sort();
        excluded.dedup();
        let started: Instant = Instant::now();
        let stats: SyncStats = sync_tree(self.platform, source, destination, SyncOptions { excluded: excluded.as_slice(), ..requested }, self.interrupt)?;
        self.line(
            format!(
                "{label}: {} entries, {} files copied ({} cloned), {} hard-linked, {} symlinks, {} unchanged, {} deleted, {} vanished, {:.2} s",
                stats.entries, stats.files_copied, stats.cloned, stats.hard_linked, stats.symlinks, stats.unchanged, stats.deleted, stats.vanished,
                started.elapsed().as_secs_f64()
            )
            .as_str(),
        );
        for text in what_a_pass_could_not_carry(label, &stats) {
            self.line(text.as_str());
        }
        return Ok(stats);
    }

    /// What:
    ///  Record the entries the pass did not carry:
    /// those it left out because they are older than the catch-up,
    /// and those whose path
    ///  in the destination holds another kind of entry,
    /// and write the list beside the checkout.
    /// Why:
    ///   A left-out entry is either a file deleted from the checkout on purpose or a file restored into the old tree
    /// after the migration,
    /// and only the person can tell which.
    /// The deletion of the old tree names them and stops until the person accepts the loss,
    /// and a list written down is what a kill between two runs keeps.
    /// An entry this user may not read is left out here too,
    /// but it is the unreadable list's entry,
    /// which names it and has its own flag:
    /// one entry never asks for two flags.
    /// A path whose entry in the destination is of another kind is not carried either,
    /// so it is recorded the
    ///  same way:
    /// otherwise the old tree's directory there,
    /// and the only copy of what it holds,
    /// goes with the tree.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// run.recordLeftOut(stats) // writes <parent>/.cli-git-migrate-<name>.left-out.txt
    /// ```
    fn record_left_out(&mut self, stats: &SyncStats) -> Result<(), MigrateError> {
        for path in &stats.conflicts {
            if !self.unreadable.contains(path) {
                self.conflicts.insert(path.clone());
            }
        }
        for path in &stats.left_out {
            if !self.unreadable.contains(path) {
                self.left_out.insert(path.clone());
            }
        }
        if self.left_out.is_empty() && self.conflicts.is_empty() {
            return Ok(());
        }
        return self.write_left_out();
    }

    /// Write the list of left-out entries beside the checkout.
    /// The write replaces a name that is there,
    /// a symbolic link included,
    /// and never writes through it.
    fn write_left_out(&self) -> Result<(), MigrateError> {
        let mut text: String = String::new();
        for path in &self.left_out {
            text.push_str(path.to_string_lossy().as_ref());
            text.push('\n');
        }
        for path in &self.conflicts {
            text.push_str(path.to_string_lossy().as_ref());
            text.push('\n');
        }
        return MigrateError::from_io_result(
            write_beside_then_rename(self.layout.left_out_path.as_path(), text.as_bytes()),
            "write the list of entries the carry left out",
            self.layout.left_out_path.as_path(),
        );
    }

    /// Write the unreadable-file list beside the checkout,
    /// through the same replacing write as the left-out list.
    fn write_unreadable(&self) -> Result<(), MigrateError> {
        let mut text: String = String::new();
        for path in &self.unreadable {
            text.push_str(path.to_string_lossy().as_ref());
            text.push('\n');
        }
        return MigrateError::from_io_result(
            crate::private_storage::replace_private_file(&self.layout.unreadable_path, text.as_bytes()),
            "write the list of unreadable files",
            self.layout.unreadable_path.as_path(),
        );
    }

    /// Stop here when the control asked for a simulated kill at this point.
    pub fn checkpoint(&self, point: &str) -> Result<(), MigrateError> {
        return self.interrupt.check(point);
    }
}
