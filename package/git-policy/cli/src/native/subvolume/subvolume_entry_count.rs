//! What:
//!  The count of the entries under a checkout:
//! every file,
//! directory and link below its root,
//! counted once each.
//! Why:
//!  The free space a migration needs is the metadata of its entries (`subvolume_preflight.rs`),
//! so the refusal for too little
//!  space starts from how many entries the copy will write.
//! The walk reads directory listings only,
//! no file contents,
//! and
//!  follows no link,
//! as the copy pass does not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entries: number = countEntries(root); // readdir on each directory, one count per name
//! ```

use std::path::{Path, PathBuf};

/// What:
///  How many entries are below `root`.
///  `&Path` borrows the path read-only (TS:
/// a `string` argument);
/// `u64` is an unsigned 64-bit integer (siblings:
/// `u32`,
///  `usize`).
/// Why:
///   A directory that cannot be listed (gone,
/// or not readable) adds nothing,
/// because the copy reports an unreadable
///  directory with its own message and the preflight must not duplicate it.
/// `u64` rather than `usize` because the figure
///  is multiplied into a byte count that is stored as `u64` in the filesystem facts.
/// Gotcha:
///   A link to a directory counts as one entry and its target is not walked,
/// unlike a TS `readdir` recursion that follows
///  `stat`;
/// the work stack below replaces recursion so a deep tree cannot overflow the call stack.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function countEntries(root: string): number {
///   let count = 0;
///   const pending: string[] = [root];
///   while (pending.length > 0) {
///     const directory = pending.pop()!;
///     let names: Dirent[] = [];
///     try { names = readdirSync(directory, { withFileTypes: true }); } catch { continue; }
///     for (const name of names) { count += 1; if (name.isDirectory()) pending.push(join(directory, name.name)); }
///   }
///   return count;
/// }
/// ```
pub fn count_entries(root: &Path) -> u64 {
    // What: `Vec<PathBuf>` is a growable list (TS: `string[]`) of owned paths; `vec![...]` builds it with one item.
    // Why: The directories still to list; a list used as a stack walks the tree without recursion.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pending: string[] = [root];
    // ```
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    let mut count: u64 = 0;
    // What: `while let Some(directory) = pending.pop()` runs while `pop()` gives `Some(...)`, the present variant of
    //       `Option`; it stops at `None`, the empty list.
    // Why: Take the next directory off the stack until none is left.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let directory: string | undefined;
    // while ((directory = pending.pop()) !== undefined) { ... }
    // ```
    while let Some(directory) = pending.pop() {
        // What: `let Ok(listing) = ... else { continue };` unpacks the success variant of the `Result` of `read_dir`, and
        //       runs the `else` block (skip this directory) when it is the failure variant. `&directory` lends the path.
        // Why: An unlistable directory adds nothing and does not stop the count.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let listing; try { listing = readdirSync(directory); } catch { continue; }
        // ```
        let Ok(listing) = std::fs::read_dir(&directory) else { continue };
        // What: `listing.flatten()` walks the `Result` items of the listing and drops the failed ones, keeping the `DirEntry`
        //       of each name that was read.
        // Why: A name that vanished between the listing and the read is not counted, as in the copy pass.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const entry of safeEntries(listing)) { ... }
        // ```
        for entry in listing.flatten() {
            count += 1;
            // What: `entry.file_type()` is the type of the entry without following a link; `.ok()` turns its `Result` into an
            //       `Option` that is `None` on failure; `is_some_and(...)` is true when it is `Some` and the closure `|kind|
            //       ...` returns true.
            // Why: Only a real directory is walked further; a link to a directory is one entry and nothing below it.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (entry.isDirectory()) pending.push(join(directory, entry.name)); // isDirectory() is false for a symlink
            // ```
            let is_directory: bool = match entry.file_type() {
                Ok(kind) => kind.is_dir(),
                Err(_) => false,
            };
            if is_directory {
                pending.push(entry.path());
            }
        }
    }
    // What: `return count;` hands the `u64` total back to the caller.
    // Why: The preflight multiplies it by the cost of an entry.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return count;
    // ```
    return count;
}

/// Controls for the count stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_entry_count_tests.rs"]
mod tests;
