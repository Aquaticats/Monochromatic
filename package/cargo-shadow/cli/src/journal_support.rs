//! Helpers the journal tests share: planting the journal of a killed run and finding journals.

use std::fs;
use std::path::{Path, PathBuf};

use crate::crates::LockSnapshot;
use crate::journal::{JOURNAL_DIR, begin};
use crate::manifest_edit::ManifestEdit;

/// Plants the journal of a run that was killed after it journaled its snapshots (and manifests): the transaction is begun and then
/// dropped, so nothing holds it any more and nothing finished it.
pub fn plant(root: &Path, snapshots: &[LockSnapshot], manifests: &[ManifestEdit]) -> PathBuf {
    let before: Vec<PathBuf> = journal_dirs(root);
    let transaction = begin(root, snapshots, !manifests.is_empty()).expect("begin");
    if !manifests.is_empty() {
        transaction.add_manifests(manifests).expect("journals the manifests");
    }
    drop(transaction);
    let mut added: Vec<PathBuf> = journal_dirs(root).into_iter().filter(|path| { return !before.contains(path); }).collect();
    assert_eq!(added.len(), 1, "expected exactly one new journal: {added:?}");
    return added.remove(0);
}

/// Every journal directory under the root (completed ones are gone; claims and `.done` directories count too).
pub fn journal_dirs(root: &Path) -> Vec<PathBuf> {
    let directory: PathBuf = root.join(JOURNAL_DIR);
    let Ok(listing) = fs::read_dir(&directory) else {
        return Vec::new();
    };
    return listing.flatten().map(|entry| { return entry.path(); }).collect();
}

/// The only journal directory under the root.
pub fn journal_dir(root: &Path) -> PathBuf {
    let mut all: Vec<PathBuf> = journal_dirs(root);
    assert_eq!(all.len(), 1, "expected exactly one journal: {all:?}");
    return all.remove(0);
}
