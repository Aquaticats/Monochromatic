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
use std::os::unix::fs::OpenOptionsExt;
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
///  The record of the entries a pass parks while it replaces them,
///  and of the parked copies an earlier run left.
/// Why:
///  A parked copy is a file of somebody's whose name says nothing,
///  so this program's own parked copies are known by this record and never by a name's ending;
///  and the line is written before the parking,
///  so a kill at any instruction after it leaves the knowledge that the name holds this program's copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TheParking implements Parking { aboutToPark(original, parked) { this.lines.set(parked, original); writeBesideThenRename(...); } }
/// ```
struct TheParking<'a> {
    /// The file beside the checkout.
    path: &'a Path,
    /// What the old tree read like when the migration copied it,
    /// when this pass carries what was written into it since.
    old_tree: Option<&'a crate::subvolume_old_tree_record::OldTreeRecord>,
    /// Whether what the pass replaces stays at the parked name.
    keeps_what_it_replaces: bool,
    /// Every parked copy the record names,
    /// by its name and the path it is a copy of.
    lines: std::collections::BTreeMap<PathBuf, PathBuf>,
    /// The handle the lines are appended to,
    /// opened when the first entry is parked.
    journal: Option<std::fs::File>,
    /// Whether this pass changed what the record holds.
    touched: bool,
    /// The failure of a write,
    /// kept so the pass stops with it once it returns.
    failed: Option<MigrateError>,
}

impl TheParking<'_> {
    /// What:
    ///  Append one line to the record,
    ///  opening it the first time,
    ///  and remember a failure.
    /// Why:
    ///   The line is on its way to the disk before the pass goes on,
    ///  so a kill at any later instruction leaves the knowledge that the name holds this program's copy;
    ///  one `write` for each entry costs a fraction of the two durability syncs an atomic replacing write costs,
    ///  and the write survives the death of this process because it is in the kernel's hands.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// parking.append(`P\t${parked}\t${original}\n`);
    /// ```
    fn append(&mut self, line: &str) {
        if self.journal.is_none() {
            if self.path.exists() && !self.path.is_file() {
                // The record is this program's own; a link or a pipe at its name is not something to write through.
                if let Err(error) = std::fs::remove_file(self.path) {
                    self.failed = Some(MigrateError::io("replace the name of the list of parked copies", self.path, &error));
                    return;
                }
            }
            match std::fs::OpenOptions::new().append(true).create(true).mode(0o600).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC).open(self.path) {
                Ok(opened) => self.journal = Some(opened),
                Err(error) => {
                    self.failed = Some(MigrateError::io("open the list of parked copies", self.path, &error));
                    return;
                }
            }
        }
        let Some(file) = self.journal.as_mut() else { return };
        if let Err(error) = std::io::Write::write_all(file, line.as_bytes()) {
            self.failed = Some(MigrateError::io("write the list of parked copies", self.path, &error));
        }
    }

    /// What:
    ///  Replace the appended lines with the record as it stands,
    ///  in one write and one durability sync,
    ///  and answer the record that stands beside the checkout.
    /// Why:
    ///   The appended lines are cheap but they grow with every replacement;
    ///  the pass writes the record once when it ends,
    ///  which is what keeps what a person waits on from growing with the number of files a pass replaces.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// parking.finish();
    /// ```
    fn finish(&mut self) -> std::collections::BTreeMap<PathBuf, PathBuf> {
        self.journal = None;
        if self.touched && self.failed.is_none() {
            let mut text: String = String::new();
            for (parked, original) in &self.lines {
                text.push_str(parked.to_string_lossy().as_ref());
                text.push('\t');
                text.push_str(original.to_string_lossy().as_ref());
                text.push('\n');
            }
            if let Err(failure) = MigrateError::from_io_result(write_beside_then_rename(self.path, text.as_bytes()), "write the list of parked copies", self.path) {
                self.failed = Some(failure);
            }
        }
        return self.lines.clone();
    }
}

impl crate::subvolume_sync_pass_state_and_checks::Parking for TheParking<'_> {
    fn the_old_tree_read_like(&self) -> Option<&crate::subvolume_old_tree_record::OldTreeRecord> {
        return self.old_tree;
    }
    fn keeps_what_it_replaces(&self) -> bool {
        return self.keeps_what_it_replaces;
    }
    fn about_to_park(&mut self, original: &Path, parked: &Path) {
        self.lines.insert(parked.to_path_buf(), original.to_path_buf());
        self.touched = true;
        let line: String = format!("{}\t{}\n", parked.to_string_lossy(), original.to_string_lossy());
        self.append(line.as_str());
    }
    fn judged(&mut self, parked: &Path) {
        if self.lines.remove(parked).is_some() {
            self.touched = true;
            let line: String = format!("-\t{}\n", parked.to_string_lossy());
            self.append(line.as_str());
        }
    }
    fn known(&self) -> &std::collections::BTreeMap<PathBuf, PathBuf> {
        return &self.lines;
    }
}

/// What:
///  The parked copies the record beside the checkout names,
///  read line by line as `parked<TAB>original`.
/// Why:
///   A line the record cannot parse names nothing,
///  and a name that leads nowhere is dropped by the next pass that writes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parked = readParked(path); // Map<parked, original>
/// ```
fn read_the_parked_record(path: &Path) -> std::io::Result<std::collections::BTreeMap<PathBuf, PathBuf>> {
    let file: std::fs::File = crate::plain_file::open_plain_file(path)?;
    let mut reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(file);
    let mut lines: std::collections::BTreeMap<PathBuf, PathBuf> = std::collections::BTreeMap::new();
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
        let text: String = String::from_utf8_lossy(line.as_slice()).into_owned();
        let Some((parked, original)) = text.split_once('\t') else { continue };
        if parked.is_empty() {
            continue;
        }
        // A judged copy is named with no path after the tab, in the records a pass appends to as it runs; the record it
        //  writes when it ends holds only the copies that still stand.
        if parked == "-" {
            lines.remove(&PathBuf::from(original));
            continue;
        }
        if original.is_empty() {
            continue;
        }
        lines.insert(PathBuf::from(parked), PathBuf::from(original));
    }
    return Ok(lines);
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
///  What the pass is told about the record of parked copies and of the old tree.
/// Why:
///   Two facts the run keeps decide how one pass behaves,
///  and they travel together so a caller cannot set one and forget the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Told = { keepsWhatItReplaces: boolean; carriesWhatChangedSinceTheRecord: boolean };
/// ```
#[derive(Clone, Copy, Default)]
struct Told {
    /// What the pass replaces stays at the parked name, named, instead of being removed.
    keeps_what_it_replaces: bool,
    /// The pass carries what was written into the old tree since the migration, which the record of the old tree names.
    carries_what_changed_since_the_record: bool,
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
    /// The parked copies the record beside the checkout names,
    /// by the name each stands under and the path it is a copy of.
    pub parked: std::collections::BTreeMap<PathBuf, PathBuf>,
    /// What the old tree read like when the migration last read it,
    /// read from the record beside the checkout;
    /// none for a checkout an earlier version of this command migrated,
    /// which has no record.
    pub old_tree: Option<crate::subvolume_old_tree_record::OldTreeRecord>,
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
        // A record that cannot be read here is asked again by the deletion, as the list of entries no pass carried is.
        let parked: std::collections::BTreeMap<PathBuf, PathBuf> = read_the_parked_record(layout.parked_path.as_path()).unwrap_or_default();
        let old_tree: Option<crate::subvolume_old_tree_record::OldTreeRecord> = match crate::subvolume_old_tree_record::OldTreeRecord::read(layout.old_tree_record_path.as_path()) {
            Ok(record) => record,
            Err(error) => return Err(MigrateError::io("read the reading of the old tree", layout.old_tree_record_path.as_path(), &error)),
        };
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
            parked,
            old_tree,
            recorded,
        });
    }

    /// What:
    ///  Write the record of what the old tree read like,
    ///  from the pass that last read it,
    ///  and keep it for this run.
    /// Why:
    ///   The record is the only evidence a later deletion can have that an entry of the old tree is the one the
    ///  migration carried and not one somebody wrote there afterwards,
    ///  and the catch-up is the last pass of the migration that reads the old tree.
    ///  It is written before the step that says the catch-up finished,
    ///  so a kill between them repeats the catch-up rather than leaving an old tree nothing has a reading of.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await run.writeTheOldTreeRecord(catchUpStats);
    /// ```
    pub fn write_the_old_tree_record(&mut self, stats: &SyncStats) -> Result<(), MigrateError> {
        let record: crate::subvolume_old_tree_record::OldTreeRecord = crate::subvolume_old_tree_record::OldTreeRecord::of(stats.source_readings.clone());
        record.write(self.layout.old_tree_record_path.as_path())?;
        self.old_tree = Some(record);
        return Ok(());
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
        return self.pass_core(label, source, destination, SyncOptions { never_replaces: false,  delete, update_only, excluded: &[], created_since: None }, Told::default());
    }

    /// What:
    ///  Run the carry back of a rollback:
    ///  put what the new root holds at every path where the two differ,
    ///  keep what it replaced at the parked name,
    ///  and name what it could not place.
    /// Why:
    ///   A rollback keeps the original directory, which holds the bytes from before the migration,
    ///  and deletes the new subvolume, which holds the person's newer work:
    ///  so where the two differ, the person's bytes are the ones that have to reach the kept tree,
    ///  and what stood there (the older copy, which the migration also carried into the new tree and Git's history)
    ///  is parked rather than removed.
    ///  Nothing is kept back on a modification time:
    ///  a file the checkout holds with an older timestamp and newer bytes is the person's.
    ///  What the pass cannot place is named, and the proof against the subvolume the rollback deletes names it too.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await run.passCarryBack('carry back', newRoot, oldTree);
    /// ```
    pub fn pass_carry_back(&mut self, label: &str, source: &Path, destination: &Path) -> Result<SyncStats, MigrateError> {
        let stats: SyncStats = self.pass_core(
            label,
            source,
            destination,
            SyncOptions { never_replaces: false,  delete: false, update_only: false, excluded: &[], created_since: None },
            Told { keeps_what_it_replaces: true, carries_what_changed_since_the_record: false },
        )?;
        for path in &stats.copied_sample {
            self.line(format!("{label}: carried {}", path.display()).as_str());
        }
        return Ok(stats);
    }

    /// What:
    ///  Run the pass that reads the old tree before it is deleted and carries what was written into it since the
    ///  migration:
    ///  an entry the record still reads the same is left alone entirely,
    ///  an entry the checkout lacks is created,
    ///  and an entry the checkout holds with other bytes is replaced only when the old tree's is newer,
    ///  with what it replaced parked at a name the run gives:
    ///  the tree this pass writes into is the one the person keeps, so a byte of it is never deleted for a newer one.
    /// Why:
    ///   Programs that kept a descriptor in the old tree can write there until the person stops them,
    /// and the deletion
    ///  of the old tree must not be the end of those writes.
    /// The record says which entries were written into the old tree,
    /// so the checkout's own later work is never replaced on a modification time:
    /// that is what deleted a file a person had restored in the checkout with an older timestamp.
    /// Nothing is written over an entry the checkout holds:
    /// what the pass cannot place is named, and the deletion keeps it out of the old tree rather than deleting it.
    /// Unreadable files found are recorded like any other pass,
    /// and the entries no pass carried are not:
    /// the deletion answers for those itself, by keeping them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const stats = await run.passLate('late writes', oldTree, root, record);
    /// ```
    pub fn pass_late(&mut self, label: &str, source: &Path, destination: &Path, since: i64) -> Result<SyncStats, MigrateError> {
        let carries_the_record: bool = self.old_tree.is_some();
        let stats: SyncStats = self.pass_core(
            label,
            source,
            destination,
            SyncOptions { never_replaces: false,  delete: false, update_only: true, excluded: &[], created_since: Some(since) },
            Told { keeps_what_it_replaces: true, carries_what_changed_since_the_record: carries_the_record },
        )?;
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
    fn pass_core(&mut self, label: &str, source: &Path, destination: &Path, requested: SyncOptions<'_>, told: Told) -> Result<SyncStats, MigrateError> {
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
            let mut parking: TheParking<'_> = TheParking {
                path: self.layout.parked_path.as_path(),
                lines: self.parked.clone(),
                journal: None,
                touched: false,
                failed: None,
                old_tree: told.carries_what_changed_since_the_record.then_some(&self.old_tree).and_then(|held| return held.as_ref()),
                keeps_what_it_replaces: told.keeps_what_it_replaces,
            };
            let mut note_uncarried = |path: &Path, _kind: Uncarried| return record.note(path);
            let stats: SyncStats = sync_tree(
                self.platform,
                source,
                destination,
                SyncOptions { excluded: excluded.as_slice(), ..requested },
                self.interrupt,
                Some(&mut note_uncarried),
                Some(&mut parking),
            )?;
            if let Some(failure) = record.failed.take() {
                return Err(failure);
            }
            self.parked = parking.finish();
            if let Some(failure) = parking.failed.take() {
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
