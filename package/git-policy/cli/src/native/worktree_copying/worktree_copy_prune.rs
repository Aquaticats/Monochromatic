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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const prune = { kind: 'main-checkout', runRecords: COPY_RUN_RECORDS };
//! if (skipsPath(prune, 'package/x/.cache/typescript')) continue;
//! const STAGE_PREFIX = '.cli-git-worktree-copy-';
//! ```

/// The prefix of every stage container any rung makes beside its destination.
use crate::worktree_copy_selection::{STAGE_PREFIX, is_separator_byte};

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
///  Whether a path is the place a snapshot's leave-out makes its trash at,
/// or anything below it.
/// Why:
///   Every worktree keeps that place ignored for the same reason,
/// and the directories below it are what an add set aside,
/// so a copy that
///  carried them would put another command's trash inside the new worktree with no manifest that names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesTheTrashParent = (path) => path === '.claude/cli-git-trash' || path.startsWith('.claude/cli-git-trash/');
/// ```
fn names_the_trash_parent(relative_path: &[u8]) -> bool {
    let parent: &[u8] = crate::worktree_trash::TRASH_PARENT.as_bytes();
    return relative_path == parent || (relative_path.starts_with(parent) && relative_path.get(parent.len()) == Some(&b'/'));
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
        // A trash an earlier add left is no state of the source either: the commit service empties it after that add returns,
        //  and until then it holds directories nothing of this copy's names, so carrying it would put an unrecorded trash of
        //  another command inside the new worktree.
        if names_the_trash_parent(relative_path) {
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
