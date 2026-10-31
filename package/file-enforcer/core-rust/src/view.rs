//! What:
//!  The virtual view of the worktree that rules plan against: the real files plus the outputs earlier rules planned.
//! Why:
//!  Downstream rules read upstream rules' new content (a catalog rule sees the Cargo.toml a key rule just changed),
//!  and every read is recorded, so the run can later check that nothing it read changed before it publishes
//!  (snapshot revalidation) and so `explain` and `watch` know the exact inputs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class View { readText(rel: string): string | undefined; list(rel: string): [string, EntryKind][]; }
//! ```

use crate::diagnostic::Diagnostic;
use crate::real_fs::{EntryKind, RealState, list_real, read_real, sha256_hex};
use crate::rule::Change;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// What:
///  A planned state of one path, as an earlier rule left it.
/// Why:
///  The overlay stores these; a removal hides the real file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Planned = { kind: 'file'; content: Uint8Array; mode?: number } | { kind: 'removed' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Planned {
    /// The path will hold these bytes (and this mode when one is set).
    File {
        /// Planned content.
        content: Vec<u8>,
        /// Planned permission bits, when the rule sets them.
        mode: Option<u32>,
    },
    /// The path will not exist.
    Removed,
}

/// What:
///  What the run observed about one real path, compact enough to compare later.
/// Why:
///  Revalidation re-observes the path and compares; content is kept as a digest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Observed = { kind: 'absent' } | { kind: 'file'; digest: string; mode: number } | { kind: 'directory' } | { kind: 'other' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Observed {
    /// Nothing there.
    Absent,
    /// A file with this content digest and mode.
    File {
        /// SHA-256 of the content.
        digest: String,
        /// Permission bits.
        mode: u32,
    },
    /// A directory, a link or a special file.
    NotAFile(String),
}

/// What:
///  The view: root, overlay, and the record of what was read.
///  `BTreeMap` is a key-sorted map (TS: `Map`, but iteration is always sorted).
/// Why:
///  Sorted maps keep every walk and every report deterministic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class View { root: string; overlay: Map<string, Planned>; files: Map<string, Observed>; dirs: Map<string, string>; }
/// ```
#[derive(Debug)]
pub struct View {
    root: PathBuf,
    overlay: BTreeMap<String, Planned>,
    files: BTreeMap<String, Observed>,
    dirs: BTreeMap<String, String>,
    accesses: Vec<(usize, String)>,
    current_rule: usize,
}

impl View {
    /// What:
    ///  Start a view over a worktree root with an empty overlay.
    /// Why:
    ///  Each run begins from the files as they are.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new View(root)
    /// ```
    pub fn new(root: &Path) -> View {
        return View {
            root: root.to_path_buf(),
            overlay: BTreeMap::new(),
            files: BTreeMap::new(),
            dirs: BTreeMap::new(),
            accesses: Vec::new(),
            current_rule: 0,
        };
    }

    /// The worktree root this view reads.
    pub fn root(&self) -> &Path {
        return &self.root;
    }

    /// What:
    ///  Tell the view which rule (by position) reads next.
    /// Why:
    ///  Access records name the reading rule so the run can catch a rule reading a later rule's output.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// beginRule(index: number): void
    /// ```
    pub fn begin_rule(&mut self, index: usize) {
        self.current_rule = index;
    }

    /// Every (rule position, path) pair read so far, in order.
    pub fn accesses(&self) -> &[(usize, String)] {
        return &self.accesses;
    }

    /// The overlay of planned paths, sorted by path.
    pub fn overlay(&self) -> &BTreeMap<String, Planned> {
        return &self.overlay;
    }

    /// What:
    ///  Record that the current rule applied a change, updating the overlay.
    /// Why:
    ///  Later rules read the planned content through `read_bytes`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// apply(change: Change): void
    /// ```
    pub fn apply(&mut self, change: &Change) -> Result<(), Diagnostic> {
        match change {
            Change::Write { path, content, mode } => {
                self.overlay.insert(path.clone(), Planned::File { content: content.clone(), mode: *mode });
            }
            Change::Remove { path } => {
                self.overlay.insert(path.clone(), Planned::Removed);
            }
            Change::SetMode { path, mode } => {
                if let Some(content) = self.read_bytes(path)? {
                    self.overlay.insert(path.clone(), Planned::File { content, mode: Some(*mode) });
                }
            }
            Change::RemoveDirectory { .. } => {}
        }
        return Ok(());
    }

    /// What:
    ///  Read a file's bytes as the run will see them: planned content first, then the real file.
    ///  `Option<Vec<u8>>` is "bytes or nothing" (TS: `Uint8Array | undefined`).
    /// Why:
    ///  `None` means the path does not exist (or is planned for removal).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// readBytes(rel: string): Uint8Array | undefined
    /// ```
    pub fn read_bytes(&mut self, rel: &str) -> Result<Option<Vec<u8>>, Diagnostic> {
        self.accesses.push((self.current_rule, rel.to_string()));
        match self.overlay.get(rel) {
            Some(Planned::File { content, .. }) => return Ok(Some(content.clone())),
            Some(Planned::Removed) => return Ok(None),
            None => {}
        }
        match self.observe_file(rel)? {
            RealState::File { content, .. } => return Ok(Some(content)),
            RealState::Absent => return Ok(None),
            RealState::Directory => {
                let message = format!("{rel} is a directory where a file was expected");
                return Err(Diagnostic::new("path-blocked", &message).with_path(rel));
            }
            RealState::Symlink | RealState::Other => {
                let message = format!("{rel} is a symbolic link or special file; it is never followed or read");
                return Err(Diagnostic::new("symlink-rejected", &message).with_path(rel));
            }
        }
    }

    /// What:
    ///  Read a file as UTF-8 text.
    /// Why:
    ///  Every input except the mirror's is text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// readText(rel: string): string | undefined
    /// ```
    pub fn read_text(&mut self, rel: &str) -> Result<Option<String>, Diagnostic> {
        match self.read_bytes(rel)? {
            None => return Ok(None),
            Some(bytes) => match String::from_utf8(bytes) {
                Ok(text) => return Ok(Some(text)),
                Err(_) => {
                    let message = format!("{rel} is not valid UTF-8 text");
                    return Err(Diagnostic::new("not-utf8", &message).with_path(rel));
                }
            },
        }
    }

    /// What:
    ///  The mode a path will have: the planned mode when set, else the real file's.
    /// Why:
    ///  Mode-only rules and copies need the current bits.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// modeOf(rel: string): number | undefined
    /// ```
    pub fn mode_of(&mut self, rel: &str) -> Result<Option<u32>, Diagnostic> {
        self.accesses.push((self.current_rule, rel.to_string()));
        if let Some(Planned::File { mode: Some(mode), .. }) = self.overlay.get(rel) {
            return Ok(Some(*mode));
        }
        if let Some(Planned::Removed) = self.overlay.get(rel) {
            return Ok(None);
        }
        match self.observe_file(rel)? {
            RealState::File { mode, .. } => return Ok(Some(mode)),
            _ => return Ok(None),
        }
    }

    /// What:
    ///  Read the real state of a path and remember how it looked.
    /// Why:
    ///  The first observation of each path is what revalidation compares against; outputs' current state is
    ///  observed through here too.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// observeFile(rel: string): RealState
    /// ```
    pub fn observe_file(&mut self, rel: &str) -> Result<RealState, Diagnostic> {
        let state = read_real(&self.root, rel)?;
        if !self.files.contains_key(rel) {
            self.files.insert(rel.to_string(), observed_of(&state));
        }
        return Ok(state);
    }

    /// What:
    ///  List a directory as the run will see it: real entries plus planned files, minus planned removals.
    /// Why:
    ///  Globs and mirrors walk through here, so planned outputs are visible and every listing is recorded.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// list(rel: string): [string, EntryKind][]
    /// ```
    pub fn list(&mut self, rel: &str) -> Result<Vec<(String, EntryKind)>, Diagnostic> {
        self.accesses.push((self.current_rule, rel.to_string()));
        let real = list_real(&self.root, rel)?;
        let mut digest_input = String::new();
        let mut merged: BTreeMap<String, EntryKind> = BTreeMap::new();
        for (name, kind) in real {
            digest_input.push_str(&format!("{name}\0{kind:?}\n"));
            merged.insert(name, kind);
        }
        if !self.dirs.contains_key(rel) {
            self.dirs.insert(rel.to_string(), sha256_hex(digest_input.as_bytes()));
        }
        let prefix = if rel.is_empty() { String::new() } else { format!("{rel}/") };
        for (path, planned) in &self.overlay {
            let Some(rest) = path.strip_prefix(&prefix) else { continue };
            let (name, deeper) = match rest.split_once('/') {
                Some((head, _)) => (head, true),
                None => (rest, false),
            };
            if deeper {
                if matches!(planned, Planned::File { .. }) {
                    merged.insert(name.to_string(), EntryKind::Directory);
                }
            } else if matches!(planned, Planned::Removed) {
                merged.remove(name);
            } else {
                merged.insert(name.to_string(), EntryKind::File);
            }
        }
        return Ok(merged.into_iter().collect());
    }

    /// What:
    ///  Re-observe every real path this run read and report the ones that changed.
    /// Why:
    ///  Snapshot revalidation: a plan built on files that changed since is discarded and redone.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// changedPaths(): string[]
    /// ```
    pub fn changed_paths(&self) -> Result<Vec<String>, Diagnostic> {
        let mut changed: Vec<String> = Vec::new();
        for (path, before) in &self.files {
            let now = observed_of(&read_real(&self.root, path)?);
            if &now != before {
                changed.push(path.clone());
            }
        }
        for (path, before) in &self.dirs {
            let real = list_real(&self.root, path)?;
            let mut digest_input = String::new();
            for (name, kind) in real {
                digest_input.push_str(&format!("{name}\0{kind:?}\n"));
            }
            if &sha256_hex(digest_input.as_bytes()) != before {
                changed.push(format!("{path}/"));
            }
        }
        return Ok(changed);
    }

    /// What:
    ///  Every real path and directory the run read, for `watch` and `explain`.
    /// Why:
    ///  The watched set is exactly what the run depended on, absent paths included.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// readPaths(): { files: string[]; directories: string[] }
    /// ```
    pub fn read_paths(&self) -> (Vec<String>, Vec<String>) {
        return (self.files.keys().cloned().collect(), self.dirs.keys().cloned().collect());
    }
}

/// What:
///  Reduce a real state to what revalidation compares.
/// Why:
///  A digest is enough to notice a changed file without keeping its bytes twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function observedOf(state: RealState): Observed
/// ```
fn observed_of(state: &RealState) -> Observed {
    match state {
        RealState::Absent => return Observed::Absent,
        RealState::File { content, mode } => return Observed::File { digest: sha256_hex(content), mode: *mode },
        RealState::Directory => return Observed::NotAFile("directory".to_string()),
        RealState::Symlink => return Observed::NotAFile("symlink".to_string()),
        RealState::Other => return Observed::NotAFile("other".to_string()),
    }
}
