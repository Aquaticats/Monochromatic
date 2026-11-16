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
use crate::subvolume_sync_pass_state_and_checks::Uncarried;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// The path,
/// relative to a checkout,
/// of the lock the migration holds during its last pass.
pub const INDEX_LOCK: &str = ".git/index.lock";

/// What:
///  The paths recorded in the plain file at `path`,
///  or none when the name holds no file,
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
/// async function readPathList(path: string): Promise<Set<string>>;
/// ```
fn read_path_list(path: &Path) -> std::io::Result<BTreeSet<PathBuf>> {
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
///  The record of entries no pass carried,
///  written beside the checkout as each one is learned.
/// Why:
///   A pass that learns an entry and is killed before it ends must still forbid the deletion:
///  the record is written with the atomic replacing write on every entry rather than once at the end of the pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TheRecord { note(path) { this.paths.add(path); writeBesideThenRename(this.name, this.paths); } }
/// ```
struct TheRecord<'a> {
    /// The file beside the checkout.
    path: &'a Path,
    /// Every path the record holds,
    /// this run's earlier record included.
    paths: BTreeSet<PathBuf>,
    /// The files this user may not read,
    /// which have their own record and their own flag:
    /// one entry never asks for two flags.
    unreadable: &'a BTreeSet<PathBuf>,
    /// The failure of a write,
    /// kept so the pass stops with it once it returns.
    failed: Option<MigrateError>,
}

impl TheRecord<'_> {
    /// What:
    ///  Write the record as it stands before the pass begins,
    ///  when it holds anything.
    /// Why:
    ///   A pass that is interrupted must not shrink what earlier passes of the run found:
    ///  the record is the union of the list the run opened with and every entry any pass of it found,
    ///  and it is written whole before each pass so an interruption cannot leave less.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// record.begin(); // write the union as it stands
    /// ```
    fn begin(&mut self) {
        if self.paths.is_empty() {
            return;
        }
        self.write();
    }

    /// Add one path and write the record,
    /// remembering a failure so the pass that called this stops with it.
    fn note(&mut self, relative: &Path) {
        if self.unreadable.contains(relative) || !self.paths.insert(relative.to_path_buf()) {
            return;
        }
        self.write();
    }

    /// Write every path the record holds,
    /// and remember a failure.
    fn write(&mut self) {
        let mut text: String = String::new();
        for path in &self.paths {
            text.push_str(path.to_string_lossy().as_ref());
            text.push('\n');
        }
        if let Err(failure) = MigrateError::from_io_result(write_beside_then_rename(self.path, text.as_bytes()), "write the list of entries no pass carried", self.path) {
            self.failed = Some(failure);
        }
    }
}

/// What:
///  Every path a run's passes and the record it opened with name,
///  which is what its next pass must write before it begins.
/// Why:
///   The record only ever grows until the entry it names is settled,
///  so a pass that is interrupted leaves the union of everything found so far and not only its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const union = new Set([...run.recorded, ...run.leftOut, ...run.conflicts, ...run.kept, ...run.changed]);
/// ```
fn everything_the_run_holds(run: &Run<'_>) -> BTreeSet<PathBuf> {
    let mut every: BTreeSet<PathBuf> = run.recorded.clone();
    every.extend(run.left_out.iter().cloned());
    every.extend(run.conflicts.iter().cloned());
    every.extend(run.kept_at_the_destination.iter().cloned());
    every.extend(run.changed_while_passing.iter().cloned());
    return every;
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
    /// Relative paths of entries the carry did not place because the checkout came to hold an entry it had not seen,
    /// which stays,
    /// so nothing from the old tree reached the checkout at that path.
    pub kept_at_the_destination: BTreeSet<PathBuf>,
    /// Relative paths of entries that were another kind of entry when the carry reached them,
    /// so nothing was written at their path.
    pub changed_while_passing: BTreeSet<PathBuf>,
    /// Relative paths an earlier command recorded as not carried,
    /// read from the list beside the checkout.
    /// They forbid the deletion until a person copies them or accepts the loss,
    /// so a run that has passed nothing still refuses:
    /// the record speaks for the runs that learned it and did not finish.
    pub recorded: BTreeSet<PathBuf>,
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
        let unreadable: BTreeSet<PathBuf> = match read_path_list(layout.unreadable_path.as_path()) {
            Ok(listed) => listed,
            // Only a missing list means "no unreadable files were recorded"; a list that exists and cannot be read is
            // the safeguard failing, and counting it as empty would let the deletion unlink files that exist only there.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => BTreeSet::new(),
            Err(error) => return Err(MigrateError::io("read the list of unreadable files", layout.unreadable_path.as_path(), &error)),
        };
        return Self::open_with(layout, platform, interrupt, say, journal, unreadable);
    }

    /// What:
    ///  Start the state with the part of it that can be read from disk,
    ///  which is what a run resumed after a kill finds.
    /// Why:
    ///   The entries no pass carried are as durable as the unreadable ones:
    ///  a kill between the pass that learned one and the deletion loses the run's memory,
    ///  not the fact.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// Run.openWith(layout, ..., recorded = readPathList(layout.leftOutPath));
    /// ```
    fn open_with<'a>(
        layout: &'a Layout,
        platform: &'a dyn Platform,
        interrupt: &'a Interrupt,
        say: &'a mut dyn FnMut(&str),
        journal: Journal,
        unreadable: BTreeSet<PathBuf>,
    ) -> Result<Run<'a>, MigrateError> {
        // A name that cannot be read here is not a reason to fail a migration: the deletion asks the record again
        //  (`Run::uncarried`), and answers then. A link at the name is such a case, and a plain write of the next list
        //  replaces that link rather than following it.
        let recorded: BTreeSet<PathBuf> = read_path_list(layout.left_out_path.as_path()).unwrap_or_default();
        return Ok(Run {
            platform,
            layout,
            journal,
            interrupt,
            say,
            unreadable,
            left_out: BTreeSet::new(),
            conflicts: BTreeSet::new(),
            kept_at_the_destination: BTreeSet::new(),
            changed_while_passing: BTreeSet::new(),
            recorded,
        });
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
        self.record_uncarried(&stats)?;
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
        self.record_uncarried(&stats)?;
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
        // The record beside the checkout is written as each entry the pass does not carry is learned, so a kill at any
        //  instruction after that leaves the fact behind, not only the run's memory of it.
        let stats: SyncStats = {
            let mut record: TheRecord<'_> =
                TheRecord { path: self.layout.left_out_path.as_path(), paths: everything_the_run_holds(self), unreadable: &self.unreadable, failed: None };
            record.begin();
            let stats: SyncStats = sync_tree(
                self.platform,
                source,
                destination,
                SyncOptions { excluded: excluded.as_slice(), ..requested },
                self.interrupt,
                Some(&mut |path: &Path, _kind: Uncarried| return record.note(path)),
            )?;
            if let Some(failure) = record.failed.take() {
                return Err(failure);
            }
            stats
        };
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
    /// those whose path
    ///  in the destination holds another kind of entry,
    /// those the destination came to hold something else at while the pass ran,
    /// and those that were another kind of entry when the pass reached them,
    /// and write the list beside the checkout,
    /// with every path an earlier run recorded,
    /// so nothing durable is lost by the run that rewrites it.
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
    fn record_uncarried(&mut self, stats: &SyncStats) -> Result<(), MigrateError> {
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
        for path in &stats.kept_at_the_destination {
            if !self.unreadable.contains(path) {
                self.kept_at_the_destination.insert(path.clone());
            }
        }
        for path in &stats.changed_while_passing {
            if !self.unreadable.contains(path) {
                self.changed_while_passing.insert(path.clone());
            }
        }
        if self.left_out.is_empty() && self.conflicts.is_empty() && self.kept_at_the_destination.is_empty() && self.changed_while_passing.is_empty() {
            return Ok(());
        }
        return self.write_left_out();
    }

    /// What:
    ///  Every path that forbids deleting the old tree:
    ///  the entries this run's passes did not carry,
    ///  and every path the record beside the checkout holds,
    ///  read again now,
    ///  so a run that has passed nothing still refuses and a run that cannot remember does not decide.
    /// Why:
    ///   Delete the old tree is the one act of this command that cannot be taken back,
    ///  and a process can be killed between the pass that learned an entry and that act,
    ///  so the question is asked of the record as it stands and not of one run's memory.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const uncarried = await run.uncarried(); // string[]
    /// ```
    pub fn uncarried(&mut self) -> Result<BTreeSet<PathBuf>, MigrateError> {
        match read_path_list(self.layout.left_out_path.as_path()) {
            Ok(listed) => self.recorded = listed,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => self.recorded = BTreeSet::new(),
            Err(error) => return Err(MigrateError::io("read the list of entries no pass carried", self.layout.left_out_path.as_path(), &error)),
        }
        let mut every: BTreeSet<PathBuf> = self.recorded.clone();
        every.extend(self.left_out.iter().cloned());
        every.extend(self.conflicts.iter().cloned());
        every.extend(self.kept_at_the_destination.iter().cloned());
        every.extend(self.changed_while_passing.iter().cloned());
        return Ok(every);
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
        for path in self.kept_at_the_destination.iter().chain(self.changed_while_passing.iter()).chain(self.recorded.iter()) {
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
