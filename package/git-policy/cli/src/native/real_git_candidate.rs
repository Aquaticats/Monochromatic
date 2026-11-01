//! What:
//!  Decide whether one PATH candidate is real Git,
//!  a cli-git wrapper,
//!  or unusable.
//! Why:
//!  The wrapper shadows `git` on PATH and must never select itself,
//!  a copy of
//!      itself,
//!  or a script that starts the TypeScript wrapper.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // classifyCandidate('/usr/bin/git', ownExecutable) === 'real-git'
//! ```

/// What:
///  Import the trait that gives files `.read(..)`.
///       A trait is an interface;
///  its methods exist only while it is in scope.
/// Why:
///   Candidates are inspected through bounded reads,
///  never loaded whole by default.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { open } from 'node:fs/promises';
/// ```
use std::io::Read;
/// `Path` is a borrowed filesystem path of raw OS bytes (owned sibling:
///  `PathBuf`).
use std::path::Path;

/// What:
///  Largest script inspected for wrapper markers,
///  64 kibibytes.
///       `usize` is the platform's index and length type (siblings `u32`,
///  `u64`).
/// Why:
///   Generated command shims are tiny launchers;
///  a fixed bound keeps an arbitrary
///       PATH file from controlling how much the resolver reads.
///  `usize` matches the
///       buffer lengths it is compared with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_SCRIPT_INSPECTION_BYTES = 64 * 1024;
/// ```
pub const MAX_SCRIPT_INSPECTION_BYTES: usize = 64 * 1024;

/// What:
///  Leading bytes of ELF,
///  PE,
///  Mach-O and universal Mach-O executables.
///       `&[&[u8]]` is a borrowed list of borrowed byte strings compiled into the program.
/// Why:
///   A native executable that is not this wrapper is treated as Git after one
///       four-byte read;
///  only scripts are searched for wrapper markers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NATIVE_EXECUTABLE_PREFIXES = ['7f454c46', '4d5a', 'feedface', ...];
/// ```
const NATIVE_EXECUTABLE_PREFIXES: &[&[u8]] = &[
    b"\x7fELF",
    b"MZ",
    b"\xfe\xed\xfa\xce",
    b"\xfe\xed\xfa\xcf",
    b"\xce\xfa\xed\xfe",
    b"\xcf\xfa\xed\xfe",
    b"\xca\xfe\xba\xbe",
    b"\xbe\xba\xfe\xca",
    b"\xca\xfe\xba\xbf",
    b"\xbf\xba\xfe\xca",
];

/// What:
///  Text that identifies a script delegating to the TypeScript cli-git wrapper:
///       its package name and bundled entry path,
///  in POSIX and Windows spellings.
/// Why:
///   Such a script is the other wrapper;
///  selecting it as real Git would make the
///       two wrappers call each other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SELF_SHIM_MARKERS = ['@monochromatic-dev/git-policy-cli', ...];
/// ```
const WRAPPER_SCRIPT_MARKERS: &[&[u8]] = &[
    b"@monochromatic-dev/git-policy-cli",
    b"package/git-policy/cli/dist/final/node/index.mjs",
    b"@monochromatic-dev\\git-policy-cli",
    b"package\\git-policy\\cli\\dist\\final\\node\\index.mjs",
];

/// What:
///  The three verdicts for one candidate path.
///       `#[derive(...)]` generates copying,
///  debug printing and `==`.
/// Why:
///   The resolver counts skipped wrappers separately from unusable entries so its
///       failure message can say why nothing was selected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CandidateKind = 'real-git' | 'wrapper' | 'unusable';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateKind {
    /// An executable regular file that is not a cli-git wrapper.
    RealGit,
    /// This executable,
    ///  a byte-identical copy of it,
    ///  or a script starting the TypeScript wrapper.
    Wrapper,
    /// Missing,
    ///  not a regular file,
    ///  not executable,
    ///  unreadable,
    ///  or an oversized script.
    Unusable,
}

/// What:
///  Report whether the bytes begin with a native executable signature.
/// Why:
///   Native candidates skip the script-marker search entirely.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isNativeHeader(header: Uint8Array): boolean;
/// ```
pub fn is_native_header(header: &[u8]) -> bool {
    // `for prefix in LIST` borrows each compiled-in signature in turn.
    for prefix in NATIVE_EXECUTABLE_PREFIXES {
        if header.starts_with(prefix) {
            return true;
        }
    }
    return false;
}

/// What:
///  Report whether `needle` occurs anywhere inside `haystack`.
/// Why:
///   Byte slices have no built-in substring search,
///  and scripts are searched as
///       bytes so a non-UTF-8 script cannot hide a marker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function containsBytes(haystack: Buffer, needle: Buffer): boolean { return haystack.includes(needle); }
/// ```
fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.len() > haystack.len() {
        return false;
    }
    // `usize` start offsets; `mut` allows advancing.
    let mut start: usize = 0;
    while start + needle.len() <= haystack.len() {
        // `&haystack[a..b]` borrows the bytes from `a` up to, not including, `b`.
        if &haystack[start..start + needle.len()] == needle {
            return true;
        }
        start += 1;
    }
    return false;
}

/// What:
///  Report whether script bytes contain a TypeScript-wrapper marker.
/// Why:
///   This is the version-independent recognition of the other wrapper's launchers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hasWrapperMarker(content: Buffer): boolean;
/// ```
pub fn has_wrapper_marker(content: &[u8]) -> bool {
    for marker in WRAPPER_SCRIPT_MARKERS {
        if contains_bytes(content, marker) {
            return true;
        }
    }
    return false;
}

/// What:
///  Report whether file metadata says the file can be run.
///       `#[cfg(unix)]` and `#[cfg(not(unix))]` each compile one of the two inner blocks.
/// Why:
///   PATH lookup only runs executable files;
///  a non-executable `git` is skipped.
///       On Unix this checks the mode bits,
///  not the calling user's access as `access(2)`
///       would.
///  Other systems have no execute bit:
///  every regular file named by PATHEXT
///       can run.
///  The Markdown policy's linter lookup applies the same test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isExecutable(stats: Stats): boolean { return process.platform === 'win32' || (stats.mode & 0o111) !== 0; }
/// ```
pub fn is_executable(metadata: &std::fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        // The trait adds `.mode()`; `use` inside a block scopes it to this block.
        use std::os::unix::fs::PermissionsExt;
        // `&` here is bitwise AND; `0o111` is octal for the three execute bits.
        return metadata.permissions().mode() & 0o111 != 0;
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        // `let _ = ...` states that the metadata is deliberately not consulted here.
        let _ = metadata;
        return true;
    }
}

/// What:
///  Report whether two metadata records name the same file on the same device.
/// Why:
///   On Unix,
///  device and inode numbers identify a file through any symbolic link,
///       hard link or repeated PATH entry.
///  Other systems expose no stable inode here,
///       so the answer is "not proven the same" and canonical paths decide instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sameInode(a: Stats, b: Stats): boolean { return process.platform !== 'win32' && a.dev === b.dev && a.ino === b.ino; }
/// ```
fn same_inode(first: &std::fs::Metadata, second: &std::fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        // The trait adds `.dev()` and `.ino()` to metadata on Unix.
        use std::os::unix::fs::MetadataExt;
        return first.dev() == second.dev() && first.ino() == second.ino();
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        // Neither record is consulted where no inode is available.
        let _ = (first, second);
        return false;
    }
}

/// What:
///  Report whether two paths name the same existing file.
/// Why:
///   Self-exclusion must hold through symbolic links,
///  hard links,
///  relative
///       spellings and repeated PATH directories.
///  Either check alone suffices;
///  an
///       inspection failure means "not proven the same".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function sameFile(first: string, second: string): Promise<boolean>;
/// ```
pub fn same_file(first: &Path, second: &Path) -> bool {
    // What: `if let (Ok(a), Ok(b)) = (x, y) && cond` runs only when both results
    //       succeeded and the extra condition holds.
    //       `std::fs::metadata` follows symbolic links to the final file.
    // Why:  A missing or unreadable path cannot be proven identical to anything.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [a, b] = await Promise.all([stat(first), stat(second)]);
    // ```
    if let (Ok(first_metadata), Ok(second_metadata)) =
        (std::fs::metadata(first), std::fs::metadata(second))
        && same_inode(&first_metadata, &second_metadata)
    {
        return true;
    }
    // `canonicalize` resolves every symbolic link and relative segment to one absolute path.
    if let (Ok(first_canonical), Ok(second_canonical)) =
        (std::fs::canonicalize(first), std::fs::canonicalize(second))
    {
        return first_canonical == second_canonical;
    }
    return false;
}

/// What:
///  Report whether two files have the same length and the same bytes.
/// Why:
///   A copy of this executable under another name or directory is still this
///       wrapper,
///  although its inode differs.
///  Lengths are compared first,
///  so real Git
///       is never read in full for this check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function identicalContent(first: string, second: string): Promise<boolean>;
/// ```
pub fn identical_content(first: &Path, second: &Path) -> bool {
    let (Ok(first_metadata), Ok(second_metadata)) =
        (std::fs::metadata(first), std::fs::metadata(second))
    else {
        return false;
    };
    if first_metadata.len() != second_metadata.len() {
        return false;
    }
    // `std::fs::read` returns the whole file as an owned byte list, or an error.
    let (Ok(first_bytes), Ok(second_bytes)) = (std::fs::read(first), std::fs::read(second)) else {
        return false;
    };
    return first_bytes == second_bytes;
}

/// What:
///  Read from a file until `limit` bytes are collected or the file ends.
///       `&mut std::fs::File` lends the open file for reading (reading moves its position).
///       `Option<Vec<u8>>` is the owned bytes,
///  or `None` when the read failed.
/// Why:
///   One `read` call may return fewer bytes than requested.
///  The standard library's
///       bounded reader repeats the read until the limit or the end of the file,
///  so the
///       bound is exact and there is no loop here to keep in step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readUpTo(file: FileHandle, limit: number): Promise<Buffer | undefined>;
/// ```
fn read_up_to(file: &mut std::fs::File, limit: usize) -> Option<Vec<u8>> {
    // `Vec::<u8>::new()` is an empty owned byte list that grows as bytes arrive.
    let mut buffer: Vec<u8> = Vec::<u8>::new();
    // What: `.take(n)` wraps the lent file in a reader that ends after `n` bytes;
    //       `limit as u64` widens the count to the 64-bit type that reader takes, which
    //       cannot lose a value. `.read_to_end(&mut buffer)` appends everything the
    //       reader yields to the list. `match` unpacks the `Result`: `Ok(_)` ignores
    //       the byte count.
    // Why:  A read error makes the candidate unusable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return (await file.read({ length: limit })).buffer; } catch { return undefined; }
    // ```
    match file.take(limit as u64).read_to_end(&mut buffer) {
        // `Some(buffer)` is the "present" variant carrying the bytes read.
        Ok(_) => return Some(buffer),
        Err(_) => return None,
    }
}

/// What:
///  Classify one candidate against this wrapper's own executable.
/// Why:
///   This is the whole self-exclusion decision:
///  the same file (any link or
///       spelling),
///  an identical copy,
///  or a TypeScript-wrapper launcher is a wrapper;
///       any other executable regular file is Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function classifyCandidate(candidate: string, ownExecutable: string): Promise<CandidateKind>;
/// ```
pub fn classify_candidate(candidate: &Path, own_executable: &Path) -> CandidateKind {
    // What: `let Ok(x) = ... else { return ... };` unwraps success or exits.
    //       `std::fs::metadata` inspects the file a symbolic link finally points at.
    // Why:  The file type is checked before opening, because opening a named pipe
    //       would wait forever for a writer. A missing candidate is simply skipped.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let stats; try { stats = await stat(candidate); } catch { return "unusable"; }
    // ```
    let Ok(metadata) = std::fs::metadata(candidate) else {
        return CandidateKind::Unusable;
    };
    if !metadata.is_file() || !is_executable(&metadata) {
        return CandidateKind::Unusable;
    }
    if same_file(candidate, own_executable) || identical_content(candidate, own_executable) {
        return CandidateKind::Wrapper;
    }
    // `mut file`: reading advances the open file, which counts as modifying it.
    let Ok(mut file) = std::fs::File::open(candidate) else {
        return CandidateKind::Unusable;
    };
    // Four bytes cover every native signature in the table.
    let Some(header) = read_up_to(&mut file, 4) else {
        return CandidateKind::Unusable;
    };
    if is_native_header(header.as_slice()) {
        return CandidateKind::RealGit;
    }
    // The header is at least one byte of a file that has any, so reading the whole bound
    // after it reaches past the bound exactly when the script is too large to inspect.
    let Some(rest) = read_up_to(&mut file, MAX_SCRIPT_INSPECTION_BYTES) else {
        return CandidateKind::Unusable;
    };
    // Join header and rest so a marker spanning the first four bytes is still found.
    let mut content: Vec<u8> = header;
    content.extend_from_slice(rest.as_slice());
    if content.len() > MAX_SCRIPT_INSPECTION_BYTES {
        return CandidateKind::Unusable;
    }
    if has_wrapper_marker(content.as_slice()) {
        return CandidateKind::Wrapper;
    }
    return CandidateKind::RealGit;
}

/// Controls for the header,
///  marker,
///  identity and content primitives.
#[cfg(test)]
#[path = "real_git_candidate_primitive_tests.rs"]
mod primitive_tests;

/// Disposable-directory classification controls stay out of the release executable.
#[cfg(test)]
#[path = "real_git_candidate_tests.rs"]
mod tests;
