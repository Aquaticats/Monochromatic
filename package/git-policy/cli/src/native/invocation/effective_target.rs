//! What:
//!  Classify the worktree an invocation targets for worktree-enforcing policies.
//! Why:
//!  Linked-worktree safeguards discipline a person's repositories,
//!  not the
//!      disposable clones a tool keeps in its own cache
//!  or in the directory a build step of that tool was given.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // classifyEffectiveTarget(identity, allowedDirs) === 'linked-worktree'
//! ```

/// Import getenv-style lookup and the resolved repository identity.
use crate::child_environment::environment_value;
use crate::worktree_identity::WorktreeIdentity;
/// What:
///  `OsString` is owned operating-system text of raw OS bytes (sibling `String`
///       must be UTF-8).
/// Why:
///   Cache directories come from environment values that need not be UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // process.env values, but byte-preserving.
/// ```
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{Path, PathBuf};

/// What:
///  The four targets worktree policies distinguish.
///       `#[derive(...)]` generates copying,
///  debug printing and `==`.
/// Why:
///   A bare repository or no repository has no worktree to protect;
///  an
///       allowlisted tool cache is exempt;
///  main versus linked decides enforcement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EffectiveTarget = 'outside-worktree' | 'main-worktree' | 'linked-worktree' | 'allowlisted';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectiveTarget {
    /// No worktree:
    ///  outside any repository,
    ///  or a bare repository.
    OutsideWorktree,
    /// The repository's main worktree.
    MainWorktree,
    /// A linked worktree.
    LinkedWorktree,
    /// A repository under a tool-cache directory exempt from enforcement.
    Allowlisted,
}

/// What:
///  One tool cache directory exempt from worktree enforcement,
///  and the variable or default that named it.
///  A `struct` is a record with named fields.
/// Why:
///   The exemption is chosen by the invocation's own environment,
///  so a repository can be exempt because a
///       variable happens to name it;
///  a person who is told which variable did that can decide whether the
///       variable means what they think.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolCacheDirectory = { path: string; namedBy: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolCacheDirectory {
    /// The directory,
    ///  as the variable named it.
    pub path: PathBuf,
    /// The environment variable that named it,
    ///  or `HOME` for the default below a home directory.
    pub named_by: &'static str,
}

/// What:
///  Resolve uv's cache directory the way uv does:
///  `UV_CACHE_DIR`,
///  then
///       `XDG_CACHE_HOME/uv`,
///  then `HOME/.cache/uv`;
///  an empty value counts as unset.
///       `Option<&Path>` is "a borrowed home directory or nothing";
///       `Option<PathBuf>` is "an owned path or nothing".
/// Why:
///   uv runs destructive Git inside throwaway clones it owns there.
///  The home
///       directory is injected so tests never read the real one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function uvCacheDir(environment: [string, string][], home?: string): string | undefined;
/// ```
pub fn uv_cache_dir(
    environment: &[(OsString, OsString)],
    home: Option<&Path>,
) -> Option<ToolCacheDirectory> {
    // `if let Some(x) = ... && cond` runs only when the variable is set and the second
    // condition also holds, like `x !== undefined && x !== ""`.
    if let Some(explicit) = environment_value(environment, "UV_CACHE_DIR")
        && !explicit.is_empty()
    {
        // `Some(...)` is the "present" variant.
        return Some(ToolCacheDirectory {
            path: PathBuf::from(explicit),
            named_by: "UV_CACHE_DIR",
        });
    }
    if let Some(cache_home) = environment_value(environment, "XDG_CACHE_HOME")
        && !cache_home.is_empty()
    {
        // `.join` appends one path segment and returns an owned path.
        return Some(ToolCacheDirectory {
            path: PathBuf::from(cache_home).join("uv"),
            named_by: "XDG_CACHE_HOME",
        });
    }
    // What: `let Some(x) = ... else { return None; };` unwraps the home or exits.
    // Why:  Without any hint and without a home directory there is no cache to exempt.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (home === undefined) return undefined;
    // ```
    let Some(home_directory) = home else {
        // `None` is the "absent" variant.
        return None;
    };
    return Some(ToolCacheDirectory {
        path: home_directory.join(".cache").join("uv"),
        named_by: "HOME",
    });
}

/// What:
///  Resolve mise's data directory the way mise does:
///  `MISE_DATA_DIR`,
///  then
///       `XDG_DATA_HOME/mise`,
///  then `HOME/.local/share/mise`;
///  an empty value counts as unset.
///       `Option<&Path>` is "a borrowed home directory or nothing";
///       `Option<PathBuf>` is "an owned path or nothing".
/// Why:
///   mise keeps each tool plugin as a Git clone under `plugins/` there and updates it with a
///       forced checkout,
/// which the main-worktree rule would reject.
///  The home directory is injected so
///       tests never read the real one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function miseDataDir(environment: [string, string][], home?: string): string | undefined;
/// ```
pub fn mise_data_dir(
    environment: &[(OsString, OsString)],
    home: Option<&Path>,
) -> Option<ToolCacheDirectory> {
    // `if let Some(x) = ... && cond` runs only when the variable is set and the second
    // condition also holds, like `x !== undefined && x !== ""`.
    if let Some(explicit) = environment_value(environment, "MISE_DATA_DIR")
        && !explicit.is_empty()
    {
        // `Some(...)` is the "present" variant.
        return Some(ToolCacheDirectory {
            path: PathBuf::from(explicit),
            named_by: "MISE_DATA_DIR",
        });
    }
    if let Some(data_home) = environment_value(environment, "XDG_DATA_HOME")
        && !data_home.is_empty()
    {
        // `.join` appends one path segment and returns an owned path.
        return Some(ToolCacheDirectory {
            path: PathBuf::from(data_home).join("mise"),
            named_by: "XDG_DATA_HOME",
        });
    }
    // What: `let Some(x) = ... else { return None; };` unwraps the home or exits.
    // Why:  Without any hint and without a home directory there is no data directory to exempt.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (home === undefined) return undefined;
    // ```
    let Some(home_directory) = home else {
        // `None` is the "absent" variant.
        return None;
    };
    return Some(ToolCacheDirectory {
        path: home_directory.join(".local").join("share").join("mise"),
        named_by: "HOME",
    });
}

/// What:
///  Cargo's output directory for one build script,
/// as the invocation's environment names it:
///  `OUT_DIR`,
///  which Cargo sets for every build script and passes on to what that script starts;
///  an empty value counts as unset.
/// Why:
///   A crate that vendors another project clones it into this directory and runs Git there
/// (`libghostty-vt-sys` cloned Ghostty into `OUT_DIR/ghostty-src`),
///  and that clone belongs to the build step rather than to the person,
///  so a guarded command inside it is exempt as one inside uv's cache or mise's data directory is.
///  Cargo decides the directory,
///  so the value is taken verbatim,
/// exactly as Cargo uses it;
///  the environment is injected so tests never read the real one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cargoBuildScriptOutputDir(environment: [string, string][]): { path: string; namedBy: string } | undefined;
/// ```
pub fn cargo_build_script_output_dir(
    environment: &[(OsString, OsString)],
) -> Option<ToolCacheDirectory> {
    // An empty value is unset: an empty path would exempt whatever the current directory is.
    if let Some(explicit) = environment_value(environment, "OUT_DIR")
        && !explicit.is_empty()
    {
        return Some(ToolCacheDirectory {
            path: PathBuf::from(explicit),
            named_by: "OUT_DIR",
        });
    }
    return None;
}

/// One name an invocation's environment can give Cargo's build directory,
///  and whether Cargo reads that name itself.
///  A `struct` is a record with named fields.
/// Why:
///   A relative value under a name Cargo reads describes a directory Cargo resolves against a base
///  this invocation cannot know,
///  so a later name does not describe the directory Cargo uses;
///  a relative value under a name Cargo never reads describes no directory at all,
///  so the later names are still worth asking
///  (`cargo_build_dir`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BuildDirectoryVariable = { name: string; readByCargo: boolean };
/// ```
struct BuildDirectoryVariable {
    /// The environment variable's name.
    name: &'static str,
    /// Whether Cargo itself reads this name,
    ///  which decides what a relative value under it means.
    read_by_cargo: bool,
}

/// The names an invocation's environment can give Cargo's build directory,
///  in the order the wrapper asks them.
/// Why:
///  The owner's answer of 2026-11-11 names `CARGO_BUILD_DIR` first;
///  `CARGO_BUILD_BUILD_DIR` is Cargo's own environment spelling of the `build.build-dir`
///  setting;
/// and `build.build-dir` defaults to `build.target-dir`,
///  which `CARGO_TARGET_DIR` names.
///  Cargo reads the last two and never the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CARGO_BUILD_DIRECTORY_VARIABLES = [{ name: 'CARGO_BUILD_DIR', readByCargo: false }, /* ... */];
/// ```
const CARGO_BUILD_DIRECTORY_VARIABLES: [BuildDirectoryVariable; 3] = [
    BuildDirectoryVariable { name: "CARGO_BUILD_DIR", read_by_cargo: false },
    BuildDirectoryVariable { name: "CARGO_BUILD_BUILD_DIR", read_by_cargo: true },
    BuildDirectoryVariable { name: "CARGO_TARGET_DIR", read_by_cargo: true },
];

/// What:
///  Cargo's build directory as the invocation's environment names it:
///  `CARGO_BUILD_DIR`,
///  then `CARGO_BUILD_BUILD_DIR`,
///  then `CARGO_TARGET_DIR`;
///  an empty value counts as unset,
///  and a relative value names nothing.
/// Why:
///   A crate that vendors another project clones it under this directory and runs Git there
/// (`libghostty-vt-sys` cloned Ghostty under the build directory),
///  and a step that runs Git in that clone works in the build's own tree,
///  so the directory is exempt as uv's cache and mise's data directory are
///  (`#3024`,
/// the owner's answer of 2026-11-11).
///  The three names are not the same setting to Cargo:
///  Cargo never reads `CARGO_BUILD_DIR`,
///  which the answer names;
///  `CARGO_BUILD_BUILD_DIR` is the environment spelling of `build.build-dir`
///  (Cargo's config reference),
/// and `build.build-dir` defaults to `build.target-dir`,
///  whose own variable is `CARGO_TARGET_DIR`.
///  A relative value names nothing because Cargo resolves one against a directory this
///  invocation cannot know:
///  `CARGO_TARGET_DIR` against Cargo's own working directory,
///  and a configuration value against the directory of the file that set it.
///  A Git process a build script starts works in that script's package directory,
///  so exempting a guess could cover a repository nobody named.
///  What a relative value leaves behind differs by the name that carried it:
///  under a name Cargo never reads it leaves the later names to be asked,
///  because it cannot describe or override the directory Cargo uses;
///  under a name Cargo reads it ends the question,
///  because that setting is what Cargo build there and no later name describes it.
///  The environment is injected so tests never read the real one.
/// Limit:
///  The exemption covers every repository under the build directory,
///  including one a person deliberately keeps there,
///  which then gets no worktree policies at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cargoBuildDir(environment: [string, string][]): { path: string; namedBy: string } | undefined;
/// ```
pub fn cargo_build_dir(
    environment: &[(OsString, OsString)],
) -> Option<ToolCacheDirectory> {
    // `for variable in CARGO_BUILD_DIRECTORY_VARIABLES` asks each name in the order the answer ranks them.
    for variable in CARGO_BUILD_DIRECTORY_VARIABLES {
        // What: `let Some(value) = ... else { continue; };` moves on when this name is absent.
        // Why:  The next name still describes the same directory.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const value = environment.get(name); if (value === undefined) continue;
        // ```
        let Some(value) = environment_value(environment, variable.name) else {
            continue;
        };
        // An empty value is unset: an empty path would exempt whatever the current directory is.
        if value.is_empty() {
            continue;
        }
        let path: PathBuf = PathBuf::from(value);
        if path.is_absolute() {
            return Some(ToolCacheDirectory { path, named_by: variable.name });
        }
        // What: `if !absolute` decides what a relative value leaves of the question.
        // Why:  Cargo resolves a relative value against its own base, which this invocation does not know,
        //       so a guess could exempt a directory nobody named. Only the name decides how much is left:
        //       a name Cargo never reads cannot describe the directory Cargo uses, so the next name is asked;
        //       a name Cargo reads is a setting Cargo build with, so no later name describes that directory.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!path.isAbsolute()) { if (readByCargo) return undefined; else continue; }
        // ```
        if variable.read_by_cargo {
            return None;
        }
    }
    return None;
}

/// What:
///  The tool directories exempt from worktree enforcement on this machine.
///       `Vec<PathBuf>` is an owned list of owned paths.
/// Why:
///   The set is a property of installed tooling,
///  not of any repository,
///  so it is
///       compiled in rather than configured.
///  Each entry is a directory whose Git clones a tool
///       creates and discards itself.
///  The Cargo build directory is the widest of them,
///  and its own doc states the limit that width costs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function defaultAllowedWorktreeDirs(environment, home): string[];
/// ```
pub fn default_allowed_worktree_dirs(
    environment: &[(OsString, OsString)],
    home: Option<&Path>,
) -> Vec<ToolCacheDirectory> {
    // `Vec::<PathBuf>::new()` is an empty owned list; `mut` allows pushing.
    let mut directories: Vec<ToolCacheDirectory> = Vec::<ToolCacheDirectory>::new();
    if let Some(uv) = uv_cache_dir(environment, home) {
        directories.push(uv);
    }
    if let Some(mise) = mise_data_dir(environment, home) {
        directories.push(mise);
    }
    // Cargo's build-script directory is the narrowest of its two: one build step's own output, not a whole build
    //  directory.
    if let Some(cargo) = cargo_build_script_output_dir(environment) {
        directories.push(cargo);
    }
    // The whole build directory covers the same clone when a step runs Git there without `OUT_DIR` in its
    //  environment; it is asked after `OUT_DIR` so the notice names the narrower directory when both cover the clone.
    if let Some(build_directory) = cargo_build_dir(environment) {
        directories.push(build_directory);
    }
    return directories;
}

/// What:
///  The first allowed directory that holds every one of `candidates`,
/// with the variable that named it.
/// Why:
///   An exemption covers a whole repository,
///  never one directory inside it:
///  a linked worktree keeps its Git directory in the main clone
///  (`<clone>/.git/worktrees/<name>`)
///  and its files wherever they were added,
///  so a Git directory alone would exempt a checkout standing outside the allowed directory
///  and let a guarded form overwrite the work kept there.
///  Each allowed directory is resolved through symbolic links first;
///  a directory that does not exist drops out,
///  and so does an empty candidate list,
///  which every directory would otherwise hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function matchingAllowedWorktreeDir(candidates: string[], allowedDirs): Promise<ToolCacheDirectory | undefined>;
/// ```
pub fn matching_allowed_worktree_dir<'dirs>(
    candidates: &[&Path],
    allowed_dirs: &'dirs [ToolCacheDirectory],
) -> Option<&'dirs ToolCacheDirectory> {
    // An empty question has no answer: with no path to ask about, every directory would qualify.
    if candidates.is_empty() {
        return None;
    }
    // `for directory in allowed_dirs` borrows each entry in turn.
    for directory in allowed_dirs {
        // What: `if let Ok(resolved) = ...` runs only when the directory could be resolved, and the loop
        //       below asks whether every candidate lies inside it.
        // Why:  A tool that is not installed has no cache directory to compare with.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let resolved; try { resolved = await realpath(directory); } catch { continue; }
        // ```
        if let Ok(resolved) = std::fs::canonicalize(directory.path.as_path()) {
            // What: every candidate is asked in turn, so the first one outside stops the question.
            // Why:  A closure would be the same test, and this form keeps the state in the loop's own names.
            let mut every_candidate_is_inside: bool = true;
            for candidate in candidates {
                if !candidate.starts_with(resolved.as_path()) {
                    every_candidate_is_inside = false;
                    break;
                }
            }
            if every_candidate_is_inside {
                return Some(directory);
            }
        }
    }
    return None;
}

/// What:
///  The paths that must all lie inside one allowed directory for a repository's work to be exempt:
///  its Git directory,
/// and the top of its worktree when it has one.
/// Why:
///  These are exactly the places a guarded command overwrites,
/// so a repository is exempt only when both stand inside the directory the variable names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exemptedRepositoryPaths(identity: WorktreeIdentity): string[];
/// ```
fn exempted_repository_paths(identity: &WorktreeIdentity) -> Vec<&Path> {
    // `match` binds the two paths each shape holds; a bare repository and no repository hold at most one.
    return match identity {
        WorktreeIdentity::OutsideWorktree => Vec::<&Path>::new(),
        WorktreeIdentity::BareRepository { git_dir, .. } => vec![git_dir.as_path()],
        WorktreeIdentity::MainWorktree {
            git_dir,
            worktree_root,
            ..
        }
        | WorktreeIdentity::LinkedWorktree {
            git_dir,
            worktree_root,
            ..
        } => vec![git_dir.as_path(), worktree_root.as_path()],
    };
}

/// What:
///  The allowed directory that exempts the whole repository `identity` names,
/// with the variable that named it.
/// Why:
///   The worktree guard protects the person's files,
/// and the exemption exists for a tool's own clone,
///  so both the Git directory and the worktree top must stand inside the same allowed directory;
/// one of them outside means part of the work is the person's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function matchingAllowedWorktreeDirForRepository(identity, allowedDirs): Promise<ToolCacheDirectory | undefined>;
/// ```
pub fn matching_allowed_worktree_dir_for_repository<'dirs>(
    identity: &WorktreeIdentity,
    allowed_dirs: &'dirs [ToolCacheDirectory],
) -> Option<&'dirs ToolCacheDirectory> {
    let paths: Vec<&Path> = exempted_repository_paths(identity);
    return matching_allowed_worktree_dir(paths.as_slice(), allowed_dirs);
}

/// What:
///  Map a repository identity to the target worktree policies act on.
/// Why:
///   Bare and absent repositories have no worktree to protect;
///  a repository whose Git directory and worktree top both stand under one allowed tool cache is exempt
///       before main/linked matters,
///  because the exemption covers the tool's own clone and nothing the tool writes elsewhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function classifyEffectiveTarget(identity: WorktreeIdentity, allowedDirs: string[]): EffectiveTarget;
/// ```
pub fn classify_effective_target(
    identity: &WorktreeIdentity,
    allowed_dirs: &[ToolCacheDirectory],
) -> EffectiveTarget {
    // What: `match` on the borrowed identity; `{ .. }` binds nothing and the shape alone decides.
    // Why:  Each shape maps to exactly one target, and the compiler checks none is missed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (identity.kind) { case 'outside-worktree': return 'outside-worktree'; ... }
    // ```
    match identity {
        WorktreeIdentity::OutsideWorktree | WorktreeIdentity::BareRepository { .. } => {
            return EffectiveTarget::OutsideWorktree;
        }
        WorktreeIdentity::MainWorktree { .. } => {
            if matching_allowed_worktree_dir_for_repository(identity, allowed_dirs).is_some() {
                return EffectiveTarget::Allowlisted;
            }
            return EffectiveTarget::MainWorktree;
        }
        WorktreeIdentity::LinkedWorktree { .. } => {
            if matching_allowed_worktree_dir_for_repository(identity, allowed_dirs).is_some() {
                return EffectiveTarget::Allowlisted;
            }
            return EffectiveTarget::LinkedWorktree;
        }
    }
}

/// Disposable-directory controls stay out of the release executable.
#[cfg(test)]
#[path = "effective_target_tests.rs"]
mod tests;
