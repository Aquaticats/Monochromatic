//! What:
//!  The paths of one migration and the free space its metadata needs:
//!  the new tree and the old tree beside the checkout,
//!  the journal,
//!  the unreadable-entry list and the left-out list beside them,
//!  the numbered claim files that give one command the migration,
//!  and the measured bytes per entry the free-space check charges.
//! Why:
//!  The names are fixed by the root's name,
//!  so a rerun finds the same files,
//!  and the two trees sit beside the checkout in one parent,
//!  so the exchange stays on one filesystem.
//!  The copy shares data blocks and writes one metadata record per entry,
//!  so the need grows with the entries and not with a fixed floor.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Layout = { root: string; parent: string; newPath: string; oldPath: string; journalPath: string; unreadablePath: string; claimPrefix: string };
//! const needed = Math.min(U64_MAX, entries * METADATA_BYTES_PER_ENTRY * METADATA_COPIES);
//! ```

use std::path::{
    Path,
    PathBuf,
};

/// The metadata the copy writes for one entry,
/// in bytes of the filesystem's own accounting:
/// the `Metadata, DUP` figure of
///  `btrfs filesystem df --block-size=1` read before and after reflink-copying a tree of 22,719 entries eight times
///  (181,752 entries) on this host,
/// three alternating rounds against a no-copy control of the same wall time:
///  957.9,
/// 979.0 and 984.8 bytes per entry,
/// while the control moved by at most 9.9 MB against 174 MB to 179 MB for a copy.
///  Rounded up to one kibibyte.
/// Recorded in `doc/planning/cli-git-shipped-limits-audit.md`,
/// "The free-space preflight".
/// This is
///  the coordinator's pick under the owner's delegation ("Pick options as if I had picked them"),
/// open to the owner's veto.
pub const METADATA_BYTES_PER_ENTRY: u64 = 1024;

/// How many physical copies of the metadata the filesystem keeps:
/// the `DUP` profile of this host writes each metadata byte twice
///  (`Metadata ratio: 2.00` in `btrfs filesystem usage`),
/// and the free space the check reads is physical.
/// A filesystem with a
///  single copy is over-estimated by this factor,
/// which only refuses earlier.
pub const METADATA_COPIES: u64 = 2;

/// What:
///  The free bytes the migration needs for the metadata of `entries` entries.
/// Why:
///   The copy shares data blocks and writes one metadata record per entry,
/// so the need grows with the entries,
/// not with a fixed
///  floor;
/// the measured cost of an entry and the copies of the metadata are the two named constants above.
/// A count so large
///  that the product overflows saturates at the largest `u64`,
/// which no filesystem has free,
/// so it refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const needed = Math.min(Number.MAX_SAFE_INTEGER, entries * METADATA_BYTES_PER_ENTRY * METADATA_COPIES);
/// ```
pub fn required_free_bytes(entries: u64) -> u64 {
    // What: `saturating_mul` multiplies two `u64` values and gives the largest `u64` instead of wrapping around when the product is too big.
    // Why: An absurd entry count must refuse, never wrap to a small number that passes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return Math.min(U64_MAX, entries * (METADATA_BYTES_PER_ENTRY * METADATA_COPIES));
    // ```
    return entries.saturating_mul(METADATA_BYTES_PER_ENTRY.saturating_mul(METADATA_COPIES));
}

/// What:
///  Every path of one migration,
/// derived from the checkout's path.
/// Why:
///   The new tree and the old tree sit beside the checkout (one parent,
/// so the exchange stays on one filesystem),
/// and
///  the journal and the unreadable-file list sit beside them,
/// outside the tree that is exchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Layout = { root: string; parent: string; newPath: string; oldPath: string; journalPath: string; unreadablePath: string; claimPrefix: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Layout {
    /// The main checkout.
    pub root: PathBuf,
    /// Its parent directory.
    pub parent: PathBuf,
    /// `<root>.subvol-new`:
    /// the subvolume being filled,
    /// and after the exchange the old tree until it is renamed.
    pub new_path: PathBuf,
    /// `<root>.before-subvolume`:
    /// the original tree,
    /// kept until removed on request.
    pub old_path: PathBuf,
    /// `<parent>/.cli-git-migrate-<name>.journal.json`.
    pub journal_path: PathBuf,
    /// `<parent>/.cli-git-migrate-<name>.unreadable.txt`.
    pub unreadable_path: PathBuf,
    /// `<parent>/.cli-git-migrate-<name>.left-out.txt`:
    /// the entries of the old tree the late-write carry left out,
    /// which the deletion of the old tree does not delete without a word.
    pub left_out_path: PathBuf,
    /// `.cli-git-migrate-<name>.owner`:
    /// the name,
    /// in the parent directory,
    /// of the numbered claim files that give one command
    ///  at a time the migration of this checkout.
    pub claim_prefix: std::ffi::OsString,
}

/// `parent/<name><suffix>`:
/// the name of a file kept beside the root under a fixed suffix.
fn with_suffix(
    parent: &Path,
    name: &std::ffi::OsStr,
    suffix: &str,
) -> PathBuf {
    let mut text: std::ffi::OsString = name.to_os_string();
    text.push(suffix);
    return parent.join(text);
}

/// `parent/.cli-git-migrate-<name><suffix>`:
/// the name of a hidden file kept beside the root under a fixed suffix.
fn dotted(
    parent: &Path,
    name: &std::ffi::OsStr,
    suffix: &str,
) -> PathBuf {
    let mut text: std::ffi::OsString = std::ffi::OsString::from(".cli-git-migrate-");
    text.push(name);
    text.push(suffix);
    return parent.join(text);
}

/// What:
///  The layout for a checkout path.
///  A root with no parent or no name (`/`) has no layout.
/// Why:
///   The names are fixed by the root's name,
/// so a rerun finds the same files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function layoutOf(root: string): Layout | undefined;
/// ```
pub fn layout_of(root: &Path) -> Option<Layout> {
    let name: &std::ffi::OsStr = root.file_name()?;
    let parent: &Path = root.parent()?;

    return Some(Layout {
        root: root.to_path_buf(),
        parent: parent.to_path_buf(),
        new_path: with_suffix(parent, name, ".subvol-new"),
        old_path: with_suffix(parent, name, ".before-subvolume"),
        journal_path: dotted(parent, name, ".journal.json"),
        unreadable_path: dotted(parent, name, ".unreadable.txt"),
        left_out_path: dotted(parent, name, ".left-out.txt"),
        claim_prefix: dotted(parent, name, ".owner")
            .file_name()
            .unwrap_or_default()
            .to_os_string(),
    });
}
