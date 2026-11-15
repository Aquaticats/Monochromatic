//! What:
//!  The name of the lock file a Cargo workspace keeps beside its manifest,
//!  and the rule that this repair never writes it.
//! Why:
//!   Seven rounds of review found seven ways for this repair to lose or mislabel a person's
//! `Cargo.lock`,
//! every one of them in a step that moved or wrote their file:
//! a placement exchanged the resolution into it,
//! a note promised to place what that displaced,
//! a recovery exchanged back,
//! a rollback judged what it displaced,
//! a file the clean created was taken away and put back by a rename,
//! and a cross-device write replaced whatever stood there.
//! The repair needs nothing from their file but its bytes:
//! Cargo resolves and cleans the same workspace in a mirror of it
//! (`package_repair_workspace_mirror.rs`,
//! `package_repair_cargo_clean_in_mirror.rs`),
//! measured on this host on 2026-11-14 to remove exactly the artifacts the workspace's own clean
//! removes.
//! So this repair reads the person's lock, if it must,
//! and never renames,
//! exchanges or writes it,
//! and no kill can leave any of their bytes anywhere but where they put them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // read only: every Cargo run that must write a lock is given a mirror to write it in
//! const LOCK_FILE = 'Cargo.lock';
//! ```

/// The lock file beside a workspace's root manifest.
///
/// It is read by this repair and never written:
/// the resolution a `--locked` clean needs is made in a mirror of the workspace,
/// and the clean itself runs there.
pub const LOCK_FILE: &str = "Cargo.lock";
