//! What:
//!  Find where a file or folder the program keeps is today:
//!  under its current name,
//!  or,
//!  the first time it is needed after the rename,
//!  moved there from its earlier name with one atomic rename in the same folder.
//! Why:
//!  The owner renamed the folders and files the wrapper keeps inside the Git directory and said "never discard user data"
//!  (`doc/planning/git-policy-cli-program-rename.md`).
//! One rename that refuses to replace copies nothing and leaves no
//!  second name;
//! a process that loses the race finds the winner's result;
//! a failure is a notice and never an error,
//!  because "commits should never fail".
//! When both names exist the current one is used and the fold of the earlier one into it
//!  (`earlier_names_folded_into_current.rs`) takes over.
//!  No lock is taken:
//! the kernel serializes the rename,
//! and a crash between the check and the rename leaves the earlier
//!  name,
//! which the next run moves.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { path, notice } = placeKeptName(gitDir, DATA_FOLDER); // renames cli-git to monochromatic-git-policy-cli if only cli-git exists
//! ```

/// The pair of names the move works on.
use crate::wrapper_names_old_and_new::{MESSAGE_PREFIX, RenamedName};
/// The rename that fails when the target exists,
///  and the test for a filesystem that cannot do it.
use crate::rename_that_never_replaces::{rename_noreplace, rename_unsupported};
/// `Write` gives standard error its `write_all` method.
use std::io::Write;
/// `Path` is a borrowed path and `PathBuf` an owned one.
use std::path::{Path, PathBuf};
/// An atomic whole number that processes of one executable share without a lock.
use std::sync::atomic::{AtomicU64, Ordering};

/// What:
///  What a look at the two names found.
/// Why:
///   The look and the move are separate steps so a control can stop between them (a crash) or move the name itself
///  in between (a competing process).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Look = 'only-earlier' | 'both' | 'settled';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Look {
    /// Only the earlier name exists:
    ///  it is the one to move.
    OnlyEarlier,
    /// Both names exist:
    ///  the current one is used and the earlier one is reported.
    Both,
    /// The current name exists,
    ///  or neither does:
    ///  nothing to move.
    Settled,
}

/// What:
///  Where the kept name is for this run,
///  and the one line to tell the person when something about it needs their attention.
///  `Option<String>` is "text or nothing".
/// Why:
///   The caller uses `path` whatever happened;
///  the notice never stops a command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Placement = { path: string; notice?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Placement {
    /// The path to use now.
    pub path: PathBuf,
    /// One line for standard error,
    ///  absent when nothing needs saying.
    pub notice: Option<String>,
}

/// Whether a name exists in a folder,
///  without following a final symbolic link;
///  any failure to look counts as absent,
///  because the later use of the path reports its own failure.
fn exists(path: &Path) -> bool {
    return std::fs::symlink_metadata(path).is_ok();
}

/// What:
///  Look at the two names in `parent`.
/// Why:
///   A symbolic link counts as present,
///  so a link is moved like any other entry and never followed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function look(parent: string, name: RenamedName): Look;
/// ```
pub fn look(parent: &Path, name: RenamedName) -> Look {
    let current_there: bool = exists(parent.join(name.current).as_path());
    let earlier_there: bool = exists(parent.join(name.earlier).as_path());
    if current_there && earlier_there {
        return Look::Both;
    }
    if earlier_there {
        return Look::OnlyEarlier;
    }
    return Look::Settled;
}

/// The line that reports an earlier name that could not be moved,
///  and is used as it is for this run.
fn unmovable_notice(parent: &Path, name: RenamedName, error: &std::io::Error) -> String {
    return format!(
        "{}: {} in {} could not be renamed to {} ({error}), so it is used under its earlier name this time and the rename is \
         tried again by the next command.\n",
        MESSAGE_PREFIX.current,
        name.earlier,
        parent.display(),
        name.current
    );
}

/// What:
///  Carry out what `look` found:
///  rename the earlier name to the current one when only the earlier one existed.
///  `Look` is a three-case enum.
/// Why:
///   `rename` is a parameter so a control can make the filesystem refuse;
///  production passes the call that refuses to replace.
///  The rename refuses to replace,
///  so a competing process that moved the name first costs nothing:
///  the earlier name is then gone (`NotFound`) or the current one exists (`AlreadyExists`),
///  and either way the current name is used.
///  A failure the filesystem gives (a read-only Git directory,
///  a filesystem without the call) keeps the earlier name in use for this run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function carryOut(looked: Look, parent: string, name: RenamedName, rename: (from: string, to: string) => void): Placement;
/// ```
pub fn carry_out(looked: Look, parent: &Path, name: RenamedName, rename: fn(&Path, &Path) -> std::io::Result<()>) -> Placement {
    let current: PathBuf = parent.join(name.current);
    let earlier: PathBuf = parent.join(name.earlier);
    match looked {
        Look::Settled => return Placement { path: current, notice: None },
        // The fold of the earlier name into the current one (`earlier_names_folded_into_current.rs`) speaks when it waits or fails.
        Look::Both => return Placement { path: current, notice: None },
        Look::OnlyEarlier => {}
    }
    // `match` on the rename's `Result`: success, a lost race, or a failure to report.
    match rename(earlier.as_path(), current.as_path()) {
        Ok(()) => return Placement { path: current, notice: None },
        Err(error) => {
            // The earlier name vanished: another process moved it, so the current name is the winner's.
            if error.kind() == std::io::ErrorKind::NotFound && exists(current.as_path()) {
                return Placement { path: current, notice: None };
            }
            // The current name appeared: another process was ahead and the earlier name came back or never left; the fold takes it from here.
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                return Placement { path: current, notice: None };
            }
            // A filesystem without the call, a read-only folder or anything else: keep working with what is there.
            let reported: std::io::Error = if rename_unsupported(&error) {
                std::io::Error::new(error.kind(), "this filesystem cannot rename without replacing")
            } else {
                error
            };
            return Placement { path: earlier, notice: Some(unmovable_notice(parent, name, &reported)) };
        }
    }
}

/// What:
///  Where the kept name is,
///  with the earlier name moved to the current one when that was needed.
/// Why:
///   Every place that writes or reads one of the renamed names settles it here first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function placeKeptName(parent: string, name: RenamedName): Placement;
/// ```
pub fn place_kept_name(parent: &Path, name: RenamedName) -> Placement {
    return carry_out(look(parent, name), parent, name, rename_noreplace);
}

/// What:
///  Where a kept name is for a reader that must not move anything,
///  such as one that visits another worktree's Git directory:
///  the current name when it exists,
///  else the earlier one when only that exists,
///  else the current name.
/// Why:
///   Another worktree's folder may be in use by an older program at that moment;
///  only its own program moves it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function existingKeptName(parent: string, name: RenamedName): string;
/// ```
pub fn existing_kept_name(parent: &Path, name: RenamedName) -> PathBuf {
    if look(parent, name) == Look::OnlyEarlier {
        return parent.join(name.earlier);
    }
    return parent.join(name.current);
}

/// What:
///  The bits of the kinds already reported by this process,
///  one per distinct renamed name.
///  `AtomicU64` is a number that threads update without a lock.
/// Why:
///   A command settles the same name many times;
///  the person reads each notice once.
static REPORTED: AtomicU64 = AtomicU64::new(0);

/// The bit of a name in `REPORTED`:
///  its current spelling folded to a position,
///  so two different names rarely share one,
///  and a shared bit only drops a repeated line.
fn report_bit(name: RenamedName) -> u64 {
    let mut sum: u64 = 0;
    for byte in name.current.bytes() {
        sum = sum.wrapping_mul(31).wrapping_add(u64::from(byte));
    }
    return 1u64 << (sum % 64);
}

/// What:
///  `place_kept_name`,
///  with the notice written to standard error once per process and name.
/// Why:
///   The notice is the only report of an earlier name left beside the current one,
///  and it must never stop the command that asked for the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keptNameForUse(parent: string, name: RenamedName): string;
/// ```
pub fn kept_name_for_use(parent: &Path, name: RenamedName) -> PathBuf {
    let placed: Placement = place_kept_name(parent, name);
    if let Some(text) = placed.notice {
        let bit: u64 = report_bit(name);
        // `fetch_or` sets the bit and returns the bits before: a set bit means this name was already reported.
        let before: u64 = REPORTED.fetch_or(bit, Ordering::Relaxed);
        if before & bit == 0 {
            let _written = std::io::stderr().write_all(text.as_bytes());
        }
    }
    return placed.path;
}

#[cfg(test)]
#[path = "earlier_name_moved_to_current_in_place_tests.rs"]
mod tests;
