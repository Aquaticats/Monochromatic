//! Language files compiled into the application executable,
//!  with the digest recorded at build time.
//!
//! - [`EmbeddedFile`] is one file:
//!    its path in the old directory layout,
//!    its bytes,
//!    and its digest.
//! - [`EmbeddedRuntime`] is the whole table that `build.rs` generates for the application binary.
//! - [`EmbeddedRuntime::verified`] finds a file and checks its digest before anyone uses the bytes.

/// The digest the build script recorded for every file.
use crate::content_digest::fnv1a;
/// Damage and absence are errors with a message and a remedy.
use anyhow::{Result, bail};
/// Compares two paths during the halving search of the embedded table.
use std::cmp::Ordering;

/// What:
///  One embedded file.
///  `&'static str` is borrowed text that lives as long as the program
///       (sibling:
///  `String`,
///  owned text);
///  `&'static [u8]` is a borrowed view of bytes stored inside
///       the executable (siblings `Vec<u8>`,
///  `[u8; N]`);
///  `u64` holds the digest.
/// Why:
///  The build script writes these as constants,
///  so nothing is copied or allocated at run time;
///      `'static` says the bytes live in the executable itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EmbeddedFile = { path: string; bytes: Uint8Array; digest: bigint };
/// ```
#[derive(Debug)]
pub struct EmbeddedFile {
    /// Path in the layout of the former application directory,
    ///  for example `runtime/grammars/sql.so`.
    pub path: &'static str,
    /// The file's bytes as they were when the executable was built.
    pub bytes: &'static [u8],
    /// [`fnv1a`] of `bytes`,
    ///  computed by the build script.
    pub digest: u64,
}

/// What:
///  The embedded table.
///  `&'static [EmbeddedFile]` is a borrowed list stored in the executable.
/// Why:
///  One value names the whole language runtime of this build,
///  and its key names the cache
///      directory,
///  so two builds with different files never share unpacked parsers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EmbeddedRuntime = { key: string; files: readonly EmbeddedFile[] }; // files sorted by path
/// ```
#[derive(Debug)]
pub struct EmbeddedRuntime {
    /// Digest over every path and file digest,
    ///  written as 16 hexadecimal digits.
    pub key: &'static str,
    /// Every embedded file,
    ///  sorted by `path` so lookups can halve the search each step.
    pub files: &'static [EmbeddedFile],
}

/// What:
///  Name the running executable for messages;
///  `std::env::current_exe()` returns
///       `Result<PathBuf>`,
///  and a fixed name is the fallback when the lookup failed.
/// Why:
///  A damaged or incomplete executable is fixed by replacing that file,
///  so the message names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function executableName(): string { try { return currentExe(); } catch { return 'the application executable'; } }
/// ```
pub fn executable_name() -> String {
    // What: a failed lookup falls back to a fixed name.
    // Why: Without a path the message still has to name what to replace.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return currentExe(); } catch { return 'the application executable'; }
    // ```
    return match std::env::current_exe() {
        Ok(path) => path.display().to_string(),
        Err(_) => "the application executable".to_string(),
    };
}

/// Lookups and digest checks over the sorted table.
impl EmbeddedRuntime {
    /// What:
    ///  Find a file by path.
    ///  `binary_search_by` halves the sorted list each step and returns
    ///       `Ok(index)` when found or `Err(insertion point)` when not;
    ///  `.ok()` keeps only the index
    ///       as `Option<usize>`.
    ///  `Option<&'static EmbeddedFile>` is the file or nothing.
    /// Why:
    ///  Query lookups happen for every language prepared;
    ///  a sorted list needs no map built at run time.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// find(path: string): EmbeddedFile | undefined { return this.files.find(file => file.path === path); }
    /// ```
    pub fn find(&self, path: &str) -> Option<&'static EmbeddedFile> {
        // What: `let files: &'static [EmbeddedFile] = self.files;` copies the borrowed list out of `self`.
        // Why: Indexing the copy yields references that live for the whole program, not only as long
        //      as this call's borrow of `self`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const files = this.files;
        // ```
        let files: &'static [EmbeddedFile] = self.files;
        // What: `file.path.cmp(path)` orders two texts; its answer tells the search which way to go.
        // Why: The build script sorted the list by the same byte order.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const index = binarySearch(files, file => compare(file.path, path));
        // ```
        let mut low = 0;
        let mut high = files.len();
        while low < high {
            let middle = low + (high - low) / 2;
            match files[middle].path.cmp(path) {
                Ordering::Less => low = middle + 1,
                Ordering::Greater => high = middle,
                Ordering::Equal => return files.get(middle),
            }
        }
        return None;
    }

    /// What:
    ///  Find a file and check its digest.
    ///  `Result<Option<&'static [u8]>>` is an error,
    ///  no file
    ///       (`Ok(None)`),
    ///  or the checked bytes (`Ok(Some(bytes))`).
    /// Why:
    ///  Bytes that no longer match their build-time digest come from a damaged executable;
    ///  loading
    ///      a damaged parser library could crash the process,
    ///  so the damage is reported instead.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// verified(path: string): Uint8Array | undefined // throws when the bytes are damaged
    /// ```
    pub fn verified(&self, path: &str) -> Result<Option<&'static [u8]>> {
        // What: `let Some(file) = ... else { ... }` unpacks the found file or runs the `else` block,
        //       which must leave the function.
        // Why: A file the table does not hold is not an error here; callers decide whether it must exist.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const file = this.find(path); if (!file) return undefined;
        // ```
        let Some(file) = self.find(path) else {
            // `Ok(None)`: success, with nothing found.
            return Ok(None);
        };
        let found = fnv1a(file.bytes);
        if found != file.digest {
            tracing::error!(
                path,
                expected = file.digest,
                found,
                "embedded file is damaged"
            );
            // What: `bail!` is a macro (the `!`) that returns an error with this formatted message.
            // Why: The message names the file, the executable, what happened, and the remedy.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // throw new Error(`The file ${path} embedded in ${exe} is damaged ...`);
            // ```
            bail!(
                "The file {path} embedded in {} is damaged: its contents no longer match the digest recorded when the application was built (expected {:016x}, found {found:016x}). Replace the executable with a fresh copy of the application, or build it again from source, then restart the application.",
                executable_name(),
                file.digest
            );
        }
        // `Ok(Some(...))`: success, with the checked bytes.
        return Ok(Some(file.bytes));
    }

    /// What:
    ///  Find a file that must exist,
    ///  check its digest,
    ///  and fail when it is absent.
    /// Why:
    ///  The manifest and every grammar it lists are required;
    ///  their absence means the executable
    ///      was built from an incomplete runtime or was cut short.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// required(path: string): Uint8Array // throws when absent or damaged
    /// ```
    pub fn required(&self, path: &str) -> Result<&'static [u8]> {
        // The trailing `?` hands a damage error to the caller unchanged.
        let Some(bytes) = self.verified(path)? else {
            tracing::error!(path, "embedded language file is missing");
            bail!(
                "The executable {} lacks the language file {path} that its embedded language manifest requires, so it is incomplete or damaged. Replace it with a fresh copy of the application, or build it again from source, then restart the application.",
                executable_name()
            );
        };
        return Ok(bytes);
    }
}

/// Lookups,
///  digest checks,
///  and the messages for a damaged or incomplete table.
#[cfg(test)]
#[path = "embedded_tests.rs"]
mod tests;
