//! What:
//! The environment and the Git program a native read runs with.
//! Why:
//! Split out of `git_refs.rs`,
//! which reads Git references without starting Git,
//! to keep that file under the code-line limit.
//! The items are re-exported from `git_refs`,
//! so their paths there are unchanged.

/// The predicate that finds a blocking variable that is set.
use crate::git_refs::is_set;
/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{PathBuf};

/// Re-exports the configuration reader,
/// which lives in `git_settings.rs`.
pub use crate::git_settings::read_config;

/// The environment variables that move the repository,
/// its references,
/// its objects or its configuration;
/// when one is set the answer is Git's,
/// not this reader's.
pub(crate) const BLOCKING_VARIABLES: [&str; 17] = [
    "GIT_DIR",
    "GIT_COMMON_DIR",
    "GIT_WORK_TREE",
    "GIT_NAMESPACE",
    "GIT_CONFIG",
    "GIT_CONFIG_GLOBAL",
    "GIT_CONFIG_SYSTEM",
    "GIT_CONFIG_NOSYSTEM",
    "GIT_CONFIG_COUNT",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CEILING_DIRECTORIES",
    "GIT_DISCOVERY_ACROSS_FILESYSTEM",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_REPLACE_REF_BASE",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_REF_PARANOIA",
];

/// The reference namespaces whose files live in the common directory and that the reader looks up.
pub(crate) const SHARED_NAMESPACES: [&str; 3] = ["refs/heads/", "refs/remotes/", "refs/tags/"];

/// The place the process environment puts Git's inputs,
/// so tests can give a disposable home.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeEnvironment {
    /// The home directory (`HOME`),
    /// where `~/.gitconfig` and `~/.config/git/config` live.
    pub home: Option<PathBuf>,
    /// `XDG_CONFIG_HOME`,
    /// when set and not empty.
    pub xdg_config_home: Option<PathBuf>,
    /// The system configuration file.
    /// This is `/etc/gitconfig`,
    /// which distribution builds of Git read;
    /// a Git built for another prefix reads elsewhere and is not seen
    /// (the limit is stated in the README of this package).
    pub system_config: Option<PathBuf>,
    /// The name of the first blocking variable that is set.
    pub blocking_variable: Option<String>,
    /// The Git program that answers when this reader declines.
    pub git: GitProgram,
    /// The user id Git compares the repository's owner with (its dubious-ownership check).
    pub user_id: u32,
}

/// How the Git program that answers a declined choice is started and bounded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitProgram {
    /// The program to start.
    pub program: PathBuf,
    /// The longest one Git call may take (owner rule of 2026-11-03:
    /// "nothing should ever wait forever or indefinitely");
    /// a bound chosen for local repository reads,
    /// not measured.
    pub bound: std::time::Duration,
    /// Start Git with an empty environment plus `environment` instead of this process's own.
    pub clean_environment: bool,
    /// Variables set for the Git call.
    pub environment: Vec<(String, String)>,
}

/// The bound of one Git call,
/// in seconds.
/// Measured 2026-11-04 on this repository (load average 6.2 to 6.9 on 32 CPUs,
/// capped scopes,
/// 3 runs each,
/// `~/temp/agent/bench_bounds.py` and `bench_online.py`):
/// `git cat-file --batch-check` with the four inputs took 0.007 s to 0.018 s,
/// `git rev-parse --verify HEAD^{commit}` 0.007 s to 0.008 s,
/// `git show HEAD:<dir>/Cargo.lock` 0.008 s to 0.009 s and `git var GIT_CONFIG_SYSTEM` 0.006 s to 0.007 s,
/// all in a warm repository.
/// A minute is about 3,000 times the slowest of these and leaves room for a cold disk or a promisor fetch,
/// neither of which was measured.
/// The owner's rule of 2026-11-03 justifies that a bound exists;
/// the number is a Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open.
pub const GIT_CALL_SECONDS: u64 = 60;

/// The methods of the type above.
impl Default for GitProgram {
    /// What:
    /// `fn default() -> GitProgram` is the Git on the search path,
    /// with the process environment and the standard bound.
    /// Why:
    /// Production starts `git` as the user's shell would.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const defaultGit = { program: 'git', boundSeconds: 60, cleanEnvironment: false, environment: [] };
    /// ```
    fn default() -> GitProgram {
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        // What:     `Duration` is a length of time and `Instant` a point on the monotonic clock (siblings: the wall-clock `SystemTime`).
        // Why:      Spacing between requests is a length of time, measured on a clock that never jumps back.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: T[] / Set / Map / number
        // ```
        return GitProgram { program: PathBuf::from("git"), bound: std::time::Duration::from_secs(GIT_CALL_SECONDS), clean_environment: false, environment: Vec::new() };
    }
}

/// What:
/// `pub fn current_user_id() -> u32` is the effective user id of this process.
/// Why:
/// Git refuses a repository that another user owns unless it is declared safe;
/// the reader leaves such a repository to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const currentUserId = (): number => process.geteuid();
/// ```
pub fn current_user_id() -> u32 {
    return rustix::process::geteuid().as_raw();
}

/// The constructor that reads the process environment.
impl NativeEnvironment {
    /// What:
    /// `pub fn from_process() -> NativeEnvironment` reads the inputs from the process environment.
    /// Why:
    /// Production code asks the process;
    /// tests build the struct by hand.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static fromProcess(): NativeEnvironment
    /// ```
    pub fn from_process() -> NativeEnvironment {
        let home: Option<PathBuf> = std::env::var_os("HOME").map(PathBuf::from);
        let xdg_config_home: Option<PathBuf> = match std::env::var_os("XDG_CONFIG_HOME") {
            Some(value) if !value.is_empty() => Some(PathBuf::from(value)),
            _ => None,
        };
        // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
        // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // (a, b) => body
        // ```
        // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
        // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // String(value)  // JS strings are always owned values
        // ```
        // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
        // Why:      The collection stays usable after the walk and no item is copied until it must be.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const item of list) { ... }  // or list.map(...)
        // ```
        // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
        // Why:      They express one step of a pipeline without an index loop.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // list.map(f).filter(g)
        // ```
        // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
        // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const name: Type = value;
        // ```
        // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
        // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: string
        // ```
        let blocking_variable: Option<String> = BLOCKING_VARIABLES.iter().find(is_set).map(ToString::to_string);
        // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
        // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Array.from(iterable)  // or [...iterable]
        // ```
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        // Adapter `.map(`: transforms or selects items with a closure.
        // Typed binding: `other_system_configs: Vec<PathBuf>`.
        return NativeEnvironment { home, xdg_config_home, system_config: None, blocking_variable, git: GitProgram::default(), user_id: current_user_id() };
    }
}
