//! What:
//!  The frozen proof: the clone's copied index and every tracked path compared
//!  with the capture taken just before the snapshot, before any byte of the
//!  clone is written.
//! Why:
//!  The snapshot is one instant; a write between the capture and the snapshot
//!  moves a file after the capture and before the clone exists, so only the
//!  clone's own bytes can settle it
//!  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "The frozen proof": D1 and D2, with D3 and D4 the digests step 11 adds).
//!  No Git run reads the copied administration: a Git run there would treat the
//!  copied index as its own and could rewrite it and move the copied refs.
//!  A difference names the path and the field, so a person reads which write
//!  refused the add.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! proveTheClone(capture, cloneRoot); // throws with the path and the field
//! ```

use super::add_capture::{CapturedEntry, CapturedKind, Capture};
use super::add_capture_tree::{FoundFacts, walk_paths};
use crate::private_storage::read_regular_file;
use crate::subvolume_sha::sha256;
use std::path::{Path, PathBuf};

/// What:
///  Prove one clone against one capture.
/// Why:
///  D1 compares the copied index's bytes first, so a staged change or a commit
///  in the window refuses before any file is read, and D2 compares every
///  tracked path's own facts.
///
/// # Errors
/// The clone's index differs,
/// a tracked path's kind,
/// size,
/// times or execute bit differs,
/// or a link's target differs.
pub fn prove_the_clone(capture: &Capture, clone_root: &Path) -> Result<(), String> {
    let index: PathBuf = clone_root.join(".git").join("index");
    let bytes: Vec<u8> = read_regular_file(index.as_path(), capture.index.size.saturating_add(1)).map_err(|refusal| {
        return format!(
            "the clone's copied index cannot be read ({refusal:?}), and the capture recorded {} bytes with SHA-256 {}. Ways forward: run the add again with nothing writing the main checkout, or make the worktree with Git's own checkout",
            capture.index.size,
            hexadecimal(capture.index.digest.as_slice())
        );
    })?;
    if bytes.len() as u64 != capture.index.size || sha256(bytes.as_slice()) != capture.index.digest {
        return Err(format!(
            "the clone's copied index differs in its length and SHA-256: the clone holds {} bytes with SHA-256 {}, and the capture recorded {} bytes with SHA-256 {}. Ways forward: run the add again with nothing writing the main checkout, or make the worktree with Git's own checkout",
            bytes.len(),
            hexadecimal(sha256(bytes.as_slice()).as_slice()),
            capture.index.size,
            hexadecimal(capture.index.digest.as_slice())
        ));
    }
    return walk_paths(clone_root, capture.entries.iter(), |entry| return entry.path.as_slice(), |entry, path, found| return compare_entry(entry, path, found));
}

/// Compare one captured entry with the clone's own facts.
fn compare_entry(entry: &CapturedEntry, _path: &Path, found: &FoundFacts) -> Result<(), String> {
    let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
    let refused = |field: &str, got: String, wanted: String| -> String {
        return format!(
            "{shown} differs in its {field}: the clone holds {got}, and the capture recorded {wanted}. Ways forward: run the add again with nothing writing the main checkout, or make the worktree with Git's own checkout"
        );
    };
    if found.kind.word() != entry.kind.word() {
        return Err(refused("kind", String::from(found.kind.word()), String::from(entry.kind.word())));
    }
    if entry.kind == CapturedKind::Absent {
        return Ok(());
    }
    let pairs: [(&str, String, String); 4] = [
        ("size", found.size.to_string(), entry.size.to_string()),
        ("modification time", format!("{}.{}", found.mtime.0, found.mtime.1), format!("{}.{}", entry.mtime.0, entry.mtime.1)),
        ("change time", format!("{}.{}", found.ctime.0, found.ctime.1), format!("{}.{}", entry.ctime.0, entry.ctime.1)),
        ("execute bit", u8::from(found.executable).to_string(), u8::from(entry.executable).to_string()),
    ];
    for (field, got, wanted) in pairs {
        if got != wanted {
            return Err(refused(field, got, wanted));
        }
    }
    if entry.kind == CapturedKind::Link && found.link_target.as_deref() != entry.link_target.as_deref() {
        let word = |bytes: Option<&[u8]>| -> String {
            return bytes.map_or_else(|| return String::from("none"), |target| return String::from_utf8_lossy(target).into_owned());
        };
        return Err(refused("link target", word(found.link_target.as_deref()), word(entry.link_target.as_deref())));
    }
    return Ok(());
}

/// Bytes as the person reads them in a proof's refusal.
fn hexadecimal(bytes: &[u8]) -> String {
    return crate::worktree_clone_binding_record::binding_encoding::bytes_as_hexadecimal(bytes);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_frozen_proof_tests.rs"]
mod tests;
