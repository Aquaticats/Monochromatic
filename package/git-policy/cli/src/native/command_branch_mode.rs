//! What:
//!  Whether a tokenized `git branch` region creates a branch in Git 2.56.0.
//! Why:
//!  `git branch` picks one action from its options and treats positional names as new
//!      branches only when no other action applies (builtin/branch.c:1083-1097,
//!  1134-1311).
//!      Presentation options such as `-v`,
//!  `--color` and `--format` do not select listing,
//!      so `git branch -v topic` creates `topic`.
//! Gotcha:
//!  The answer reads option names and argument counts only.
//!  A command Git refuses
//!         for an option value (`--track=bogus`,
//!  `--column` with `-v`),
//!  for configuration
//!         (`--recurse-submodules` without `submodule.propagateBranches`) or for repository
//!         state (an existing name,
//!  a missing copy source) still answers "creates".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // branchCreates(parsed, positionalCount): boolean
//! ```

/// What:
///  Bring the branch option identifiers and the tokenizer questions into this file.
/// Why:
///   The decision reads final option states,
///  never spellings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { COPY, DELETE /* ... */ } from './command_branch_table.ts';
/// ```
use super::command_branch_table::{
    COPY, COPY_FORCE, DELETE, DELETE_FORCE, DELETE_MERGED, DRY_RUN, EDIT_DESCRIPTION, FILTER, KIND,
    LIST, MOVE, MOVE_FORCE, POINTS_AT, RECURSE_SUBMODULES, SET_UPSTREAM, SET_UPSTREAM_TO,
    SHOW_CURRENT, TRACK, UNSET_UPSTREAM,
};
use super::command_options::ParsedOptions;
use super::command_options_query::{is_enabled, is_stated};

/// What:
///  Whether the tracking mode ends as the one `--set-upstream` writes.
/// Why:
///   Git refuses to create a branch in that mode (branch.c:1298-1299).
///  `--track`,
///       `--no-track` and `--no-set-upstream` write the same variable,
///  so the last counts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let override = false; for (const o of occurrences) { /* last writer wins */ }
/// ```
fn track_is_override(parsed: &ParsedOptions) -> bool {
    let mut is_override: bool = false;
    // `for occurrence in &parsed.occurrences` borrows each record in the order Git applied it.
    for occurrence in &parsed.occurrences {
        if occurrence.id == SET_UPSTREAM {
            is_override = !occurrence.negated;
        }
        if occurrence.id == TRACK {
            is_override = false;
        }
    }
    return is_override;
}

/// What:
///  Count the `true` entries of a list.
///  `&[bool]` borrows the list;
///  `usize` is the
///       count type every list length uses.
/// Why:
///   Git adds up the selected actions and refuses more than one (branch.c:1093-1097).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const count = flags.filter(Boolean).length;
/// ```
fn count_true(flags: &[bool]) -> usize {
    let mut count: usize = 0;
    for flag in flags {
        // `*flag` reads the `bool` the loop variable points at.
        if *flag {
            count += 1;
        }
    }
    return count;
}

/// What:
///  Decide whether the region creates a branch.
///  `positional_count` is the number of
///       non-option arguments.
/// Why:
///   A copy creates its target;
///  otherwise a branch is created only when no action was
///       selected,
///  one or two names are given,
///  and no option makes Git refuse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function branchCreates(parsed: ParsedOptions, positionalCount: number): boolean;
/// ```
pub(crate) fn branch_creates(parsed: &ParsedOptions, positional_count: usize) -> bool {
    // `-d`/`--delete` is one bit that `--no-delete` clears; `-D` is another bit.
    let delete: bool = is_enabled(parsed, DELETE) || is_stated(parsed, DELETE_FORCE);
    let rename: bool = is_enabled(parsed, MOVE) || is_stated(parsed, MOVE_FORCE);
    let copy: bool = is_enabled(parsed, COPY) || is_stated(parsed, COPY_FORCE);
    let new_upstream: bool = is_enabled(parsed, SET_UPSTREAM_TO);
    let show_current: bool = is_enabled(parsed, SHOW_CURRENT);
    let edit_description: bool = is_enabled(parsed, EDIT_DESCRIPTION);
    let unset_upstream: bool = is_enabled(parsed, UNSET_UPSTREAM);
    let delete_merged: bool = is_stated(parsed, DELETE_MERGED);
    let filtered: bool = is_stated(parsed, FILTER) || is_enabled(parsed, POINTS_AT);
    let selected: usize = count_true(&[
        delete,
        rename,
        copy,
        new_upstream,
        show_current,
        edit_description,
        unset_upstream,
        delete_merged,
    ]);
    // Listing is explicit or implied by a filter (1088-1091). Git also lists when no action
    // and no name is given (1083-1086); with no name nothing is created either way, so that
    // default changes no answer here and is left out.
    let list: bool = is_enabled(parsed, LIST) || filtered;
    let actions: usize = selected + count_true(&[list]);
    if actions > 1 {
        // Git prints usage and exits (1096-1097).
        return false;
    }
    if is_enabled(parsed, DRY_RUN) && !delete_merged {
        // "--dry-run requires --delete-merged" (1099-1100).
        return false;
    }
    if is_enabled(parsed, RECURSE_SUBMODULES) && actions != 0 {
        // "--recurse-submodules can only be used to create branches" (1102-1107).
        return false;
    }
    let named: bool = positional_count == 1 || positional_count == 2;
    if copy {
        return named;
    }
    if actions != 0 {
        return false;
    }
    // Creation refuses `-a`/`-r` and the `--set-upstream` tracking mode (1294-1299).
    return named && !is_stated(parsed, KIND) && !track_is_override(parsed);
}
