//! What:
//!  The shadow repository's `packed-refs` damaged the way a power loss can damage a file that was written and never
//!  synced, at every phase a killed commit reaches, and the outcome of the next wrapped commands.
//! Why:
//!  The snapshot of the real refs inside the throwaway shadow is written without a sync on the files ref backend.
//!  The sweep of `binary_recovery_tearing_tests.rs` walks the journal, the capture store and the locks, not the
//!  shadow, so this module adds the cases that decide whether that write may stay unsynced: recovery must reach the
//!  same outcome as the undamaged commit whether the file is emptied, cut to half, zero-filled to its length, or gone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const damage of ['empty', 'half', 'zeros', 'delete']) {
//!   const paused = await pauseAt(phase); await damageFile(shadowPackedRefs(paused), damage);
//!   expect(await classify(paused)).toBe(recorded[`${phase}|(undamaged)|none`]);
//! }
//! ```

use super::{Damage, Paused, apply, classify, expected, pause_at};
use crate::commit::remove_settled;
use std::path::{Path, PathBuf};

/// The `packed-refs` file of every shadow repository under the Git directory.
fn shadow_packed_refs(git_dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(git_dir.join("cli-git/shadow"))
        .map(|entries| {
            return entries
                .flatten()
                .map(|entry| { return entry.path().join("packed-refs"); })
                .filter(|path| { return path.is_file(); })
                .collect();
        })
        .unwrap_or_default();
    found.sort();
    return found;
}

/// What:
///  Fill the file with zero bytes up to its length, the other state an unsynced extending write can leave.
/// Why:
///   A filesystem that persists a file's size before its data shows zeros, which is neither emptied nor halved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(file, Buffer.alloc((await stat(file)).size));
/// ```
fn zero_fill(file: &Path) {
    let length: usize = usize::try_from(std::fs::metadata(file).expect("the file to damage").len()).expect("length fits");
    assert!(length > 0, "{} holds the snapshot header at least", file.display());
    std::fs::write(file, vec![0_u8; length]).expect("zero fill");
}

/// What:
///  The outcome the undamaged commit has at a phase, from the recorded table.
/// Why:
///   A damaged shadow must change nothing, so the recorded control is the expected cell of every case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recorded[`${phase}|(undamaged)|none`]
/// ```
fn recorded_control(phase: &str) -> &'static str {
    let key: String = format!("{phase}|(undamaged)|none");
    return expected::OUTCOMES
        .iter()
        .find(|(name, _)| { return *name == key; })
        .map(|(_, outcome)| { return *outcome; })
        .unwrap_or_else(|| { panic!("the table records {key}"); });
}

/// What:
///  Kill a commit at `phase`, damage the shadow's `packed-refs` four ways in turn, and require the recorded control
///  outcome after each.
/// Why:
///   Each damage starts from a fresh killed commit so no case sees another's recovery.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const damage of damages) expect(await classify(await damaged(phase, damage))).toBe(control(phase));
/// ```
fn shadow_refs_do_not_change_the_outcome(phase: &str) {
    let control: &str = recorded_control(phase);
    let names: [&str; 4] = ["empty", "half", "zeros", "delete"];
    for name in names {
        let paused: Paused = pause_at(phase);
        let git_dir: PathBuf = paused.repo.join(".git");
        let files: Vec<PathBuf> = shadow_packed_refs(git_dir.as_path());
        assert_eq!(files.len(), 1, "exactly one shadow holds packed-refs at {phase}: {files:?}");
        match name {
            "zeros" => zero_fill(files[0].as_path()),
            "empty" => assert!(apply(Damage::Empty, files[0].as_path())),
            "half" => assert!(apply(Damage::Half, files[0].as_path())),
            _ => assert!(apply(Damage::Dropped, files[0].as_path())),
        }
        let (cell, note): (String, String) = classify(&paused);
        remove_settled(&paused.fixture);
        assert_eq!(cell, control, "{phase}: shadow packed-refs {name}: {note}");
    }
}

/// A commit killed with its capture lock held; the shadow and its snapshot exist already.
#[test]
fn shadow_refs_at_capture_locked() {
    shadow_refs_do_not_change_the_outcome("capture-locked");
}

/// A commit killed after preparation.
#[test]
fn shadow_refs_at_preparation_done() {
    shadow_refs_do_not_change_the_outcome("preparation-done");
}

/// A commit killed with the landing and index locks held.
#[test]
fn shadow_refs_at_landing_locked() {
    shadow_refs_do_not_change_the_outcome("landing-locked");
}

/// A commit killed after the objects migrated into the kept pack.
#[test]
fn shadow_refs_at_objects_migrated() {
    shadow_refs_do_not_change_the_outcome("objects-migrated");
}

/// A commit killed after the branch advanced.
#[test]
fn shadow_refs_at_ref_updated() {
    shadow_refs_do_not_change_the_outcome("ref-updated");
}

/// A commit killed after the index was installed.
#[test]
fn shadow_refs_at_index_installed() {
    shadow_refs_do_not_change_the_outcome("index-installed");
}
