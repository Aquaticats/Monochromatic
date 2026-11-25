//! What:
//!  What one forwarded Git command does when it meets a managed clone's slot:
//!  the refusal of a move,
//!  the pass a prune runs over the wrapper's own records,
//!  and the repair report a repair command prints before Git starts.
//! Why:
//!  The design refuses `git worktree move` for a bound slot whether the clone is
//!  mounted or not,
//!  and its commands report the slot,
//!  its dataset and the one next safe action rather than acting on absent files
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  plan step 12).
//!  The words live in the ZFS folders that own them,
//!  and this file decides only when a command asks for them,
//!  so the command path stays one reading of Git's own words.
//!  Every refusal here changes nothing:
//!  it prints what happened,
//!  what is kept and the next safe command,
//!  and exits with the wrapper's refusal status.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = moveRefusalOf(command, commonDir) ?? pruneRefusalOf(command, commonDir, stderr);
//! ```

use crate::action::Action;
use crate::forwarded_command::ResolvedCommand;
use crate::subvolume_command::REFUSED_EXIT_CODE;
use crate::worktree_add_plan::start_directory;
use crate::worktree_clone_link_repair::live_link_report::{repair_paths, the_links_report};
use crate::worktree_clone_move_refusal::{ManagedMoveRefusal, refuse_move_of_a_bound_slot};
use crate::worktree_clone_prune_pointer::prune_guard::{PruneGuard, PruneRefusal, guard_the_prune};
use crate::worktree_clone_link_repair::link_report_lines;
use crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  The one refusal status a command that changed nothing ends in.
/// Why:
///  Every refusal of this wrapper uses it,
///  so a caller distinguishes a condition from a failure.
pub const REFUSAL_STATUS: i32 = REFUSED_EXIT_CODE;

/// What:
///  The subcommand word of one `git worktree` command,
///  which Git never abbreviates.
/// Why:
///  The move refusal,
///  the prune pass and the repair notice run for their own subcommand only,
///  and `git worktree` declares its subcommands rather than options;
///  each of them runs for every forwarded command,
///  so each reads this word before it reads anything else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const word = worktreeSubcommand('worktree', ['prune', '--verbose']); // 'prune'
/// ```
fn worktree_subcommand(command: &ResolvedCommand) -> Option<&[u8]> {
    if command.word() != b"worktree" {
        return None;
    }
    let first: &OsString = command.region().iter().find(|token| return !token.as_encoded_bytes().starts_with(b"-"))?;
    return Some(first.as_encoded_bytes());
}

/// What:
///  Whether this prune would remove nothing anyway.
/// Why:
///  A dry run must not write,
///  so the pass that restores a pointer stays out of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dry = pruneIsADryRun(['prune', '-nv']);
/// ```
fn prune_is_a_dry_run(command: &ResolvedCommand) -> bool {
    return command.region().iter().skip(1).any(|token| {
        let bytes: &[u8] = token.as_encoded_bytes();
        return bytes == b"--dry-run" || (bytes.starts_with(b"-") && !bytes.starts_with(b"--") && bytes.contains(&b'n'));
    });
}

/// What:
///  The refusal of one `git worktree move` of a bound managed clone,
///  as the action the command ends in.
/// Why:
///  The mounted and the unmounted slot are the same code path,
///  and the kernel protects only the mounted one,
///  so the wrapper is the whole protection for the other.
///  Every forwarded command asks,
///  so the move word is read before the working directory is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ending = moveRefusalOf(command, commonDir); // undefined leaves the move to Git
/// ```
pub fn move_refusal_of(command: &ResolvedCommand, common_dir: Option<&Path>) -> Option<Action> {
    // Every forwarded command asks, so the move word is read before the working directory is.
    if worktree_subcommand(command) != Some(b"move") {
        return None;
    }
    let common: &Path = common_dir?;
    let current: PathBuf = std::env::current_dir().ok()?;
    let start: PathBuf = start_directory(&command.arguments[..command.subcommand_index], current.as_path())?;
    let source = crate::worktree_move_source::parse_move_source(command.arguments.as_slice(), command.subcommand_index, start.as_path(), Some(common))?;
    let refusal: ManagedMoveRefusal = refuse_move_of_a_bound_slot(common, source.source.as_path())?;
    return Some(Action::Exit {
        code: REFUSAL_STATUS,
        stdout: String::new(),
        stderr: format!("{}\n", refusal.sentence),
    });
}

/// What:
///  The pass before one `git worktree prune`:
///  its notices go to standard error,
///  and a slot nothing restores ends the command as a refusal.
/// Why:
///  An explicit prune drops a registration whose pointer is gone at once,
///  so the pointer is put back first,
///  and a prune that would lose a registration nothing can restore stops
///  before Git starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ending = beforeAPrune(command, commonDir, stderr); // undefined lets the prune run
/// ```
pub fn before_a_prune(command: &ResolvedCommand, common_dir: Option<&Path>, stderr: &mut String) -> Option<Action> {
    let common: &Path = common_dir?;
    if worktree_subcommand(command) != Some(b"prune") || prune_is_a_dry_run(command) {
        return None;
    }
    let guard: PruneGuard = guard_the_prune(common, &LiveMountedDatasets);
    for line in guard.lines.iter() {
        stderr.push_str(line.as_str());
        stderr.push('\n');
    }
    if guard.refused.is_empty() {
        return None;
    }
    let mut refusal: String = String::new();
    for refused in guard.refused.iter() {
        match refused {
            PruneRefusal::Record { message, .. } => {
                refusal.push_str(message.as_str());
                refusal.push('\n');
            }
            PruneRefusal::Link(facts) => {
                for line in link_report_lines(facts) {
                    refusal.push_str(line.as_str());
                    refusal.push('\n');
                }
            }
        }
    }
    refusal.push_str(&format!(
        "nothing was pruned: Git would drop the registration of a managed clone and the clone dataset would then be reachable by nobody. Restore the pointer from the record, or run the removal, then run the prune again.\n"
    ));
    return Some(Action::Exit { code: REFUSAL_STATUS, stdout: String::new(), stderr: refusal });
}

/// What:
///  The report one `git worktree repair` prints before Git runs.
/// Why:
///  Git `2.56.1` rebuilds a missing pointer and still exits 1,
///  and a registration Git already pruned loses HEAD,
///  the index and the lock,
///  so a person reads what the repair can and cannot restore before they act.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// beforeARepair(command, commonDir, start, stderr);
/// ```
pub fn before_a_repair(command: &ResolvedCommand, common_dir: Option<&Path>, stderr: &mut String) {
    let Some(common) = common_dir else {
        return;
    };
    // Every forwarded command asks, so the repair word is read before the working directory is.
    let repair: bool = command.word() == b"worktree" && command.arguments.get(command.subcommand_index + 1).is_some_and(|token| return token.as_encoded_bytes() == b"repair");
    if !repair {
        return;
    }
    let Ok(current) = std::env::current_dir() else {
        return;
    };
    let Some(start) = start_directory(&command.arguments[..command.subcommand_index], current.as_path()) else {
        return;
    };
    let paths: Vec<PathBuf> = repair_paths(command.arguments.as_slice(), command.subcommand_index, start.as_path(), Some(common));
    for path in paths {
        let Some(report) = the_links_report(common, path.as_path(), &LiveMountedDatasets) else {
            continue;
        };
        for line in report.lines() {
            stderr.push_str(line.as_str());
            stderr.push('\n');
        }
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "entry_zfs_slot_commands_tests.rs"]
mod tests;
