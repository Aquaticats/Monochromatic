//! What:
//!  The worktree one `git worktree move <worktree> <new-path>` renames,
//!  resolved the way Git resolves that operand.
//! Why:
//!  The wrapper refuses a move of a managed clone's slot,
//!  and the refusal must read the slot the command actually renames
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  plan step 12).
//!  Git matches the operand against the last components of the registered
//!  paths first and reads it as a path only when that match is not unique,
//!  so the reading is the removal's own rule rather than a second one written
//!  here;
//!  the plan is absent for a form the reading does not know,
//!  and Git then answers the command as it always does.
//!  The operands themselves come from `worktree_operands`,
//!  so this reading and
//!  the removal's cannot disagree about which word names the worktree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const source = parseMoveSource(['worktree', 'move', 'topic', '../elsewhere'], 0, cwd, commonDir);
//! ```

use crate::worktree_operands::operands_after_options;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The worktree a `git worktree move` renames and the operand that named it.
/// Why:
///  A person may name the worktree by its whole path or by a last component,
///  and the refusal compares the record with the path Git picked,
///  so both the resolved path and the typed name travel together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MoveSource = { source: string; operand: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MoveSource {
    /// The worktree directory Git renames.
    pub source: PathBuf,
    /// The operand as the person typed it.
    pub operand: OsString,
}

/// What:
///  The source of one `git worktree move`,
///  when the form is the plain one.
/// Why:
///  The source is the first operand after `move`,
///  where an option or a `--` separator may stand before it,
///  and the destination is the next operand,
///  and a form without the destination operand is Git's to refuse.
///  The destination is not read:
///  the refusal is the same wherever the slot would be renamed to.
///  The start directory is optional because Git's own match against the
///  registered paths needs none:
///  a spelling whose global options the caller does not serve can still name a
///  worktree by a name the registry knows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseMoveSource(args: string[], subcommandIndex: number, start?: string, commonDir?: string): MoveSource | undefined;
/// ```
pub fn parse_move_source(
    arguments: &[OsString],
    subcommand_index: usize,
    start: Option<&Path>,
    common_dir: Option<&Path>,
) -> Option<MoveSource> {
    if arguments.get(subcommand_index)?.as_encoded_bytes() != b"worktree"
        || arguments.get(subcommand_index + 1)?.as_encoded_bytes() != b"move"
    {
        return None;
    }
    let rest: &[OsString] = &arguments[subcommand_index + 2..];
    let operands: Vec<&OsString> = operands_after_options(rest);
    let [operand, _destination, ..] = operands.as_slice() else {
        return None;
    };
    if operand.is_empty() {
        return None;
    }
    // Git matches the typed name against the registered paths first, and a name that matches one of them names that worktree.
    if let Some(common) = common_dir
        && let Some(source) =
            crate::worktree_remove_registry::path_git_picks_for_operand(common, operand.as_os_str())
    {
        return Some(MoveSource {
            source,
            operand: (*operand).clone(),
        });
    }
    let source: PathBuf = std::fs::canonicalize(start?.join(operand)).ok()?;
    return source.is_dir().then_some(MoveSource {
        source,
        operand: (*operand).clone(),
    });
}

/// The move source's controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_move_source_tests.rs"]
mod tests;
