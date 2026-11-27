//! What:
//! The transaction handle of a run:
//! a journal directory of lockfile and manifest snapshots,
//! kept alive by an exclusive lock
//! that the run holds for as long as it lives.
//! Why:
//! The reviews of 2026-11-03 showed that naming a journal by process id and deciding liveness from `/proc` fails open
//! (a reused id,
//! an unreadable `/proc`,
//! a recorded identity from another namespace).
//! The kernel releases a lock when its holder dies,
//! so "the lock can be taken" is the proof that the owner is gone,
//! and trying for it never waits (owner rule of 2026-11-03:
//! "nothing should ever wait forever or indefinitely").
//! Every journal has its own unique name and is created,
//! never reused;
//! a `ready` record,
//! written last,
//! lists every file with its
//! length,
//! so a recovery can tell a complete journal from a partial one.
//! How a journal is made (its name,
//! its ready record,
//! the workspace lock taken first) lives in `journal_transaction_creation.rs`;
//! this file keeps the handle those rules fill in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal-transaction.ts
//! export function begin(root: string, snapshots: LockSnapshot[], expectsManifests: boolean): Transaction
//! export class Transaction { addManifests(edits: ManifestEdit[]): void; finish(): void }
//! ```

/// Imports `std::fs::{self, File}` for this file.
use std::fs::{
    self,
    File,
};
/// Imports `std::io` for this file.
use std::io;
/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{
    Path,
    PathBuf,
};

/// Imports `crate::bounded_run::RunScope` for this file.
use crate::bounded_run::RunScope;
/// Imports `crate::crates::LockSnapshot` for this file.
use crate::crates::LockSnapshot;
// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports `crate::constants::LOCKFILE_NAME` for this file.
use crate::constants::LOCKFILE_NAME;
/// Imports the journal's names and its durable writes for this file.
use crate::journal::{
    GROUPS_FILE,
    INDEX_FILE,
    JOURNAL_DIR,
    MANIFEST_INDEX_FILE,
    OWNER_FILE,
    atomic_write,
    create_directories_synced,
    make_durable,
    sync_directory,
    write_synced,
};
// Imports `crate::manifest_edit::ManifestEdit`.
/// Imports `crate::manifest_edit::ManifestEdit` for this file.
use crate::manifest_edit::ManifestEdit;

/// Imports `crate::journal_finish::FinishError` for this file.
use crate::journal_finish::FinishError;

/// The hook that only the tests of the transaction use.
#[cfg(test)]
pub(crate) use crate::journal_transaction_creation::begin_step_for_test;
/// The steps `begin` takes,
/// and the hook a test stops it at (split out of the journal transaction module).
use crate::journal_transaction_creation::{
    MAX_JOURNAL_NAME_ATTEMPTS,
    at_step,
    journal_name,
    write_ready,
};
/// The workspace lock,
/// the journal's unique name,
/// the index text and the ready record (split out of the journal transaction module).
pub use crate::journal_transaction_creation::{
    encode_path,
    take_workspace,
    try_lock,
};

/// The notices about files a journal keeps beside the lockfiles (split out of the journal transaction module).
pub(crate) use crate::journal_kept_notices::{
    announce_kept_files_for,
    announce_kept_notice,
    kept_folder_notice,
};

/// The completion marker of a journal (split out of the journal transaction module).
pub(crate) use crate::journal_done_marker::finish_journal;
/// The completion marker item that only the tests of the transaction use.
#[cfg(test)]
pub(crate) use crate::journal_done_marker::publish_done;

/// A transaction held by its owner:
/// the workspace lock and the journal.
/// Dropping it without calling `finish` is what a killed run looks like to a recovery:
/// the journal stays,
/// the lock is free,
/// and the control groups of the children it started are recorded in the journal.
pub struct Transaction {
    /// The repository root.
    root: PathBuf,
    /// The journal directory.
    directory: PathBuf,
    /// The locked workspace file;
    /// closing it (by drop or by death) releases the lock.
    _workspace: File,
    /// The files already listed for the ready record.
    listed: std::cell::RefCell<Vec<String>>,
    /// The crate roots whose lockfiles this transaction covers.
    crate_dirs: Vec<String>,
    /// The manifests journaled for it.
    manifests: std::cell::RefCell<Vec<PathBuf>>,
    /// Whether the manifests were journaled already.
    manifests_added: std::cell::Cell<bool>,
    /// The notice that the kept folder is large,
    /// when it is.
    kept_notice: Option<String>,
    /// Where the control groups of this transaction's children are recorded,
    /// and whether any of them left writers behind;
    /// it also keeps the transaction on the thread that began it.
    scope: RunScope,
}

/// What:
/// `pub fn begin(root, snapshots, expects_manifests) -> io::Result<Transaction>` takes the workspace lock,
/// creates a new journal
/// directory,
/// writes the snapshots and,
/// when no manifests are expected,
/// the ready record.
/// Why:
/// Nothing may be mutated without a complete recovery record,
/// and no other transaction or recovery may run meanwhile.
/// The directory is created,
/// never reused,
/// so an earlier journal's snapshots are never truncated.
/// The control group of every child this thread starts is recorded in the journal before the child starts,
/// so a recovery can see a survivor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function begin(root: string, snapshots: LockSnapshot[], expectsManifests: boolean): Transaction
/// ```
pub fn begin(
    root: &Path,
    snapshots: &[LockSnapshot],
    expects_manifests: bool,
) -> io::Result<Transaction> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `workspace: File`.
    let workspace: File = match take_workspace(root)? {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(held) => held,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(message) => return Err(io::Error::other(message)),
    };
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    let crate_roots: Vec<String> = snapshots.iter().map(LockSnapshot::crate_dir_text).collect();
    let kept_notice: Option<String> = kept_folder_notice(root, &crate_roots);
    // A child of an earlier killed run may still be writing the files this transaction would snapshot:
    // starting now would let two runs mutate one workspace, so the transaction is refused.
    let earlier: Vec<PathBuf> = match crate::journal_recover::list_journals(&root.join(JOURNAL_DIR))
    {
        Ok(Some(listing)) => listing.open,
        // Only an absent journal directory is "no earlier journals"; an entry that vanished mid-listing is unknown (7b P1-2).
        Ok(None) => Vec::new(),
        Err(error) => {
            return Err(io::Error::other(format!(
                "the journals of {} could not be listed ({error}), so a surviving child of an earlier run cannot be ruled out; no transaction was started",
                root.display()
            )));
        }
    };
    for path in earlier {
        if let Some(note) = crate::journal_recover::survivor_note(&path) {
            return Err(io::Error::other(note));
        }
    }
    let journals: PathBuf = root.join(JOURNAL_DIR);
    create_directories_synced(&journals)?;
    let mut created: Option<PathBuf> = None;
    for _attempt in 0..MAX_JOURNAL_NAME_ATTEMPTS {
        let candidate: PathBuf = journals.join(journal_name());
        match fs::create_dir(&candidate) {
            Ok(()) => {
                created = Some(candidate);
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    let Some(directory) = created else {
        return Err(io::Error::other(format!(
            "{} could not be given a new journal name; run the command again",
            journals.display()
        )));
    };
    at_step("directory");
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&directory.join`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    // The record of children comes first: a journal directory without it holds nothing that could have started a child
    // (an empty directory), and one that has other files but lost it is uncertain (round 7, 6b2 2).
    write_synced(&directory.join(GROUPS_FILE), "")?;
    at_step("groups");
    // How this journal directory is seen now (mount and inode); a recovery that sees another view refuses (contract assumption c).
    // With the mount view go the machine id and the boot id: a recovery after a reboot of the same machine knows that the run's
    // processes ended (contract assumption c). An identity that cannot be read is not recorded as a guess: the journal then has none,
    // and a recovery treats that as uncertainty.
    let identity_text: String = match crate::control_group::current_identity() {
        Ok(identity) => format!("boot {}\nmachine {}\n", identity.boot, identity.machine),
        Err(_) => String::new(),
    };
    write_synced(
        &directory.join(crate::journal::VIEW_FILE),
        &format!(
            "view {}\n{identity_text}",
            crate::control_group::view_of(&directory)?
        ),
    )?;
    at_step("view");
    write_synced(
        &directory.join(OWNER_FILE),
        &format!("v3\n{}\n", std::process::id()),
    )?;
    at_step("owner");
    let mut index: String = String::from("v2\n");
    let mut listed: Vec<String> = Vec::new();
    for (position, snapshot) in snapshots.iter().enumerate() {
        let name: String = format!("{position}.lock");
        write_synced(&directory.join(&name), &snapshot.source)?;
        at_step(&format!("lock-{position}"));
        listed.push(name);
        index.push_str(&snapshot.crate_dir);
        index.push('\n');
    }
    index.push_str(&format!("end {}\n", snapshots.len()));
    atomic_write(&directory.join(INDEX_FILE), &index)?;
    at_step("index");
    listed.insert(0, INDEX_FILE.to_string());
    if !expects_manifests {
        write_ready(&directory, &listed, false)?;
        at_step("ready");
    }
    // The journal and then the name that lists it are made durable before any recorded child can start. A test replaces the result of
    // each barrier, so a failure takes the path of a failure of the kernel (round 21, sixth pass, V3).
    #[cfg(test)]
    let own_barrier: io::Result<()> =
        if crate::failpoints::fires(crate::failpoints::Point::JournalDirectorySync) {
            Err(io::Error::other(
                "injected failure to make the journal directory durable",
            ))
        } else {
            sync_directory(&directory)
        };
    #[cfg(not(test))]
    let own_barrier: io::Result<()> = sync_directory(&directory);
    own_barrier?;
    #[cfg(test)]
    crate::bounded_run::trace(format_args!("journal directory synced"));
    #[cfg(test)]
    let parent_barrier: io::Result<()> =
        if crate::failpoints::fires(crate::failpoints::Point::JournalParentSync) {
            Err(io::Error::other(
                "injected failure to make the journal's name durable",
            ))
        } else {
            sync_directory(&journals)
        };
    #[cfg(not(test))]
    let parent_barrier: io::Result<()> = sync_directory(&journals);
    parent_barrier?;
    #[cfg(test)]
    crate::bounded_run::trace(format_args!("journal published"));
    // `Some`: the present or the absent value of an `Option`.
    let scope: RunScope = RunScope::open(directory.join(GROUPS_FILE));
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(Transaction {
        root: root.to_path_buf(),
        directory,
        _workspace: workspace,
        // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
        listed: std::cell::RefCell::new(listed),
        // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
        // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Array.from(iterable)  // or [...iterable]
        // ```
        // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
        // Why:      The collection stays usable after the walk and no item is copied until it must be.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const item of list) { ... }  // or list.map(...)
        // ```
        // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
        // Why:      They express one step of a pipeline without an index loop.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // list.map(f).filter(g)
        // ```
        crate_dirs: snapshots.iter().map(LockSnapshot::crate_dir_text).collect(),
        // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
        manifests: std::cell::RefCell::new(Vec::new()),
        // Constructor `Cell::new(`: an empty owned collection or a boxed value.
        manifests_added: std::cell::Cell::new(false),
        kept_notice,
        scope,
    });
}

/// The methods of the type above.
impl Transaction {
    /// What:
    /// `pub fn kept_notice(&self) -> Option<&str>` is the warning to print when the kept folder holds many files.
    /// Why:
    /// The owner decided to warn past a count and never to refuse.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// keptNotice(): string | undefined
    /// ```
    pub fn kept_notice(&self) -> Option<&str> {
        return self.kept_notice.as_deref();
    }

    /// What:
    /// `pub fn writers_may_remain(&self) -> bool` says whether a run of this transaction could not show that everything it started
    /// was stopped.
    /// Why:
    /// Then the lockfiles can be neither rolled back nor finished safely,
    /// and the command must stop with the journal kept.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// writersMayRemain(): boolean
    /// ```
    pub fn writers_may_remain(&self) -> bool {
        return self.scope.writers_may_remain();
    }

    /// What:
    /// `pub fn process_group_only(&self) -> bool` says whether a run of this transaction was contained by a process group only,
    /// because no control group could be had.
    /// Why:
    /// Quiescence was not shown for what leaves a process group.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// processGroupOnly(): boolean
    /// ```
    pub fn process_group_only(&self) -> bool {
        return self.scope.process_group_only();
    }

    /// What:
    /// `pub fn add_manifests(&self, edits) -> io::Result<()>` journals the manifests a `--breaking` run may rewrite and writes the
    /// ready record;
    /// it can be called once.
    /// Why:
    /// Their original text is the only copy once a run rewrites them,
    /// so it is synced before the run starts.
    /// The payload of a ready transaction is frozen:
    /// a second call would rewrite snapshots a recovery may rely on.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// addManifests(edits: ManifestEdit[]): void
    /// ```
    pub fn add_manifests(
        &self,
        edits: &[ManifestEdit],
    ) -> io::Result<()> {
        if self.manifests_added.replace(true) {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            return Err(io::Error::other(
                "the manifests of this transaction were journaled already; its recovery payload is not rewritten",
            ));
        }
        let mut index: String = String::from("v1\n");
        let mut listed: Vec<String> = self.listed.borrow().clone();
        for (position, edit) in edits.iter().enumerate() {
            if edit.crate_dir.contains('\n') || edit.crate_dir.contains('\t') {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "the crate root {:?} contains a line feed or tab, which the journal cannot list",
                        edit.crate_dir
                    ),
                ));
            }
            let name: String = format!("m{position}.toml");
            write_synced(&self.directory.join(&name), &edit.before)?;
            listed.push(name);
            index.push_str(&format!(
                "{}\t{}\n",
                edit.crate_dir,
                encode_path(&edit.path)
            ));
        }
        index.push_str(&format!("end {}\n", edits.len()));
        atomic_write(&self.directory.join(MANIFEST_INDEX_FILE), &index)?;
        listed.push(MANIFEST_INDEX_FILE.to_string());
        write_ready(&self.directory, &listed, true)?;
        *self.listed.borrow_mut() = listed;
        // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
        // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // (a, b) => body
        // ```
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        // `.collect()`: runs the iterator and gathers the items into the bound collection type.
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        // Adapter `.map(`: transforms or selects items with a closure.
        let mut paths: Vec<PathBuf> = Vec::new();
        for edit in edits {
            paths.push(edit.path.clone());
        }
        *self.manifests.borrow_mut() = paths;
        return sync_directory(&self.directory);
    }

    /// What:
    /// `pub fn finish(self) -> io::Result<()>` settles the transaction:
    /// the outputs are made durable,
    /// then the journal is renamed
    /// to its `.done` name (the durable completion marker) and removed.
    /// Why:
    /// The owner's rule is "Force at leaving":
    /// the lockfiles and manifests the run leaves are on disk before the snapshots that
    /// could undo them go.
    /// "Else background" has no background carrier in a command that exits,
    /// so this barrier forces what must be durable and nothing else is synced
    /// (the coordinator's pick of round 7 under the owner's delegation,
    /// veto open).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// finish(): void
    /// ```
    pub fn finish(self) -> Result<(), FinishError> {
        return self.finish_with(&make_durable);
    }

    /// What:
    /// `pub fn finish_with(self, durable) -> Result<(), FinishError>` is `finish` with the barrier that makes an output durable
    /// given by the caller (tests record the order).
    /// Why:
    /// The error says whether the outputs were forced to disk before the failure,
    /// so a caller stops the command on a failed barrier and only warns about a journal it could not remove.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// finishWith(durable: (path: string) => void): void
    /// ```
    pub fn finish_with(
        self,
        durable: &dyn Fn(&Path) -> io::Result<()>,
    ) -> Result<(), FinishError> {
        // Phase one: the outputs the run leaves are on disk before anything that could undo them goes.
        #[cfg(test)]
        if crate::failpoints::fires(crate::failpoints::Point::FinishDurability) {
            return Err(FinishError::outputs(io::Error::other(
                "injected durability failure",
            )));
        }
        for crate_dir in &self.crate_dirs {
            durable(&self.root.join(crate_dir).join(LOCKFILE_NAME))
                .map_err(FinishError::outputs)?;
        }
        for manifest in self.manifests.borrow().iter() {
            durable(manifest).map_err(FinishError::outputs)?;
        }
        // Phase two: the completion marker is durable, or the command stops (round 7, 6b2 5).
        #[cfg(test)]
        if crate::failpoints::fires(crate::failpoints::Point::FinishCompletion) {
            return Err(FinishError::completion(io::Error::other(
                "injected completion failure",
            )));
        }
        // Phase three (inside the same function the recovery uses): only the removal of a journal that is already marked done is a warning.
        return finish_journal(&self.directory);
    }
}
