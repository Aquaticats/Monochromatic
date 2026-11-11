//! What:
//!  What a copy from a main checkout leaves out of the ignored state it hands over:
//!  caches,
//! optionally the records of earlier runs,
//! the stage container of another command,
//! and every directory that is another checkout.
//! Why:
//!  A worktree added from the main checkout should work at once,
//! so builds and installed dependencies are copied,
//!  but a cache is rebuilt by its tool,
//! a log of an earlier run is no input,
//! another command's stage container holds
//!  another worktree's ignored state (credentials included),
//! and another checkout's files would make the
//!  new worktree a second claimant of that checkout's administrative state.
//!  The rule is a handful of directory names rather than a list of paths,
//! so a reader can predict it
//!  (`doc/planning/cli-git-main-worktree-ignored-copy.md` measures the set it leaves out).
//! A folder that held nothing but a container is no state either:
//! the tree walk of a conversion
//!  leaves it out whole and the comparison that follows does not demand it,
//! so the converted worktree gains no empty
//!  folder nothing names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const prune = { kind: 'main-checkout', runRecords: COPY_RUN_RECORDS };
//! if (skipsPath(prune, 'package/x/.cache/typescript')) continue;
//! const STAGE_PREFIX = '.cli-git-worktree-copy-';
//! ```

/// The prefix of every stage container any rung makes beside its destination.
use crate::path_bytes::path_bytes_of;
use crate::worktree_copy_selection::{STAGE_PREFIX, is_separator_byte};
use crate::worktree_trash_manifest::is_trash_name;
use std::path::{Path, PathBuf};

/// What:
///  Whether the records of earlier runs (logs,
/// mutation and test reports,
/// verification output) are copied.
/// Why:
///   The owner answered on 2026-11-02 "Skip run records (Recommended)",
/// so they are left out;
///  setting it to `true` is the one-line change that copies them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const COPY_RUN_RECORDS = false;
/// ```
pub const COPY_RUN_RECORDS: bool = false;

/// What:
///  Directory names that are caches wherever they sit below the source root.
/// Why:
///   A package-manager store or a build tool's project cache is rebuilt by the tool that made it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CACHE_NAMES = ['.pnpm-store', '.gradle', '.kotlin', '.zig-cache'] as const;
/// ```
const CACHE_NAMES: [&str; 4] = [".pnpm-store", ".gradle", ".kotlin", ".zig-cache"];

/// What:
///  Directory names that are caches only inside a Cargo `target` directory.
/// Why:
///   `incremental` holds rustc's incremental-compilation state and `zig-global-cache` the Zig compiler's cache;
///  the compiled dependencies beside them are the build the new worktree needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TARGET_CACHE_NAMES = ['incremental', 'zig-global-cache'] as const;
/// ```
const TARGET_CACHE_NAMES: [&str; 2] = ["incremental", "zig-global-cache"];

/// What:
///  Directory names that hold the output of earlier test,
/// mutation and lint runs wherever they sit.
/// Why:
///   They are reports,
/// not inputs;
/// they are copied only while `COPY_RUN_RECORDS` is set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const RUN_RECORD_NAMES = ['mutants.out', 'mutants.out.old', 'playwright-report', 'test-results'] as const;
/// ```
const RUN_RECORD_NAMES: [&str; 4] = ["mutants.out", "mutants.out.old", "playwright-report", "test-results"];

/// What:
///  The name Git gives the entry that makes a directory a checkout of its own.
/// Why:
///   A directory that holds it,
/// file or directory,
/// is another checkout,
/// never ignored state of this one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CHECKOUT_MARKER = '.git';
/// ```
const CHECKOUT_MARKER: &str = ".git";

/// What:
///  The leave-out rule one copy applies while it walks a source.
/// Why:
///   A linked or bare source hands over everything it ignores,
/// so only the main checkout has a rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Prune = { kind: 'never' } | { kind: 'main-checkout'; runRecords: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Prune {
    /// Nothing is left out beyond the registered worktrees and the stage.
    Never,
    /// The main checkout's rule;
    /// `run_records` says whether the records of earlier runs are copied.
    MainCheckout {
        /// Whether run records (reports and logs of earlier runs) are copied.
        run_records: bool,
    },
}

/// What:
///  Whether one component's bytes are one of these names.
/// Why:
///   The stage prefix and the leave-out rule compare a path's own bytes with a short list of
/// names,
/// so one loop answers for every list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// names.includes(component)
/// ```
fn component_is_one_of(component: &[u8], names: &[&str]) -> bool {
    for name in names {
        if component == name.as_bytes() {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether any child name is the marker of a checkout.
/// Why:
///   A directory holding the marker is another checkout and is left out whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// childNames.includes(CHECKOUT_MARKER)
/// ```
fn names_checkout_marker(child_names: &[Vec<u8>]) -> bool {
    for name in child_names {
        if name.as_slice() == CHECKOUT_MARKER.as_bytes() {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether a path is a trash directory the wrapper made,
/// or anything below it.
/// Why:
///   A worktree keeps the trash parent out of every commit,
/// so a person may keep files the repository ignores there;
///  only a directory named the way the wrapper names a trash is a trash,
/// and the person's own directories below the parent are state of the
///  source.
/// A copy that took the whole parent for trash dropped those files from the new worktree,
/// and a snapshot took
///  them into this add's trash,
/// where the service deleted them.
/// The path is anchored at the source root,
/// so a directory
///  of that name deeper down is not what the wrapper leaves out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesAWrappersTrash = (path) => { const [below, ...rest] = splitFirst(path, '/.claude/cli-git-trash/'); return isTrashName(below); };
/// ```
fn names_a_wrappers_trash(relative_path: &[u8]) -> bool {
    let parent: &[u8] = crate::worktree_trash::TRASH_PARENT.as_bytes();
    let Some(below_parent) = relative_path.strip_prefix(parent) else { return false };
    let Some(below_slash) = below_parent.strip_prefix(b"/") else { return false };
    let mut components = below_slash.split(is_separator_byte);
    let Some(component) = components.next() else { return false };
    let Ok(name) = std::str::from_utf8(component) else { return false };
    return is_trash_name(name);
}

/// What:
///  Whether any component of a repository path begins the name of a private stage container.
/// Why:
///   A stage container belongs to the one add or copy that is filling it and holds another worktree's ignored state,
/// so no walk of a source may enter it and no manifest may carry it;
/// the rule is about a component,
///  so a file in a directory whose name merely begins with the prefix is not left out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namesAPrivateStage('.claude/worktrees/.cli-git-worktree-copy-8f14e45fceea167a5a36dedd4bea2543/payload/node_modules/pkg/index.js') // true
/// ```
pub fn names_a_private_stage(relative_path: &[u8]) -> bool {
    for component in relative_path.split(is_separator_byte) {
        if component.starts_with(STAGE_PREFIX.as_bytes()) {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether everything a folder holds is another command's stage container,
///  at any depth,
///  with `empty_folders_are_state` saying what a folder with nothing in it is.
/// Why:
///   A container holds the whole ignored state another worktree's copy is filling and no copy carries it,
/// so the folders
///  that held nothing else are no state of the tree either:
/// a copy leaves them out and a comparison must not demand them.
///  A folder that a command emptied by removing only its container is the one case the two callers
///  answer differently:
/// a copy that made a destination folder and finds its source empty has nothing to hand over,
///  while a folder the installed worktree does not hold at all was left out of the copy because it
///  held only containers then,
/// so what arrived in it since is what the comparison has to judge.
///  Everything else is state,
/// so a folder holding one other entry is state,
/// and a name that begins the stage prefix
///  without being a directory is not a container and is state too.
/// A folder this
///  user cannot list is not proven to hold nothing but a container,
/// so it counts as state as well.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// leavesOutWhole(directory, true)
/// // true when that folder holds only `.cli-git-worktree-copy-*` folders and the folders around those
/// ```
fn leaves_out_whole(directory: &Path, empty_folders_are_state: bool) -> bool {
    // The walk uses a work stack, not recursion, so a deep tree cannot overflow the stack.
    let mut pending: Vec<PathBuf> = vec![directory.to_path_buf()];
    while let Some(current) = pending.pop() {
        let listing: std::fs::ReadDir = match std::fs::read_dir(current.as_path()) {
            Ok(found) => found,
            Err(_unreadable) => return false,
        };
        let mut holds_entry: bool = false;
        for item in listing {
            let entry: std::fs::DirEntry = match item {
                Ok(found) => found,
                Err(_unreadable) => return false,
            };
            holds_entry = true;
            let file_name: std::ffi::OsString = entry.file_name();
            // Only a folder is another command's stage container, and only a folder can hold nothing but one further down; a
            // link is not followed, because the folder it names is somewhere else and is nobody's to leave out.
            match entry.file_type() {
                Ok(kind) if kind.is_dir() => {}
                _ => return false,
            }
            // The rule for a name is the one `names_a_private_stage` states, read on the one component this entry is.
            if names_a_private_stage(path_bytes_of(Path::new(file_name.as_os_str()))) {
                continue;
            }
            pending.push(current.join(file_name));
        }
        if !holds_entry && empty_folders_are_state {
            return false;
        }
    }
    return true;
}

/// What:
///  Whether a source folder holds nothing but another command's stage containers,
///  every folder below it holding at least one.
/// Why:
///   The tree walk of a conversion asks this before it forgets the destination folder it made,
/// so a folder that held an empty folder of the person's own as well keeps its counterpart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdsOnlyPrivateStages('.claude/worktrees')
/// ```
pub fn holds_only_private_stages(directory: &Path) -> bool {
    return leaves_out_whole(directory, true);
}

/// What:
///  Whether a folder the installed worktree does not hold carries nothing a copy would have carried:
///  every entry below it is another command's stage container or a folder of them,
///  and a folder emptied by removing only its container counts as nothing.
/// Why:
///   Such a folder was left out of the copy because it held only containers at the time,
/// so its absence is what the copy asked for and the comparison must not demand it;
///  anything that arrived in it since is named by the entry walk of the comparison itself,
///  which is why only the emptiness of the folder is judged here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdsNothingACopyCarries('.claude/worktrees')
/// ```
pub fn holds_nothing_a_copy_carries(directory: &Path) -> bool {
    return leaves_out_whole(directory, false);
}

impl Prune {
    /// What:
    ///  The rule the main checkout uses,
    /// with the run-record setting the crate ships.
    /// Why:
    ///   One place reads the shipped constant,
    /// so changing the owner's answer is a one-line change.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// Prune.mainCheckout() // { kind: 'main-checkout', runRecords: COPY_RUN_RECORDS }
    /// ```
    pub fn main_checkout() -> Prune {
        return Prune::MainCheckout { run_records: COPY_RUN_RECORDS };
    }

    /// What:
    ///  Whether the path,
    /// relative to the source root,
    /// and everything below it is left out.
    /// Why:
    ///   The decision uses directory names only,
    /// so it is the same for a root Git listed and a child the walk found.
    /// A stage container is left out by every rule,
    /// because it belongs to the one add or copy that is
    ///  filling it and holds another worktree's ignored state.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// skipsPath(prune, 'package/x/target/debug/incremental') // true
    /// skipsPath(prune, '.claude/worktrees/.cli-git-worktree-copy-8f14e45fceea167a5a36dedd4bea2543') // true
    /// ```
    pub fn skips_path(self, relative_path: &[u8]) -> bool {
        // Another command's stage container is no state of the source, whatever the rule: its payload is a full copy of
        //  another worktree's ignored state, made while its own copy runs, and nothing names it in the new worktree.
        if names_a_private_stage(relative_path) {
            return true;
        }
        // A trash a wrapper command made and recorded is no state of the source either: what it holds is another add's left-out
        //  directory, and a copy that carried it would put a trash nothing names inside the new worktree. A directory below the
        //  trash parent that is not named like one is the person's own state, and is copied.
        if names_a_wrappers_trash(relative_path) {
            return true;
        }
        let components: Vec<&[u8]> = relative_path.split(is_separator_byte).collect();
        let Prune::MainCheckout { run_records } = self else {
            return false;
        };
        for (index, component) in components.iter().enumerate() {
            let parent: Option<&[u8]> = match index.checked_sub(1) {
                Some(before) => Some(components[before]),
                None => None,
            };
            let below_target: bool = components[..index].contains(&"target".as_bytes());
            // The repository root's own `.cache` is where the repository keeps tools and generated inputs that tasks read.
            let nested_cache: bool = *component == b".cache" && index > 0;
            let cache: bool = component_is_one_of(component, &CACHE_NAMES)
                || nested_cache
                || (below_target && component_is_one_of(component, &TARGET_CACHE_NAMES));
            let record: bool = component_is_one_of(component, &RUN_RECORD_NAMES)
                || (*component == b".monochromatic" && components[..index].contains(&"node_modules".as_bytes()))
                || (*component == b"verification" && parent == Some(b"target".as_slice()))
                || (*component == b"cache" && parent == Some(b".serena".as_slice()));
            if cache || (record && !run_records) {
                return true;
            }
        }
        return false;
    }

    /// What:
    ///  Whether a directory with these child names is another checkout and is left out whole.
    /// Why:
    ///   The registered worktrees are excluded by path;
    /// this also catches a clone or a stale worktree directory
    ///  that Git no longer registers.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// skipsCheckout(prune, ['.git', 'src']) // true
    /// ```
    pub fn skips_checkout(self, child_names: &[Vec<u8>]) -> bool {
        return match self {
            Prune::Never => false,
            Prune::MainCheckout { .. } => names_checkout_marker(child_names),
        };
    }
}

/// Prune rule controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_prune_tests.rs"]
mod tests;
