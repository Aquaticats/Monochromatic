//! What:
//! Finds the real `mpm` on PATH while skipping the shadow itself,
//! and records every `mpm` it saw with the reason it took none of them.
//! Why:
//! The shadow is first on PATH under the name `mpm`,
//! so a plain PATH search would find the shadow again and
//!           run itself forever.
//! The repository and global shadow directories can both be on PATH,
//! so every candidate
//!           that resolves to this executable is skipped,
//! not only the first.
//! A person who reads "no real `mpm` found on PATH" has to know which files were skipped and why,
//! because a file without its execute bit,
//! a directory of that name and a link that leads nowhere
//! all look the same from outside and need three different repairs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // real-binary.ts
//! export function lookupInPath(name: string, pathVariable: string, skip: string): { candidates: Candidate[]; chosen?: string } { ... }
//! export function findInPath(name: string, pathVariable: string, skip: string): string | undefined { return lookupInPath(name, pathVariable, skip).chosen; }
//! ```

/// Imports the file-system module (resolves symlinks,
/// reads permissions and file types).
use std::fs;

/// What:
/// `use std::os::unix::fs::PermissionsExt;` imports a trait (a named bundle of methods,
/// like a TS interface
///           that types can opt into) that makes `.mode()` available on file permissions.
/// Why:
/// The execute bits live in the Unix mode number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No equivalent: in TS `stats.mode` is always available.
/// ```
use std::os::unix::fs::PermissionsExt;

/// Imports the borrowed and owned path types.
use std::path::{
    Path,
    PathBuf,
};

/// Imports the lookup that names the shadow a program is or leads to.
use crate::shadow_identity::shadow_behind;

/// Imports the launcher record reader.
use crate::toolchain::launcher::{
    parse_record,
    record_path,
};

/// What:
/// `const EXECUTE_BITS: u32 = 0o111;` is a constant unsigned 32-bit integer (siblings `i32`,
/// `u64`).
/// The `0o`
///           prefix writes it in base 8:
/// the owner,
/// group and other execute bits.
/// Why:
/// A file is runnable when any of those bits is set;
/// `u32` because `.mode()` returns that type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const EXECUTE_BITS = 0o111;
/// ```
const EXECUTE_BITS: u32 = 0o111;

/// What:
/// `const PERMISSION_BITS: u32 = 0o777;` is the twelve-bit-free part of a Unix mode:
/// the nine permission bits,
/// written in base 8.
/// Why:
/// The mode a message prints is the one a person changes with `chmod`,
/// and the file-type bits above those nine would only confuse that number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PERMISSION_BITS = 0o777;
/// ```
const PERMISSION_BITS: u32 = 0o777;

/// What:
/// `pub enum SkipReason` says why the lookup passed over one `mpm` it saw.
/// Why:
/// Each of these has its own repair,
/// and a message that names none of them sends the person to the wrong step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SkipReason = { kind: 'this-shadow' } | { kind: 'launcher-of-this-shadow' } | { kind: 'shadow'; identity: string } | { kind: 'directory' } | { kind: 'no-execute-bit'; mode: number } | { kind: 'broken-link'; target: string } | { kind: 'not-a-regular-file' };
/// ```
#[derive(Debug)]
pub enum SkipReason {
    /// The file is this running shadow.
    ThisShadow,
    /// The file is a launcher whose record names this running shadow.
    LauncherOfThisShadow,
    /// The file carries a shadow's identity,
    /// of this version or another.
    Shadow {
        /// What that shadow says it is,
        /// as its own file writes it.
        identity: String,
    },
    /// The path is a directory,
    /// not a program.
    Directory,
    /// The path is a regular file with no execute bit.
    NoExecuteBit {
        /// The permission bits a `chmod` would change.
        mode: u32,
    },
    /// The path is a symbolic link that leads to no file.
    BrokenLink {
        /// What the link points at,
        /// as the link itself writes it.
        target: PathBuf,
    },
    /// The path is something that is neither a file nor a directory (a named pipe,
    /// a socket or a device).
    NotARegularFile,
}

/// What:
/// `pub struct Candidate` is one `mpm` the lookup saw,
/// with the reason it took it or passed it over.
/// Why:
/// The message lists these in the order the lookup read them,
/// so a person can tell a file that needs its execute bit from one that is a directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Candidate = { path: string; skippedBecause?: SkipReason };
/// ```
#[derive(Debug)]
pub struct Candidate {
    /// The path the lookup joined its directory with,
    /// as it was on PATH.
    pub path: PathBuf,
    /// Why the lookup passed it over,
    /// or `None` when this is the one it chose.
    pub skipped_because: Option<SkipReason>,
}

/// What:
/// `pub struct Lookup` is everything one PATH search saw and what it chose.
/// Why:
/// The same walk has to answer two questions at once:
/// which program to start,
/// and what to tell a person when there is none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Lookup = { candidates: Candidate[]; chosen?: string };
/// ```
#[derive(Debug)]
pub struct Lookup {
    /// Every `mpm` the lookup saw,
    /// in PATH order.
    /// A directory on PATH with no `mpm` in it adds nothing here:
    /// the list is of programs found,
    /// not of directories read.
    pub candidates: Vec<Candidate>,
    /// The real `mpm` to start,
    /// or `None` when every candidate was skipped and none was left.
    pub chosen: Option<PathBuf>,
}

/// What:
/// What the lookup found at one path:
/// a program to start,
/// nothing to mention,
/// or a reason it was passed over.
/// Why:
/// A PATH directory that holds no `mpm` is the ordinary case and must not appear in a message,
/// while a file that is there and was skipped must.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Inspection = { kind: 'runnable' } | { kind: 'nothing-there' } | { kind: 'skipped'; reason: SkipReason };
/// ```
enum Inspection {
    /// The path holds a program the lookup starts.
    Runnable,
    /// Nothing is at this path:
    /// neither a file nor a link.
    NothingThere,
    /// Something is at this path and the lookup passed it over.
    Skipped {
        /// Why the lookup passed this path over.
        reason: SkipReason,
    },
}

/// What:
/// Whether `candidate` (already resolved) is a per-tree launcher whose record names `shadow` as the binary it starts.
/// Why:
/// `install` links `mpm` to a launcher,
/// which replaces itself with the real shadow binary under the name `mpm`;
/// that binary's own path is not the launcher's,
/// so comparing paths alone takes the launcher for the real `mpm`,
/// starts it,
/// and the chain repeats until the host refuses new processes (issue 6259).
/// A file with no record,
/// or an unreadable one,
/// is not a launcher.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isLauncherOf(candidate: string, shadow: string): boolean
/// ```
fn is_launcher_of(
    candidate: &Path,
    shadow: &Path,
) -> bool {
    let Ok(text) = fs::read_to_string(record_path(candidate)) else {
        return false;
    };
    let Some(record) = parse_record(&text) else {
        return false;
    };
    if record.real == shadow {
        return true;
    }
    return match fs::canonicalize(&record.real) {
        Ok(resolved) => resolved == shadow,
        Err(_) => false,
    };
}

/// What:
/// `fn inspect_candidate(candidate: &Path, skip: &Path) -> Inspection` says what one PATH entry holds.
/// Why:
/// The decision and the reason for it are one walk:
/// a second function deciding whether to skip would let the two disagree about a file,
/// and the message would then name a reason the start did not use.
fn inspect_candidate(
    candidate: &Path,
    skip: &Path,
) -> Inspection {
    // `fs::symlink_metadata` reads the link itself rather than what it points at, so a link that leads nowhere is still seen.
    let Ok(link) = fs::symlink_metadata(candidate) else {
        return Inspection::NothingThere;
    };
    let target: fs::Metadata = if link.file_type().is_symlink() {
        match fs::metadata(candidate) {
            Ok(followed) => followed,
            // `fs::read_link` reads the text the link itself holds; an unreadable one leaves an empty path.
            Err(_) => {
                return Inspection::Skipped {
                    reason: SkipReason::BrokenLink {
                        target: fs::read_link(candidate).unwrap_or_default(),
                    },
                };
            }
        }
    } else {
        link
    };
    if target.is_dir() {
        return Inspection::Skipped {
            reason: SkipReason::Directory,
        };
    }
    if !target.is_file() {
        return Inspection::Skipped {
            reason: SkipReason::NotARegularFile,
        };
    }
    let mode: u32 = target.permissions().mode();
    if mode & EXECUTE_BITS == 0 {
        return Inspection::Skipped {
            reason: SkipReason::NoExecuteBit {
                mode: mode & PERMISSION_BITS,
            },
        };
    }
    // What:     `fs::canonicalize` resolves every symlink to the final path (a `Result`); the `&` lends the path.
    // Why:      Every link to the shadow resolves to the one executable, so comparing resolved paths recognizes
    //           the shadow under whatever name it was found. A path that cannot be resolved is judged by its
    //           own bytes below instead of being skipped.
    if let Ok(resolved) = fs::canonicalize(candidate) {
        if resolved == skip {
            return Inspection::Skipped {
                reason: SkipReason::ThisShadow,
            };
        }
        if is_launcher_of(&resolved, skip) {
            return Inspection::Skipped {
                reason: SkipReason::LauncherOfThisShadow,
            };
        }
    }
    // A candidate that is a shadow of any version, or a launcher that leads to one, is skipped without being started:
    // what a file is does not depend on its path or its version (`shadow_identity.rs`).
    if let Some(identity) = shadow_behind(candidate) {
        return Inspection::Skipped {
            reason: SkipReason::Shadow {
                identity,
            },
        };
    }
    return Inspection::Runnable;
}

/// What:
/// `pub fn lookup_in_path(name: &str, path_variable: &str, skip: &Path) -> Lookup` walks PATH in order and
/// answers the program to start and every `mpm` it passed over.
/// Why:
/// `skip` is the shadow executable's own resolved path,
/// so the loop never returns the shadow,
/// and the list it keeps is the evidence a person needs when it returns
/// none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function lookupInPath(name: string, pathVariable: string, skip: string): Lookup
/// ```
pub fn lookup_in_path(
    name: &str,
    path_variable: &str,
    skip: &Path,
) -> Lookup {
    let mut candidates: Vec<Candidate> = Vec::new();
    for directory in path_variable.split(':') {
        // An empty PATH entry means the current directory in a shell; the shadow ignores it on purpose.
        if directory.is_empty() {
            continue;
        }
        let candidate: PathBuf = Path::new(directory).join(name);
        match inspect_candidate(&candidate, skip) {
            Inspection::NothingThere => continue,
            Inspection::Skipped {
                reason,
            } => candidates.push(Candidate {
                path: candidate,
                skipped_because: Some(reason),
            }),
            Inspection::Runnable => {
                candidates.push(Candidate {
                    path: candidate.clone(),
                    skipped_because: None,
                });
                return Lookup {
                    candidates,
                    chosen: Some(candidate),
                };
            }
        }
    }
    return Lookup {
        candidates,
        chosen: None,
    };
}

/// What:
/// `pub fn find_in_path(name: &str, path_variable: &str, skip: &Path) -> Option<PathBuf>` answers only which
/// program to start,
/// and nothing about the ones passed over.
/// Why:
/// A caller that starts the program has no message to build,
/// and this is the whole of what it wants from the same walk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findInPath(name: string, pathVariable: string, skip: string): string | undefined
/// ```
pub fn find_in_path(
    name: &str,
    path_variable: &str,
    skip: &Path,
) -> Option<PathBuf> {
    return lookup_in_path(name, path_variable, skip).chosen;
}
