//! What:
//! `mpm-shadow bin [--json]`:
//! plain output prints the exposure directory,
//! and `--json` prints the document the IDE launcher reads
//! (`exposureDir`,
//! `pathDirs` and `bins`).
//! Why:
//! Plain output is what `pnpm bin` prints,
//! "the directory into which the executables of dependencies are linked",
//! and pnpm's `bin` page has no `--json`,
//! so `--json` follows the `--json` of pnpm's `list` and `outdated` pages.
//! The JSON document reads only the plan and the file system:
//! the IDE sandbox may run this before any lock exists,
//! so a missing lock is not a refusal here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function binVerb(matches: ParsedArgs, scope: Scope, env: EnvMap): number
//! ```

/// Imports the `clap` type for reading a parsed command line.
use clap::ArgMatches;

/// Imports what this module uses from `super::super::bin_paths`.
use super::super::bin_paths::{
    bin_paths_output,
    render_bin_paths,
};
/// Imports `read_plan` from `super::super::plan`.
use super::super::plan::read_plan;
/// Imports what this module uses from `super::super::scope`.
use super::super::scope::{
    EnvMap,
    Scope,
};
/// Imports what this module uses from `super::what_every_verb_prints`.
use super::what_every_verb_prints::{
    report,
    warn_older_locks,
};

/// What:
/// Runs `bin` with the arguments `matches` in `scope` and returns the exit code.
/// Why:
/// The plain path is printed without reading the plan,
/// so it answers in a tree whose plan cannot be read,
/// and both forms warn about an older lock on standard error,
/// since the path or the JSON is on standard output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function binVerb(matches: ParsedArgs, scope: Scope, env: EnvMap): number
/// ```
pub(super) fn bin_verb(
    matches: &ArgMatches,
    scope: &Scope,
    env: &EnvMap,
) -> i32 {
    // Plain output is what `pnpm bin` prints: the directory the executables are linked into.
    if !matches.get_flag("json") {
        println!("{}", scope.exposure_dir.display());
        warn_older_locks("bin", scope);
        return 0;
    }
    // Only the plan is read: the IDE sandbox may run this before any lock exists.
    let outcome = match read_plan(&scope.plan_file) {
        Ok(plan) => bin_paths_output(&plan, scope, env),
        Err(error) => Err(error),
    };
    return match outcome {
        Ok(document) => {
            print!("{}", render_bin_paths(&document));
            warn_older_locks("bin", scope);
            0
        }
        Err(error) => report(&error),
    };
}
